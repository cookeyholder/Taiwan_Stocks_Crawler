use std::path::PathBuf;

use clap::{Parser, Subcommand};
use stock_core::db::MarketRepository;
use stock_core::fetcher::{HttpClient, HttpPolicy};
use stock_core::pipeline::Pipeline;

#[derive(Parser)]
#[command(name = "stock-cli", version, about = "台股選股工具 CLI")]
struct Cli {
    #[arg(short, long, default_value = "market_data.db")]
    db: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 下載市場資料
    Fetch {
        #[arg(long)]
        today: bool,
        #[arg(long)]
        date: Option<String>,
        #[arg(long)]
        start: Option<String>,
        #[arg(long)]
        end: Option<String>,
    },
    /// 布林通道選股
    Screen {
        #[arg(long, default_value = "0.08")]
        max_bandwidth: f64,
        #[arg(long, default_value = "5")]
        min_days: i32,
        #[arg(long, value_delimiter = ',')]
        types: Option<Vec<String>>,
        #[arg(long, default_value = "0")]
        min_volume: i64,
    },
    /// 同步 yfinance 歷史資料
    Sync {
        #[arg(long)]
        symbol: Option<String>,
    },
    /// 回測選股策略
    Backtest {
        #[arg(long)]
        date: Option<String>,
        #[arg(long)]
        start: Option<String>,
        #[arg(long)]
        end: Option<String>,
        #[arg(long, default_value = "0.08")]
        max_bandwidth: f64,
        #[arg(long, default_value = "5")]
        min_days: usize,
        #[arg(long, value_delimiter = ',')]
        types: Option<Vec<String>>,
        #[arg(long, default_value = "0")]
        min_volume: i64,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let http = HttpClient::new(HttpPolicy::default());
    let repo = MarketRepository::open(cli.db.to_str().unwrap()).expect("Failed to open DB");
    let pipeline = Pipeline::new(http, repo);

    match cli.command {
        Commands::Fetch {
            today,
            date,
            start: _,
            end: _,
        } => {
            if today {
                match pipeline.fetch_today().await {
                    Ok(outcome) => println!("{}", outcome.message),
                    Err(e) => eprintln!("Error: {}", e),
                }
            } else if let Some(d) = date {
                match pipeline.fetch_date(&d).await {
                    Ok(outcome) => println!("{}: {} records", d, outcome.records_inserted),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        }
        Commands::Screen {
            max_bandwidth,
            min_days,
            types,
            min_volume,
        } => {
            match pipeline
                .screen(max_bandwidth, min_days, types, min_volume)
                .await
            {
                Ok(results) => {
                    println!(
                        "{:<8} {:<12} {:>10} {:>12} {:>12}",
                        "Symbol", "Name", "Close", "Volume", "Bandwidth"
                    );
                    for r in &results {
                        let name = r.name.as_deref().unwrap_or("-");
                        let close = r
                            .latest_close
                            .map(|c| format!("{:.2}", c))
                            .unwrap_or_else(|| "-".into());
                        let vol = r
                            .latest_volume
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "-".into());
                        println!(
                            "{:<8} {:<12} {:>10} {:>12} {:>12.4}",
                            r.symbol, name, close, vol, r.latest_bandwidth
                        );
                    }
                    println!("\nTotal: {} stocks", results.len());
                }
                Err(e) => eprintln!("Screen error: {}", e),
            }
        }
        Commands::Backtest {
            date,
            start,
            end,
            max_bandwidth,
            min_days,
            types,
            min_volume,
        } => {
            use stock_core::backtest::BacktestEngine;
            use stock_core::backtest::bollinger::BollingerSqueezeBreakout;

            if min_days == 0 {
                eprintln!("Error: --min-days must be > 0");
                return;
            }
            let engine = BacktestEngine::new(&pipeline.repo);
            let strategy = BollingerSqueezeBreakout {
                bandwidth_limit: max_bandwidth,
                consecutive_days: min_days,
                min_volume,
            };
            let types_ref = types.as_deref();

            let output = if let Some(d) = date {
                engine.run_date(&d, &strategy, types_ref)
            } else if let (Some(s), Some(e)) = (start.as_ref(), end.as_ref()) {
                engine.run_range(s, e, &strategy, types_ref)
            } else {
                let latest = match pipeline.repo.get_latest_trade_date() {
                    Ok(Some(d)) => d,
                    _ => {
                        eprintln!("No data in database");
                        return;
                    }
                };
                engine.run_date(&latest, &strategy, types_ref)
            };

            match output {
                Ok(output) => {
                    println!(
                        "{:<10} {:<8} {:<12} {:>10} {:>12} {:>8} {:>8}",
                        "Date", "Symbol", "Name", "Close", "Volume", "Dir", "Strength"
                    );
                    for s in &output.signals {
                        let name = s.name.as_deref().unwrap_or("-");
                        let close = s
                            .close_price
                            .map(|c| format!("{:.2}", c))
                            .unwrap_or_else(|| "-".into());
                        let vol = s
                            .volume
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "-".into());
                        println!(
                            "{:<10} {:<8} {:<12} {:>10} {:>12} {:>8} {:>8.4}",
                            s.trade_date, s.symbol, name, close, vol, s.direction, s.strength
                        );
                    }
                    println!("\nTotal: {} signals", output.signals.len());
                }
                Err(e) => eprintln!("Backtest error: {}", e),
            }
        }
        Commands::Sync { symbol } => {
            if let Some(sym) = symbol {
                println!("Syncing yfinance data for {}...", sym);
                println!("Sync complete (yfinance not yet implemented in Rust backend)");
            }
        }
    }
}
