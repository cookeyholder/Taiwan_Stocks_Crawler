//! 集中市場券商日報表 BSR 網路連線管理與客戶端
//!
//! 負責連線 Session 管理（Cookie Jar）、GET bsMenu.aspx 表單狀態抓取、
//! 驗證碼下載與辨識、POST 查詢提交與雙欄 CSV 內容下載，並內建換碼重試機制。

use super::captcha::{CaptchaError, CaptchaSolver};
use reqwest::header::{HeaderMap, HeaderValue, ORIGIN, REFERER, USER_AGENT};
use std::time::Duration;
use thiserror::Error;

const DEFAULT_USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

#[derive(Debug, Error)]
pub enum BsrClientError {
    #[error("網路連線錯誤: {0}")]
    Network(#[from] reqwest::Error),
    #[error("解析 HTML 失敗，缺少欄位: {0}")]
    HtmlParseError(String),
    #[error("驗證碼處理失敗: {0}")]
    Captcha(#[from] CaptchaError),
    #[error("驗證碼重試已達上限 ({0} 次)")]
    CaptchaExceededRetries(u32),
}

/// ASP.NET WebForm 表單狀態資訊
#[derive(Debug, Clone, PartialEq)]
pub struct BsrFormState {
    pub viewstate: String,
    pub viewstate_generator: String,
    pub event_validation: String,
    pub captcha_url: String,
}

/// POST 查詢回應狀態
#[derive(Debug, Clone, PartialEq)]
pub enum BsrQueryResult {
    /// 查詢成功，已生成報表
    Success,
    /// 驗證碼錯誤，需換碼重試
    CaptchaError,
    /// 查無資料或當日非交易日
    NoData,
}

/// BSR 客戶端
pub struct BsrClient {
    client: reqwest::Client,
    base_url: String,
}

impl BsrClient {
    /// 建立新的 BSR 客戶端
    pub fn new() -> Result<Self, BsrClientError> {
        Self::with_base_url("https://bsr.twse.com.tw/bshtm")
    }

    /// 使用自訂 base_url 建立客戶端（主要用於測試與 Mock 伺服器）
    pub fn with_base_url(base_url: &str) -> Result<Self, BsrClientError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(DEFAULT_USER_AGENT),
        );

        let client = reqwest::Client::builder()
            .cookie_store(true)
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }

    /// 從 HTML 內容中萃取表單狀態參數與 Captcha 圖片網址
    pub fn parse_form_state_from_html(html: &str) -> Result<BsrFormState, BsrClientError> {
        let extract_attr = |name: &str| -> Option<String> {
            let pattern = format!("id=\"{}\"", name);
            if let Some(pos) = html.find(&pattern) {
                let slice = &html[pos..];
                if let Some(val_pos) = slice.find("value=\"") {
                    let val_slice = &slice[val_pos + 7..];
                    if let Some(end_pos) = val_slice.find('"') {
                        return Some(val_slice[..end_pos].to_string());
                    }
                }
            }
            None
        };

        let viewstate = extract_attr("__VIEWSTATE")
            .ok_or_else(|| BsrClientError::HtmlParseError("__VIEWSTATE".to_string()))?;
        let viewstate_generator = extract_attr("__VIEWSTATEGENERATOR")
            .unwrap_or_default();
        let event_validation = extract_attr("__EVENTVALIDATION")
            .ok_or_else(|| BsrClientError::HtmlParseError("__EVENTVALIDATION".to_string()))?;

        // 尋找 CaptchaImage.aspx?guid=...
        let captcha_url = if let Some(src_pos) = html.find("CaptchaImage.aspx?guid=") {
            let slice = &html[src_pos..];
            let end_char = slice.find(|c| c == '\'' || c == '"' || c == ' ' || c == '>').unwrap_or(slice.len());
            slice[..end_char].to_string()
        } else {
            return Err(BsrClientError::HtmlParseError("CaptchaImage.aspx".to_string()));
        };

        Ok(BsrFormState {
            viewstate,
            viewstate_generator,
            event_validation,
            captcha_url,
        })
    }

