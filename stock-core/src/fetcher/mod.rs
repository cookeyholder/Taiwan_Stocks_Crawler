pub mod tpex;
pub mod twse;

use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use std::time::Duration;

const DEFAULT_USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
];

#[derive(Debug, Clone)]
pub struct HttpPolicy {
    pub timeout_secs: u64,
    pub retry_count: u32,
    pub backoff_secs: u64,
    pub ssl_verify: bool,
}

impl Default for HttpPolicy {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            retry_count: 3,
            backoff_secs: 1,
            ssl_verify: true,
        }
    }
}

pub struct HttpClient {
    client: reqwest::Client,
    policy: HttpPolicy,
}

impl HttpClient {
    pub fn new(policy: HttpPolicy) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(DEFAULT_USER_AGENTS[0]));
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(policy.timeout_secs))
            .danger_accept_invalid_certs(!policy.ssl_verify)
            .build()
            .expect("Failed to build HTTP client");
        Self { client, policy }
    }

    pub async fn get(&self, url: &str) -> Result<String, HttpError> {
        let mut last_error = HttpError::RequestFailed("no retry".into());
        for attempt in 0..=self.policy.retry_count {
            let ua = DEFAULT_USER_AGENTS[attempt as usize % DEFAULT_USER_AGENTS.len()];
            let mut headers = HeaderMap::new();
            headers.insert(USER_AGENT, HeaderValue::from_str(ua).unwrap());
            match self.client.get(url).headers(headers).send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        return resp
                            .text()
                            .await
                            .map_err(|e| HttpError::RequestFailed(e.to_string()));
                    } else if resp.status().is_server_error() && attempt < self.policy.retry_count {
                        last_error = HttpError::HttpStatus(resp.status().as_u16());
                    } else {
                        return Err(HttpError::HttpStatus(resp.status().as_u16()));
                    }
                }
                Err(e) => {
                    if e.is_timeout() && attempt < self.policy.retry_count {
                        last_error = HttpError::Timeout;
                    } else if e.is_connect() && attempt < self.policy.retry_count {
                        last_error = HttpError::ConnectionFailed(e.to_string());
                    } else {
                        return Err(HttpError::RequestFailed(e.to_string()));
                    }
                }
            }
            let delay = self.policy.backoff_secs * (2u64.pow(attempt));
            tokio::time::sleep(Duration::from_secs(delay)).await;
        }
        Err(last_error)
    }

    pub async fn get_bytes(&self, url: &str) -> Result<Vec<u8>, HttpError> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(DEFAULT_USER_AGENTS[0]));
        let resp = self
            .client
            .get(url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    HttpError::Timeout
                } else {
                    HttpError::RequestFailed(e.to_string())
                }
            })?;
        if resp.status().is_success() {
            Ok(resp
                .bytes()
                .await
                .map_err(|e| HttpError::RequestFailed(e.to_string()))?
                .to_vec())
        } else {
            Err(HttpError::HttpStatus(resp.status().as_u16()))
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error("HTTP status {0}")]
    HttpStatus(u16),
    #[error("timeout")]
    Timeout,
    #[error("connection failed: {0}")]
    ConnectionFailed(String),
    #[error("request failed: {0}")]
    RequestFailed(String),
}
