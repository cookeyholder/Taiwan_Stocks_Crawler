use encoding_rs::BIG5;
use scraper::{Html, Selector};

use crate::models::{InstitutionalRecord, ParsedRecord};

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
            instrument_type: None,
            source_file: url.to_string(),
            source_line_no: (i + 1) as i64,
            source_date: item["Date"].as_str().map(|s| s.to_string()),
        });
    }
    Ok(results)
}

/// Fetch historical market data from TWSE CSV API
pub async fn fetch_history_csv(
    http: &HttpClient,
    date_str: &str,
) -> Result<Vec<ParsedRecord>, HttpError> {
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
    if !cow.is_empty() && !cow.contains('\u{FFFD}') {
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

        let symbol = record[0]
            .trim_start_matches('=')
            .trim_matches('"')
            .trim()
            .to_string();
        if symbol.is_empty() || symbol == "證券代號" || !symbol.chars().all(|c| c.is_ascii_digit())
        {
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
            instrument_type: None,
            source_file: source_file.to_string(),
            source_line_no: line_no,
            source_date: None,
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
            let cells: Vec<String> = row
                .select(&cell_selector)
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
                instrument_type: None,
                source_file: String::new(),
                source_line_no: 0,
                source_date: None,
            });
        }
    }
    results
}

fn parse_shares(s: &str) -> i64 {
    let clean = s.replace([',', '+'], "");
    clean.trim().parse::<i64>().unwrap_or(0)
}

/// 解析 TWSE T86 三大法人買賣超日報 JSON 資料
pub fn parse_twse_t86_json(
    trade_date: &str,
    json_str: &str,
) -> Result<Vec<InstitutionalRecord>, String> {
    let root: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| format!("解析 T86 JSON 失敗: {}", e))?;

    let stat = root.get("stat").and_then(|v| v.as_str()).unwrap_or("");
    if stat != "OK" && stat != "ok" {
        return Ok(Vec::new());
    }

    let rows = root.get("data").and_then(|v| v.as_array()).or_else(|| {
        root.get("tables")
            .and_then(|t| t.get(0))
            .and_then(|t0| t0.get("data"))
            .and_then(|d| d.as_array())
    });

    let Some(rows) = rows else {
        return Ok(Vec::new());
    };

    let mut results = Vec::new();

    for row_val in rows {
        let Some(cols) = row_val.as_array() else {
            continue;
        };
        if cols.len() < 12 {
            continue;
        }

        let raw_symbol = cols[0].as_str().unwrap_or("").trim();
        let symbol = raw_symbol
            .trim_start_matches('=')
            .trim_matches('"')
            .trim()
            .to_string();

        if symbol.is_empty() || symbol == "0000" {
            continue;
        }

        let foreign_buy;
        let foreign_sell;
        let foreign_net;
        let trust_buy;
        let trust_sell;
        let trust_net;
        let dealer_buy;
        let dealer_sell;
        let dealer_net;
        let total_net;

        if cols.len() >= 19 {
            // 標準 19 欄格式（含外資自營商與自營商避險）
            let f_buy_main = parse_shares(cols[2].as_str().unwrap_or("0"));
            let f_sell_main = parse_shares(cols[3].as_str().unwrap_or("0"));
            let f_net_main = parse_shares(cols[4].as_str().unwrap_or("0"));
            let f_buy_dealer = parse_shares(cols[5].as_str().unwrap_or("0"));
            let f_sell_dealer = parse_shares(cols[6].as_str().unwrap_or("0"));
            let f_net_dealer = parse_shares(cols[7].as_str().unwrap_or("0"));

            foreign_buy = f_buy_main + f_buy_dealer;
            foreign_sell = f_sell_main + f_sell_dealer;
            foreign_net = f_net_main + f_net_dealer;

            trust_buy = parse_shares(cols[8].as_str().unwrap_or("0"));
            trust_sell = parse_shares(cols[9].as_str().unwrap_or("0"));
            trust_net = parse_shares(cols[10].as_str().unwrap_or("0"));

            dealer_net = parse_shares(cols[11].as_str().unwrap_or("0"));
            let d_buy_prop = parse_shares(cols[12].as_str().unwrap_or("0"));
            let d_sell_prop = parse_shares(cols[13].as_str().unwrap_or("0"));
            let d_buy_hedge = parse_shares(cols[15].as_str().unwrap_or("0"));
            let d_sell_hedge = parse_shares(cols[16].as_str().unwrap_or("0"));

            dealer_buy = d_buy_prop + d_buy_hedge;
            dealer_sell = d_sell_prop + d_sell_hedge;

            total_net = parse_shares(cols[18].as_str().unwrap_or("0"));
        } else {
            // 緊湊格式（外資、投信、自營商）
            foreign_buy = parse_shares(cols[2].as_str().unwrap_or("0"));
            foreign_sell = parse_shares(cols[3].as_str().unwrap_or("0"));
            foreign_net = parse_shares(cols[4].as_str().unwrap_or("0"));

            trust_buy = parse_shares(cols[5].as_str().unwrap_or("0"));
            trust_sell = parse_shares(cols[6].as_str().unwrap_or("0"));
            trust_net = parse_shares(cols[7].as_str().unwrap_or("0"));

            dealer_buy = parse_shares(cols[8].as_str().unwrap_or("0"));
            dealer_sell = parse_shares(cols[9].as_str().unwrap_or("0"));
            dealer_net = parse_shares(cols[10].as_str().unwrap_or("0"));

            total_net = foreign_net + trust_net + dealer_net;
        }

        results.push(InstitutionalRecord {
            trade_date: trade_date.to_string(),
            symbol,
            foreign_buy,
            foreign_sell,
            foreign_net,
            trust_buy,
            trust_sell,
            trust_net,
            dealer_buy,
            dealer_sell,
            dealer_net,
            total_net,
        });
    }

    Ok(results)
}