    /// 初次訪問 bsMenu.aspx 取得 WebForm 表單狀態與驗證碼 URL
    pub async fn fetch_form_state(&self) -> Result<BsrFormState, BsrClientError> {
        let menu_url = format!("{}/bsMenu.aspx", self.base_url);
        let resp = self.client.get(&menu_url).send().await?;
        let html = resp.text().await?;
        Self::parse_form_state_from_html(&html)
    }

    /// 下載驗證碼圖片原始位元組
    pub async fn fetch_captcha_image(&self, captcha_url: &str) -> Result<Vec<u8>, BsrClientError> {
        let full_url = if captcha_url.starts_with("http") {
            captcha_url.to_string()
        } else {
            format!("{}/{}", self.base_url, captcha_url.trim_start_matches('/'))
        };

        let menu_url = format!("{}/bsMenu.aspx", self.base_url);
        let resp = self
            .client
            .get(&full_url)
            .header(REFERER, menu_url)
            .send()
            .await?;

        let bytes = resp.bytes().await?;
        Ok(bytes.to_vec())
    }

    /// 提交查詢表單
    pub async fn submit_query(
        &self,
        state: &BsrFormState,
        symbol: &str,
        captcha_code: &str,
    ) -> Result<BsrQueryResult, BsrClientError> {
        let menu_url = format!("{}/bsMenu.aspx", self.base_url);

        let params = [
            ("__VIEWSTATE", state.viewstate.as_str()),
            ("__VIEWSTATEGENERATOR", state.viewstate_generator.as_str()),
            ("__EVENTVALIDATION", state.event_validation.as_str()),
            ("__VIEWSTATEENCRYPTED", ""),
            ("RadioButton_Normal", "RadioButton_Normal"),
            ("TextBox_Stkno", symbol),
            ("CaptchaControl1", captcha_code),
            ("btnOK", "查詢"),
        ];

        let resp = self
            .client
            .post(&menu_url)
            .header(REFERER, &menu_url)
            .header(ORIGIN, &self.base_url)
            .form(&params)
            .send()
            .await?;

        let html = resp.text().await?;

        if html.contains("驗證碼不符") || html.contains("驗證碼錯誤") {
            Ok(BsrQueryResult::CaptchaError)
        } else if html.contains("查無資料")
            || html.contains("查無當日成交資料")
            || html.contains("非交易日")
            || html.contains("股票代號不存在")
        {
            Ok(BsrQueryResult::NoData)
        } else {
            Ok(BsrQueryResult::Success)
        }
    }

    /// 下載 bsContent.aspx 的 CSV 內容（自動辨識 Big5 或 UTF-8 編碼）
    pub async fn fetch_csv_content(&self) -> Result<String, BsrClientError> {
        let content_url = format!("{}/bsContent.aspx", self.base_url);
        let menu_url = format!("{}/bsMenu.aspx", self.base_url);

        let resp = self
            .client
            .get(&content_url)
            .header(REFERER, menu_url)
            .send()
            .await?;

        let raw_bytes = resp.bytes().await?;

        // 先檢查是否已為有效 UTF-8 且包含關鍵字
        if let Ok(utf8_str) = std::str::from_utf8(&raw_bytes) {
            if utf8_str.contains("券商") || utf8_str.contains("股票") {
                return Ok(utf8_str.to_string());
            }
        }

        // 以 BIG-5 解碼
        let (cow, _, _) = encoding_rs::BIG5.decode(&raw_bytes);
        Ok(cow.into_owned())
    }

