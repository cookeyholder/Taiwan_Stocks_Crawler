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
    /// 布林擠壓篩選
    Screen {
        #[arg(long, default_value = "0.08")]
        max_bandwidth: f64,
        #[arg(long, default_value = "5")]
        min_days: i32,
        #[arg(long)]
        types: Option<Vec<String>>,
        #[arg(long, default_value = "0")]
        min_volume: i64,
    },
    /// 同步 yfinance 歷史資料
    Sync {
        #[arg(long)]
        symbol: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let http = HttpClient::new(HttpPolicy::default());
    let repo = MarketRepository::open(cli.db.to_str().unwrap()).expect("Failed to open DB");
    let pipeline = Pipeline::new(http, repo);

    match cli.command {
        Commands::Fetch { today, date, start: _, end: _ } => {
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
        Commands::Screen { max_bandwidth, min_days, types, min_volume } => {
            match pipeline.screen(max_bandwidth, min_days, types, min_volume).await {
                Ok(results) => {
                    println!("{:<8} {:<12} {:>10} {:>12} {:>12}", "Symbol", "Name", "Close", "Volume", "Bandwidth");
                    for r in &results {
                        let name = r.name.as_deref().unwrap_or("-");
                        let close = r.latest_close.map(|c| format!("{:.2}", c)).unwrap_or_else(|| "-".into());
                        let vol = r.latest_volume.map(|v| v.to_string()).unwrap_or_else(|| "-".into());
                        println!("{:<8} {:<12} {:>10} {:>12} {:>12.4}", r.symbol, name, close, vol, r.latest_bandwidth);
                    }
                    println!("\nTotal: {} stocks", results.len());
                }
                Err(e) => eprintln!("Screen error: {}", e),
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
