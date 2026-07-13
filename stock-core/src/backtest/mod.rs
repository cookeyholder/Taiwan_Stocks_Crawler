pub mod bollinger;
pub mod engine;

use crate::models::OhlcvRecord;
pub use crate::models::{BacktestOutput, EngineError, Signal, SignalRecord};
pub use engine::BacktestEngine;

pub trait Strategy: Send + Sync {
    fn id(&self) -> &str;
    fn min_history(&self) -> usize;
    fn evaluate(&self, ohlcv: &[OhlcvRecord], date_index: usize) -> Signal;
}