    /// 執行單一個股抓取流程（包含自動換碼重試）
    ///
    /// * 若成功抓取，回傳 `Ok(Some(csv_content))`
    /// * 若當日查無成交資料，回傳 `Ok(None)`
    /// * 若驗證碼重試超過次數，回傳 `Err(BsrClientError::CaptchaExceededRetries)`
    pub async fn fetch_stock_report(
        &self,
        symbol: &str,
        max_retries: u32,
    ) -> Result<Option<String>, BsrClientError> {
        for attempt in 1..=max_retries {
            log::debug!("正在嘗試抓取個股 {} (第 {}/{} 次)", symbol, attempt, max_retries);

            // 1. 取得最新表單狀態
            let state = self.fetch_form_state().await?;

            // 2. 下載驗證碼圖片
            let img_bytes = self.fetch_captcha_image(&state.captcha_url).await?;

            // 3. 辨識驗證碼
            let captcha_code = match CaptchaSolver::solve(&img_bytes) {
                Ok(code) => code,
                Err(e) => {
                    log::warn!("驗證碼辨識失敗: {}，即將重試", e);
                    continue;
                }
            };

            // 4. 提交查詢
            match self.submit_query(&state, symbol, &captcha_code).await? {
                BsrQueryResult::Success => {
                    let csv = self.fetch_csv_content().await?;
                    return Ok(Some(csv));
                }
                BsrQueryResult::NoData => {
                    return Ok(None);
                }
                BsrQueryResult::CaptchaError => {
                    log::info!("驗證碼 '{}' 不符，重新換碼...", captcha_code);
                    continue;
                }
            }
        }

        Err(BsrClientError::CaptchaExceededRetries(max_retries))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn test_parse_form_state_from_html() {
        let html = r#"
            <html>
                <input type="hidden" name="__VIEWSTATE" id="__VIEWSTATE" value="VS_VAL_123" />
                <input type="hidden" name="__VIEWSTATEGENERATOR" id="__VIEWSTATEGENERATOR" value="GEN_456" />
                <input type="hidden" name="__EVENTVALIDATION" id="__EVENTVALIDATION" value="EV_789" />
                <img src='CaptchaImage.aspx?guid=abc-123-xyz' />
            </html>
        "#;

        let state = BsrClient::parse_form_state_from_html(html).unwrap();
        assert_eq!(state.viewstate, "VS_VAL_123");
        assert_eq!(state.viewstate_generator, "GEN_456");
        assert_eq!(state.event_validation, "EV_789");
        assert_eq!(state.captcha_url, "CaptchaImage.aspx?guid=abc-123-xyz");
    }

    #[test]
    fn test_parse_form_state_missing_field() {
        let html = "<html>no fields</html>";
        let res = BsrClient::parse_form_state_from_html(html);
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_bsr_client_flow_with_mock_server() {
        let mock_server = MockServer::start().await;

        let sample_html = r#"
            <html>
                <input type="hidden" id="__VIEWSTATE" value="MOCK_VS" />
                <input type="hidden" id="__VIEWSTATEGENERATOR" value="MOCK_GEN" />
                <input type="hidden" id="__EVENTVALIDATION" value="MOCK_EV" />
                <img src='CaptchaImage.aspx?guid=mock-guid-999' />
            </html>
        "#;

        // 載入測試用真實 Captcha 圖片
        let fixture_path = format!(
            "{}/../tests/fixtures/captcha/live_cap.png",
            env!("CARGO_MANIFEST_DIR")
        );
        let cap_bytes = std::fs::read(fixture_path).expect("需存在 live_cap.png");

        // Mock 1: GET /bsMenu.aspx
        Mock::given(method("GET"))
            .and(path("/bsMenu.aspx"))
            .respond_with(ResponseTemplate::new(200).set_body_string(sample_html))
            .mount(&mock_server)
            .await;

        // Mock 2: GET /CaptchaImage.aspx
        Mock::given(method("GET"))
            .and(path("/CaptchaImage.aspx"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(cap_bytes))
            .mount(&mock_server)
            .await;

        // Mock 3: POST /bsMenu.aspx (成功回應)
        let post_success_html = r#"
            <html>
                <a href="bsContent.aspx">下載</a>
            </html>
        "#;
        Mock::given(method("POST"))
            .and(path("/bsMenu.aspx"))
            .respond_with(ResponseTemplate::new(200).set_body_string(post_success_html))
            .mount(&mock_server)
            .await;

        // Mock 4: GET /bsContent.aspx
        let mock_csv = "券商買賣股票成交價量資訊\n股票代碼,=\"2330\"\n序號,券商,價格,買進股數,賣出股數";
        Mock::given(method("GET"))
            .and(path("/bsContent.aspx"))
            .respond_with(ResponseTemplate::new(200).set_body_string(mock_csv))
            .mount(&mock_server)
            .await;

        let client = BsrClient::with_base_url(&mock_server.uri()).unwrap();
        let report = client.fetch_stock_report("2330", 3).await.unwrap();

        assert!(report.is_some());
        let csv_text = report.unwrap();
        assert!(csv_text.contains("2330"));
        assert!(csv_text.contains("券商"));
    }
}
