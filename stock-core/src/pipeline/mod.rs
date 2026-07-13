use std::sync::atomic::AtomicBool;

use crate::db::MarketRepository;
use crate::fetcher::{tpex, twse};
use crate::models::*;
use crate::{date_utils, fetcher::HttpClient};

use chrono::NaiveDate;

pub struct Pipeline {
    pub http: HttpClient,
    pub repo: MarketRepository,
    pub cancel_flag: Option<AtomicBool>,
}

impl Pipeline {
    pub fn new(http: HttpClient, repo: MarketRepository) -> Self {
        Self {
            http,
            repo,
            cancel_flag: None,
        }
    }

    pub async fn fetch_today(&self) -> Result<IngestionOutcome, PipelineError> {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        self.fetch_date(&today).await
    }

    pub async fn fetch_date(&self, date_str: &str) -> Result<IngestionOutcome, PipelineError> {
        let mut outcome = IngestionOutcome {
            day_type: DayType::Unknown,
            ok: false,
            message: String::new(),
            records_inserted: 0,
        };

        let mut total = 0;
        let ingest_run_id = uuid();
        let ingested_at = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();

        let is_today = date_str == chrono::Local::now().format("%Y-%m-%d").to_string();

        // TWSE: today JSON or history CSV
        let mut twse_result = if is_today {
            twse::fetch_today_json(&self.http).await
        } else {
            twse::fetch_history_csv(&self.http, date_str).await
        };

        if is_today
            && let Some(parsed_date) = twse_result
                .as_ref()
                .ok()
                .and_then(|records| records.first())
                .and_then(|first| first.source_date.as_ref())
                .and_then(|s_date| date_utils::roc_date_compact_to_ad(s_date))
            && parsed_date != date_str
        {
            eprintln!(
                "[pipeline] TWSE OpenAPI date {} does not match requested date {}. Fallback to CSV.",
                parsed_date, date_str
            );
            twse_result = twse::fetch_history_csv(&self.http, date_str).await;
        }

        match twse_result {
            Ok(records) if !records.is_empty() => {
                let quotes = normalize_records(
                    "TWSE",
                    date_str,
                    &records,
                    "utf-8",
                    &ingest_run_id,
                    &ingested_at,
                );
                total += self.repo.upsert_core_batch(&quotes)?;
            }
            Ok(_) => {}
            Err(e) => {
                outcome.message = format!("TWSE: {}", e);
            }
        }

        // TPEX daily JSON (works for both today and history)
        match tpex::fetch_daily_json(&self.http, date_str).await {
            Ok(records) if !records.is_empty() => {
                let quotes = normalize_records(
                    "TPEX",
                    date_str,
                    &records,
                    "utf-8",
                    &ingest_run_id,
                    &ingested_at,
                );
                total += self.repo.upsert_core_batch(&quotes)?;
            }
            Ok(_) => {}
            Err(e) => {
                if !outcome.message.is_empty() {
                    outcome.message.push_str("; ");
                }
                outcome.message.push_str(&format!("TPEX: {}", e));
            }
        }

        if total > 0 {
            outcome.day_type = DayType::Trading;
            outcome.ok = true;
            outcome.message = format!("OK: {} records", total);
        } else {
            outcome.day_type = DayType::MarketClosed;
            if outcome.message.is_empty() {
                outcome.message = "no data (market closed or error)".into();
            }
        }
        outcome.records_inserted = total;
        Ok(outcome)
    }

    pub async fn screen(
        &self,
        bandwidth_limit: f64,
        consecutive_days: i32,
        instrument_types: Option<Vec<String>>,
        min_volume: i64,
    ) -> Result<Vec<ScreenResult>, PipelineError> {
        use crate::backtest::bollinger::BollingerSqueezeBreakout;
        use crate::backtest::BacktestEngine;

        if consecutive_days <= 0 {
            return Err(PipelineError::Internal("consecutive_days must be > 0".into()));
        }
        let date = match self.repo.get_latest_trade_date()? {
            Some(d) => d,
            None => return Ok(vec![]),
        };
        let engine = BacktestEngine::new(&self.repo);
        let output = engine
            .run_date(
                &date,
                &BollingerSqueezeBreakout {
                    bandwidth_limit,
                    consecutive_days: consecutive_days as usize,
                    min_volume,
                },
                instrument_types.as_deref(),
            )
            .map_err(|e| PipelineError::Internal(e.to_string()))?;
        Ok(output.signals.iter().map(|s| s.to_screen_result()).collect())
    }

