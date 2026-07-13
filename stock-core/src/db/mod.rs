pub mod migrations;

<<<<<<< HEAD
use rusqlite::{params, Connection, Result as SqlResult};
use std::collections::HashMap;
=======
use rusqlite::{Connection, Result as SqlResult, params};
>>>>>>> 71bcc8b (ci: 建立 Github Actions CI 檢查工作流並修正 Rust Clippy 警告與格式)
use std::sync::Mutex;

use crate::models::{CanonicalQuote, OhlcvRecord};

pub struct MarketRepository {
    conn: Mutex<Connection>,
}

impl MarketRepository {
    pub fn open(path: &str) -> SqlResult<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let repo = Self {
            conn: Mutex::new(conn),
        };
        migrations::run_migrations(&repo.conn.lock().unwrap())?;
        Ok(repo)
    }

    pub fn open_in_memory() -> SqlResult<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let repo = Self {
            conn: Mutex::new(conn),
        };
        migrations::run_migrations(&repo.conn.lock().unwrap())?;
        Ok(repo)
    }

    pub fn upsert_core_batch(&self, quotes: &[CanonicalQuote]) -> SqlResult<usize> {
        let conn = self.conn.lock().unwrap();
        let mut count = 0;
        let tx = conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO market_quote_core (
                    trade_date, market_code, symbol_raw, symbol_clean,
                    security_name, instrument_type, instrument_subtype,
                    isin_code, cfi_code, section_label,
                    open_price, high_price, low_price, close_price,
                    volume_shares, turnover_value, trade_count,
                    change_sign, change_value,
                    last_bid_price, last_bid_qty_raw,
                    last_ask_price, last_ask_qty_raw,
                    pe_ratio, issued_shares,
                    next_limit_up, next_limit_down,
                    source_file, source_encoding, source_line_no,
                    ingest_run_id, ingested_at
                ) VALUES (
                    ?1, ?2, ?3, ?4,
                    ?5, ?6, ?7,
                    ?8, ?9, ?10,
                    ?11, ?12, ?13, ?14,
                    ?15, ?16, ?17,
                    ?18, ?19,
                    ?20, ?21,
                    ?22, ?23,
                    ?24, ?25,
                    ?26, ?27,
                    ?28, ?29, ?30,
                    ?31, ?32
                ) ON CONFLICT(trade_date, market_code, symbol_clean) DO UPDATE SET
                    security_name     = COALESCE(excluded.security_name,     market_quote_core.security_name),
                    instrument_type   = excluded.instrument_type,
                    instrument_subtype= excluded.instrument_subtype,
                    open_price        = excluded.open_price,
                    high_price        = excluded.high_price,
                    low_price         = excluded.low_price,
                    close_price       = excluded.close_price,
                    volume_shares     = excluded.volume_shares,
                    turnover_value    = excluded.turnover_value,
                    trade_count       = excluded.trade_count,
                    change_sign       = excluded.change_sign,
                    change_value      = excluded.change_value,
                    last_bid_price    = excluded.last_bid_price,
                    last_bid_qty_raw  = excluded.last_bid_qty_raw,
                    last_ask_price    = excluded.last_ask_price,
                    last_ask_qty_raw  = excluded.last_ask_qty_raw,
                    pe_ratio          = excluded.pe_ratio,
                    issued_shares     = excluded.issued_shares,
                    next_limit_up     = excluded.next_limit_up,
                    next_limit_down   = excluded.next_limit_down,
                    source_file       = excluded.source_file,
                    source_encoding   = excluded.source_encoding,
                    source_line_no    = excluded.source_line_no,
                    ingest_run_id     = excluded.ingest_run_id,
                    ingested_at       = excluded.ingested_at
            ")?;
            for q in quotes {
                stmt.execute(params![
                    q.trade_date,
                    q.market_code,
                    q.symbol_raw,
                    q.symbol_clean,
                    q.security_name,
                    q.instrument_type,
                    q.instrument_subtype,
                    q.isin_code,
                    q.cfi_code,
                    q.section_label,
                    q.open_price,
                    q.high_price,
                    q.low_price,
                    q.close_price,
                    q.volume_shares,
                    q.turnover_value,
                    q.trade_count,
                    q.change_sign,
                    q.change_value,
                    q.last_bid_price,
                    q.last_bid_qty_raw,
                    q.last_ask_price,
                    q.last_ask_qty_raw,
                    q.pe_ratio,
                    q.issued_shares,
                    q.next_limit_up,
                    q.next_limit_down,
                    q.source_file,
                    q.source_encoding,
                    q.source_line_no,
                    q.ingest_run_id,
                    q.ingested_at,
                ])?;
                count += 1;
            }
        }
        tx.commit()?;
        Ok(count)
    }

    pub fn upsert_extension_batch(
        &self,
        trade_date: &str,
        market_code: &str,
        symbol_clean: &str,
        raw_change: Option<&str>,
        raw_payload: Option<&str>,
    ) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO market_quote_extension (trade_date, market_code, symbol_clean, raw_change, raw_payload)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(trade_date, market_code, symbol_clean) DO UPDATE SET
                raw_change = COALESCE(excluded.raw_change, market_quote_extension.raw_change),
                raw_payload = excluded.raw_payload",
            params![trade_date, market_code, symbol_clean, raw_change, raw_payload],
        )?;
        Ok(())
    }

    pub fn get_ohlcv(
        &self,
        symbol: &str,
        start_date: &str,
        end_date: &str,
    ) -> SqlResult<Vec<OhlcvRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare_cached(
            "SELECT trade_date, open_price, high_price, low_price, close_price, volume_shares
             FROM market_quote_core
             WHERE symbol_clean = ?1 AND trade_date >= ?2 AND trade_date <= ?3
             ORDER BY trade_date ASC",
        )?;
        let rows = stmt.query_map(params![symbol, start_date, end_date], |row| {
            Ok(OhlcvRecord {
                trade_date: row.get(0)?,
                open_price: row.get(1)?,
                high_price: row.get(2)?,
                low_price: row.get(3)?,
                close_price: row.get(4)?,
                volume_shares: row.get(5)?,
            })
        })?;
        rows.collect()
    }

    pub fn get_distinct_symbols(&self) -> SqlResult<Vec<(String, String, Option<String>)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare_cached(
            "SELECT symbol_clean, MAX(market_code), MAX(security_name)
             FROM market_quote_core
             GROUP BY symbol_clean
             ORDER BY symbol_clean",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })?;
        rows.collect()
    }

    pub fn get_latest_trade_date(&self) -> SqlResult<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare_cached("SELECT MAX(trade_date) FROM market_quote_core")?;
        stmt.query_row([], |row| row.get(0))
    }

    pub fn sync_securities(&self) -> SqlResult<usize> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        let tx = conn.unchecked_transaction()?;
        tx.execute("DELETE FROM securities", [])?;
        tx.execute(
            "INSERT INTO securities (symbol_clean, market_code, security_name, updated_at)
             SELECT symbol_clean, MAX(market_code), MAX(security_name), ?1
             FROM market_quote_core
             WHERE symbol_clean LIKE '00%'
                OR (symbol_clean GLOB '[1-9]???')
             GROUP BY symbol_clean",
            params![now],
        )?;
        let count: usize = tx.changes().try_into().unwrap();
        tx.commit()?;
        Ok(count)
    }

    pub fn get_securities(&self) -> SqlResult<Vec<(String, String, Option<String>)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare_cached(
            "SELECT symbol_clean, market_code, security_name
             FROM securities
             ORDER BY
               CASE WHEN symbol_clean GLOB '[1-9]???' THEN 0 ELSE 1 END,
               symbol_clean",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })?;
        rows.collect()
    }

    pub fn get_symbol_name(&self, symbol: &str) -> SqlResult<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare_cached(
            "SELECT security_name FROM market_quote_core
             WHERE symbol_clean = ?1 AND security_name IS NOT NULL
             LIMIT 1",
        )?;
        stmt.query_row(params![symbol], |row| row.get(0))
    }

    pub fn delete_date_range(&self, start: &str, end: &str) -> SqlResult<usize> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM market_quote_extension WHERE trade_date >= ?1 AND trade_date <= ?2",
            params![start, end],
        )?;
        let count = conn.execute(
            "DELETE FROM market_quote_core WHERE trade_date >= ?1 AND trade_date <= ?2",
            params![start, end],
        )?;
        Ok(count)
    }

    pub fn get_date_range(&self) -> SqlResult<(Option<String>, Option<String>, usize)> {
        let conn = self.conn.lock().unwrap();
        let min: Option<String> =
            conn.query_row("SELECT MIN(trade_date) FROM market_quote_core", [], |row| {
                row.get(0)
            })?;
        let max: Option<String> =
            conn.query_row("SELECT MAX(trade_date) FROM market_quote_core", [], |row| {
                row.get(0)
            })?;
        let count: usize = conn.query_row(
            "SELECT COUNT(DISTINCT trade_date) FROM market_quote_core",
            [],
            |row| row.get(0),
        )?;
        Ok((min, max, count))
    }

    pub fn get_effective_trade_date(&self, max_date: &str) -> SqlResult<Option<String>> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT MAX(trade_date) FROM market_quote_core WHERE trade_date <= ?1",
            params![max_date],
            |row| row.get(0),
        )
    }

    pub fn get_last_n_trade_dates(&self, up_to: &str, limit: usize) -> SqlResult<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare_cached(
            "SELECT DISTINCT trade_date FROM market_quote_core
             WHERE trade_date <= ?1
             ORDER BY trade_date DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![up_to, limit as i64], |row| row.get(0))?;
        rows.collect()
    }

    pub fn get_trade_dates_in_range(&self, start: &str, end: &str) -> SqlResult<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare_cached(
            "SELECT DISTINCT trade_date FROM market_quote_core
             WHERE trade_date >= ?1 AND trade_date <= ?2
             ORDER BY trade_date ASC",
        )?;
        let rows = stmt.query_map(params![start, end], |row| row.get(0))?;
        rows.collect()
    }

    pub fn get_active_symbols(
        &self,
        start_date: &str,
        instrument_types: Option<&[String]>,
    ) -> SqlResult<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut sql = "SELECT DISTINCT symbol_clean FROM market_quote_core WHERE trade_date >= ?1"
            .to_string();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> =
            vec![Box::new(start_date.to_string())];
        if let Some(types) = instrument_types
            && !types.is_empty()
        {
            let placeholders: Vec<String> =
                (0..types.len()).map(|i| format!("?{}", i + 2)).collect();
            sql.push_str(&format!(
                " AND instrument_type IN ({})",
                placeholders.join(",")
            ));
            for t in types {
                param_values.push(Box::new(t.clone()));
            }
        }
        let mut stmt = conn.prepare_cached(&sql)?;
        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let rows = stmt.query_map(param_refs.as_slice(), |row| row.get(0))?;
        rows.collect()
    }

    pub fn get_batch_ohlcv(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> SqlResult<Vec<(String, OhlcvRecord)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare_cached(
            "SELECT symbol_clean, trade_date, open_price, high_price, low_price, close_price, volume_shares
             FROM market_quote_core
             WHERE trade_date >= ?1 AND trade_date <= ?2
             ORDER BY symbol_clean, trade_date ASC"
        )?;
        let rows = stmt.query_map(params![start_date, end_date], |row| {
            Ok((
                row.get::<_, String>(0)?,
                OhlcvRecord {
                    trade_date: row.get(1)?,
                    open_price: row.get(2)?,
                    high_price: row.get(3)?,
                    low_price: row.get(4)?,
                    close_price: row.get(5)?,
                    volume_shares: row.get(6)?,
                },
            ))
        })?;
        rows.collect()
    }

    pub fn get_name_map(&self, as_of_date: &str) -> SqlResult<HashMap<String, Option<String>>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare_cached(
            "SELECT symbol_clean, security_name FROM market_quote_core
             WHERE (symbol_clean, trade_date) IN (
                 SELECT symbol_clean, MAX(trade_date)
                 FROM market_quote_core
                 WHERE security_name IS NOT NULL AND trade_date <= ?1
                 GROUP BY symbol_clean
             )",
        )?;
        let rows = stmt.query_map(params![as_of_date], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
        })?;
        let mut map = HashMap::new();
        for result in rows {
            let (sym, name) = result?;
            map.entry(sym).or_insert(name);
        }
        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_quote(symbol: &str, date: &str, close: f64) -> CanonicalQuote {
        CanonicalQuote {
            trade_date: date.to_string(),
            market_code: "TWSE".to_string(),
            symbol_raw: symbol.to_string(),
            symbol_clean: symbol.to_string(),
            security_name: Some(symbol.to_string()),
            instrument_type: "stock".to_string(),
            instrument_subtype: None,
            isin_code: None,
            cfi_code: None,
            section_label: None,
            open_price: Some(close),
            high_price: Some(close),
            low_price: Some(close),
            close_price: Some(close),
            volume_shares: Some(1000),
            turnover_value: None,
            trade_count: None,
            change_sign: None,
            change_value: None,
            last_bid_price: None,
            last_bid_qty_raw: None,
            last_ask_price: None,
            last_ask_qty_raw: None,
            pe_ratio: None,
            issued_shares: None,
            next_limit_up: None,
            next_limit_down: None,
            source_file: "test".to_string(),
            source_encoding: "utf-8".to_string(),
            source_line_no: 0,
            ingest_run_id: "test-ingest".to_string(),
            ingested_at: "2024-01-01T00:00:00".to_string(),
            extension: HashMap::new(),
        }
    }

    #[test]
    fn test_upsert_and_retrieve_ohlcv() {
        let repo = MarketRepository::open_in_memory().unwrap();
        let quotes = vec![
            make_quote("2330", "2024-01-02", 500.0),
            make_quote("2330", "2024-01-03", 505.0),
        ];
        repo.upsert_core_batch(&quotes).unwrap();
        let ohlcv = repo.get_ohlcv("2330", "2024-01-01", "2024-01-31").unwrap();
        assert_eq!(ohlcv.len(), 2);
        assert_eq!(ohlcv[0].close_price, Some(500.0));
        assert_eq!(ohlcv[1].close_price, Some(505.0));
    }

    #[test]
    fn test_get_distinct_symbols() {
        let repo = MarketRepository::open_in_memory().unwrap();
        let quotes = vec![
            make_quote("2330", "2024-01-02", 500.0),
            make_quote("1101", "2024-01-02", 30.0),
        ];
        repo.upsert_core_batch(&quotes).unwrap();
        let symbols = repo.get_distinct_symbols().unwrap();
        assert_eq!(symbols.len(), 2);
    }
}
