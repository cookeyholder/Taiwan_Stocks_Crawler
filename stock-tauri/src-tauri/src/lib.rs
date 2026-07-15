use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};

use stock_core::db::MarketRepository;
use stock_core::fetcher::{HttpClient, HttpPolicy};
use stock_core::models::*;
use stock_core::pipeline::Pipeline;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;

struct AppState {
    pipeline: Mutex<Pipeline>,
    cancel_flag: Arc<AtomicBool>,
}

#[derive(Serialize, Deserialize)]
struct AnalysisResponse {
    ohlcv: Vec<OhlcvRecord>,
    bollinger_upper: Vec<Option<f64>>,
    bollinger_middle: Vec<Option<f64>>,
    bollinger_lower: Vec<Option<f64>>,
    ma5: Vec<Option<f64>>,
    ma10: Vec<Option<f64>>,
    ma20: Vec<Option<f64>>,
    rsi: Vec<Option<f64>>,
    macd_line: Vec<Option<f64>>,
    macd_signal: Vec<Option<f64>>,
    macd_histogram: Vec<Option<f64>>,
    symbol: String,
    name: Option<String>,
}

#[tauri::command]
async fn fetch_today(state: State<'_, AppState>) -> Result<String, String> {
    let pipeline = state.pipeline.lock().await;
    let outcome = pipeline.fetch_today().await.map_err(|e| e.to_string())?;
    if outcome.records_inserted > 0 {
        let _ = pipeline.repo.sync_securities();
    }
    Ok(outcome.message)
}

