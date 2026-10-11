pub mod bollinger;
pub mod engine;
pub mod ma_alignment;
pub mod ma_cross;
pub mod new_high;
pub mod performance;
pub mod portfolio;
pub mod rsi;
pub mod strategies;

use crate::models::OhlcvRecord;
pub use crate::models::{BacktestOutput, EngineError, Signal, SignalRecord, TradeRecord};
pub use engine::BacktestEngine;
pub use strategies::StrategyKind;

pub trait Strategy: Send + Sync {
    fn id(&self) -> &str;
    fn min_history(&self) -> usize;
    fn evaluate(&self, ohlcv: &[OhlcvRecord], date_index: usize) -> Signal;
}
