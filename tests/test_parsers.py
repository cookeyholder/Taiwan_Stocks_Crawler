from __future__ import annotations

from market_crawler.parsers import (
    parse_tpex_daily_close,
    parse_twse_daily_close,
    parse_twse_daily_close_json,
)


def test_parse_twse_extracts_daily_close_section(twse_fixture_bytes: bytes) -> None:
    result = parse_twse_daily_close(twse_fixture_bytes, source_file="twse_sample.csv")
    assert result.market_code == "TWSE"
    assert result.errors == []
    assert len(result.records) == 2


def test_parse_twse_keeps_symbol_with_equals_prefix(twse_fixture_bytes: bytes) -> None:
    result = parse_twse_daily_close(twse_fixture_bytes, source_file="twse_sample.csv")
    assert result.records[0]["symbol"] == '="0050"'


def test_parse_tpex_reads_17_columns(tpex_fixture_bytes: bytes) -> None:
    result = parse_tpex_daily_close(tpex_fixture_bytes, source_file="tpex_sample.csv")
    assert result.market_code == "TPEX"
    assert result.errors == []
    assert len(result.records) == 2
    assert result.records[0]["symbol"] == "00679B"


def test_parse_tpex_cp950_decoding() -> None:
    raw = '代號,名稱,收盤 ,漲跌,開盤 ,最高 ,最低,成交股數  , 成交金額(元), 成交筆數 ,最後買價,最後買量(張數),最後賣價,最後賣量(張數),發行股數 ,次日漲停價 ,次日跌停價\n"1234","測試","1","+0.1","1","1","1","1","1","1","1","1","1","1","1","1","1"\n'
    result = parse_tpex_daily_close(raw.encode("cp950"), source_file="x.csv")
    assert result.source_encoding == "cp950"
    assert result.records[0]["name"] == "測試"


def test_parse_twse_big5hkscs_decoding() -> None:
    raw = '115年03月18日 每日收盤行情(全部)\n證券代號,證券名稱,成交股數,成交筆數,成交金額,開盤價,最高價,最低價,收盤價,漲跌(+/-),漲跌價差,最後揭示買價,最後揭示買量,最後揭示賣價,最後揭示賣量,本益比\n="0050",元大台灣50,1,1,1,1,1,1,1,+,0.1,1,1,1,1,1\n'
    result = parse_twse_daily_close(raw.encode("big5hkscs"), source_file="x.csv")
    assert result.source_encoding in {"cp950", "big5hkscs"}
    assert result.records[0]["symbol"] == '="0050"'


def test_parse_twse_reports_missing_header() -> None:
    result = parse_twse_daily_close(b"hello", source_file="x.csv")
    assert result.records == []
    assert result.errors


def test_parse_tpex_reports_missing_header() -> None:
    result = parse_tpex_daily_close(b"hello", source_file="x.csv")
    assert result.records == []
    assert result.errors


def test_parse_twse_records_error_on_short_row() -> None:
    raw = (
        "115年03月18日 每日收盤行情(全部)\n"
        "證券代號,證券名稱,成交股數,成交筆數,成交金額,開盤價,最高價,最低價,收盤價,漲跌(+/-),漲跌價差,最後揭示買價,最後揭示買量,最後揭示賣價,最後揭示賣量,本益比\n"
        "0050,元大台灣50,100\n"
    )
    result = parse_twse_daily_close(raw.encode("cp950"), source_file="x.csv")
    assert result.errors
    assert result.records == []


def test_parse_twse_stops_at_note_row() -> None:
    raw = (
        "115年03月18日 每日收盤行情(全部)\n"
        "證券代號,證券名稱,成交股數,成交筆數,成交金額,開盤價,最高價,最低價,收盤價,漲跌(+/-),漲跌價差,最後揭示買價,最後揭示買量,最後揭示賣價,最後揭示賣量,本益比\n"
        '="0050",元大台灣50,1,1,1,1,1,1,1,+,0.1,1,1,1,1,1\n'
        "說明,這是註解\n"
        "1101,台泥,1,1,1,1,1,1,1,+,0.1,1,1,1,1,1\n"
    )
    result = parse_twse_daily_close(raw.encode("cp950"), source_file="x.csv")
    assert len(result.records) == 1


def test_parse_tpex_short_row_records_error_and_skips_blank_rows() -> None:
    raw = (
        "代號,名稱,收盤 ,漲跌,開盤 ,最高 ,最低,成交股數  , 成交金額(元), 成交筆數 ,最後買價,最後買量(張數),最後賣價,最後賣量(張數),發行股數 ,次日漲停價 ,次日跌停價\n"
        "\n"
        "1234,測試,1,+0.1\n"
    )
    result = parse_tpex_daily_close(raw.encode("cp950"), source_file="x.csv")
    assert result.records == []
    assert result.errors


# ---------------------------------------------------------------------------
# parse_twse_daily_close_json — TWSE OpenAPI STOCK_DAY_ALL
# ---------------------------------------------------------------------------

_TWSE_JSON_SAMPLE = [
    {
        "Date": "1150320",
        "Code": "0050",
        "Name": "元大台灣50",
        "TradeVolume": "72638922",
        "TradeValue": "5517624557",
        "OpeningPrice": "76.20",
        "HighestPrice": "76.40",
        "LowestPrice": "75.50",
        "ClosingPrice": "75.90",
        "Change": "-0.1000",
        "Transaction": "101109",
    },
    {
        "Date": "1150320",
        "Code": "2330",
        "Name": "台積電",
        "TradeVolume": "53705722",
        "TradeValue": "99109745990",
        "OpeningPrice": "1860.00",
        "HighestPrice": "1870.00",
        "LowestPrice": "1840.00",
        "ClosingPrice": "1840.00",
        "Change": "-10.0000",
        "Transaction": "208042",
    },
]

