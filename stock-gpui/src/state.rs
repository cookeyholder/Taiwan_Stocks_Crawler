use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, RwLock};

pub static TOKIO_RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("初始化 Tokio 執行期失敗")
});

use stock_core::backtest::bollinger::BollingerSqueezeBreakout;
use stock_core::backtest::engine::BacktestEngine;
use stock_core::backtest::ma_alignment::MaBullishAlignmentStrategy;
use stock_core::backtest::new_high::NewHighBreakoutStrategy;
use stock_core::backtest::performance::calculate_metrics;
use stock_core::backtest::portfolio::PortfolioSimulator;
use stock_core::backtest::strategies::StrategyKind;
use stock_core::db::MarketRepository;
use stock_core::fetcher::{HttpClient, HttpPolicy};
use stock_core::models::*;
use stock_core::pipeline::Pipeline;

fn ingestion_has_source_error(outcome: &stock_core::pipeline::IngestionOutcome) -> bool {
    (!outcome.ok && outcome.message != "no data (market closed or error)")
        || ["TWSE:", "TPEX:", "T86:"]
            .iter()
            .any(|prefix| outcome.message.contains(prefix))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ScreenerMode {
    #[default]
    Bollinger,
    MaAlignment,
    NewHigh,
    Composite,
}

impl ScreenerMode {
    pub fn label(&self) -> &'static str {
        match self {
            ScreenerMode::Bollinger => "布林通道擠壓",
            ScreenerMode::MaAlignment => "均線多頭排列",
            ScreenerMode::NewHigh => "創波段新高",
            ScreenerMode::Composite => "複合選股（多頭排列 + 創新高）",
        }
    }
}

#[derive(Clone, Debug)]
pub struct ScreenerParams {
    pub mode: ScreenerMode,
    pub max_bandwidth: f64,
    pub min_days: i32,
    pub require_break_upper: bool,
    pub latest_day_only: bool,
    pub short_period: usize,
    pub mid_period: usize,
    pub long_period: usize,
    pub lookback_period: usize,
    pub use_high: bool,
    pub min_volume: i64,
    pub instrument_types: Option<Vec<String>>,
}

impl Default for ScreenerParams {
    fn default() -> Self {
        Self {
            mode: ScreenerMode::Bollinger,
            max_bandwidth: 0.08,
            min_days: 5,
            require_break_upper: true,
            latest_day_only: true,
            short_period: 5,
            mid_period: 10,
            long_period: 20,
            lookback_period: 20,
            use_high: true,
            min_volume: 0,
            instrument_types: None,
        }
    }
}

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
    pub institutional: Vec<Option<InstitutionalRecord>>,
}

#[derive(Clone, Debug)]
pub struct SecurityItem {
    pub symbol: String,
    pub market: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairError {
    pub message: String,
    pub remaining_dates: Vec<String>,
    pub inserted_days: usize,
    pub total_records: usize,
}

impl std::ops::Deref for RepairError {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.message
    }
}

impl std::fmt::Display for RepairError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RepairError {}

#[derive(Clone)]
pub struct CoreState {
    pub pipeline: Arc<tokio::sync::Mutex<Pipeline>>,
    pub cancel_flag: Arc<AtomicBool>,
    pub watchlist_symbols: Arc<RwLock<HashSet<String>>>,
}

