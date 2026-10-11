use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};

pub static TOKIO_RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("初始化 Tokio 執行期失敗")
});

use stock_core::backtest::bollinger::BollingerSqueezeBreakout;
use stock_core::backtest::engine::BacktestEngine;
use stock_core::backtest::performance::calculate_metrics;
use stock_core::backtest::portfolio::PortfolioSimulator;
use stock_core::backtest::strategies::StrategyKind;
use stock_core::db::MarketRepository;
use stock_core::fetcher::{HttpClient, HttpPolicy};
use stock_core::models::*;
use stock_core::pipeline::Pipeline;

pub fn default_db_path() -> PathBuf {
    if let Ok(dir) = std::env::var("STOCK_DATA_DIR") {
        let path = PathBuf::from(dir);
        let _ = std::fs::create_dir_all(&path);
        return path.join("market_data.db");
    }
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("stock-screener");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("market_data.db")
}

#[derive(Clone, Debug)]
pub struct AnalysisData {
    pub symbol: String,
    pub name: Option<String>,
    pub ohlcv: Vec<OhlcvRecord>,
    pub bollinger_upper: Vec<Option<f64>>,
    pub bollinger_middle: Vec<Option<f64>>,
    pub bollinger_lower: Vec<Option<f64>>,
    pub ma5: Vec<Option<f64>>,
    pub ma10: Vec<Option<f64>>,
    pub ma20: Vec<Option<f64>>,
    pub rsi: Vec<Option<f64>>,
    pub macd_line: Vec<Option<f64>>,
    pub macd_signal: Vec<Option<f64>>,
    pub macd_histogram: Vec<Option<f64>>,
}

#[derive(Clone, Debug)]
pub struct SecurityItem {
    pub symbol: String,
    pub market: String,
    pub name: String,
}

#[derive(Clone)]
pub struct CoreState {
    pub pipeline: Arc<tokio::sync::Mutex<Pipeline>>,
    pub cancel_flag: Arc<AtomicBool>,
}

