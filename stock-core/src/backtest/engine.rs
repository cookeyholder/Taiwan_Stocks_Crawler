use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;

use crate::backtest::{BacktestOutput, EngineError, SignalRecord, Strategy};
use crate::db::MarketRepository;
use crate::models::OhlcvRecord;

pub struct BacktestEngine<'a> {
    repo: &'a MarketRepository,
}

impl<'a> BacktestEngine<'a> {
    pub fn new(repo: &'a MarketRepository) -> Self {
        Self { repo }
    }

    pub fn run_date(
        &self,
        date: &str,
        strategy: &dyn Strategy,
        instrument_types: Option<&[String]>,
    ) -> Result<BacktestOutput, EngineError> {
        let _ = NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .map_err(|_| EngineError::InvalidDate(date.to_string()))?;

        let effective_date = match self.repo.get_effective_trade_date(date)? {
            Some(d) => d,
            None => return Ok(BacktestOutput { signals: vec![] }),
        };

        let required_days = strategy.min_history();
        let mut dates = self.repo.get_last_n_trade_dates(&effective_date, required_days)?;
        if dates.len() < required_days {
            return Ok(BacktestOutput { signals: vec![] });
        }
        dates.reverse();
        let start_date = dates[0].clone();
        let end_date = dates.last().cloned().unwrap_or_default();

        let symbols = self.repo.get_active_symbols(&start_date, instrument_types)?;
        let rows = self.repo.get_batch_ohlcv(&start_date, &end_date)?;
        let mut ohlcv_map: HashMap<String, Vec<OhlcvRecord>> = HashMap::new();
        for (sym, record) in rows {
            ohlcv_map.entry(sym).or_default().push(record);
        }
        let name_map = self.repo.get_name_map(&end_date)?;

        let mut signals = Vec::new();
        let target_date = &effective_date;
        for symbol in &symbols {
            let data = match ohlcv_map.get(symbol) {
                Some(d) if d.len() >= strategy.min_history() => d,
                _ => continue,
            };
            let date_index = data.len() - 1;
            if data[date_index].trade_date != *target_date {
                continue;
            }
            let signal = strategy.evaluate(data, date_index);
            if signal.has_signal {
                let last = &data[date_index];
                signals.push(SignalRecord {
                    trade_date: target_date.clone(),
                    symbol: symbol.clone(),
                    name: name_map.get(symbol).and_then(|n| n.clone()),
                    close_price: last.close_price,
                    volume: last.volume_shares,
                    direction: signal.direction,
                    strength: signal.strength,
                    strategy: strategy.id().to_string(),
                    metadata: signal.metadata,
                });
            }
        }
        Ok(BacktestOutput { signals })
    }

    pub fn run_range(
        &self,
        start: &str,
        end: &str,
        strategy: &dyn Strategy,
        instrument_types: Option<&[String]>,
    ) -> Result<BacktestOutput, EngineError> {
        let start_parsed = NaiveDate::parse_from_str(start, "%Y-%m-%d")
            .map_err(|_| EngineError::InvalidDate(start.to_string()))?;
        let end_parsed = NaiveDate::parse_from_str(end, "%Y-%m-%d")
            .map_err(|_| EngineError::InvalidDate(end.to_string()))?;
        if start_parsed > end_parsed {
            return Err(EngineError::InvalidDate(format!(
                "start {} after end {}",
                start, end
            )));
        }

        let range_dates = self.repo.get_trade_dates_in_range(start, end)?;
        if range_dates.is_empty() {
            return Ok(BacktestOutput { signals: vec![] });
        }
        let data_start = range_dates[0].clone();
        let data_end = range_dates.last().unwrap().clone();

        let mut warmup_dates = self
            .repo
            .get_last_n_trade_dates(&data_start, strategy.min_history())?;
        warmup_dates.reverse();
        let full_start = warmup_dates
            .first()
            .cloned()
            .unwrap_or(data_start.clone());

        let symbols = self.repo.get_active_symbols(&data_start, instrument_types)?;
        let rows = self.repo.get_batch_ohlcv(&full_start, &data_end)?;
        let mut ohlcv_map: HashMap<String, Vec<OhlcvRecord>> = HashMap::new();
        for (sym, record) in rows {
            ohlcv_map.entry(sym).or_default().push(record);
        }
        let name_map = self.repo.get_name_map(&data_end)?;

        let date_set: HashSet<&str> = range_dates.iter().map(|s| s.as_str()).collect();

        let mut signals = Vec::new();
        for symbol in &symbols {
            let data = match ohlcv_map.get(symbol) {
                Some(d) if d.len() >= strategy.min_history() => d,
                _ => continue,
            };
            for i in (strategy.min_history() - 1)..data.len() {
                let trade_date = &data[i].trade_date;
                if !date_set.contains(trade_date.as_str()) {
                    continue;
                }
                let signal = strategy.evaluate(data, i);
                if signal.has_signal {
                    let last = &data[i];
                    signals.push(SignalRecord {
                        trade_date: trade_date.clone(),
                        symbol: symbol.clone(),
                        name: name_map.get(symbol).and_then(|n| n.clone()),
                        close_price: last.close_price,
                        volume: last.volume_shares,
                        direction: signal.direction,
                        strength: signal.strength,
                        strategy: strategy.id().to_string(),
                        metadata: signal.metadata,
                    });
                }
            }
        }
        Ok(BacktestOutput { signals })
    }
}
