use super::{HttpClient, HttpError};
use crate::models::ParsedRecord;

/// Fetch TPEX daily quotes JSON API
pub async fn fetch_daily_json(http: &HttpClient, date_str: &str) -> Result<Vec<ParsedRecord>, HttpError> {
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return Err(HttpError::RequestFailed("invalid date format".into()));
    }
    let year: i32 = parts[0].parse().unwrap_or(0);
    if year < 1911 {
        return Err(HttpError::RequestFailed("year out of range".into()));
    }
    let roc_year = (year - 1911).to_string();
    let month = parts[1];
    let day = parts[2];
    let url = format!(
        "https://www.tpex.org.tw/www/zh-tw/afterTrading/dailyQuotes?date={}/{}/{}&type=S&response=json",
        roc_year, month, day
    );
    let text = http.get(&url).await?;
    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| HttpError::RequestFailed(format!("TPEX JSON parse: {}", e)))?;
    let mut results = Vec::new();
    if let Some(tables) = json["tables"].as_array() {
        for table in tables {
            if let Some(title) = table["title"].as_str() {
                if !title.contains("上櫃股票") {
                    continue;
                }
            }
            if let Some(rows) = table["data"].as_array() {
                for (i, row) in rows.iter().enumerate() {
                    let fields: Vec<&str> = row.as_array()
                        .map(|arr| arr.iter().map(|v| v.as_str().unwrap_or("")).collect())
                        .unwrap_or_default();
                    if fields.len() < 12 {
                        continue;
                    }
                    let symbol = fields[0].trim().to_string();
                    if symbol.is_empty() {
                        continue;
                    }
                    results.push(ParsedRecord {
                        symbol,
                        name: Some(fields[1].trim().to_string()),
                        open: Some(fields[5].trim().to_string()),
                        high: Some(fields[6].trim().to_string()),
                        low: Some(fields[7].trim().to_string()),
                        close: Some(fields[8].trim().to_string()),
                        volume: Some(fields[10].trim().to_string()),
                        turnover: Some(fields[11].trim().to_string()),
                        trade_count: Some(fields[12].trim().to_string()),
                        change: Some(fields[4].trim().to_string()),
                        change_sign: None,
                        change_value: None,
                        last_bid: None,
                        last_bid_qty: None,
                        last_ask: None,
                        last_ask_qty: None,
                        pe_ratio: None,
                        issued_shares: fields.get(13).map(|s| s.trim().to_string()),
                        next_limit_up: None,
                        next_limit_down: None,
                        source_file: url.clone(),
                        source_line_no: (i + 1) as i64,
                    });
                }
            }
        }
    }
    Ok(results)
}