impl CoreState {
    pub fn new_with_db_path(db_path: &str) -> Result<Self, String> {
        let repo = MarketRepository::open(db_path).map_err(|e| e.to_string())?;
        let initial_watchlist = repo
            .get_watchlist(None)
            .unwrap_or_default()
            .into_iter()
            .collect::<HashSet<String>>();
        let http = HttpClient::new(HttpPolicy::default());
        let pipeline = Pipeline::new(http, repo);
        Ok(Self {
            pipeline: Arc::new(tokio::sync::Mutex::new(pipeline)),
            cancel_flag: Arc::new(AtomicBool::new(false)),
            watchlist_symbols: Arc::new(RwLock::new(initial_watchlist)),
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

    /// 判斷股票是否在自選清單中（同步讀取快取）
    pub fn is_favorited(&self, symbol: &str) -> bool {
        self.watchlist_symbols
            .read()
            .map(|set| set.contains(symbol))
            .unwrap_or(false)
    }

    /// 取得目前所有自選股票代號集合快取
    pub fn get_watchlist_symbols(&self) -> HashSet<String> {
        self.watchlist_symbols
            .read()
            .map(|set| set.clone())
            .unwrap_or_default()
    }

    /// 一鍵切換自選股狀態（非同步寫入資料庫並同步更新快取）
    pub async fn toggle_watchlist(
        &self,
        symbol: &str,
        group: Option<&str>,
    ) -> Result<bool, String> {
        let pipeline = self.pipeline.lock().await;
        let is_fav = pipeline
            .repo
            .is_in_watchlist(symbol, group)
            .map_err(|e| e.to_string())?;
        if is_fav {
            pipeline
                .repo
                .remove_from_watchlist(symbol, group)
                .map_err(|e| e.to_string())?;
            let still_in = pipeline
                .repo
                .is_in_watchlist(symbol, None)
                .map_err(|e| e.to_string())?;
            if !still_in && let Ok(mut set) = self.watchlist_symbols.write() {
                set.remove(symbol);
            }
            Ok(false)
        } else {
            pipeline
                .repo
                .add_to_watchlist(symbol, group.unwrap_or("預設自選"))
                .map_err(|e| e.to_string())?;
            if let Ok(mut set) = self.watchlist_symbols.write() {
                set.insert(symbol.to_string());
            }
            Ok(true)
        }
    }

    /// 從資料庫重新載入自選股快取
    pub async fn refresh_watchlist(&self) -> Result<(), String> {
        let pipeline = self.pipeline.lock().await;
        let list = pipeline
            .repo
            .get_watchlist(None)
            .map_err(|e| e.to_string())?;
        if let Ok(mut set) = self.watchlist_symbols.write() {
            *set = list.into_iter().collect();
        }
        Ok(())
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

        let inst_records =
            if let (Some(first), Some(last)) = (output.ohlcv.first(), output.ohlcv.last()) {
                pipeline
                    .repo
                    .get_institutional_records(symbol, &first.trade_date, &last.trade_date)
                    .unwrap_or_default()
            } else {
                vec![]
            };

        let inst_map: HashMap<String, InstitutionalRecord> = inst_records
            .into_iter()
            .map(|r| (r.trade_date.clone(), r))
            .collect();

        let institutional: Vec<Option<InstitutionalRecord>> = output
            .ohlcv
            .iter()
            .map(|bar| inst_map.get(&bar.trade_date).cloned())
            .collect();

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
            institutional,
        })
    }

    pub async fn screen_composite_range<F>(
        &self,
        start_date: &str,
        end_date: &str,
        params: ScreenerParams,
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

        let types_ref = params.instrument_types.as_deref();
        let engine = BacktestEngine::new(&pipeline.repo);

        let chunk_size = 10;
        let chunks: Vec<_> = trade_dates.chunks(chunk_size).collect();
        let total_chunks = chunks.len();
        let mut all_results = Vec::new();

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

            match params.mode {
                ScreenerMode::Bollinger => {
                    let strategy = BollingerSqueezeBreakout {
                        bandwidth_limit: params.max_bandwidth,
                        consecutive_days: params.min_days as usize,
                        min_volume: params.min_volume,
                        require_break_upper: params.require_break_upper,
                    };
                    let output = engine
                        .run_range(chunk_start, chunk_end, &strategy, types_ref)
                        .map_err(|e| e.to_string())?;
                    for s in output.signals {
                        all_results.push(s.to_screen_result());
                    }
                }
                ScreenerMode::MaAlignment => {
                    let strategy = MaBullishAlignmentStrategy {
                        short_period: params.short_period,
                        mid_period: params.mid_period,
                        long_period: params.long_period,
                        min_volume: params.min_volume,
                    };
                    let output = engine
                        .run_range(chunk_start, chunk_end, &strategy, types_ref)
                        .map_err(|e| e.to_string())?;
                    for s in output.signals {
                        all_results.push(s.to_screen_result());
                    }
                }
                ScreenerMode::NewHigh => {
                    let strategy = NewHighBreakoutStrategy {
                        lookback_period: params.lookback_period,
                        use_high: params.use_high,
                        min_volume: params.min_volume,
                    };
                    let output = engine
                        .run_range(chunk_start, chunk_end, &strategy, types_ref)
                        .map_err(|e| e.to_string())?;
                    for s in output.signals {
                        all_results.push(s.to_screen_result());
                    }
                }
                ScreenerMode::Composite => {
                    let ma_strategy = MaBullishAlignmentStrategy {
                        short_period: params.short_period,
                        mid_period: params.mid_period,
                        long_period: params.long_period,
                        min_volume: params.min_volume,
                    };
                    let nh_strategy = NewHighBreakoutStrategy {
                        lookback_period: params.lookback_period,
                        use_high: params.use_high,
                        min_volume: params.min_volume,
                    };
                    let ma_output = engine
                        .run_range(chunk_start, chunk_end, &ma_strategy, types_ref)
                        .map_err(|e| e.to_string())?;
                    let nh_output = engine
                        .run_range(chunk_start, chunk_end, &nh_strategy, types_ref)
                        .map_err(|e| e.to_string())?;

                    let mut nh_set: HashSet<(String, String)> = HashSet::new();
                    for s in nh_output.signals {
                        if s.direction > 0 {
                            nh_set.insert((s.trade_date, s.symbol));
                        }
                    }

                    for s in ma_output.signals {
                        if s.direction > 0
                            && nh_set.contains(&(s.trade_date.clone(), s.symbol.clone()))
                        {
                            let mut item = s.to_screen_result();
                            item.tags = vec![
                                "多頭排列".to_string(),
                                format!("創{}日新高", params.lookback_period),
                            ];
                            all_results.push(item);
                        }
                    }
                }
            }
        }

        if self.cancel_flag.load(Ordering::SeqCst) {
            return Err("選股已被使用者取消".into());
        }

        if params.latest_day_only
            && let Some(last_date) = trade_dates.last()
        {
            all_results.retain(|item| &item.trade_date == last_date);
        }

        let mut symbol_map: HashMap<String, (String, ScreenResult)> = HashMap::new();
        for item in all_results {
            let trade_date = item.trade_date.clone();
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
        match params.mode {
            ScreenerMode::Bollinger => {
                results.sort_by(|a, b| {
                    a.latest_bandwidth
                        .partial_cmp(&b.latest_bandwidth)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            _ => {
                results.sort_by(|a, b| {
                    b.latest_volume
                        .unwrap_or(0)
                        .cmp(&a.latest_volume.unwrap_or(0))
                });
            }
        }

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
        let params = ScreenerParams {
            mode: ScreenerMode::Bollinger,
            max_bandwidth,
            min_days,
            require_break_upper,
            min_volume,
            instrument_types,
            ..Default::default()
        };
        self.screen_composite_range(start_date, end_date, params, on_progress)
            .await
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

    /// 診斷指定日期區間之缺漏平日交易日
    pub async fn diagnose_gaps(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> Result<GapReport, String> {
        let pipeline = self.pipeline.lock().await;
        pipeline
            .repo
            .diagnose_gaps(start_date, end_date)
            .map_err(|e| e.to_string())
    }

    /// 定向修補缺漏之交易日清單
    pub async fn repair_gaps<F>(
        &self,
        missing_dates: &[String],
        on_progress: F,
    ) -> Result<String, RepairError>
    where
        F: Fn(usize, usize, String) + Send + Sync,
    {
        if missing_dates.is_empty() {
            return Ok("目前沒有缺漏交易日需要修補".to_string());
        }

        let total_days = missing_dates.len();
        let mut days_done = 0usize;
        let mut total_records = 0usize;
        let mut inserted_days = 0usize;
        let mut failure_count = 0usize;
        let mut failure_details = Vec::new();
        let mut failed_dates = Vec::new();

        self.cancel_flag.store(false, Ordering::SeqCst);

        for (idx, date_str) in missing_dates.iter().enumerate() {
            if self.cancel_flag.load(Ordering::SeqCst) {
                failed_dates.extend(missing_dates[idx..].iter().cloned());
                break;
            }

            on_progress(days_done, total_days, date_str.clone());

            let pipeline = self.pipeline.clone();
            let date_clone = date_str.clone();
            let fetch_task = TOKIO_RUNTIME.spawn(async move {
                let p = pipeline.lock().await;
                p.fetch_date(&date_clone).await
            });

            let failure = match fetch_task.await {
                Ok(Ok(outcome)) => {
                    if outcome.records_inserted > 0 {
                        total_records += outcome.records_inserted;
                        inserted_days += 1;
                    }
                    ingestion_has_source_error(&outcome).then_some(outcome.message)
                }
                Ok(Err(error)) => Some(error.to_string()),
                Err(error) => Some(error.to_string()),
            };
            if let Some(error) = failure {
                failure_count += 1;
                failed_dates.push(date_str.clone());
                if failure_details.len() < 5 {
                    failure_details.push(format!("{}：{}", date_str, error));
                }
            }

            days_done += 1;
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
                format!("已取消修補（處理至第 {} 日）", days_done),
            );
            return Err(RepairError {
                message: format!(
                    "修補作業已被使用者取消（已完成 {}/{} 日，成功入庫 {} 日共 {} 筆資料）",
                    days_done, total_days, inserted_days, total_records
                ),
                remaining_dates: failed_dates,
                inserted_days,
                total_records,
            });
        }

        if failure_count > 0 {
            let extra = if failure_count > failure_details.len() {
                format!(
                    "；另有 {} 個日期未列出",
                    failure_count - failure_details.len()
                )
            } else {
                String::new()
            };
            return Err(RepairError {
                message: format!(
                    "修補未完成：{} 個日期抓取失敗，已入庫 {} 個交易日共 {} 筆資料。{}{}",
                    failure_count,
                    inserted_days,
                    total_records,
                    failure_details.join("；"),
                    extra
                ),
                remaining_dates: failed_dates,
                inserted_days,
                total_records,
            });
        }

        on_progress(total_days, total_days, "完成".into());

        Ok(format!(
            "修補完成！共檢查 {} 個缺漏日，成功入庫 {} 個交易日共 {} 筆資料（其餘為休市或無行情）",
            total_days, inserted_days, total_records
        ))
    }

    /// 取得資料庫中 TWSE 上市普通股總檔數（排除 ETF 與權證）
    pub async fn get_twse_stock_count(&self) -> Result<usize, String> {
        let pipeline = self.pipeline.lock().await;
        let syms = pipeline
            .get_twse_stock_symbols()
            .map_err(|e| e.to_string())?;
        Ok(syms.len())
    }

    /// 批次抓取指定交易日之集中市場券商日報表（分點成交明細）
    pub async fn fetch_broker_trades<F>(
        &self,
        trade_date: &str,
        target_symbols: Option<Vec<String>>,
        on_progress: F,
    ) -> Result<stock_core::pipeline::broker::BrokerFetchSummary, String>
    where
        F: Fn(stock_core::pipeline::broker::BrokerProgress) + Send + Sync + 'static,
    {
        self.cancel_flag.store(false, Ordering::SeqCst);
        let date_str = trade_date.to_string();
        let cancel_flag = self.cancel_flag.clone();

        let path = default_db_path();
        let repo = MarketRepository::open(path.to_str().unwrap_or("market_data.db"))
            .map_err(|e| format!("開啟資料庫失敗: {}", e))?;

        let client = stock_core::fetcher::bsr::client::BsrClient::new()
            .map_err(|e| format!("建立集中市場 BSR 連線客戶端失敗: {}", e))?;

        let broker_pipeline = stock_core::pipeline::broker::BrokerFetchPipeline::new(client, repo)
            .with_cancel_flag(cancel_flag);

        let handle = TOKIO_RUNTIME.spawn(async move {
            broker_pipeline
                .fetch_date(&date_str, target_symbols, None, on_progress)
                .await
        });

        handle
            .await
            .map_err(|e| format!("執行批次抓取任務異常: {}", e))?
            .map_err(|e| format!("集中市場券商日報表抓取失敗: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::ingestion_has_source_error;
    use stock_core::pipeline::{DayType, IngestionOutcome};

    #[test]
    fn gap_repair_distinguishes_closed_days_from_source_errors() {
        let closed = IngestionOutcome {
            day_type: DayType::MarketClosed,
            ok: false,
            message: "no data (market closed or error)".to_string(),
            records_inserted: 0,
        };
        let failed = IngestionOutcome {
            day_type: DayType::MarketClosed,
            ok: false,
            message: "TWSE: request timeout".to_string(),
            records_inserted: 0,
        };
        let partial = IngestionOutcome {
            day_type: DayType::Trading,
            ok: true,
            message: "OK: 10 records; T86: request timeout".to_string(),
            records_inserted: 10,
        };

        assert!(!ingestion_has_source_error(&closed));
        assert!(ingestion_has_source_error(&failed));
        assert!(ingestion_has_source_error(&partial));
    }
}
