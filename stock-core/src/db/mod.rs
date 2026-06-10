pub mod migrations;

use rusqlite::{params, Connection, Result as SqlResult};
use std::sync::Mutex;

use crate::models::{CanonicalQuote, OhlcvRecord, ScreenResult};

pub struct MarketRepository {
    conn: Mutex<Connection>,
}

impl MarketRepository {
    pub fn open(path: &str) -> SqlResult<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let repo = Self { conn: Mutex::new(conn) };
        migrations::run_migrations(&repo.conn.lock().unwrap())?;
        Ok(repo)
    }

    pub fn open_in_memory() -> SqlResult<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let repo = Self { conn: Mutex::new(conn) };
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
                    q.trade_date, q.market_code, q.symbol_raw, q.symbol_clean,
                    q.security_name, q.instrument_type, q.instrument_subtype,
                    q.isin_code, q.cfi_code, q.section_label,
                    q.open_price, q.high_price, q.low_price, q.close_price,
                    q.volume_shares, q.turnover_value, q.trade_count,
                    q.change_sign, q.change_value,
                    q.last_bid_price, q.last_bid_qty_raw,
                    q.last_ask_price, q.last_ask_qty_raw,
                    q.pe_ratio, q.issued_shares,
                    q.next_limit_up, q.next_limit_down,
                    q.source_file, q.source_encoding, q.source_line_no,
                    q.ingest_run_id, q.ingested_at,
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
             ORDER BY trade_date ASC"
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

    pub fn screen_bollinger_squeeze(
        &self,
        bandwidth_limit: f64,
        consecutive_days: i32,
        instrument_types: Option<&[String]>,
        min_volume: i64,
    ) -> SqlResult<Vec<ScreenResult>> {
        let conn = self.conn.lock().unwrap();
        let required_days = consecutive_days + 40;
        let mut dates: Vec<String> = {
            let mut stmt = conn.prepare_cached(
                "SELECT DISTINCT trade_date FROM market_quote_core ORDER BY trade_date DESC LIMIT ?1"
            )?;
            let rows = stmt.query_map(params![required_days], |row| row.get(0))?;
            rows.collect::<SqlResult<Vec<String>>>()?
        };
        dates.reverse();
        if dates.is_empty() {
            return Ok(vec![]);
        }
        let start_date = &dates[0];

        let symbols: Vec<String> = {
            let mut sql = "SELECT DISTINCT symbol_clean FROM market_quote_core WHERE trade_date >= ?1".to_string();
            if let Some(types) = instrument_types {
                if !types.is_empty() {
                    let placeholders: Vec<String> = types.iter().enumerate().map(|(i, _)| format!("?{}", i + 2)).collect();
                    sql.push_str(&format!(" AND instrument_type IN ({})", placeholders.join(",")));
                }
            }
            let mut stmt = conn.prepare_cached(&sql)?;
            let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(start_date.to_string())];
            if let Some(types) = instrument_types {
                for t in types {
                    param_values.push(Box::new(t.clone()));
                }
            }
            let param_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
            let rows = stmt.query_map(param_refs.as_slice(), |row| row.get(0))?;
            rows.collect::<SqlResult<Vec<String>>>()?
        };

        let mut results = Vec::new();
        for symbol in &symbols {
            let data = self.get_ohlcv(symbol, start_date, &dates.last().cloned().unwrap_or_default())?;
            if data.len() < 20 {
                continue;
            }
            let prices: Vec<f64> = data.iter().filter_map(|r| r.close_price).collect();
            if prices.len() < 20 {
                continue;
            }
            let bollinger = crate::analysis::bollinger(&prices, 20, 2.0);
            let bandwidth: Vec<Option<f64>> = bollinger.upper.iter().zip(bollinger.lower.iter()).zip(bollinger.middle.iter())
                .map(|((u, l), m)| {
                    match (u, l, m) {
                        (Some(u), Some(l), Some(m)) if *m != 0.0 => Some((u - l) / m),
                        _ => None,
                    }
                })
                .collect();
            let valid_bw: Vec<f64> = bandwidth.iter().filter_map(|b| *b).collect();
            if valid_bw.len() < consecutive_days as usize {
                continue;
            }
            let recent: Vec<f64> = valid_bw.iter().rev().take(consecutive_days as usize).copied().rev().collect();
            let max_bw = recent.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            if max_bw <= bandwidth_limit {
                if let Some(latest) = data.last() {
                    if min_volume > 0 {
                        if let Some(v) = latest.volume_shares {
                            if v < min_volume {
                                continue;
                            }
                        } else {
                            continue;
                        }
                    }
                    let security_name = data.iter().find_map(|r| {
                        let conn = self.conn.lock().unwrap();
                        let mut stmt = conn.prepare_cached(
                            "SELECT security_name FROM market_quote_core WHERE symbol_clean = ?1 AND trade_date = ?2 AND security_name IS NOT NULL LIMIT 1"
                        ).ok()?;
                        stmt.query_row(params![symbol, r.trade_date], |row| row.get::<_, Option<String>>(0)).ok()?
                    });
                    results.push(ScreenResult {
                        symbol: symbol.clone(),
                        name: security_name,
                        latest_close: latest.close_price,
                        latest_volume: latest.volume_shares,
                        max_bandwidth: max_bw,
                        latest_bandwidth: *recent.last().unwrap_or(&0.0),
                    });
                }
            }
        }
        Ok(results)
    }

    pub fn get_distinct_symbols(&self) -> SqlResult<Vec<(String, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare_cached(
            "SELECT DISTINCT symbol_clean, market_code FROM market_quote_core ORDER BY symbol_clean"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        rows.collect()
    }

    pub fn get_latest_trade_date(&self) -> SqlResult<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare_cached(
            "SELECT MAX(trade_date) FROM market_quote_core"
        )?;
        stmt.query_row([], |row| row.get(0))
    }
}