    pub async fn get_analysis(
        &self,
        symbol: &str,
        ma_periods: &[usize],
        rsi_period: Option<usize>,
        macd_fast: Option<usize>,
        macd_slow: Option<usize>,
        macd_signal: Option<usize>,
    ) -> Result<AnalysisOutput, PipelineError> {
        eprintln!(
            "[pipeline::get_analysis] symbol={}, ma_periods={:?}",
            symbol, ma_periods
        );
        let end = match self.repo.get_latest_trade_date() {
            Ok(Some(d)) => d,
            _ => {
                let tz_offset = chrono::FixedOffset::east_opt(8 * 3600).unwrap();
                let now_taiwan = chrono::Utc::now().with_timezone(&tz_offset);
                now_taiwan.format("%Y-%m-%d").to_string()
            }
        };
        let start = {
            let d = NaiveDate::parse_from_str(&end, "%Y-%m-%d")
                .map_err(|e| PipelineError::Internal(e.to_string()))?;
            let start = d - chrono::Duration::days(365 * 2);
            start.format("%Y-%m-%d").to_string()
        };
        eprintln!("[pipeline::get_analysis] date range: {} -> {}", start, end);
        let ohlcv = self.repo.get_ohlcv(symbol, &start, &end)?;
        eprintln!(
            "[pipeline::get_analysis] got {} OHLCV rows from DB",
            ohlcv.len()
        );
        if ohlcv.is_empty() {
            return Err(PipelineError::NoData(symbol.to_string()));
        }
        let prices: Vec<f64> = ohlcv.iter().filter_map(|r| r.close_price).collect();
        let mut output = AnalysisOutput {
            ohlcv,
            ma_lines: Vec::new(),
            bollinger: None,
            rsi: None,
            macd: None,
        };
        if !ma_periods.is_empty() {
            for &p in ma_periods {
                let line = crate::analysis::sma(&prices, p);
                output.ma_lines.push(line);
            }
        }
        if prices.len() >= 20 {
            output.bollinger = Some(crate::analysis::bollinger(&prices, 20, 2.0));
        }
        if let Some(period) = rsi_period
            && prices.len() > period
        {
            output.rsi = Some(crate::analysis::rsi(&prices, period));
        }
        if let (Some(f), Some(s), Some(sig)) = (macd_fast, macd_slow, macd_signal)
            && prices.len() > s
        {
            output.macd = Some(crate::analysis::macd(&prices, f, s, sig));
        }
        eprintln!(
            "[pipeline::get_analysis] DONE: {} prices, {} ma_lines, bollinger={}, rsi={}, macd={}",
            prices.len(),
            output.ma_lines.len(),
            output.bollinger.is_some(),
            output.rsi.is_some(),
            output.macd.is_some()
        );
        Ok(output)
    }
}

fn normalize_records(
    market_code: &str,
    trade_date: &str,
    records: &[ParsedRecord],
    encoding: &str,
    ingest_run_id: &str,
    ingested_at: &str,
) -> Vec<CanonicalQuote> {
    records
        .iter()
        .map(|r| {
            let symbol_clean = r.symbol.trim_matches('=').trim_matches('"').to_string();
            let td = r
                .source_date
                .as_deref()
                .filter(|s| s.len() == 7 && s.bytes().all(|b| b.is_ascii_digit()))
                .and_then(date_utils::roc_date_compact_to_ad)
                .or_else(|| date_utils::roc_to_ad_date(trade_date))
                .unwrap_or_else(|| trade_date.to_string());
            let (change_sign, change_value) = parse_change_field(r.change.as_deref());
            CanonicalQuote {
                trade_date: td,
                market_code: market_code.to_string(),
                symbol_raw: r.symbol.clone(),
                symbol_clean,
                security_name: r.name.clone(),
                instrument_type: "other".to_string(),
                instrument_subtype: None,
                isin_code: None,
                cfi_code: None,
                section_label: None,
                open_price: parse_float(r.open.as_deref()),
                high_price: parse_float(r.high.as_deref()),
                low_price: parse_float(r.low.as_deref()),
                close_price: parse_float(r.close.as_deref()),
                volume_shares: parse_int(r.volume.as_deref()),
                turnover_value: parse_float(r.turnover.as_deref()),
                trade_count: parse_int(r.trade_count.as_deref()),
                change_sign,
                change_value,
                last_bid_price: parse_float(r.last_bid.as_deref()),
                last_bid_qty_raw: r.last_bid_qty.clone(),
                last_ask_price: parse_float(r.last_ask.as_deref()),
                last_ask_qty_raw: r.last_ask_qty.clone(),
                pe_ratio: parse_float(r.pe_ratio.as_deref()),
                issued_shares: parse_int(r.issued_shares.as_deref()),
                next_limit_up: parse_float(r.next_limit_up.as_deref()),
                next_limit_down: parse_float(r.next_limit_down.as_deref()),
                source_file: r.source_file.clone(),
                source_encoding: encoding.to_string(),
                source_line_no: r.source_line_no,
                ingest_run_id: ingest_run_id.to_string(),
                ingested_at: ingested_at.to_string(),
                extension: std::collections::HashMap::new(),
            }
        })
        .collect()
}

