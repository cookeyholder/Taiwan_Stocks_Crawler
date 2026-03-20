from __future__ import annotations

from market_crawler.parsers import parse_tpex_daily_close, parse_twse_daily_close


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
