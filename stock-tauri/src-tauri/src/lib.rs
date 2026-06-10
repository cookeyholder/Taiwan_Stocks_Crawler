use serde::{Deserialize, Serialize};
use stock_core::analysis;
use stock_core::db::MarketRepository;
use stock_core::fetcher::{HttpClient, HttpPolicy};
use stock_core::models::*;
use stock_core::pipeline::Pipeline;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;

struct AppState {
    pipeline: Mutex<Pipeline>,
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
    pipeline.fetch_today().await
        .map(|o| format!("{}: {} records", o.message, o.records_inserted))
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn fetch_date(state: State<'_, AppState>, date: String) -> Result<String, String> {
    let pipeline = state.pipeline.lock().await;
    pipeline.fetch_date(&date).await
        .map(|o| format!("{} records", o.records_inserted))
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn screen_bollinger(
    state: State<'_, AppState>,
    max_bandwidth: f64,
    min_days: i32,
    instrument_types: Option<Vec<String>>,
    min_volume: i64,
) -> Result<Vec<ScreenResult>, String> {
    let pipeline = state.pipeline.lock().await;
    pipeline.screen(max_bandwidth, min_days, instrument_types, min_volume)
        .await
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
    let output = pipeline.get_analysis(&symbol, &ma_periods, rsi_period, macd_fast, macd_slow, macd_signal)
        .await
        .map_err(|e| e.to_string())?;

    let prices: Vec<f64> = output.ohlcv.iter().filter_map(|r| r.close_price).collect();
    let bollinger = if prices.len() >= 20 {
        Some(analysis::bollinger(&prices, 20, 2.0))
    } else {
        None
    };

    let name = pipeline.repo.get_distinct_symbols().ok().and_then(|syms| {
        syms.iter().find(|(s, _)| s == &symbol).map(|(_, m)| m.clone())
    });

    let empty = vec![];
    Ok(AnalysisResponse {
        symbol,
        name,
        ohlcv: output.ohlcv,
        bollinger_upper: bollinger.as_ref().map(|b| b.upper.clone()).unwrap_or(empty.clone()),
        bollinger_middle: bollinger.as_ref().map(|b| b.middle.clone()).unwrap_or(empty.clone()),
        bollinger_lower: bollinger.as_ref().map(|b| b.lower.clone()).unwrap_or(empty),
        ma5: output.ma_lines.get(0).cloned().unwrap_or_default(),
        ma10: output.ma_lines.get(1).cloned().unwrap_or_default(),
        ma20: output.ma_lines.get(2).cloned().unwrap_or_default(),
        rsi: output.rsi.map(|r| r.values).unwrap_or_default(),
        macd_line: output.macd.as_ref().map(|m| m.macd_line.clone()).unwrap_or_default(),
        macd_signal: output.macd.as_ref().map(|m| m.signal_line.clone()).unwrap_or_default(),
        macd_histogram: output.macd.as_ref().map(|m| m.histogram.clone()).unwrap_or_default(),
    })
}

#[tauri::command]
async fn get_symbols(state: State<'_, AppState>) -> Result<Vec<(String, String)>, String> {
    let pipeline = state.pipeline.lock().await;
    pipeline.repo.get_distinct_symbols().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_latest_trade_date(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let pipeline = state.pipeline.lock().await;
    pipeline.repo.get_latest_trade_date().map_err(|e| e.to_string())
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
    let mut processed = 0usize;

    while current <= end {
        let date_str = current.format("%Y-%m-%d").to_string();
        let _ = app.emit("download-progress", DownloadProgress {
            current: processed,
            total: total_days,
            date: date_str.clone(),
        });

        let pipeline = state.pipeline.lock().await;
        match pipeline.fetch_date(&date_str).await {
            Ok(outcome) => {
                processed += outcome.records_inserted;
            }
            Err(_) => {}
        }
        current += chrono::Duration::days(1);
    }

    let _ = app.emit("download-progress", DownloadProgress {
        current: total_days,
        total: total_days,
        date: "完成".into(),
    });

    Ok(format!("Downloaded {} records across {} dates", processed, total_days))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_path = dirs_data_dir();
    let _ = std::fs::create_dir_all(&db_path);
    let db_file = db_path.join("market_data.db");

    let repo = MarketRepository::open(db_file.to_str().unwrap())
        .expect("Failed to open database");
    let http = HttpClient::new(HttpPolicy::default());
    let pipeline = Pipeline::new(http, repo);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            pipeline: Mutex::new(pipeline),
        })
        .invoke_handler(tauri::generate_handler![
            fetch_today,
            fetch_date,
            screen_bollinger,
            get_analysis,
            get_symbols,
            get_latest_trade_date,
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
