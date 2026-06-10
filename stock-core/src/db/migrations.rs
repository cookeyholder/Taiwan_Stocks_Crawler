use rusqlite::{Connection, Result};

const MIGRATIONS: &[&str] = &[
    // V1: market_quote_core
    "CREATE TABLE IF NOT EXISTS market_quote_core (
        trade_date      TEXT NOT NULL,
        market_code     TEXT NOT NULL,
        symbol_raw      TEXT NOT NULL,
        symbol_clean    TEXT NOT NULL,
        security_name   TEXT,
        instrument_type TEXT NOT NULL DEFAULT 'other',
        instrument_subtype TEXT,
        isin_code       TEXT,
        cfi_code        TEXT,
        section_label   TEXT,
        open_price      REAL,
        high_price      REAL,
        low_price       REAL,
        close_price     REAL,
        volume_shares   INTEGER,
        turnover_value  REAL,
        trade_count     INTEGER,
        change_sign     TEXT,
        change_value    REAL,
        last_bid_price  REAL,
        last_bid_qty_raw TEXT,
        last_ask_price  REAL,
        last_ask_qty_raw TEXT,
        pe_ratio        REAL,
        issued_shares   INTEGER,
        next_limit_up   REAL,
        next_limit_down REAL,
        source_file     TEXT NOT NULL DEFAULT '',
        source_encoding TEXT NOT NULL DEFAULT '',
        source_line_no  INTEGER NOT NULL DEFAULT 0,
        ingest_run_id   TEXT NOT NULL DEFAULT '',
        ingested_at     TEXT NOT NULL DEFAULT '',
        PRIMARY KEY (trade_date, market_code, symbol_clean)
    );",
    // V2: market_quote_extension
    "CREATE TABLE IF NOT EXISTS market_quote_extension (
        trade_date    TEXT NOT NULL,
        market_code   TEXT NOT NULL,
        symbol_clean  TEXT NOT NULL,
        raw_change    TEXT,
        raw_payload   TEXT,
        PRIMARY KEY (trade_date, market_code, symbol_clean),
        FOREIGN KEY (trade_date, market_code, symbol_clean)
          REFERENCES market_quote_core(trade_date, market_code, symbol_clean)
    );",
];

pub fn run_migrations(conn: &Connection) -> Result<()> {
    let current_version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    for (i, sql) in MIGRATIONS.iter().enumerate() {
        let version = (i + 1) as i32;
        if version > current_version {
            conn.execute_batch(sql)?;
            conn.execute(&format!("PRAGMA user_version = {}", version), [])?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_fresh_migration() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        let version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0)).unwrap();
        assert_eq!(version, MIGRATIONS.len() as i32);
        conn.execute("SELECT 1 FROM market_quote_core LIMIT 1", []).unwrap();
        conn.execute("SELECT 1 FROM market_quote_extension LIMIT 1", []).unwrap();
    }

    #[test]
    fn test_idempotent_migration() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap();
        let version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0)).unwrap();
        assert_eq!(version, MIGRATIONS.len() as i32);
    }
}
