//! 集中市場券商日報表批次抓取流水線 (BrokerFetchPipeline)
//!
//! 整合 BsrClient、雙欄 CSV 剖析器、資料庫倉儲與斷點續傳、隨機延遲 Jitter、
//! 網路異常冷卻退避與取消旗標控制。

use crate::db::MarketRepository;
use crate::fetcher::bsr::client::{BsrClient, BsrClientError};
use crate::fetcher::bsr::parser::parse_bsr_csv;
use crate::pipeline::filter_twse_common_stock_symbols;
use rand::Rng;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BrokerPipelineError {
    #[error("資料庫操作失敗: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("網路客戶端錯誤: {0}")]
    Client(#[from] BsrClientError),
    #[error("無有效之上市普通股標的")]
    NoTargetSymbols,
}

/// 批次抓取組態選項
#[derive(Debug, Clone)]
pub struct BrokerPipelineOptions {
    /// 最小延遲毫秒（預設 1,500ms）
    pub delay_min_ms: u64,
    /// 最大延遲毫秒（預設 3,000ms）
    pub delay_max_ms: u64,
    /// 驗證碼辨識重試上限（預設 3 次）
    pub max_retries: u32,
    /// 遭遇嚴重網路異常時冷卻等待秒數（預設 30 秒）
    pub cooldown_secs: u64,
}

impl Default for BrokerPipelineOptions {
    fn default() -> Self {
        Self {
            delay_min_ms: 1500,
            delay_max_ms: 3000,
            max_retries: 3,
            cooldown_secs: 30,
        }
    }
}

/// 即時進度通知
#[derive(Debug, Clone)]
pub struct BrokerProgress {
    pub current_index: usize,
    pub total_stocks: usize,
    pub current_symbol: String,
    pub success_count: usize,
    pub failed_count: usize,
    pub skipped_count: usize,
    pub message: String,
}

/// 批次下載成果摘要
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrokerFetchSummary {
    pub trade_date: String,
    pub total_stocks: usize,
    pub success_count: usize,
    pub failed_count: usize,
    pub skipped_count: usize,
    pub cancelled: bool,
}

/// 券商分點明細批次抓取流水線
pub struct BrokerFetchPipeline {
    client: Arc<BsrClient>,
    repo: Arc<MarketRepository>,
    cancel_flag: Option<Arc<AtomicBool>>,
}

impl BrokerFetchPipeline {
    pub fn new(client: BsrClient, repo: MarketRepository) -> Self {
        Self {
            client: Arc::new(client),
            repo: Arc::new(repo),
            cancel_flag: None,
        }
    }

    pub fn with_shared(client: Arc<BsrClient>, repo: Arc<MarketRepository>) -> Self {
        Self {
            client,
            repo,
            cancel_flag: None,
        }
    }

    pub fn with_cancel_flag(mut self, flag: Arc<AtomicBool>) -> Self {
        self.cancel_flag = Some(flag);
        self
    }

    /// 執行指定交易日的集中市場券商日報表批次抓取
    ///
    /// * `trade_date`: 交易日字串，例如 "2026-10-08"
    /// * `target_symbols`: 可選之自訂股票代碼清單；若為 None 則自動提取資料庫中所有 TWSE 普通股
    /// * `options`: 延遲與重試等選項；若為 None 則採用預設值
    /// * `progress_cb`: 每檔處理完畢時的進度回呼函式
    pub async fn fetch_date<F>(
        &self,
        trade_date: &str,
        target_symbols: Option<Vec<String>>,
        options: Option<BrokerPipelineOptions>,
        progress_cb: F,
    ) -> Result<BrokerFetchSummary, BrokerPipelineError>
    where
        F: Fn(BrokerProgress),
    {
        let opts = options.unwrap_or_default();

        // 1. 決定目標股票清單（嚴格限定 TWSE 普通股：4 碼純數字且非 0 開頭）
        let raw_symbols: Vec<String> = match target_symbols {
            Some(syms) => syms,
            None => self
                .repo
                .get_distinct_symbols()
                .unwrap_or_default()
                .into_iter()
                .map(|(s, _, _)| s)
                .collect(),
        };

        let symbols = filter_twse_common_stock_symbols(&raw_symbols);
        if symbols.is_empty() {
            return Err(BrokerPipelineError::NoTargetSymbols);
        }

        let total_stocks = symbols.len();

        // 2. 斷點續傳檢核：比對資料庫當日已入庫之代號清單
        let completed_list = self
            .repo
            .get_broker_completed_symbols(trade_date)?;
        let completed_set: HashSet<String> = completed_list.into_iter().collect();

        let mut success_count = 0;
        let mut failed_count = 0;
        let mut skipped_count = 0;
        let mut cancelled = false;

        for (idx, symbol) in symbols.iter().enumerate() {
            // 檢查是否收到取消訊號
            if let Some(flag) = &self.cancel_flag {
                if flag.load(Ordering::Relaxed) {
                    cancelled = true;
                    log::info!("接收到使用者取消訊號，批次下載提前中斷");
                    break;
                }
            }

            // 斷點續傳：已完成者直接略過
            if completed_set.contains(symbol) {
                skipped_count += 1;
                progress_cb(BrokerProgress {
                    current_index: idx + 1,
                    total_stocks,
                    current_symbol: symbol.clone(),
                    success_count,
                    failed_count,
                    skipped_count,
                    message: format!("個股 {} 已存在當日資料，自動跳過", symbol),
                });
                continue;
            }

            // 執行網路抓取與辨識
            match self.client.fetch_stock_report(symbol, opts.max_retries).await {
                Ok(Some(csv_content)) => {
                    // 剖析雙欄 CSV
                    match parse_bsr_csv(&csv_content, symbol, trade_date) {
                        Ok(report) => {
                            if !report.records.is_empty() {
                                self.repo.upsert_broker_trades_batch(&report.records)?;
                            }
                            success_count += 1;
                            progress_cb(BrokerProgress {
                                current_index: idx + 1,
                                total_stocks,
                                current_symbol: symbol.clone(),
                                success_count,
                                failed_count,
                                skipped_count,
                                message: format!(
                                    "個股 {} 下載並入庫完成 (共 {} 筆明細)",
                                    symbol,
                                    report.records.len()
                                ),
                            });
                        }
                        Err(e) => {
                            failed_count += 1;
                            progress_cb(BrokerProgress {
                                current_index: idx + 1,
                                total_stocks,
                                current_symbol: symbol.clone(),
                                success_count,
                                failed_count,
                                skipped_count,
                                message: format!("個股 {} CSV 剖析錯誤: {}", symbol, e),
                            });
                        }
                    }
                }
                Ok(None) => {
                    // 當日無交易
                    success_count += 1;
                    progress_cb(BrokerProgress {
                        current_index: idx + 1,
                        total_stocks,
                        current_symbol: symbol.clone(),
                        success_count,
                        failed_count,
                        skipped_count,
                        message: format!("個股 {} 當日查無成交資料", symbol),
                    });
                }
                Err(e) => {
                    failed_count += 1;
                    let is_network_err = matches!(e, BsrClientError::Network(_));
                    progress_cb(BrokerProgress {
                        current_index: idx + 1,
                        total_stocks,
                        current_symbol: symbol.clone(),
                        success_count,
                        failed_count,
                        skipped_count,
                        message: format!("個股 {} 抓取失敗: {}", symbol, e),
                    });

                    // 異常連線指數退避：遭遇嚴重網路逾時或連線中斷時進行冷卻
                    if is_network_err && opts.cooldown_secs > 0 {
                        log::warn!(
                            "遭遇網路異常，啟動冷卻退避暫停 {} 秒...",
                            opts.cooldown_secs
                        );
                        tokio::time::sleep(Duration::from_secs(opts.cooldown_secs)).await;
                    }
                }
            }

            // 智慧隨機延遲（Jitter）
            if idx + 1 < total_stocks && opts.delay_max_ms > 0 {
                let delay = if opts.delay_max_ms > opts.delay_min_ms {
                    let mut rng = rand::thread_rng();
                    rng.gen_range(opts.delay_min_ms..=opts.delay_max_ms)
                } else {
                    opts.delay_min_ms
                };
                tokio::time::sleep(Duration::from_millis(delay)).await;
            }
        }

        Ok(BrokerFetchSummary {
            trade_date: trade_date.to_string(),
            total_stocks,
            success_count,
            failed_count,
            skipped_count,
            cancelled,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::MarketRepository;
    use crate::models::BrokerTradeRecord;
    use std::sync::atomic::AtomicBool;
    use std::sync::Mutex;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_broker_pipeline_resume_and_cancellation() {
        let repo = MarketRepository::open_in_memory().unwrap();

        let trade_date = "2026-10-08";

        // 先寫入 2330 當日資料以模擬斷點續傳
        let pre_record = BrokerTradeRecord {
            trade_date: trade_date.to_string(),
            symbol: "2330".to_string(),
            broker_id: "1020".to_string(),
            broker_name: "合庫".to_string(),
            price: 2550.0,
            buy_shares: 1000,
            sell_shares: 0,
        };
        repo.upsert_broker_trades_batch(&[pre_record]).unwrap();

        // 啟動 Mock 伺服器
        let mock_server = MockServer::start().await;
        let menu_html = r#"
            <html>
                <input type="hidden" id="__VIEWSTATE" value="VS" />
                <input type="hidden" id="__VIEWSTATEGENERATOR" value="GEN" />
                <input type="hidden" id="__EVENTVALIDATION" value="EV" />
                <img src='CaptchaImage.aspx?guid=xyz' />
            </html>
        "#;
        let fixture_path = format!(
            "{}/../tests/fixtures/captcha/live_cap.png",
            env!("CARGO_MANIFEST_DIR")
        );
        let cap_bytes = std::fs::read(fixture_path).unwrap();

        Mock::given(method("GET"))
            .and(path("/bsMenu.aspx"))
            .respond_with(ResponseTemplate::new(200).set_body_string(menu_html))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/CaptchaImage.aspx"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(cap_bytes))
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/bsMenu.aspx"))
            .respond_with(ResponseTemplate::new(200).set_body_string("<html>OK</html>"))
            .mount(&mock_server)
            .await;

        let mock_csv = "券商買賣股票成交價量資訊\n股票代碼,=\"2317\"\n序號,券商,價格,買進股數,賣出股數,,序號,券商,價格,買進股數,賣出股數\n\"1\",\"1020合庫\",\"200.0\",\"5000\",\"0\",,\"\",\"\",\"\",\"\",\"\"";
        Mock::given(method("GET"))
            .and(path("/bsContent.aspx"))
            .respond_with(ResponseTemplate::new(200).set_body_string(mock_csv))
            .mount(&mock_server)
            .await;

        let client = BsrClient::with_base_url(&mock_server.uri()).unwrap();
        let cancel_flag = Arc::new(AtomicBool::new(false));

        let pipeline = BrokerFetchPipeline::new(client, repo)
            .with_cancel_flag(cancel_flag.clone());

        let target_symbols = vec!["2330".to_string(), "2317".to_string()];
        let opts = BrokerPipelineOptions {
            delay_min_ms: 1,
            delay_max_ms: 5,
            max_retries: 2,
            cooldown_secs: 0,
        };

        let progress_logs = Arc::new(Mutex::new(Vec::new()));
        let logs_clone = progress_logs.clone();

        let summary = pipeline
            .fetch_date(
                trade_date,
                Some(target_symbols),
                Some(opts),
                move |p| {
                    logs_clone.lock().unwrap().push(p);
                },
            )
            .await
            .unwrap();

        assert_eq!(summary.total_stocks, 2);
        assert_eq!(summary.skipped_count, 1, "2330 應被自動跳過 (斷點續傳)");
        assert_eq!(summary.success_count, 1, "2317 應成功下載入庫");
        assert!(!summary.cancelled);

        // 驗證 2317 確實入庫
        let res_2317 = pipeline.repo.get_broker_trades("2317", trade_date).unwrap();
        assert_eq!(res_2317.len(), 1);
        assert_eq!(res_2317[0].price, 200.0);
    }
}
