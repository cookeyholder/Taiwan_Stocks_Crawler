pub mod bollinger;
pub mod engine;

pub use crate::models::{BacktestOutput, EngineError, Signal, SignalRecord};
pub use engine::BacktestEngine;
use crate::models::OhlcvRecord;

pub trait Strategy: Send + Sync {
    fn id(&self) -> &str;
    fn min_history(&self) -> usize;
    fn evaluate(&self, ohlcv: &[OhlcvRecord], date_index: usize) -> Signal;
}

