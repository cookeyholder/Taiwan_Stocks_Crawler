pub mod migrations;

use chrono::Datelike;
use rusqlite::{Connection, Result as SqlResult, params};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use crate::models::{
    BrokerSummary, BrokerTradeRecord, CacheParams, CanonicalQuote, GapReport, InstitutionalRecord,
    OhlcvRecord, ScreenResult,
};

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

    /// 查詢指定日期區間內所有未入庫之平日工作日（自動排除週六與週日）
    pub fn find_missing_dates(&self, start: &str, end: &str) -> SqlResult<Vec<String>> {
        let start_date = chrono::NaiveDate::parse_from_str(start, "%Y-%m-%d")
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let end_date = chrono::NaiveDate::parse_from_str(end, "%Y-%m-%d")
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        if start_date > end_date {
            return Ok(Vec::new());
        }

        let recorded_dates: std::collections::HashSet<String> = self
            .get_trade_dates_in_range(start, end)?
            .into_iter()
            .collect();

        let mut missing = Vec::new();
        let mut curr = start_date;
        while curr <= end_date {
            let weekday = curr.weekday();
            if weekday != chrono::Weekday::Sat && weekday != chrono::Weekday::Sun {
                let date_str = curr.format("%Y-%m-%d").to_string();
                if !recorded_dates.contains(&date_str) {
                    missing.push(date_str);
                }
            }
            curr += chrono::Duration::days(1);
        }

        Ok(missing)
    }

    /// 診斷指定日期區間之平日交易日缺漏狀況並回傳診斷報告
    pub fn diagnose_gaps(&self, start: &str, end: &str) -> SqlResult<GapReport> {
        let start_date = chrono::NaiveDate::parse_from_str(start, "%Y-%m-%d")
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let end_date = chrono::NaiveDate::parse_from_str(end, "%Y-%m-%d")
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        if start_date > end_date {
            return Ok(GapReport {
                start_date: start.to_string(),
                end_date: end.to_string(),
                total_workdays: 0,
                recorded_dates: 0,
                missing_dates: Vec::new(),
            });
        }

        let recorded_dates_vec = self.get_trade_dates_in_range(start, end)?;
        let recorded_set: std::collections::HashSet<String> =
            recorded_dates_vec.into_iter().collect();

        let mut total_workdays = 0;
        let mut recorded_workdays = 0;
        let mut missing_dates = Vec::new();
        let mut curr = start_date;

        while curr <= end_date {
            let weekday = curr.weekday();
            if weekday != chrono::Weekday::Sat && weekday != chrono::Weekday::Sun {
                total_workdays += 1;
                let date_str = curr.format("%Y-%m-%d").to_string();
                if recorded_set.contains(&date_str) {
                    recorded_workdays += 1;
                } else {
                    missing_dates.push(date_str);
                }
            }
            curr += chrono::Duration::days(1);
        }

        Ok(GapReport {
            start_date: start.to_string(),
            end_date: end.to_string(),
            total_workdays,
            recorded_dates: recorded_workdays,
            missing_dates,
        })
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

    pub fn get_cached_screening(&self, key: &str) -> SqlResult<Option<Vec<ScreenResult>>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare_cached("SELECT results_json FROM screener_cache WHERE cache_key = ?1")?;
        let result = stmt.query_row(params![key], |row| row.get::<_, String>(0));
        match result {
            Ok(json) => match serde_json::from_str::<Vec<ScreenResult>>(&json) {
                Ok(screen_results) => Ok(Some(screen_results)),
                Err(e) => {
                    eprintln!("[cache] corrupted JSON for key {}: {}", key, e);
                    Ok(None)
                }
            },
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn set_cached_screening(
        &self,
        key: &str,
        params: &CacheParams,
        results: &[ScreenResult],
    ) -> SqlResult<()> {
        let json = serde_json::to_string(results).map_err(|e| {
            eprintln!("[cache] failed to serialize results for key {}: {}", key, e);
            rusqlite::Error::ToSqlConversionFailure(Box::new(e))
        })?;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO screener_cache (cache_key, query_date, instrument_types, max_bandwidth, min_days, min_volume, results_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![key, params.query_date, params.instrument_types, params.max_bandwidth, params.min_days, params.min_volume, json],
        )?;
        Ok(())
    }

    pub fn clear_screener_cache(&self) -> SqlResult<usize> {
        let conn = self.conn.lock().unwrap();
        let deleted = conn.execute("DELETE FROM screener_cache", [])?;
        Ok(deleted)
    }

    pub fn get_screener_cache_count(&self) -> SqlResult<usize> {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT COUNT(*) FROM screener_cache", [], |row| row.get(0))
    }

    /// 加入自選股（冪等寫入）
    pub fn add_to_watchlist(&self, symbol_clean: &str, group_name: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO watchlists (symbol_clean, group_name) VALUES (?1, ?2)",
            params![symbol_clean, group_name],
        )?;
        Ok(())
    }

    /// 移出自選股（若 group_name 為 None 則移除所有群組的該股票）
    pub fn remove_from_watchlist(
        &self,
        symbol_clean: &str,
        group_name: Option<&str>,
    ) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        match group_name {
            Some(group) => {
                conn.execute(
                    "DELETE FROM watchlists WHERE symbol_clean = ?1 AND group_name = ?2",
                    params![symbol_clean, group],
                )?;
            }
            None => {
                conn.execute(
                    "DELETE FROM watchlists WHERE symbol_clean = ?1",
                    params![symbol_clean],
                )?;
            }
        }
        Ok(())
    }

    /// 查詢自選股清單（回傳 symbol_clean 列表）
    pub fn get_watchlist(&self, group_name: Option<&str>) -> SqlResult<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut symbols = Vec::new();
        match group_name {
            Some(group) => {
                let mut stmt = conn.prepare(
                    "SELECT symbol_clean FROM watchlists WHERE group_name = ?1 ORDER BY created_at DESC, symbol_clean ASC",
                )?;
                let rows = stmt.query_map(params![group], |row| row.get(0))?;
                for sym in rows {
                    symbols.push(sym?);
                }
            }
            None => {
                let mut stmt = conn.prepare(
                    "SELECT DISTINCT symbol_clean FROM watchlists ORDER BY created_at DESC, symbol_clean ASC",
                )?;
                let rows = stmt.query_map([], |row| row.get(0))?;
                for sym in rows {
                    symbols.push(sym?);
                }
            }
        }
        Ok(symbols)
    }

    /// 取得所有已建立之自選群組名稱
    pub fn get_watchlist_groups(&self) -> SqlResult<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT DISTINCT group_name FROM watchlists ORDER BY group_name ASC")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        let mut groups = Vec::new();
        for g in rows {
            groups.push(g?);
        }
        if groups.is_empty() {
            groups.push("預設自選".to_string());
        }
        Ok(groups)
    }

    /// 判斷股票是否已加入自選股
    pub fn is_in_watchlist(&self, symbol_clean: &str, group_name: Option<&str>) -> SqlResult<bool> {
        let conn = self.conn.lock().unwrap();
        let query = match group_name {
            Some(group) => {
                let mut stmt = conn.prepare_cached(
                    "SELECT 1 FROM watchlists WHERE symbol_clean = ?1 AND group_name = ?2 LIMIT 1",
                )?;
                stmt.exists(params![symbol_clean, group])?
            }
            None => {
                let mut stmt = conn
                    .prepare_cached("SELECT 1 FROM watchlists WHERE symbol_clean = ?1 LIMIT 1")?;
                stmt.exists(params![symbol_clean])?
            }
        };
        Ok(query)
    }

    /// 批次寫入或更新三大法人買賣超資料
    pub fn upsert_institutional_batch(&self, records: &[InstitutionalRecord]) -> SqlResult<usize> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        let mut count = 0;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO institutional_investors (
                    trade_date, symbol_clean,
                    foreign_buy, foreign_sell, foreign_net,
                    trust_buy, trust_sell, trust_net,
                    dealer_buy, dealer_sell, dealer_net,
                    total_net
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                ON CONFLICT(trade_date, symbol_clean) DO UPDATE SET
                    foreign_buy = excluded.foreign_buy,
                    foreign_sell = excluded.foreign_sell,
                    foreign_net = excluded.foreign_net,
                    trust_buy = excluded.trust_buy,
                    trust_sell = excluded.trust_sell,
                    trust_net = excluded.trust_net,
                    dealer_buy = excluded.dealer_buy,
                    dealer_sell = excluded.dealer_sell,
                    dealer_net = excluded.dealer_net,
                    total_net = excluded.total_net
                ",
            )?;
            for r in records {
                stmt.execute(params![
                    r.trade_date,
                    r.symbol,
                    r.foreign_buy,
                    r.foreign_sell,
                    r.foreign_net,
                    r.trust_buy,
                    r.trust_sell,
                    r.trust_net,
                    r.dealer_buy,
                    r.dealer_sell,
                    r.dealer_net,
                    r.total_net,
                ])?;
                count += 1;
            }
        }
        tx.commit()?;
        Ok(count)
    }

    /// 查詢指定個股在指定日期區間之三大法人買賣超資料（依日期遞增排序）
    pub fn get_institutional_records(
        &self,
        symbol: &str,
        start_date: &str,
        end_date: &str,
    ) -> SqlResult<Vec<InstitutionalRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare_cached(
            "SELECT trade_date, symbol_clean,
                    foreign_buy, foreign_sell, foreign_net,
                    trust_buy, trust_sell, trust_net,
                    dealer_buy, dealer_sell, dealer_net,
                    total_net
             FROM institutional_investors
             WHERE symbol_clean = ?1 AND trade_date >= ?2 AND trade_date <= ?3
             ORDER BY trade_date ASC",
        )?;
        let rows = stmt.query_map(params![symbol, start_date, end_date], |row| {
            Ok(InstitutionalRecord {
                trade_date: row.get(0)?,
                symbol: row.get(1)?,
                foreign_buy: row.get(2)?,
                foreign_sell: row.get(3)?,
                foreign_net: row.get(4)?,
                trust_buy: row.get(5)?,
                trust_sell: row.get(6)?,
                trust_net: row.get(7)?,
                dealer_buy: row.get(8)?,
                dealer_sell: row.get(9)?,
                dealer_net: row.get(10)?,
                total_net: row.get(11)?,
            })
        })?;
        rows.collect()
    }

    /// 批次寫入或更新券商買賣分點成交明細資料
    pub fn upsert_broker_trades_batch(&self, records: &[BrokerTradeRecord]) -> SqlResult<usize> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        let mut count = 0;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO broker_trades (
                    trade_date, symbol_clean, broker_id, broker_name,
                    price, buy_shares, sell_shares
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                ON CONFLICT(trade_date, symbol_clean, broker_id, price) DO UPDATE SET
                    broker_name = excluded.broker_name,
                    buy_shares = excluded.buy_shares,
                    sell_shares = excluded.sell_shares
                ",
            )?;
            for r in records {
                stmt.execute(params![
                    r.trade_date,
                    r.symbol,
                    r.broker_id,
                    r.broker_name,
                    r.price,
                    r.buy_shares,
                    r.sell_shares,
                ])?;
                count += 1;
            }
        }
        tx.commit()?;
        Ok(count)
    }

    /// 查詢指定個股與交易日之券商分點成交明細（依券商代號與價格排序）
    pub fn get_broker_trades(
        &self,
        symbol: &str,
        trade_date: &str,
    ) -> SqlResult<Vec<BrokerTradeRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare_cached(
            "SELECT trade_date, symbol_clean, broker_id, broker_name,
                    price, buy_shares, sell_shares
             FROM broker_trades
             WHERE symbol_clean = ?1 AND trade_date = ?2
             ORDER BY broker_id ASC, price ASC",
        )?;
        let rows = stmt.query_map(params![symbol, trade_date], |row| {
            Ok(BrokerTradeRecord {
                trade_date: row.get(0)?,
                symbol: row.get(1)?,
                broker_id: row.get(2)?,
                broker_name: row.get(3)?,
                price: row.get(4)?,
                buy_shares: row.get(5)?,
                sell_shares: row.get(6)?,
            })
        })?;
        rows.collect()
    }

    /// 查詢指定交易日中已完成分點數據入庫之所有個股代號集合（供斷點續傳比對）
    pub fn get_broker_completed_symbols(&self, trade_date: &str) -> SqlResult<HashSet<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare_cached(
            "SELECT DISTINCT symbol_clean FROM broker_trades WHERE trade_date = ?1",
        )?;
        let rows = stmt.query_map(params![trade_date], |row| row.get(0))?;
        let mut set = HashSet::new();
        for r in rows {
            set.insert(r?);
        }
        Ok(set)
    }

    /// 查詢指定個股與交易日之券商分點買賣超彙總（依淨買賣張數絕對值降冪排序）
    pub fn get_broker_summary(
        &self,
        symbol: &str,
        trade_date: &str,
    ) -> SqlResult<Vec<BrokerSummary>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare_cached(
            "SELECT broker_id, broker_name,
                    SUM(buy_shares) as total_buy,
                    SUM(sell_shares) as total_sell,
                    SUM(buy_shares - sell_shares) as net_shares
             FROM broker_trades
             WHERE symbol_clean = ?1 AND trade_date = ?2
             GROUP BY broker_id, broker_name
             ORDER BY ABS(SUM(buy_shares - sell_shares)) DESC",
        )?;
        let rows = stmt.query_map(params![symbol, trade_date], |row| {
            Ok(BrokerSummary {
                broker_id: row.get(0)?,
                broker_name: row.get(1)?,
                total_buy: row.get(2)?,
                total_sell: row.get(3)?,
                net_shares: row.get(4)?,
            })
        })?;
        rows.collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CacheParams, ScreenResult};

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
    fn test_institutional_investors_upsert_and_query() {
        let repo = MarketRepository::open_in_memory().unwrap();
        let records = vec![
            InstitutionalRecord {
                trade_date: "2024-01-02".to_string(),
                symbol: "2330".to_string(),
                foreign_buy: 10_000,
                foreign_sell: 5_000,
                foreign_net: 5_000,
                trust_buy: 2_000,
                trust_sell: 1_000,
                trust_net: 1_000,
                dealer_buy: 500,
                dealer_sell: 100,
                dealer_net: 400,
                total_net: 6_400,
            },
            InstitutionalRecord {
                trade_date: "2024-01-03".to_string(),
                symbol: "2330".to_string(),
                foreign_buy: 8_000,
                foreign_sell: 9_000,
                foreign_net: -1_000,
                trust_buy: 500,
                trust_sell: 0,
                trust_net: 500,
                dealer_buy: 200,
                dealer_sell: 300,
                dealer_net: -100,
                total_net: -600,
            },
        ];

        let count = repo.upsert_institutional_batch(&records).unwrap();
        assert_eq!(count, 2);

        let queried = repo
            .get_institutional_records("2330", "2024-01-01", "2024-01-03")
            .unwrap();
        assert_eq!(queried.len(), 2);
        assert_eq!(queried[0].trade_date, "2024-01-02");
        assert_eq!(queried[0].total_net, 6_400);
        assert_eq!(queried[1].trade_date, "2024-01-03");
        assert_eq!(queried[1].total_net, -600);
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

    #[test]
    fn test_screener_cache_write_and_read() {
        let repo = MarketRepository::open_in_memory().unwrap();
        let key = "v1|2026-07-14|stock|0.0800|5|1000000";
        let params = CacheParams {
            query_date: "2026-07-14".into(),
            instrument_types: "stock".into(),
            max_bandwidth: 0.08,
            min_days: 5,
            min_volume: 1000000,
        };
        let results = vec![ScreenResult {
            symbol: "2330".into(),
            name: Some("台積電".into()),
            trade_date: "2026-07-14".into(),
            latest_close: Some(1000.0),
            latest_volume: Some(10000000),
            max_bandwidth: 0.08,
            latest_bandwidth: 0.04,
            latest_upper_band: Some(1050.0),
            break_upper: false,
            tags: vec![],
        }];
        repo.set_cached_screening(key, &params, &results).unwrap();
        let cached = repo.get_cached_screening(key).unwrap().unwrap();
        assert_eq!(cached.len(), 1);
        assert_eq!(cached[0].symbol, "2330");
        assert_eq!(cached[0].name.as_deref(), Some("台積電"));
    }

    #[test]
    fn test_screener_cache_miss() {
        let repo = MarketRepository::open_in_memory().unwrap();
        let result = repo.get_cached_screening("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_screener_cache_overwrite() {
        let repo = MarketRepository::open_in_memory().unwrap();
        let key = "test|overwrite";
        let params = CacheParams {
            query_date: "2026-07-14".into(),
            instrument_types: "stock".into(),
            max_bandwidth: 0.08,
            min_days: 5,
            min_volume: 1000000,
        };
        let first = vec![ScreenResult {
            symbol: "2330".into(),
            name: None,
            trade_date: "2026-07-14".into(),
            latest_close: None,
            latest_volume: None,
            max_bandwidth: 0.0,
            latest_bandwidth: 0.0,
            latest_upper_band: None,
            break_upper: false,
            tags: vec![],
        }];
        let second = vec![ScreenResult {
            symbol: "1101".into(),
            name: None,
            trade_date: "2026-07-14".into(),
            latest_close: None,
            latest_volume: None,
            max_bandwidth: 0.0,
            latest_bandwidth: 0.0,
            latest_upper_band: None,
            break_upper: false,
            tags: vec![],
        }];
        repo.set_cached_screening(key, &params, &first).unwrap();
        repo.set_cached_screening(key, &params, &second).unwrap();
        let cached = repo.get_cached_screening(key).unwrap().unwrap();
        assert_eq!(cached.len(), 1);
        assert_eq!(cached[0].symbol, "1101");
    }

    #[test]
    fn test_screener_cache_clear() {
        let repo = MarketRepository::open_in_memory().unwrap();
        let params = CacheParams {
            query_date: "2026-07-14".into(),
            instrument_types: "stock".into(),
            max_bandwidth: 0.08,
            min_days: 5,
            min_volume: 1000000,
        };
        repo.set_cached_screening("k1", &params, &[]).unwrap();
        repo.set_cached_screening("k2", &params, &[]).unwrap();
        let deleted = repo.clear_screener_cache().unwrap();
        assert_eq!(deleted, 2);
        assert!(repo.get_cached_screening("k1").unwrap().is_none());
        assert!(repo.get_cached_screening("k2").unwrap().is_none());
    }

    #[test]
    fn test_screener_cache_corrupted_json() {
        let repo = MarketRepository::open_in_memory().unwrap();
        let key = "corrupted";
        {
            let conn = repo.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO screener_cache (cache_key, query_date, instrument_types, max_bandwidth, min_days, min_volume, results_json)
                 VALUES (?1, '2026-07-14', 'stock', 0.08, 5, 1000000, '{bad json}')",
                params![key],
            ).unwrap();
        }
        let result = repo.get_cached_screening(key).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_watchlist_add_and_query() {
        let repo = MarketRepository::open_in_memory().unwrap();
        assert_eq!(repo.get_watchlist(None).unwrap().len(), 0);

        repo.add_to_watchlist("2330", "預設自選").unwrap();
        repo.add_to_watchlist("2454", "預設自選").unwrap();
        repo.add_to_watchlist("2317", "科技股").unwrap();

        let all = repo.get_watchlist(None).unwrap();
        assert_eq!(all.len(), 3);
        assert!(all.contains(&"2330".to_string()));
        assert!(all.contains(&"2454".to_string()));
        assert!(all.contains(&"2317".to_string()));

        let default_group = repo.get_watchlist(Some("預設自選")).unwrap();
        assert_eq!(default_group.len(), 2);
        assert!(default_group.contains(&"2330".to_string()));
        assert!(default_group.contains(&"2454".to_string()));
        assert!(!default_group.contains(&"2317".to_string()));

        let tech_group = repo.get_watchlist(Some("科技股")).unwrap();
        assert_eq!(tech_group.len(), 1);
        assert_eq!(tech_group[0], "2317");
    }

    #[test]
    fn test_watchlist_remove() {
        let repo = MarketRepository::open_in_memory().unwrap();
        repo.add_to_watchlist("2330", "預設自選").unwrap();
        repo.add_to_watchlist("2330", "科技股").unwrap();
        repo.add_to_watchlist("2454", "預設自選").unwrap();

        assert!(repo.is_in_watchlist("2330", Some("預設自選")).unwrap());
        assert!(repo.is_in_watchlist("2330", Some("科技股")).unwrap());

        // 移出單人群組
        repo.remove_from_watchlist("2330", Some("預設自選"))
            .unwrap();
        assert!(!repo.is_in_watchlist("2330", Some("預設自選")).unwrap());
        assert!(repo.is_in_watchlist("2330", Some("科技股")).unwrap());
        assert!(repo.is_in_watchlist("2330", None).unwrap());

        // 移出全部群組
        repo.remove_from_watchlist("2330", None).unwrap();
        assert!(!repo.is_in_watchlist("2330", None).unwrap());
    }

    #[test]
    fn test_watchlist_idempotency() {
        let repo = MarketRepository::open_in_memory().unwrap();
        repo.add_to_watchlist("2330", "預設自選").unwrap();
        repo.add_to_watchlist("2330", "預設自選").unwrap();
        let list = repo.get_watchlist(Some("預設自選")).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0], "2330");

        // 移除不存在的項目亦不應報錯
        repo.remove_from_watchlist("9999", None).unwrap();
    }

    #[test]
    fn test_watchlist_groups() {
        let repo = MarketRepository::open_in_memory().unwrap();
        assert_eq!(repo.get_watchlist_groups().unwrap(), vec!["預設自選"]);
        repo.add_to_watchlist("2330", "群組B").unwrap();
        repo.add_to_watchlist("2454", "群組A").unwrap();
        repo.add_to_watchlist("2317", "群組B").unwrap();

        let groups = repo.get_watchlist_groups().unwrap();
        assert_eq!(groups, vec!["群組A".to_string(), "群組B".to_string()]);
    }

    #[test]
    fn test_diagnose_gaps_and_find_missing() {
        let repo = MarketRepository::open_in_memory().unwrap();

        // 2026-07-06(週一) ~ 2026-07-12(週日)
        // 平日為 07-06, 07-07, 07-08, 07-09, 07-10 (共 5 天)
        // 週末為 07-11(六), 07-12(日)
        let report_empty = repo.diagnose_gaps("2026-07-06", "2026-07-12").unwrap();
        assert_eq!(report_empty.total_workdays, 5);
        assert_eq!(report_empty.recorded_dates, 0);
        assert_eq!(report_empty.missing_dates.len(), 5);
        assert_eq!(
            report_empty.missing_dates,
            vec![
                "2026-07-06",
                "2026-07-07",
                "2026-07-08",
                "2026-07-09",
                "2026-07-10"
            ]
        );

        // 插入週一 (07-06) 與週三 (07-08)
        let quotes = vec![
            make_quote("2330", "2026-07-06", 500.0),
            make_quote("2330", "2026-07-08", 505.0),
        ];
        repo.upsert_core_batch(&quotes).unwrap();

        let report_partial = repo.diagnose_gaps("2026-07-06", "2026-07-12").unwrap();
        assert_eq!(report_partial.total_workdays, 5);
        assert_eq!(report_partial.recorded_dates, 2);
        assert_eq!(report_partial.missing_dates.len(), 3);
        assert_eq!(
            report_partial.missing_dates,
            vec!["2026-07-07", "2026-07-09", "2026-07-10"]
        );

        // 插入剩餘平日
        let quotes_rest = vec![
            make_quote("2330", "2026-07-07", 510.0),
            make_quote("2330", "2026-07-09", 515.0),
            make_quote("2330", "2026-07-10", 520.0),
            make_quote("2330", "2026-07-11", 521.0),
        ];
        repo.upsert_core_batch(&quotes_rest).unwrap();

        let report_full = repo.diagnose_gaps("2026-07-06", "2026-07-12").unwrap();
        assert_eq!(report_full.total_workdays, 5);
        assert_eq!(report_full.recorded_dates, 5);
        assert_eq!(report_full.missing_dates.len(), 0);
        assert!(report_full.missing_dates.is_empty());
    }

    #[test]
    fn test_diagnose_gaps_invalid_range() {
        let repo = MarketRepository::open_in_memory().unwrap();
        let report = repo.diagnose_gaps("2026-07-10", "2026-07-01").unwrap();
        assert_eq!(report.total_workdays, 0);
        assert_eq!(report.missing_dates.len(), 0);
    }

    #[test]
    fn test_broker_trades_upsert_and_query() {
        let repo = MarketRepository::open_in_memory().unwrap();
        let records = vec![
            BrokerTradeRecord {
                trade_date: "2026-10-08".to_string(),
                symbol: "2330".to_string(),
                broker_id: "1020".to_string(),
                broker_name: "合庫".to_string(),
                price: 2550.0,
                buy_shares: 8000,
                sell_shares: 0,
            },
            BrokerTradeRecord {
                trade_date: "2026-10-08".to_string(),
                symbol: "2330".to_string(),
                broker_id: "1020".to_string(),
                broker_name: "合庫".to_string(),
                price: 2555.0,
                buy_shares: 5018,
                sell_shares: 8,
            },
            BrokerTradeRecord {
                trade_date: "2026-10-08".to_string(),
                symbol: "2317".to_string(),
                broker_id: "9A00".to_string(),
                broker_name: "永豐金".to_string(),
                price: 200.0,
                buy_shares: 10000,
                sell_shares: 2000,
            },
        ];

        let count = repo.upsert_broker_trades_batch(&records).unwrap();
        assert_eq!(count, 3);

        // 測試重複寫入冪等性
        let count_again = repo.upsert_broker_trades_batch(&records).unwrap();
        assert_eq!(count_again, 3);

        // 查詢 2330 交易日明細
        let trades_2330 = repo.get_broker_trades("2330", "2026-10-08").unwrap();
        assert_eq!(trades_2330.len(), 2);
        assert_eq!(trades_2330[0].broker_id, "1020");
        assert_eq!(trades_2330[0].price, 2550.0);
        assert_eq!(trades_2330[1].price, 2555.0);

        // 查詢已完成代號集合
        let completed = repo.get_broker_completed_symbols("2026-10-08").unwrap();
        assert_eq!(completed.len(), 2);
        assert!(completed.contains("2330"));
        assert!(completed.contains("2317"));
        assert!(!completed.contains("2454"));

        // 測試買賣超彙總
        let summary_2330 = repo.get_broker_summary("2330", "2026-10-08").unwrap();
        assert_eq!(summary_2330.len(), 1);
        assert_eq!(summary_2330[0].broker_id, "1020");
        assert_eq!(summary_2330[0].total_buy, 13018);
        assert_eq!(summary_2330[0].total_sell, 8);
        assert_eq!(summary_2330[0].net_shares, 13010);
    }
}