/// 自 TWSE 抓取三大法人買賣超日報
pub async fn fetch_institutional_twse(
    http: &HttpClient,
    date_str: &str,
) -> Result<Vec<InstitutionalRecord>, HttpError> {
    let date_compact = date_str.replace('-', "");
    let url = format!(
        "https://www.twse.com.tw/rwd/zh/fund/T86?response=json&date={}&selectType=ALLBUT0999",
        date_compact
    );
    let text = http.get(&url).await?;
    parse_twse_t86_json(date_str, &text)
        .map_err(|e| HttpError::RequestFailed(format!("T86 解析失敗: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_twse_t86_json() {
        let json = r#"{
            "stat": "OK",
            "date": "20240102",
            "title": "三大法人買賣超日報",
            "fields": ["證券代號","證券名稱","外陸資買進股數","外陸資賣出股數","外陸資買賣超股數","外資自營商買進股數","外資自營商賣出股數","外資自營商買賣超股數","投信買進股數","投信賣出股數","投信買賣超股數","自營商買賣超股數","自營商買進股數(自行買賣)","自營商賣出股數(自行買賣)","自營商買賣超股數(自行買賣)","自營商買進股數(避險)","自營商賣出股數(避險)","自營商買賣超股數(避險)","三大法人買賣超股數"],
            "data": [
                ["=\"2330\"","台積電","15,000,000","5,000,000","10,000,000","0","0","0","2,000,000","500,000","1,500,000","800,000","500,000","200,000","300,000","600,000","100,000","500,000","12,300,000"],
                ["2317","鴻海","3,000,000","4,000,000","-1,000,000","0","0","0","100,000","200,000","-100,000","-200,000","100,000","150,000","-50,000","50,000","200,000","-150,000","-1,300,000"]
            ]
        }"#;

        let records = parse_twse_t86_json("2024-01-02", json).unwrap();
        assert_eq!(records.len(), 2);

        let tsmc = &records[0];
        assert_eq!(tsmc.symbol, "2330");
        assert_eq!(tsmc.trade_date, "2024-01-02");
        assert_eq!(tsmc.foreign_net, 10_000_000);
        assert_eq!(tsmc.trust_net, 1_500_000);
        assert_eq!(tsmc.dealer_net, 800_000);
        assert_eq!(tsmc.total_net, 12_300_000);

        let foxconn = &records[1];
        assert_eq!(foxconn.symbol, "2317");
        assert_eq!(foxconn.foreign_net, -1_000_000);
        assert_eq!(foxconn.trust_net, -100_000);
        assert_eq!(foxconn.dealer_net, -200_000);
        assert_eq!(foxconn.total_net, -1_300_000);
    }

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