fn parse_float(s: Option<&str>) -> Option<f64> {
    let s = s?.trim();
    if s.is_empty() || s == "--" || s == "---" {
        return None;
    }
    let cleaned = s.replace(",", "");
    cleaned.parse::<f64>().ok()
}

fn parse_int(s: Option<&str>) -> Option<i64> {
    let s = s?.trim();
    if s.is_empty() || s == "--" || s == "---" {
        return None;
    }
    let cleaned = s.replace(",", "");
    cleaned.parse::<i64>().ok()
}

fn parse_change_field(change: Option<&str>) -> (Option<String>, Option<f64>) {
    let s = match change {
        Some(s) if !s.trim().is_empty() && s.trim() != "--" => s.trim(),
        _ => return (None, None),
    };
    let chars: Vec<char> = s.chars().collect();
    if chars.len() >= 2 && (chars[0] == '+' || chars[0] == '-') {
        let magnitude: String = chars[1..].iter().collect();
        (Some(chars[0].to_string()), magnitude.parse::<f64>().ok())
    } else if chars.len() >= 2 && (chars[0] == '＋' || chars[0] == '－') {
        let sign = if chars[0] == '＋' { "+" } else { "-" };
        let magnitude: String = chars[1..].iter().collect();
        (Some(sign.to_string()), magnitude.parse::<f64>().ok())
    } else {
        (None, s.parse::<f64>().ok())
    }
}

fn uuid() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:x}", nanos)
}

#[derive(Debug)]
pub struct IngestionOutcome {
    pub day_type: DayType,
    pub ok: bool,
    pub message: String,
    pub records_inserted: usize,
}

#[derive(Debug)]
pub enum DayType {
    Unknown,
    Trading,
    MarketClosed,
}

#[derive(Debug)]
pub struct AnalysisOutput {
    pub ohlcv: Vec<OhlcvRecord>,
    pub ma_lines: Vec<Vec<Option<f64>>>,
    pub bollinger: Option<crate::models::BollingerResult>,
    pub rsi: Option<crate::models::RsiResult>,
    pub macd: Option<crate::models::MacdResult>,
}

