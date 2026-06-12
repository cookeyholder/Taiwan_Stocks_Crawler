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
    parse_tpex_json(&text, &url)
}

fn parse_tpex_json(text: &str, source_file: &str) -> Result<Vec<ParsedRecord>, HttpError> {
    let json: serde_json::Value = serde_json::from_str(text)
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
                    if fields.len() < 11 {
                        continue;
                    }
                    let symbol = fields[0].trim().to_string();
                    if symbol.is_empty() {
                        continue;
                    }
                    results.push(ParsedRecord {
                        symbol,
                        name: Some(fields[1].trim().to_string()),
                        open: Some(fields[4].trim().to_string()),
                        high: Some(fields[5].trim().to_string()),
                        low: Some(fields[6].trim().to_string()),
                        close: Some(fields[2].trim().to_string()),
                        volume: Some(fields[8].trim().to_string()),
                        turnover: Some(fields[9].trim().to_string()),
                        trade_count: Some(fields[10].trim().to_string()),
                        change: Some(fields[3].trim().to_string()),
                        change_sign: None,
                        change_value: None,
                        last_bid: None,
                        last_bid_qty: None,
                        last_ask: None,
                        last_ask_qty: None,
                        pe_ratio: None,
                        issued_shares: fields.get(15).map(|s| s.trim().to_string()),
                        next_limit_up: None,
                        next_limit_down: None,
                        source_file: source_file.to_string(),
                        source_line_no: (i + 1) as i64,
                        source_date: None,
                    });
                }
            }
        }
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tpex_json() -> String {
        r#"{
  "tables": [
    {
      "title": "上櫃股票",
      "data": [
        ["3105","穩懋","150.00","-2.00","152.00","153.00","149.50","151.20","5000000","756000000","3500","149.50","100","150.00","200","250000000"],
        ["6488","環球晶","450.00","5.00","445.00","455.00","443.00","449.50","3000000","1350000000","2800","449.00","50","450.00","100","200000000"],
        ["8069","元太","180.00","1.50","179.00","182.00","178.50","180.50","8000000","1444000000","6200","180.00","80","180.50","120","150000000"]
      ]
    }
  ]
}"#.to_string()
    }

    #[test]
    fn test_parse_tpex_json_correct_field_mapping() {
        let records = parse_tpex_json(&sample_tpex_json(), "test.json").unwrap();
        assert_eq!(records.len(), 3);

        assert_eq!(records[0].symbol, "3105");
        assert_eq!(records[0].name.as_deref(), Some("穩懋"));
        assert_eq!(records[0].open.as_deref(), Some("152.00"));
        assert_eq!(records[0].high.as_deref(), Some("153.00"));
        assert_eq!(records[0].low.as_deref(), Some("149.50"));
        assert_eq!(records[0].close.as_deref(), Some("150.00"));
        assert_eq!(records[0].change.as_deref(), Some("-2.00"));
        assert_eq!(records[0].volume.as_deref(), Some("5000000"));
        assert_eq!(records[0].turnover.as_deref(), Some("756000000"));
        assert_eq!(records[0].trade_count.as_deref(), Some("3500"));
        assert_eq!(records[0].issued_shares.as_deref(), Some("250000000"));

        assert_eq!(records[1].symbol, "6488");
        assert_eq!(records[1].open.as_deref(), Some("445.00"));
        assert_eq!(records[1].close.as_deref(), Some("450.00"));
        assert_eq!(records[1].change.as_deref(), Some("5.00"));

        assert_eq!(records[2].symbol, "8069");
        assert_eq!(records[2].close.as_deref(), Some("180.00"));
        assert_eq!(records[2].volume.as_deref(), Some("8000000"));
    }

    #[test]
    fn test_parse_tpex_json_skips_short_rows() {
        let json = r#"{
  "tables": [{
    "title": "上櫃股票",
    "data": [
      ["3105","穩懋"],
      ["3105","穩懋","150.00","-2.00","152.00","153.00","149.50","151.20","5000000","756000000","3500","149.50","100","150.00","200","250000000"]
    ]
  }]
}"#;
        let records = parse_tpex_json(json, "test.json").unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].symbol, "3105");
        assert_eq!(records[0].close.as_deref(), Some("150.00"));
    }

    #[test]
    fn test_parse_tpex_json_skips_empty_symbol() {
        let json = r#"{
  "tables": [{
    "title": "上櫃股票",
    "data": [
      ["","","","","","","","","","","","","","","",""]
    ]
  }]
}"#;
        let records = parse_tpex_json(json, "test.json").unwrap();
        assert_eq!(records.len(), 0);
    }

    #[test]
    fn test_parse_tpex_json_invalid_json() {
        let result = parse_tpex_json("not json", "test.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_tpex_json_issued_shares_fallback_none() {
        let json = r#"{
  "tables": [{
    "title": "上櫃股票",
    "data": [
      ["3105","穩懋","150.00","-2.00","152.00","153.00","149.50","151.20","5000000","756000000","3500","149.50","100","150.00","200"]
    ]
  }]
}"#;
        let records = parse_tpex_json(json, "test.json").unwrap();
        assert_eq!(records.len(), 1);
        assert!(records[0].issued_shares.is_none());
    }
}
