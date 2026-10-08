use super::Signal;
use super::Strategy;
use crate::backtest::bollinger::BollingerSqueezeBreakout;
use crate::backtest::ma_cross::MaCrossStrategy;
use crate::backtest::rsi::RsiStrategy;
use crate::models::{OhlcvRecord, ParamDef, StrategyInfo};

pub enum StrategyKind {
    BollingerSqueeze(BollingerSqueezeBreakout),
    MaCross(MaCrossStrategy),
    Rsi(RsiStrategy),
}

impl Strategy for StrategyKind {
    fn id(&self) -> &str {
        match self {
            StrategyKind::BollingerSqueeze(_) => "bollinger_squeeze",
            StrategyKind::MaCross(_) => "ma_cross",
            StrategyKind::Rsi(_) => "rsi",
        }
    }

    fn min_history(&self) -> usize {
        match self {
            StrategyKind::BollingerSqueeze(s) => s.min_history(),
            StrategyKind::MaCross(s) => s.min_history(),
            StrategyKind::Rsi(s) => s.min_history(),
        }
    }

    fn evaluate(&self, ohlcv: &[OhlcvRecord], date_index: usize) -> Signal {
        match self {
            StrategyKind::BollingerSqueeze(s) => s.evaluate(ohlcv, date_index),
            StrategyKind::MaCross(s) => s.evaluate(ohlcv, date_index),
            StrategyKind::Rsi(s) => s.evaluate(ohlcv, date_index),
        }
    }
}

impl StrategyKind {
    pub fn list() -> Vec<StrategyInfo> {
        vec![
            StrategyInfo {
                id: "bollinger_squeeze".into(),
                name: "布林擠壓突破".into(),
                params: vec![
                    ParamDef {
                        name: "bandwidth_limit".into(),
                        param_type: "float".into(),
                        default: 0.08,
                        min: 0.0,
                        max: 1.0,
                    },
                    ParamDef {
                        name: "consecutive_days".into(),
                        param_type: "int".into(),
                        default: 5.0,
                        min: 1.0,
                        max: 100.0,
                    },
                    ParamDef {
                        name: "min_volume".into(),
                        param_type: "int".into(),
                        default: 0.0,
                        min: 0.0,
                        max: 1e9,
                    },
                ],
            },
            StrategyInfo {
                id: "ma_cross".into(),
                name: "移動平均線交叉".into(),
                params: vec![
                    ParamDef {
                        name: "fast_period".into(),
                        param_type: "int".into(),
                        default: 5.0,
                        min: 2.0,
                        max: 100.0,
                    },
                    ParamDef {
                        name: "slow_period".into(),
                        param_type: "int".into(),
                        default: 20.0,
                        min: 5.0,
                        max: 200.0,
                    },
                    ParamDef {
                        name: "min_volume".into(),
                        param_type: "int".into(),
                        default: 0.0,
                        min: 0.0,
                        max: 1e9,
                    },
                ],
            },
            StrategyInfo {
                id: "rsi".into(),
                name: "RSI 超買超賣".into(),
                params: vec![
                    ParamDef {
                        name: "period".into(),
                        param_type: "int".into(),
                        default: 14.0,
                        min: 2.0,
                        max: 100.0,
                    },
                    ParamDef {
                        name: "overbought".into(),
                        param_type: "float".into(),
                        default: 70.0,
                        min: 50.0,
                        max: 100.0,
                    },
                    ParamDef {
                        name: "oversold".into(),
                        param_type: "float".into(),
                        default: 30.0,
                        min: 0.0,
                        max: 50.0,
                    },
                    ParamDef {
                        name: "min_volume".into(),
                        param_type: "int".into(),
                        default: 0.0,
                        min: 0.0,
                        max: 1e9,
                    },
                ],
            },
        ]
    }

    pub fn create(id: &str, params: &[(String, f64)]) -> Result<Self, String> {
        let get = |name: &str, default: f64| -> f64 {
            params
                .iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| *v)
                .unwrap_or(default)
        };
        match id {
            "bollinger_squeeze" => Ok(StrategyKind::BollingerSqueeze(BollingerSqueezeBreakout {
                bandwidth_limit: get("bandwidth_limit", 0.08),
                consecutive_days: get("consecutive_days", 5.0) as usize,
                min_volume: get("min_volume", 0.0) as i64,
                require_break_upper: get("require_break_upper", 0.0) > 0.5,
            })),
            "ma_cross" => Ok(StrategyKind::MaCross(MaCrossStrategy {
                fast_period: get("fast_period", 5.0) as usize,
                slow_period: get("slow_period", 20.0) as usize,
                min_volume: get("min_volume", 0.0) as i64,
            })),
            "rsi" => Ok(StrategyKind::Rsi(RsiStrategy {
                period: get("period", 14.0) as usize,
                overbought: get("overbought", 70.0),
                oversold: get("oversold", 30.0),
                min_volume: get("min_volume", 0.0) as i64,
            })),
            _ => Err(format!("unknown strategy: {}", id)),
        }
    }
}