_TWSE_JSON_UNTRADEABLE = {
    "Date": "1150320",
    "Code": "00707R",
    "Name": "期元大S&P日圓反1",
    "TradeVolume": "",
    "TradeValue": "",
    "OpeningPrice": "",
    "HighestPrice": "",
    "LowestPrice": "",
    "ClosingPrice": "",
    "Change": "0.0000",
    "Transaction": "",
}


def test_parse_twse_json_basic_parsing() -> None:
    """正常資料應解析出正確筆數與欄位。"""
    result = parse_twse_daily_close_json(_TWSE_JSON_SAMPLE, source_file="test_url")
    assert result.market_code == "TWSE"
    assert result.source_encoding == "utf-8"
    assert result.errors == []
    assert len(result.records) == 2


def test_parse_twse_json_field_mapping() -> None:
    """驗證 JSON 欄位對應到 record dict 鍵名。"""
    result = parse_twse_daily_close_json(_TWSE_JSON_SAMPLE, source_file="test_url")
    rec = result.records[0]
    assert rec["symbol"] == "0050"
    assert rec["name"] == "元大台灣50"
    assert rec["volume"] == "72638922"
    assert rec["turnover"] == "5517624557"
    assert rec["open"] == "76.20"
    assert rec["high"] == "76.40"
    assert rec["low"] == "75.50"
    assert rec["close"] == "75.90"
    assert rec["change"] == "-0.1000"
    assert rec["trade_count"] == "101109"


def test_parse_twse_json_change_negative() -> None:
    """Change 欄位應保留負號。"""
    result = parse_twse_daily_close_json(_TWSE_JSON_SAMPLE, source_file="test_url")
    assert result.records[0]["change"] == "-0.1000"
    assert result.records[1]["change"] == "-10.0000"


def test_parse_twse_json_accepts_empty_trade_fields() -> None:
    """當日未成交（欄位為空字串）的代號仍應被解析入庫。"""
    data = [_TWSE_JSON_UNTRADEABLE]
    result = parse_twse_daily_close_json(data, source_file="test_url")
    assert result.errors == []
    assert len(result.records) == 1
    assert result.records[0]["volume"] == ""
    assert result.records[0]["close"] == ""


def test_parse_twse_json_skips_empty_code() -> None:
    """代號為空的列應被跳過，不新增錯誤。"""
    data = [
        {"Date": "1150320", "Code": "", "Name": "空代號",
         "TradeVolume": "1", "TradeValue": "1", "OpeningPrice": "1",
         "HighestPrice": "1", "LowestPrice": "1", "ClosingPrice": "1",
         "Change": "0.0000", "Transaction": "1"},
    ]
    result = parse_twse_daily_close_json(data, source_file="test_url")
    assert result.records == []
    assert result.errors == []


def test_parse_twse_json_error_on_missing_keys() -> None:
    """缺少必要欄位時應記錄 ParseError。"""
    data = [{"Date": "1150320", "Code": "1234", "Name": "缺欄位"}]
    result = parse_twse_daily_close_json(data, source_file="test_url")
    assert result.records == []
    assert len(result.errors) == 1
    assert "缺少欄位" in result.errors[0].reason


def test_parse_twse_json_error_on_non_list_input() -> None:
    """傳入非 list 型態時應回傳空結果並帶有 ParseError。"""
    result = parse_twse_daily_close_json({"key": "val"}, source_file="test_url")
    assert result.records == []
    assert len(result.errors) == 1
    assert "非 list" in result.errors[0].reason


def test_parse_twse_json_error_on_non_dict_item() -> None:
    """list 中有非 dict 元素時應記錄 ParseError。"""
    result = parse_twse_daily_close_json(["not_a_dict"], source_file="test_url")
    assert result.records == []
    assert len(result.errors) == 1


def test_parse_twse_json_source_file_recorded() -> None:
    """source_file 應被記錄到每筆 record 中。"""
    result = parse_twse_daily_close_json(_TWSE_JSON_SAMPLE, source_file="https://example.com/api")
    assert result.records[0]["source_file"] == "https://example.com/api"


def test_parse_twse_json_source_line_no_starts_at_one() -> None:
    """source_line_no 應從 1 開始遞增。"""
    result = parse_twse_daily_close_json(_TWSE_JSON_SAMPLE, source_file="url")
    assert result.records[0]["source_line_no"] == 1
    assert result.records[1]["source_line_no"] == 2


def test_parse_twse_json_empty_input() -> None:
    """空 list 輸入應回傳空 records 且無錯誤。"""
    result = parse_twse_daily_close_json([], source_file="url")
    assert result.records == []
    assert result.errors == []


def test_parse_twse_json_mixed_valid_and_invalid() -> None:
    """有效與無效列混合時，有效列應解析成功，無效列應記錄錯誤。"""
    data = [
        *_TWSE_JSON_SAMPLE,
        {"Date": "1150320", "Code": "9999", "Name": "不完整"},  # 缺必要欄位
    ]
    result = parse_twse_daily_close_json(data, source_file="url")
    assert len(result.records) == 2
    assert len(result.errors) == 1