#[tauri::command]
async fn fetch_date(state: State<'_, AppState>, date: String) -> Result<String, String> {
    let pipeline = state.pipeline.lock().await;
    pipeline
        .fetch_date(&date)
        .await
        .map(|o| o.message)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn screen_bollinger_range(
    app: AppHandle,
    state: State<'_, AppState>,
    start_date: String,
    end_date: String,
    max_bandwidth: f64,
    min_days: i32,
    instrument_types: Option<Vec<String>>,
    min_volume: i64,
) -> Result<RangeScreenResult, String> {
    use stock_core::pipeline::build_cache_key;

    const SCREEN_CACHE_VERSION: &str = "v1";
    const MAX_TRADING_DAYS: usize = 60;

    let _start = chrono::NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
        .map_err(|_| format!("日期格式錯誤：{}", start_date))?;
    let _end = chrono::NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
        .map_err(|_| format!("日期格式錯誤：{}", end_date))?;
    if start_date > end_date {
        return Err("起始日期不能晚於結束日期".into());
    }

    let cancel_flag = state.cancel_flag.clone();
    cancel_flag.store(false, Ordering::SeqCst);

    let pipeline = state.pipeline.lock().await;

    let types = instrument_types.as_deref().unwrap_or(&[]);
    let mut sorted_types = types.to_vec();
    sorted_types.sort();
    let types_str = sorted_types.join(",");

    let trade_dates = pipeline
        .repo
        .get_trade_dates_in_range(&start_date, &end_date)
        .map_err(|e| e.to_string())?;

    if trade_dates.len() > MAX_TRADING_DAYS {
        return Err(format!(
            "區間超過上限：{} 個交易日（上限 {} 天）",
            trade_dates.len(),
            MAX_TRADING_DAYS
        ));
    }

    let total = trade_dates.len();
    let mut cached_days = 0usize;
    let mut calculated_days = 0usize;
    let mut failed_days = Vec::new();
    let mut last_results: Vec<ScreenResult> = Vec::new();

    for (i, date) in trade_dates.iter().enumerate() {
        if cancel_flag.load(Ordering::SeqCst) {
            break;
        }

        let cache_key = build_cache_key(
            SCREEN_CACHE_VERSION,
            date,
            &sorted_types,
            max_bandwidth,
            min_days,
            min_volume,
        );

        if let Some(cached) = pipeline
            .repo
            .get_cached_screening(&cache_key)
            .map_err(|e| e.to_string())?
        {
            cached_days += 1;
            last_results = cached;
            let _ = app.emit(
                "range-progress",
                RangeProgress {
                    current: i + 1,
                    total,
                    date: date.clone(),
                    cached: true,
                },
            );
            continue;
        }

        match pipeline
            .screen(
                max_bandwidth,
                min_days,
                instrument_types.clone(),
                min_volume,
                Some(date),
            )
            .await
        {
            Ok(results) => {
                calculated_days += 1;
                let cache_params = CacheParams {
                    query_date: date.clone(),
                    instrument_types: types_str.clone(),
                    max_bandwidth,
                    min_days,
                    min_volume,
                };
                if let Err(e) =
                    pipeline
                        .repo
                        .set_cached_screening(&cache_key, &cache_params, &results)
                {
                    eprintln!("[cache] failed to write {}: {}", date, e);
                }
                last_results = results;
            }
            Err(e) => {
                eprintln!("[range] date {} failed: {}", date, e);
                failed_days.push(date.clone());
            }
        }

        let _ = app.emit(
            "range-progress",
            RangeProgress {
                current: i + 1,
                total,
                date: date.clone(),
                cached: false,
            },
        );
    }

    Ok(RangeScreenResult {
        results: last_results,
        meta: RangeMeta {
            start_date,
            end_date,
            trading_days: trade_dates.len(),
            cached_days,
            calculated_days,
            failed_days,
        },
    })
}

#[tauri::command]
async fn cancel_screening(state: State<'_, AppState>) -> Result<(), String> {
    state.cancel_flag.store(true, Ordering::SeqCst);
    Ok(())
}

#[derive(Clone, Serialize)]
struct RangeProgress {
    current: usize,
    total: usize,
    date: String,
    cached: bool,
}

#[tauri::command]
async fn clear_screener_cache(state: State<'_, AppState>) -> Result<usize, String> {
    let pipeline = state.pipeline.lock().await;
    pipeline
        .repo
        .clear_screener_cache()
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_screener_cache_count(state: State<'_, AppState>) -> Result<usize, String> {
    let pipeline = state.pipeline.lock().await;
    pipeline
        .repo
        .get_screener_cache_count()
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_analysis(
    state: State<'_, AppState>,
    symbol: String,
    ma_periods: Vec<usize>,
    rsi_period: Option<usize>,
    macd_fast: Option<usize>,
    macd_slow: Option<usize>,
    macd_signal: Option<usize>,
) -> Result<AnalysisResponse, String> {
    let pipeline = state.pipeline.lock().await;
    let output = pipeline
        .get_analysis(
            &symbol,
            &ma_periods,
            rsi_period,
            macd_fast,
            macd_slow,
            macd_signal,
        )
        .await
        .map_err(|e| e.to_string())?;

    let name = pipeline.repo.get_symbol_name(&symbol).ok().flatten();

    let empty = vec![];
    Ok(AnalysisResponse {
        symbol,
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

#[tauri::command]
async fn get_symbols(
    state: State<'_, AppState>,
) -> Result<Vec<(String, String, Option<String>)>, String> {
    let pipeline = state.pipeline.lock().await;
    pipeline.repo.get_securities().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_latest_trade_date(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let pipeline = state.pipeline.lock().await;
    pipeline
        .repo
        .get_latest_trade_date()
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_date_range(
    state: State<'_, AppState>,
    start: String,
    end: String,
) -> Result<String, String> {
    if chrono::NaiveDate::parse_from_str(&start, "%Y-%m-%d").is_err()
        || chrono::NaiveDate::parse_from_str(&end, "%Y-%m-%d").is_err()
    {
        return Err("日期格式錯誤，請使用 YYYY-MM-DD 格式".into());
    }
    let pipeline = state.pipeline.lock().await;
    let count = pipeline
        .repo
        .delete_date_range(&start, &end)
        .map_err(|e| e.to_string())?;
    let _ = pipeline.repo.sync_securities();
    Ok(format!("已刪除 {} 筆記錄（{} ~ {}）", count, start, end))
}

#[tauri::command]
async fn get_date_range(
    state: State<'_, AppState>,
) -> Result<(Option<String>, Option<String>, usize), String> {
    let pipeline = state.pipeline.lock().await;
    pipeline.repo.get_date_range().map_err(|e| e.to_string())
}

#[derive(Clone, Serialize)]
struct DownloadProgress {
    current: usize,
    total: usize,
    date: String,
}

#[tauri::command]
async fn batch_download(
    app: AppHandle,
    state: State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<String, String> {
    let start = chrono::NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid start date: {}", e))?;
    let end = chrono::NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid end date: {}", e))?;

    let mut current = start;
    let total_days = (end - start).num_days() as usize + 1;
    let mut days_done = 0usize;
    let mut total_records = 0usize;

    while current <= end {
        let date_str = current.format("%Y-%m-%d").to_string();
        let _ = app.emit(
            "download-progress",
            DownloadProgress {
                current: days_done,
                total: total_days,
                date: date_str.clone(),
            },
        );

        let pipeline = state.pipeline.lock().await;
        if let Ok(outcome) = pipeline.fetch_date(&date_str).await {
            total_records += outcome.records_inserted;
        }
        days_done += 1;
        current += chrono::Duration::days(1);
    }

    if total_records > 0 {
        let pipeline = state.pipeline.lock().await;
        let _ = pipeline.repo.sync_securities();
    }

    let _ = app.emit(
        "download-progress",
        DownloadProgress {
            current: total_days,
            total: total_days,
            date: "完成".into(),
        },
    );

    Ok(format!(
        "Downloaded {} records across {} dates",
        total_records, total_days
    ))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_path = dirs_data_dir();
    let _ = std::fs::create_dir_all(&db_path);
    let db_file = db_path.join("market_data.db");

    let repo = MarketRepository::open(db_file.to_str().unwrap()).expect("Failed to open database");
    let http = HttpClient::new(HttpPolicy::default());
    let pipeline = Pipeline::new(http, repo);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            pipeline: Mutex::new(pipeline),
            cancel_flag: Arc::new(AtomicBool::new(false)),
        })
        .invoke_handler(tauri::generate_handler![
            fetch_today,
            fetch_date,
            screen_bollinger_range,
            cancel_screening,
            clear_screener_cache,
            get_screener_cache_count,
            get_analysis,
            get_symbols,
            get_latest_trade_date,
            get_date_range,
            delete_date_range,
            batch_download,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn dirs_data_dir() -> std::path::PathBuf {
    if let Ok(dir) = std::env::var("STOCK_DATA_DIR") {
        return std::path::PathBuf::from(dir);
    }
    let base = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    base.join("stock-screener")
}
