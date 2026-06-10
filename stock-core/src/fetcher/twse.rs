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

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(text.as_bytes());

    for (line_no, result) in reader.records().enumerate() {
        let line_no = (line_no + 1) as i64;
        let record = match result {
            Ok(r) => r,
            Err(_) => continue,
        };

        if record.is_empty() || (record.len() == 1 && record[0].trim().is_empty()) {
            continue;
        }

        let joined: String = record.iter().collect();
        if joined.contains("每日收盤行情") || joined.contains("證券代號") {
            in_data = true;
            continue;
        }
        if !in_data || joined.starts_with('＊') || joined.starts_with('|') {
            continue;
        }
        if record.len() < 16 {
            continue;
        }

        let symbol = record[0].trim_matches('=').trim().to_string();
        if symbol.is_empty() || symbol == "證券代號" || !symbol.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        results.push(ParsedRecord {
            symbol,
            name: Some(record[1].to_string()),
            open: Some(record[5].to_string()),
            high: Some(record[6].to_string()),
            low: Some(record[7].to_string()),
            close: Some(record[8].to_string()),
            volume: Some(record[2].to_string()),
            turnover: Some(record[3].to_string()),
            trade_count: Some(record[4].to_string()),
            change: None,
            change_sign: Some(record[9].to_string()),
            change_value: Some(record[10].to_string()),
            last_bid: Some(record[11].to_string()),
            last_bid_qty: Some(record[12].to_string()),
            last_ask: Some(record[13].to_string()),
            last_ask_qty: Some(record[14].to_string()),
            pe_ratio: Some(record[15].to_string()),
            issued_shares: record.get(16).map(|s| s.to_string()),
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
