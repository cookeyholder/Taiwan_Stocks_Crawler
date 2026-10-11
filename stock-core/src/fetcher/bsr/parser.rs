//! 集中市場買賣日報表雙欄 CSV 剖析器

use crate::models::BrokerTradeRecord;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum BsrParserError {
    #[error("CSV 內容為空或行數不足")]
    EmptyContent,
    #[error("找不到股票代號")]
    MissingSymbol,
    #[error("無效的資料格式: {0}")]
    InvalidFormat(String),
}

/// 剖析後之買賣日報表結果彙總
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedBsrReport {
    pub symbol: String,
    pub records: Vec<BrokerTradeRecord>,
    pub total_buy_volume: i64,
    pub total_sell_volume: i64,
}

/// 將券商字串分離為券商代碼（4 碼）與券商名稱（去除全形/半形空白）
/// 例如："1020合　　庫" -> ("1020", "合庫")
/// 例如："9B2Z台新安南" -> ("9B2Z", "台新安南")
pub fn split_broker_cell(cell: &str) -> (String, String) {
    let clean = cell.trim().trim_matches('"');
    let chars: Vec<char> = clean.chars().collect();
    if chars.len() >= 4 {
        let id: String = chars[..4].iter().collect();
        let name: String = chars[4..]
            .iter()
            .filter(|c| !c.is_whitespace() && **c != '\u{3000}')
            .collect();
        (id, name)
    } else {
        (clean.to_string(), String::new())
    }
}

/// 從 CSV 前幾行萃取股票代碼（例如從 `股票代碼,="2330"`）
pub fn extract_symbol_from_header(first_few_lines: &str) -> Option<String> {
    for line in first_few_lines.lines() {
        if line.contains("股票代碼") {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 2 {
                let raw_val = parts[1].trim();
                let clean = raw_val
                    .trim_start_matches('=')
                    .trim_matches('"')
                    .trim();
                if !clean.is_empty() {
                    return Some(clean.to_string());
                }
            }
        }
    }
    None
}

/// 剖析單側 5 欄數據（序號, 券商, 價格, 買進股數, 賣出股數）
fn parse_single_side(
    row: &[String],
    symbol: &str,
    trade_date: &str,
) -> Result<Option<BrokerTradeRecord>, BsrParserError> {
    if row.len() < 5 {
        return Ok(None);
    }

    let seq = row[0].trim().trim_matches('"');
    let broker_str = row[1].trim().trim_matches('"');

    // 若序號或券商為空，代表該側無成交明細
    if seq.is_empty() || broker_str.is_empty() {
        return Ok(None);
    }

    let price_str = row[2].trim().trim_matches('"').replace(',', "");
    let buy_str = row[3].trim().trim_matches('"').replace(',', "");
    let sell_str = row[4].trim().trim_matches('"').replace(',', "");

    let price: f64 = price_str.parse().map_err(|e| {
        BsrParserError::InvalidFormat(format!("價格解析失敗: '{}', 原因: {}", price_str, e))
    })?;

    let buy_qty: i64 = buy_str.parse().map_err(|e| {
        BsrParserError::InvalidFormat(format!("買進股數解析失敗: '{}', 原因: {}", buy_str, e))
    })?;

    let sell_qty: i64 = sell_str.parse().map_err(|e| {
        BsrParserError::InvalidFormat(format!("賣出股數解析失敗: '{}', 原因: {}", sell_str, e))
    })?;

    let (broker_id, broker_name) = split_broker_cell(broker_str);

    Ok(Some(BrokerTradeRecord {
        trade_date: trade_date.to_string(),
        symbol: symbol.to_string(),
        broker_id,
        broker_name,
        price,
        buy_shares: buy_qty,
        sell_shares: sell_qty,
    }))
}

