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

        let symbol = record[0].trim_start_matches('=').trim_matches('"').trim().to_string();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_twse_csv_parses_quoted_fields() {
        let csv = "\"證券代號\",\"證券名稱\",\"成交股數\",\"成交金額\",\"成交筆數\",\"開盤價\",\"最高價\",\"最低價\",\"收盤價\",\"漲跌(+/-)\",\"漲跌價差\",\"最後揭示買價\",\"最後揭示買量\",\"最後揭示賣價\",\"最後揭示賣量\",\"本益比\"\n\"1101\",\"台泥\",\"10,053,434\",\"304,574,770\",\"4,723\",\"30.30\",\"30.35\",\"30.15\",\"30.20\",\"+\",\"0.10\",\"30.20\",\"10\",\"30.25\",\"30\",\"10.75\"\n\"2330\",\"台積電\",\"20000000\",\"1200000000\",\"15000\",\"500.00\",\"505.00\",\"498.00\",\"502.00\",\"+\",\"2.00\",\"502.00\",\"20\",\"502.50\",\"15\",\"25.50\"\n=\"0050\",\"元大台灣50\",\"5000000\",\"300000000\",\"2500\",\"60.00\",\"60.50\",\"59.80\",\"60.20\",\"+\",\"0.20\",\"60.20\",\"5\",\"60.30\",\"10\",\"\"";
        let records = parse_twse_csv(csv, "test.csv");
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].symbol, "1101");
        assert_eq!(records[0].name.as_deref(), Some("台泥"));
        assert_eq!(records[0].volume.as_deref(), Some("10,053,434"));
        assert_eq!(records[1].symbol, "2330");
        assert_eq!(records[1].name.as_deref(), Some("台積電"));
        assert_eq!(records[1].close.as_deref(), Some("502.00"));
        assert_eq!(records[2].symbol, "0050");
        assert_eq!(records[2].name.as_deref(), Some("元大台灣50"));
    }

    #[test]
    fn test_parse_twse_csv_skips_metadata_lines() {
        let csv = "\"每日收盤行情\"\n\"證券代號\",\"證券名稱\",\"成交股數\",\"成交金額\",\"成交筆數\",\"開盤價\",\"最高價\",\"最低價\",\"收盤價\",\"漲跌(+/-)\",\"漲跌價差\",\"最後揭示買價\",\"最後揭示買量\",\"最後揭示賣價\",\"最後揭示賣量\",\"本益比\"\n\"1101\",\"台泥\",\"10000000\",\"300000000\",\"5000\",\"30.00\",\"30.50\",\"29.80\",\"30.20\",\"+\",\"0.20\",\"30.20\",\"10\",\"30.30\",\"15\",\"10.00\"\n=\"0050\",\"元大台灣50\",\"5000000\",\"300000000\",\"2500\",\"60.00\",\"60.50\",\"59.80\",\"60.20\",\"+\",\"0.20\",\"60.20\",\"5\",\"60.30\",\"10\",\"\"\n\"＊ 資料來源：臺灣證券交易所\"\n\"| 製表時間：2024/01/02\"";
        let records = parse_twse_csv(csv, "test.csv");
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].symbol, "1101");
        assert_eq!(records[1].symbol, "0050");
    }

    #[test]
    fn test_parse_twse_csv_issued_shares() {
        let csv = "\"證券代號\",\"證券名稱\",\"成交股數\",\"成交金額\",\"成交筆數\",\"開盤價\",\"最高價\",\"最低價\",\"收盤價\",\"漲跌(+/-)\",\"漲跌價差\",\"最後揭示買價\",\"最後揭示買量\",\"最後揭示賣價\",\"最後揭示賣量\",\"本益比\",\"發行量\"\n\"1101\",\"台泥\",\"10000000\",\"300000000\",\"5000\",\"30.00\",\"30.50\",\"29.80\",\"30.20\",\"+\",\"0.20\",\"30.20\",\"10\",\"30.30\",\"15\",\"10.00\",\"50000000\"";
        let records = parse_twse_csv(csv, "test.csv");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].issued_shares.as_deref(), Some("50000000"));
    }

    #[test]
    fn test_parse_twse_csv_no_issued_shares() {
        let csv = "\"證券代號\",\"證券名稱\",\"成交股數\",\"成交金額\",\"成交筆數\",\"開盤價\",\"最高價\",\"最低價\",\"收盤價\",\"漲跌(+/-)\",\"漲跌價差\",\"最後揭示買價\",\"最後揭示買量\",\"最後揭示賣價\",\"最後揭示賣量\",\"本益比\"\n\"1101\",\"台泥\",\"10000000\",\"300000000\",\"5000\",\"30.00\",\"30.50\",\"29.80\",\"30.20\",\"+\",\"0.20\",\"30.20\",\"10\",\"30.30\",\"15\",\"10.00\"";
        let records = parse_twse_csv(csv, "test.csv");
        assert_eq!(records.len(), 1);
        assert!(records[0].issued_shares.is_none());
    }

    #[test]
    fn test_parse_twse_html_extracts_table_data() {
        let html = "<html><body><table>\n<tr><td>1</td><td>1101</td><td>台泥</td><td>30.30</td><td>30.35</td><td>30.15</td><td>30.20</td><td>10053434</td></tr>\n<tr><td>2</td><td>2330</td><td>台積電</td><td>500.00</td><td>505.00</td><td>498.00</td><td>502.00</td><td>20000000</td></tr>\n</table></body></html>";
        let records = parse_twse_html(html);
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].symbol, "1101");
        assert_eq!(records[0].name.as_deref(), Some("台泥"));
        assert_eq!(records[0].close.as_deref(), Some("30.20"));
        assert_eq!(records[1].symbol, "2330");
        assert_eq!(records[1].name.as_deref(), Some("台積電"));
    }
}
