use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalQuote {
    pub trade_date: String,
    pub market_code: String,
    pub symbol_raw: String,
    pub symbol_clean: String,
    pub security_name: Option<String>,
    pub instrument_type: String,
    pub instrument_subtype: Option<String>,
    pub isin_code: Option<String>,
    pub cfi_code: Option<String>,
    pub section_label: Option<String>,
    pub open_price: Option<f64>,
    pub high_price: Option<f64>,
    pub low_price: Option<f64>,
    pub close_price: Option<f64>,
    pub volume_shares: Option<i64>,
    pub turnover_value: Option<f64>,
    pub trade_count: Option<i64>,
    pub change_sign: Option<String>,
    pub change_value: Option<f64>,
    pub last_bid_price: Option<f64>,
    pub last_bid_qty_raw: Option<String>,
    pub last_ask_price: Option<f64>,
    pub last_ask_qty_raw: Option<String>,
    pub pe_ratio: Option<f64>,
    pub issued_shares: Option<i64>,
    pub next_limit_up: Option<f64>,
    pub next_limit_down: Option<f64>,
    pub source_file: String,
    pub source_encoding: String,
    pub source_line_no: i64,
    pub ingest_run_id: String,
    pub ingested_at: String,
    pub extension: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedRecord {
    pub symbol: String,
    pub name: Option<String>,
    pub open: Option<String>,
    pub high: Option<String>,
    pub low: Option<String>,
    pub close: Option<String>,
    pub volume: Option<String>,
    pub turnover: Option<String>,
    pub trade_count: Option<String>,
    pub change: Option<String>,
    pub change_sign: Option<String>,
    pub change_value: Option<String>,
    pub last_bid: Option<String>,
    pub last_bid_qty: Option<String>,
    pub last_ask: Option<String>,
    pub last_ask_qty: Option<String>,
    pub pe_ratio: Option<String>,
    pub issued_shares: Option<String>,
    pub next_limit_up: Option<String>,
    pub next_limit_down: Option<String>,
    pub source_file: String,
    pub source_line_no: i64,
    pub source_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenResult {
    pub symbol: String,
    pub name: Option<String>,
    pub latest_close: Option<f64>,
    pub latest_volume: Option<i64>,
    pub max_bandwidth: f64,
    pub latest_bandwidth: f64,
    pub latest_upper_band: Option<f64>,
    pub break_upper: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub has_signal: bool,
    pub direction: i8,
    pub strength: f64,
    pub metadata: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalRecord {
    pub trade_date: String,
    pub symbol: String,
    pub name: Option<String>,
    pub close_price: Option<f64>,
    pub volume: Option<i64>,
    pub direction: i8,
    pub strength: f64,
    pub strategy: String,
    pub metadata: HashMap<String, f64>,
}

impl SignalRecord {
    pub fn to_screen_result(&self) -> ScreenResult {
        let get_f64 = |key: &str| self.metadata.get(key).copied().unwrap_or(0.0);
        ScreenResult {
            symbol: self.symbol.clone(),
            name: self.name.clone(),
            latest_close: self.close_price,
            latest_volume: self.volume,
            max_bandwidth: get_f64("max_bandwidth"),
            latest_bandwidth: get_f64("bandwidth"),
            latest_upper_band: self.metadata.get("upper_band").copied(),
            break_upper: self.metadata.get("break_upper").copied().unwrap_or(0.0) > 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestOutput {
    pub signals: Vec<SignalRecord>,
}

#[derive(Debug)]
pub enum EngineError {
    Db(rusqlite::Error),
    InvalidDate(String),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::Db(e) => write!(f, "DB error: {}", e),
            EngineError::InvalidDate(d) => write!(f, "invalid date: {}", d),
        }
    }
}

impl std::error::Error for EngineError {}

impl From<rusqlite::Error> for EngineError {
    fn from(e: rusqlite::Error) -> Self {
        EngineError::Db(e)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OhlcvRecord {
    pub trade_date: String,
    pub open_price: Option<f64>,
    pub high_price: Option<f64>,
    pub low_price: Option<f64>,
    pub close_price: Option<f64>,
    pub volume_shares: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeQuote {
    pub symbol: String,
    pub name: Option<String>,
    pub price: Option<f64>,
    pub change: Option<f64>,
    pub change_percent: Option<f64>,
    pub high: Option<f64>,
    pub low: Option<f64>,
    pub volume: Option<i64>,
    pub bids: Vec<BidAsk>,
    pub asks: Vec<BidAsk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidAsk {
    pub price: f64,
    pub volume: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BollingerResult {
    pub upper: Vec<Option<f64>>,
    pub middle: Vec<Option<f64>>,
    pub lower: Vec<Option<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsiResult {
    pub values: Vec<Option<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacdResult {
    pub macd_line: Vec<Option<f64>>,
    pub signal_line: Vec<Option<f64>>,
    pub histogram: Vec<Option<f64>>,
}