/// 剖析集中市場當日交易資料雙欄 CSV
///
/// * `csv_content`: CSV 文字內容
/// * `symbol_hint`: 若已知個股代碼可傳入，若傳入空字串則自動由檔頭嘗試萃取
/// * `trade_date`: 交易日期，格式例如 "2026-10-08"
pub fn parse_bsr_csv(
    csv_content: &str,
    symbol_hint: &str,
    trade_date: &str,
) -> Result<ParsedBsrReport, BsrParserError> {
    if csv_content.trim().is_empty() {
        return Err(BsrParserError::EmptyContent);
    }

    let resolved_symbol = if !symbol_hint.trim().is_empty() {
        symbol_hint.trim().to_string()
    } else {
        extract_symbol_from_header(csv_content).ok_or(BsrParserError::MissingSymbol)?
    };

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(csv_content.as_bytes());

    let mut records = Vec::new();
    let mut total_buy_volume: i64 = 0;
    let mut total_sell_volume: i64 = 0;

    let mut line_index = 0;
    for result in rdr.records() {
        let record = match result {
            Ok(r) => r,
            Err(_) => continue,
        };

        line_index += 1;
        // 略過前 3 列（標題、股票代碼、欄位名稱）
        if line_index <= 3 {
            continue;
        }

        let fields: Vec<String> = record.iter().map(|s| s.to_string()).collect();
        if fields.is_empty() {
            continue;
        }

        // 解析左側（第 0 ~ 4 欄）
        if fields.len() >= 5 {
            if let Some(item) = parse_single_side(&fields[0..5], &resolved_symbol, trade_date)? {
                total_buy_volume += item.buy_shares;
                total_sell_volume += item.sell_shares;
                records.push(item);
            }
        }

        // 解析右側（第 6 ~ 10 欄，若存在）
        if fields.len() >= 11 {
            if let Some(item) = parse_single_side(&fields[6..11], &resolved_symbol, trade_date)? {
                total_buy_volume += item.buy_shares;
                total_sell_volume += item.sell_shares;
                records.push(item);
            }
        }
    }

    if line_index < 3 {
        return Err(BsrParserError::EmptyContent);
    }

    Ok(ParsedBsrReport {
        symbol: resolved_symbol,
        records,
        total_buy_volume,
        total_sell_volume,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_broker_cell() {
        let (id1, name1) = split_broker_cell("\"1020合　　庫\"");
        assert_eq!(id1, "1020");
        assert_eq!(name1, "合庫");

        let (id2, name2) = split_broker_cell("9B2Z台新安南");
        assert_eq!(id2, "9B2Z");
        assert_eq!(name2, "台新安南");

        let (id3, name3) = split_broker_cell("9B2v台新桃園");
        assert_eq!(id3, "9B2v");
        assert_eq!(name3, "台新桃園");

        let (id4, name4) = split_broker_cell("123");
        assert_eq!(id4, "123");
        assert_eq!(name4, "");
    }

    #[test]
    fn test_extract_symbol_from_header() {
        let content = "券商買賣股票成交價量資訊\n股票代碼,=\"2330\"\n序號,券商,價格,買進股數,賣出股數\n";
        let sym = extract_symbol_from_header(content);
        assert_eq!(sym, Some("2330".to_string()));
    }

    #[test]
    fn test_parse_bsr_csv_synthetic() {
        let content = "\
券商買賣股票成交價量資訊
股票代碼,=\"2330\"
序號,券商,價格,買進股數,賣出股數,,序號,券商,價格,買進股數,賣出股數
\"1\",\"1020合　　庫\",\"2550.00\",\"8000\",\"0\",,\"2\",\"1020合　　庫\",\"2555.00\",\"5018\",\"8\"
\"3\",\"1021合庫台中\",\"2560.00\",\"1000\",\"500\",,\"\",\"\",\"\",\"\",\"\"
";
        let report = parse_bsr_csv(content, "", "2026-10-08").unwrap();
        assert_eq!(report.symbol, "2330");
        assert_eq!(report.records.len(), 3);

        assert_eq!(report.records[0].broker_id, "1020");
        assert_eq!(report.records[0].broker_name, "合庫");
        assert_eq!(report.records[0].price, 2550.0);
        assert_eq!(report.records[0].buy_shares, 8000);
        assert_eq!(report.records[0].sell_shares, 0);

        assert_eq!(report.records[1].broker_id, "1020");
        assert_eq!(report.records[1].price, 2555.0);
        assert_eq!(report.records[1].buy_shares, 5018);
        assert_eq!(report.records[1].sell_shares, 8);

        assert_eq!(report.records[2].broker_id, "1021");
        assert_eq!(report.records[2].broker_name, "合庫台中");
        assert_eq!(report.records[2].price, 2560.0);
        assert_eq!(report.records[2].buy_shares, 1000);
        assert_eq!(report.records[2].sell_shares, 500);

        assert_eq!(report.total_buy_volume, 8000 + 5018 + 1000);
        assert_eq!(report.total_sell_volume, 0 + 8 + 500);
    }

    #[test]
    fn test_parse_bsr_csv_real_2330_sample() {
        let sample_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../tests/fixtures/bsr_2330_sample.csv"
        );
        let content = std::fs::read_to_string(sample_path)
            .expect("應成功讀取測試用 2330 分點交易 CSV");

        let report = parse_bsr_csv(&content, "2330", "2026-10-08").unwrap();
        assert_eq!(report.symbol, "2330");
        // 驗證解析出 4,431 筆明細，完全無遺漏
        assert_eq!(report.records.len(), 4431);

        // 驗證第一筆
        let first = &report.records[0];
        assert_eq!(first.broker_id, "1020");
        assert_eq!(first.broker_name, "合庫");
        assert_eq!(first.price, 2550.0);
        assert_eq!(first.buy_shares, 8000);
        assert_eq!(first.sell_shares, 0);

        // 驗證最後一筆 (第 4431 筆: "4431","9B2Z台新安南","2575.00","11","5")
        let last = &report.records[4430];
        assert_eq!(last.broker_id, "9B2Z");
        assert_eq!(last.broker_name, "台新安南");
        assert_eq!(last.price, 2575.0);
        assert_eq!(last.buy_shares, 11);
        assert_eq!(last.sell_shares, 5);

        assert!(report.total_buy_volume > 0);
        assert!(report.total_sell_volume > 0);
    }

    #[test]
    fn test_parse_bsr_csv_empty_or_invalid() {
        assert!(parse_bsr_csv("", "2330", "2026-10-08").is_err());
        assert!(parse_bsr_csv("   \n\n", "2330", "2026-10-08").is_err());
        assert!(parse_bsr_csv("第一行\n第二行", "", "2026-10-08").is_err());
    }

    #[test]
    fn test_parse_bsr_csv_number_error() {
        let invalid = "\
券商買賣股票成交價量資訊
股票代碼,=\"2330\"
序號,券商,價格,買進股數,賣出股數,,序號,券商,價格,買進股數,賣出股數
\"1\",\"1020合　　庫\",\"ABC\",\"8000\",\"0\",,\"\",\"\",\"\",\"\",\"\"
";
        let res = parse_bsr_csv(invalid, "2330", "2026-10-08");
        assert!(res.is_err());
    }
}
