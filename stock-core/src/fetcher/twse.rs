use encoding_rs::BIG5;
use scraper::{Html, Selector};

use crate::models::ParsedRecord;

use super::{HttpClient, HttpError};

/// Fetch today's market data from TWSE OpenAPI (JSON, no date param)
pub async fn fetch_today_json(http: &HttpClient) -> Result<Vec<ParsedRecord>, HttpError> {
    let url = "https://openapi.twse.com.tw/v1/exchangeReport/STOCK_DAY_ALL";
    let text = http.get(url).await?;
    let records: Vec<serde_json::Value> = serde_json::from_str(&text)
        .map_err(|e| HttpError::RequestFailed(format!("JSON parse: {}", e)))?;
    let mut results = Vec::new();
    for (i, item) in records.iter().enumerate() {
        let symbol = item["Code"].as_str().unwrap_or("").to_string();
        if symbol.is_empty() || symbol == "0000" {
            continue;
        }
        results.push(ParsedRecord {
            symbol,
            name: item["Name"].as_str().map(|s| s.to_string()),
            open: item["OpeningPrice"].as_str().map(|s| s.to_string()),
            high: item["HighestPrice"].as_str().map(|s| s.to_string()),
            low: item["LowestPrice"].as_str().map(|s| s.to_string()),
            close: item["ClosingPrice"].as_str().map(|s| s.to_string()),
            volume: item["TradeVolume"].as_str().map(|s| s.to_string()),
            turnover: item["TradeValue"].as_str().map(|s| s.to_string()),
            trade_count: item["Transaction"].as_str().map(|s| s.to_string()),
            change: item["Change"].as_str().map(|s| s.to_string()),
            change_sign: None,
            change_value: None,
            last_bid: None,
            last_bid_qty: None,
            last_ask: None,
            last_ask_qty: None,
            pe_ratio: None,
            issued_shares: None,
            next_limit_up: None,
            next_limit_down: None,
            source_file: url.to_string(),
            source_line_no: (i + 1) as i64,
        });
    }
    Ok(results)
}

/// Fetch historical market data from TWSE CSV API
pub async fn fetch_history_csv(http: &HttpClient, date_str: &str) -> Result<Vec<ParsedRecord>, HttpError> {
    let date_compact = date_str.replace("-", "");
    let url = format!(
        "https://www.twse.com.tw/rwd/zh/afterTrading/MI_INDEX?response=csv&date={}&type=ALL",
        date_compact
    );
    let bytes = http.get_bytes(&url).await?;
    let text = decode_big5_fallback(&bytes);
    let records = parse_twse_csv(&text, &url);
    Ok(records)
}

fn decode_big5_fallback(bytes: &[u8]) -> String {
    let (cow, _) = BIG5.decode_without_bom_handling(bytes);
    if cow.len() > 0 && !cow.contains('\u{FFFD}') {
        return cow.into_owned();
    }
    String::from_utf8_lossy(bytes).to_string()
}

fn parse_twse_csv(text: &str, source_file: &str) -> Vec<ParsedRecord> {
    let mut results = Vec::new();
    let mut in_data = false;
    let mut line_no = 0;
    for line in text.lines() {
        line_no += 1;
        let trimmed = line.trim();
        if trimmed.contains("每日收盤行情") || trimmed.contains("證券代號") {
            in_data = true;
            continue;
        }
        if !in_data || trimmed.is_empty() || trimmed.starts_with("＊") || trimmed.starts_with("|") {
            continue;
        }
        let fields: Vec<&str> = trimmed.split(',').collect();
        if fields.len() < 16 {
            continue;
        }
        let symbol = fields[0].trim_matches('"').trim().to_string();
        if symbol.is_empty() || symbol == "證券代號" || !symbol.chars().all(|c| c.is_ascii_digit() || c == '"' || c == '=') {
            continue;
        }
        let name = fields[1].trim_matches('"').trim().to_string();
        results.push(ParsedRecord {
            symbol,
            name: Some(name),
            open: Some(fields[5].trim_matches('"').to_string()),
            high: Some(fields[6].trim_matches('"').to_string()),
            low: Some(fields[7].trim_matches('"').to_string()),
            close: Some(fields[8].trim_matches('"').to_string()),
            volume: Some(fields[2].trim_matches('"').to_string()),
            turnover: Some(fields[3].trim_matches('"').to_string()),
            trade_count: Some(fields[4].trim_matches('"').to_string()),
            change: None,
            change_sign: Some(fields[9].trim_matches('"').to_string()),
            change_value: Some(fields[10].trim_matches('"').to_string()),
            last_bid: Some(fields[11].trim_matches('"').to_string()),
            last_bid_qty: Some(fields[12].trim_matches('"').to_string()),
            last_ask: Some(fields[13].trim_matches('"').to_string()),
            last_ask_qty: Some(fields[14].trim_matches('"').to_string()),
            pe_ratio: Some(fields[15].trim_matches('"').to_string()),
            issued_shares: fields.get(16).map(|s| s.trim_matches('"').to_string()),
            next_limit_up: None,
            next_limit_down: None,
            source_file: source_file.to_string(),
            source_line_no: line_no,
        });
    }
    results
}

/// Parse TWSE HTML response for Big5-encoded pages
pub fn parse_twse_html(html: &str) -> Vec<ParsedRecord> {
    let document = Html::parse_document(html);
    let table_selector = Selector::parse("table").unwrap();
    let row_selector = Selector::parse("tr").unwrap();
    let cell_selector = Selector::parse("td").unwrap();

    let mut results = Vec::new();
    for table in document.select(&table_selector) {
        for row in table.select(&row_selector) {
            let cells: Vec<String> = row.select(&cell_selector)
                .map(|c| c.text().collect::<String>().trim().to_string())
                .collect();
            if cells.len() < 8 {
                continue;
            }
            let symbol = cells[1].trim().to_string();
            if symbol.is_empty() || !symbol.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            results.push(ParsedRecord {
                symbol,
                name: Some(cells[2].clone()),
                open: Some(cells[3].clone()),
                high: Some(cells[4].clone()),
                low: Some(cells[5].clone()),
                close: Some(cells[6].clone()),
                volume: Some(cells[7].clone()),
                turnover: None,
                trade_count: None,
                change: None,
                change_sign: None,
                change_value: None,
                last_bid: None,
                last_bid_qty: None,
                last_ask: None,
                last_ask_qty: None,
                pe_ratio: None,
                issued_shares: None,
                next_limit_up: None,
                next_limit_down: None,
                source_file: String::new(),
                source_line_no: 0,
            });
        }
    }
    results
}
