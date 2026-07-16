use std::path::PathBuf;

use clap::{Parser, Subcommand};
use stock_core::db::MarketRepository;
use stock_core::fetcher::{HttpClient, HttpPolicy};
use stock_core::models::OhlcvRecord;
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
        #[arg(long)]
        date: Option<String>,
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
        #[arg(long, default_value = "bollinger_squeeze")]
        strategy: String,
        #[arg(long, value_parser = parse_key_val)]
        param: Vec<(String, f64)>,
        #[arg(long, default_value = "1000000")]
        capital: f64,
        #[arg(long, value_delimiter = ',')]
        types: Option<Vec<String>>,
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
            date,
            max_bandwidth,
            min_days,
            types,
            min_volume,
        } => {
            match pipeline
                .screen(max_bandwidth, min_days, types, min_volume, date.as_deref())
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
            strategy,
            param,
            capital,
            types,
        } => {
            use std::collections::HashMap;
            use stock_core::backtest::engine::BacktestEngine;
            use stock_core::backtest::performance::calculate_metrics;
            use stock_core::backtest::portfolio::PortfolioSimulator;
            use stock_core::backtest::strategies::StrategyKind;

            let engine = BacktestEngine::new(&pipeline.repo);
            let strategy = match StrategyKind::create(&strategy, &param) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("策略錯誤: {}", e);
                    return;
                }
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
                    // Get trade dates for equity curve
                    let trade_dates: Vec<String> =
                        if let (Some(s), Some(e)) = (start.as_ref(), end.as_ref()) {
                            pipeline
                                .repo
                                .get_trade_dates_in_range(s, e)
                                .unwrap_or_default()
                        } else {
                            vec![]
                        };
                    let start_date = trade_dates.first().cloned().unwrap_or_default();
                    let end_date = trade_dates.last().cloned().unwrap_or_default();

                    // Build ohlcv_map for portfolio simulation
                    let rows = pipeline
                        .repo
                        .get_batch_ohlcv(&start_date, &end_date)
                        .unwrap_or_default();
                    let mut ohlcv_map: HashMap<String, Vec<OhlcvRecord>> = HashMap::new();
                    for (sym, record) in rows {
                        ohlcv_map.entry(sym).or_default().push(record);
                    }

                    let (trades, equity_curve) = PortfolioSimulator::simulate(
                        &output.signals,
                        capital,
                        &ohlcv_map,
                        &trade_dates,
                    );
                    let metrics = calculate_metrics(&trades, capital, &equity_curve);

                    println!("\n─── 績效摘要 ───");
                    println!(
                        "總報酬率: {:+.1}%    勝率: {:.1}%    Sharpe: {:.2}",
                        metrics.total_return_pct,
                        metrics.win_rate * 100.0,
                        metrics.sharpe_ratio
                    );
                    println!(
                        "最大回撤: {:.1}%     交易次數: {}    最後價值: {:.0}",
                        metrics.max_drawdown_pct, metrics.total_trades, metrics.final_value
                    );

                    if !trades.is_empty() {
                        println!("\n─── 交易明細 ───");
                        println!(
                            "{:<8} {:<6} {:<12} {:>8} {:>8} {:>10} {:>8}",
                            "買入日", "代號", "名稱", "買價", "賣價", "損益", "報酬率"
                        );
                        for t in &trades {
                            let name = t.name.as_deref().unwrap_or("-");
                            let sell_p = t
                                .sell_price
                                .map(|p| format!("{:.2}", p))
                                .unwrap_or_else(|| "持有中".into());
                            let pnl_str = if t.pnl >= 0.0 {
                                format!("+{:.0}", t.pnl)
                            } else {
                                format!("{:.0}", t.pnl)
                            };
                            println!(
                                "{:<8} {:<6} {:<12} {:>8.2} {:>8} {:>10} {:>7.1}%",
                                t.buy_date, t.symbol, name, t.buy_price, sell_p, pnl_str, t.pnl_pct
                            );
                        }
                    }

                    if !output.signals.is_empty() {
                        println!("\n─── 訊號明細 ───");
                        println!(
                            "{:<10} {:<8} {:<12} {:>8} {:>10} {:>8}",
                            "日期", "代號", "名稱", "方向", "收盤價", "強度"
                        );
                        for s in &output.signals {
                            let name = s.name.as_deref().unwrap_or("-");
                            let dir = if s.direction > 0 {
                                "買"
                            } else if s.direction < 0 {
                                "賣"
                            } else {
                                "-"
                            };
                            let close = s
                                .close_price
                                .map(|c| format!("{:.2}", c))
                                .unwrap_or_else(|| "-".into());
                            println!(
                                "{:<10} {:<8} {:<12} {:>8} {:>10} {:>8.4}",
                                s.trade_date, s.symbol, name, dir, close, s.strength
                            );
                        }
                    }
                    println!(
                        "\nTotal: {} signals, {} trades",
                        output.signals.len(),
                        trades.len()
                    );
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

fn parse_key_val(s: &str) -> Result<(String, f64), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(format!("invalid KEY=value: {}", s));
    }
    let key = parts[0].to_string();
    let val = parts[1]
        .parse::<f64>()
        .map_err(|e| format!("invalid number: {}", e))?;
    Ok((key, val))
}