#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("DB error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] crate::fetcher::HttpError),
    #[error("No data for {0}")]
    NoData(String),
    #[error("Internal: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_float_normal() {
        assert_eq!(parse_float(Some("30.20")), Some(30.20));
    }

    #[test]
    fn test_parse_float_with_commas() {
        assert_eq!(parse_float(Some("10,053,434")), Some(10053434.0));
    }

    #[test]
    fn test_parse_float_empty_or_placeholder() {
        assert_eq!(parse_float(Some("")), None);
        assert_eq!(parse_float(Some("--")), None);
        assert_eq!(parse_float(Some("---")), None);
        assert_eq!(parse_float(None), None);
    }

    #[test]
    fn test_parse_int_normal() {
        assert_eq!(parse_int(Some("10053434")), Some(10053434));
    }

    #[test]
    fn test_parse_int_with_commas() {
        assert_eq!(parse_int(Some("10,053,434")), Some(10053434));
    }

    #[test]
    fn test_parse_int_empty_or_placeholder() {
        assert_eq!(parse_int(Some("")), None);
        assert_eq!(parse_int(Some("--")), None);
        assert_eq!(parse_int(None), None);
    }

    #[test]
    fn test_parse_change_field_ascii_plus_minus() {
        assert_eq!(
            parse_change_field(Some("+0.10")),
            (Some("+".into()), Some(0.10))
        );
        assert_eq!(
            parse_change_field(Some("-5.0")),
            (Some("-".into()), Some(5.0))
        );
    }

    #[test]
    fn test_parse_change_field_fullwidth() {
        assert_eq!(
            parse_change_field(Some("＋0.10")),
            (Some("+".into()), Some(0.10))
        );
        assert_eq!(
            parse_change_field(Some("－5.0")),
            (Some("-".into()), Some(5.0))
        );
    }

    #[test]
    fn test_parse_change_field_plain_number() {
        assert_eq!(parse_change_field(Some("0.10")), (None, Some(0.10)));
    }

    #[test]
    fn test_parse_change_field_empty_or_placeholder() {
        assert_eq!(parse_change_field(Some("")), (None, None));
        assert_eq!(parse_change_field(Some("--")), (None, None));
        assert_eq!(parse_change_field(None), (None, None));
    }

    #[test]
    fn test_normalize_records_sets_market_code() {
        let records = vec![ParsedRecord {
            symbol: "1101".into(),
            name: Some("台泥".into()),
            open: Some("30.00".into()),
            high: Some("30.50".into()),
            low: Some("29.80".into()),
            close: Some("30.20".into()),
            volume: Some("10000000".into()),
            turnover: Some("300000000".into()),
            trade_count: Some("5000".into()),
            change: None,
            change_sign: None,
            change_value: None,
            last_bid: None,
            last_bid_qty: None,
            last_ask: None,
            last_ask_qty: None,
            pe_ratio: None,
            issued_shares: None,
            next_limit_up: None,
            next_limit_down: None,
            source_file: "test".into(),
            source_line_no: 1,
            source_date: None,
        }];
        let quotes = normalize_records(
            "TWSE",
            "2024-01-02",
            &records,
            "utf-8",
            "test-ingest",
            "2024-01-01T00:00:00",
        );
        assert_eq!(quotes.len(), 1);
        assert_eq!(quotes[0].market_code, "TWSE");
        assert_eq!(quotes[0].symbol_clean, "1101");
        assert_eq!(quotes[0].close_price, Some(30.20));
        assert_eq!(quotes[0].volume_shares, Some(10000000));
    }

    #[test]
    fn test_normalize_records_uses_source_date_when_available() {
        let records = vec![ParsedRecord {
            symbol: "1101".into(),
            name: Some("台泥".into()),
            open: Some("30.00".into()),
            high: Some("30.50".into()),
            low: Some("29.80".into()),
            close: Some("30.20".into()),
            volume: Some("10000000".into()),
            turnover: Some("300000000".into()),
            trade_count: Some("5000".into()),
            change: None,
            change_sign: None,
            change_value: None,
            last_bid: None,
            last_bid_qty: None,
            last_ask: None,
            last_ask_qty: None,
            pe_ratio: None,
            issued_shares: None,
            next_limit_up: None,
            next_limit_down: None,
            source_file: "test".into(),
            source_line_no: 1,
            source_date: Some("1150610".into()),
        }];
        let quotes = normalize_records(
            "TWSE",
            "2026-06-11",
            &records,
            "utf-8",
            "test-ingest",
            "2024-01-01T00:00:00",
        );
        assert_eq!(quotes.len(), 1);
        assert_eq!(quotes[0].trade_date, "2026-06-10");
    }

    #[test]
    fn test_normalize_records_strips_equals_prefix() {
        let records = vec![ParsedRecord {
            symbol: "=1101".into(),
            name: Some("台泥".into()),
            open: None,
            high: None,
            low: None,
            close: None,
            volume: None,
            turnover: None,
            trade_count: None,
            change: None,
            change_sign: None,
            change_value: None,
            last_bid: None,
            last_bid_qty: None,
            last_ask: None,
            last_ask_qty: None,
            pe_ratio: None,
            issued_shares: None,
            next_limit_up: None,
            next_limit_down: None,
            source_file: "test".into(),
            source_line_no: 1,
            source_date: None,
        }];
        let quotes = normalize_records(
            "TWSE",
            "2024-01-02",
            &records,
            "utf-8",
            "test-ingest",
            "2024-01-01T00:00:00",
        );
        assert_eq!(quotes[0].symbol_clean, "1101");
    }
}