impl CoreState {
    pub fn new_with_db_path(db_path: &str) -> Result<Self, String> {
        let repo = MarketRepository::open(db_path).map_err(|e| e.to_string())?;
        let http = HttpClient::new(HttpPolicy::default());
        let pipeline = Pipeline::new(http, repo);
        Ok(Self {
            pipeline: Arc::new(tokio::sync::Mutex::new(pipeline)),
            cancel_flag: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn new() -> Result<Self, String> {
        let path = default_db_path();
        Self::new_with_db_path(path.to_str().unwrap_or("market_data.db"))
    }

    pub fn shutdown(&self) {
        self.cancel_flag.store(true, Ordering::SeqCst);
    }

    pub fn cancel(&self) {
        self.cancel_flag.store(true, Ordering::SeqCst);
    }

    pub async fn clear_screener_cache(&self) -> Result<usize, String> {
        let pipeline = self.pipeline.lock().await;
        pipeline
            .repo
            .clear_screener_cache()
            .map_err(|e| e.to_string())
    }

    pub async fn get_screener_cache_count(&self) -> Result<usize, String> {
        let pipeline = self.pipeline.lock().await;
        pipeline
            .repo
            .get_screener_cache_count()
            .map_err(|e| e.to_string())
    }

    pub async fn get_securities(&self) -> Result<Vec<SecurityItem>, String> {
        let pipeline = self.pipeline.lock().await;
        let rows = pipeline.repo.get_securities().map_err(|e| e.to_string())?;
        Ok(rows
            .into_iter()
            .map(|(symbol, market, name)| SecurityItem {
                symbol,
                market,
                name: name.unwrap_or_default(),
            })
            .collect())
    }

    pub async fn get_latest_trade_date(&self) -> Result<Option<String>, String> {
        let pipeline = self.pipeline.lock().await;
        pipeline
            .repo
            .get_latest_trade_date()
            .map_err(|e| e.to_string())
    }

    pub async fn get_date_range(&self) -> Result<(Option<String>, Option<String>, usize), String> {
        let pipeline = self.pipeline.lock().await;
        pipeline.repo.get_date_range().map_err(|e| e.to_string())
    }

    pub async fn delete_date_range(&self, start: &str, end: &str) -> Result<String, String> {
        let start_date = chrono::NaiveDate::parse_from_str(start, "%Y-%m-%d")
            .map_err(|_| format!("無效的起始日期格式：{}，請使用 YYYY-MM-DD", start))?;
        let end_date = chrono::NaiveDate::parse_from_str(end, "%Y-%m-%d")
            .map_err(|_| format!("無效的結束日期格式：{}，請使用 YYYY-MM-DD", end))?;
        if start_date > end_date {
            return Err("起始日期不可晚於結束日期".to_string());
        }
        let pipeline = self.pipeline.lock().await;
        let count = pipeline
            .repo
            .delete_date_range(start, end)
            .map_err(|e| e.to_string())?;
        let _ = pipeline.repo.sync_securities();
        Ok(format!("已刪除 {} 筆記錄（{} ~ {}）", count, start, end))
    }

    pub async fn get_analysis(
        &self,
        symbol: &str,
        ma_periods: &[usize],
        rsi_period: Option<usize>,
        macd_fast: Option<usize>,
        macd_slow: Option<usize>,
        macd_signal: Option<usize>,
    ) -> Result<AnalysisData, String> {
        let pipeline = self.pipeline.lock().await;
        let output = pipeline
            .get_analysis(
                symbol,
                ma_periods,
                rsi_period,
                macd_fast,
                macd_slow,
                macd_signal,
            )
            .await
            .map_err(|e| e.to_string())?;

        let name = pipeline.repo.get_symbol_name(symbol).ok().flatten();

        let empty = vec![];
        Ok(AnalysisData {
            symbol: symbol.to_string(),
            name,
            ohlcv: output.ohlcv,
            bollinger_upper: output
                .bollinger
                .as_ref()
                .map(|b| b.upper.clone())
                .unwrap_or(empty.clone()),
            bollinger_middle: output
                .bollinger
                .as_ref()
                .map(|b| b.middle.clone())
                .unwrap_or(empty.clone()),
            bollinger_lower: output
                .bollinger
                .as_ref()
                .map(|b| b.lower.clone())
                .unwrap_or(empty),
            ma5: output.ma_lines.first().cloned().unwrap_or_default(),
            ma10: output.ma_lines.get(1).cloned().unwrap_or_default(),
            ma20: output.ma_lines.get(2).cloned().unwrap_or_default(),
            rsi: output.rsi.map(|r| r.values).unwrap_or_default(),
            macd_line: output
                .macd
                .as_ref()
                .map(|m| m.macd_line.clone())
                .unwrap_or_default(),
            macd_signal: output
                .macd
                .as_ref()
                .map(|m| m.signal_line.clone())
                .unwrap_or_default(),
            macd_histogram: output
                .macd
                .as_ref()
                .map(|m| m.histogram.clone())
                .unwrap_or_default(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn screen_bollinger_range<F>(
        &self,
        start_date: &str,
        end_date: &str,
        max_bandwidth: f64,
        min_days: i32,
        instrument_types: Option<Vec<String>>,
        min_volume: i64,
        require_break_upper: bool,
        on_progress: F,
    ) -> Result<RangeScreenResult, String>
    where
        F: Fn(usize, usize, String, bool) + Send + Sync,
    {
        const MAX_TRADING_DAYS: usize = 120;

        let _start = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
            .map_err(|_| format!("日期格式錯誤：{}", start_date))?;
        let _end = chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d")
            .map_err(|_| format!("日期格式錯誤：{}", end_date))?;
        if start_date > end_date {
            return Err("起始日期不能晚於結束日期".into());
        }

        self.cancel_flag.store(false, Ordering::SeqCst);
        let pipeline = self.pipeline.lock().await;

        let trade_dates = pipeline
            .repo
            .get_trade_dates_in_range(start_date, end_date)
            .map_err(|e| e.to_string())?;

        if trade_dates.is_empty() {
            return Ok(RangeScreenResult {
                results: vec![],
                meta: RangeMeta {
                    start_date: start_date.to_string(),
                    end_date: end_date.to_string(),
                    trading_days: 0,
                    cached_days: 0,
                    calculated_days: 0,
                    failed_days: vec![],
                },
            });
        }

        if trade_dates.len() > MAX_TRADING_DAYS {
            return Err(format!(
                "區間超過上限：{} 個交易日（上限 {} 天）",
                trade_dates.len(),
                MAX_TRADING_DAYS
            ));
        }

        on_progress(1, 2, "正在批次載入歷史報價與計算技術指標...".into(), false);

        let strategy = BollingerSqueezeBreakout {
            bandwidth_limit: max_bandwidth,
            consecutive_days: min_days as usize,
            min_volume,
            require_break_upper,
        };

        let types_ref = instrument_types.as_deref();
        let engine = BacktestEngine::new(&pipeline.repo);

        let chunk_size = 10;
        let chunks: Vec<_> = trade_dates.chunks(chunk_size).collect();
        let total_chunks = chunks.len();
        let mut all_signals = Vec::new();

        for (idx, chunk) in chunks.iter().enumerate() {
            if self.cancel_flag.load(Ordering::SeqCst) {
                return Err("選股已被使用者取消".into());
            }
            let chunk_start = chunk.first().unwrap();
            let chunk_end = chunk.last().unwrap();
            on_progress(
                idx + 1,
                total_chunks + 1,
                format!("正在篩選區間 {} ~ {}...", chunk_start, chunk_end),
                false,
            );

            let output = engine
                .run_range(chunk_start, chunk_end, &strategy, types_ref)
                .map_err(|e| e.to_string())?;
            all_signals.extend(output.signals);
        }

        if self.cancel_flag.load(Ordering::SeqCst) {
            return Err("選股已被使用者取消".into());
        }

        let mut symbol_map: HashMap<String, (String, ScreenResult)> = HashMap::new();
        for signal in all_signals {
            let trade_date = signal.trade_date.clone();
            let item = signal.to_screen_result();
            match symbol_map.get_mut(&item.symbol) {
                Some((existing_date, existing_item)) => {
                    if trade_date >= *existing_date {
                        *existing_date = trade_date;
                        *existing_item = item;
                    }
                }
                None => {
                    symbol_map.insert(item.symbol.clone(), (trade_date, item));
                }
            }
        }

        let mut results: Vec<ScreenResult> =
            symbol_map.into_values().map(|(_, item)| item).collect();
        results.sort_by(|a, b| {
            a.latest_bandwidth
                .partial_cmp(&b.latest_bandwidth)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        on_progress(total_chunks + 1, total_chunks + 1, "完成".into(), false);

        Ok(RangeScreenResult {
            results,
            meta: RangeMeta {
                start_date: start_date.to_string(),
                end_date: end_date.to_string(),
                trading_days: trade_dates.len(),
                cached_days: 0,
                calculated_days: trade_dates.len(),
                failed_days: vec![],
            },
        })
    }

    pub async fn run_backtest(
        &self,
        strategy_id: &str,
        strategy_params: &[(String, f64)],
        start_date: &str,
        end_date: &str,
        capital: f64,
        instrument_types: Option<Vec<String>>,
    ) -> Result<BacktestReport, String> {
        let pipeline = self.pipeline.clone();
        let s_id = strategy_id.to_string();
        let s_params = strategy_params.to_vec();
        let start_d = start_date.to_string();
        let end_d = end_date.to_string();

        let handle = TOKIO_RUNTIME.spawn(async move {
            let pipeline = pipeline.lock().await;

            let strategy = StrategyKind::create(&s_id, &s_params)?;

            let trade_dates = pipeline
                .repo
                .get_trade_dates_in_range(&start_d, &end_d)
                .map_err(|e| e.to_string())?;

            let start = trade_dates.first().cloned().unwrap_or_default();
            let end = trade_dates.last().cloned().unwrap_or_default();

            let types_ref = instrument_types.as_deref();

            let output = BacktestEngine::new(&pipeline.repo)
                .run_range(&start_d, &end_d, &strategy, types_ref)
                .map_err(|e| e.to_string())?;

            let rows = pipeline
                .repo
                .get_batch_ohlcv(&start, &end)
                .map_err(|e| e.to_string())?;
            let mut ohlcv_map: HashMap<String, Vec<OhlcvRecord>> = HashMap::new();
            for (sym, record) in rows {
                ohlcv_map.entry(sym).or_default().push(record);
            }

            let (trades, equity_curve) =
                PortfolioSimulator::simulate(&output.signals, capital, &ohlcv_map, &trade_dates);
            let metrics = calculate_metrics(&trades, capital, &equity_curve);

            Ok(BacktestReport {
                signals: output.signals,
                metrics,
                trades,
                equity_curve,
                trade_dates,
            })
        });

        handle
            .await
            .map_err(|e| format!("執行回測工作失敗: {}", e))?
    }

    pub async fn batch_download<F>(
        &self,
        start_date: &str,
        end_date: &str,
        on_progress: F,
    ) -> Result<String, String>
    where
        F: Fn(usize, usize, String) + Send + Sync,
    {
        let start = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
            .map_err(|e| format!("無效的起始日期：{}", e))?;
        let end = chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d")
            .map_err(|e| format!("無效的結束日期：{}", e))?;

        if start > end {
            return Err("起始日期不可晚於結束日期".to_string());
        }

        let mut current = start;
        let total_days = (end - start).num_days() as usize + 1;
        let mut days_done = 0usize;
        let mut total_records = 0usize;

        self.cancel_flag.store(false, Ordering::SeqCst);

        while current <= end {
            if self.cancel_flag.load(Ordering::SeqCst) {
                break;
            }
            let date_str = current.format("%Y-%m-%d").to_string();
            on_progress(days_done, total_days, date_str.clone());

            let pipeline = self.pipeline.clone();
            let date_clone = date_str.clone();
            let fetch_task = TOKIO_RUNTIME.spawn(async move {
                let p = pipeline.lock().await;
                p.fetch_date(&date_clone).await
            });

            if let Ok(Ok(outcome)) = fetch_task.await {
                total_records += outcome.records_inserted;
            }

            days_done += 1;
            current += chrono::Duration::days(1);
        }

        if total_records > 0 {
            let pipeline = self.pipeline.clone();
            let _ = TOKIO_RUNTIME
                .spawn(async move {
                    let p = pipeline.lock().await;
                    p.repo.sync_securities()
                })
                .await;
        }

        if self.cancel_flag.load(Ordering::SeqCst) {
            on_progress(
                days_done,
                total_days,
                format!("已取消（處理至 {} 日）", days_done),
            );
            return Err(format!(
                "下載作業已被使用者取消（已完成 {}/{} 日，共 {} 筆資料）",
                days_done, total_days, total_records
            ));
        }

        on_progress(total_days, total_days, "完成".into());

        Ok(format!(
            "已下載共 {} 筆資料，涵蓋 {} 個日期",
            total_records, total_days
        ))
    }
}
