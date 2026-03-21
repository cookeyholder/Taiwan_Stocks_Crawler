from __future__ import annotations

import pytest

from market_crawler.normalize import (
    clean_numeric,
    clean_symbol,
    normalize_record,
    parse_change_field,
    roc_to_ad_date,
)


@pytest.mark.parametrize(
    ("raw", "cleaned"),
    [
        ('="0050"', "0050"),
        ("0050", "0050"),
        (' "00679B" ', "00679B"),
        ("00625K", "00625K"),
        (" T2330 ", "T2330"),
    ],
)
def test_clean_symbol(raw: str, cleaned: str) -> None:
    assert clean_symbol(raw) == cleaned


@pytest.mark.parametrize(
    ("raw", "cleaned"),
    [
        ("1,234", "1234"),
        ("--", None),
        ("", None),
        (" 12.34 ", "12.34"),
        (None, None),
    ],
)
def test_clean_numeric(raw: str | None, cleaned: str | None) -> None:
    assert clean_numeric(raw) == cleaned


@pytest.mark.parametrize(
    ("roc", "ad"),
    [
        ("115/03/18", "2026-03-18"),
        ("114/12/31", "2025-12-31"),
        ("2026-03-18", "2026-03-18"),
        ("001/01/01", "1912-01-01"),
        ("1911/01/01", "1911-01-01"),
    ],
)
def test_roc_to_ad_date(roc: str, ad: str) -> None:
    assert roc_to_ad_date(roc) == ad


@pytest.mark.parametrize(
    ("raw", "expected_sign", "expected_value"),
    [
        ("+0.18", "+", 0.18),
        ("-0.04", "-", 0.04),
        ("0.00", None, 0.0),
        ("--", None, None),
        ("", None, None),
    ],
)
def test_parse_change_field(
    raw: str, expected_sign: str | None, expected_value: float | None
) -> None:
    sign, value = parse_change_field(raw)
    assert sign == expected_sign
    assert value == expected_value


def test_normalize_twse_record() -> None:
    record = {
        "symbol": '="0050"',
        "name": "元大台灣50",
        "volume": "78,000,193",
        "trade_count": "65,062",
        "turnover": "6,064,253,100",
        "open": "77.75",
        "high": "78.05",
        "low": "77.45",
        "close": "77.80",
        "change_sign": "+",
        "change_value": "1.15",
        "last_bid": "77.75",
        "last_bid_qty": "203",
        "last_ask": "77.80",
        "last_ask_qty": "115",
        "pe_ratio": "0.00",
        "source_file": "MI_INDEX_ALL_20260318.csv",
        "source_line_no": 314,
    }
    normalized = normalize_record(
        market_code="TWSE",
        trade_date="115/03/18",
        record=record,
        source_encoding="cp950",
        ingest_run_id="run-1",
        ingested_at="2026-03-19T00:00:00Z",
    )
    assert normalized["trade_date"] == "2026-03-18"
    assert normalized["symbol_raw"] == '="0050"'
    assert normalized["symbol_clean"] == "0050"
    assert normalized["volume_shares"] == 78000193


def test_normalize_tpex_record_with_missing_numeric() -> None:
    record = {
        "symbol": "00679B",
        "name": "元大美債20年",
        "close": "27.50",
        "change": "+0.18",
        "open": "--",
        "high": "27.54",
        "low": "27.41",
        "volume": "51,751,000",
        "turnover": "1,423,092,210",
        "trade_count": "12,128",
        "last_bid": "27.50",
        "last_bid_qty": "770",
        "last_ask": "27.51",
        "last_ask_qty": "5,953",
        "issued_shares": "7,862,192,000",
        "next_limit_up": "9,999.95",
        "next_limit_down": "0.01",
        "source_file": "SQUOTE_AL_1150318.csv",
        "source_line_no": 5,
    }
    normalized = normalize_record(
        market_code="TPEX",
        trade_date="115/03/18",
        record=record,
        source_encoding="cp950",
        ingest_run_id="run-2",
        ingested_at="2026-03-19T00:00:00Z",
    )
    assert normalized["trade_date"] == "2026-03-18"
    assert normalized["open_price"] is None
    assert normalized["change_sign"] == "+"


def test_roc_to_ad_invalid_format() -> None:
    with pytest.raises(ValueError):
        roc_to_ad_date("20260318")


def test_twse_change_sign_empty_string_normalized_to_none() -> None:
    """TWSE change_sign 為空字串時（收盤價未變動），應正規化為 None。"""
    record = {
        "symbol": "0050",
        "name": "元大台灣50",
        "volume": "1000",
        "open": "150.00",
        "high": "152.00",
        "low": "149.00",
        "close": "150.00",
        "change_sign": "",
        "change_value": "0.00",
        "turnover": "150000000",
        "trade_count": "500",
        "source_file": "MI_INDEX_ALL.csv",
        "source_line_no": 1,
    }
    normalized = normalize_record(
        market_code="TWSE",
        trade_date="114/01/02",
        record=record,
        source_encoding="cp950",
        ingest_run_id="run-1",
        ingested_at="2025-01-03T00:00:00Z",
    )
    assert normalized["change_sign"] is None


def test_twse_change_sign_whitespace_normalized_to_none() -> None:
    """TWSE change_sign 為空白字元時，應正規化為 None。"""
    record = {
        "symbol": "2330",
        "name": "台積電",
        "volume": "5000",
        "open": "900.00",
        "high": "910.00",
        "low": "895.00",
        "close": "900.00",
        "change_sign": " ",
        "change_value": "0",
        "turnover": "4500000000",
        "trade_count": "2000",
        "source_file": "MI_INDEX_ALL.csv",
        "source_line_no": 2,
    }
    normalized = normalize_record(
        market_code="TWSE",
        trade_date="114/01/02",
        record=record,
        source_encoding="cp950",
        ingest_run_id="run-1",
        ingested_at="2025-01-03T00:00:00Z",
    )
    assert normalized["change_sign"] is None


# --- 防呆：除息/除權當日漲跌欄位為文字 ---


@pytest.mark.parametrize(
    "marker",
    ["除息", "除權", "除息除權"],
)
def test_parse_change_field_ex_dividend_markers_return_none(marker: str) -> None:
    """TPEX 在除息/除權日，漲跌欄位會填入文字標記，應傳回 (None, None) 而非拋出例外。"""
    sign, value = parse_change_field(marker)
    assert sign is None
    assert value is None


def test_to_float_non_numeric_returns_none() -> None:
    """to_float 遇到非數字文字時應傳回 None，不應拋出 ValueError。"""
    from market_crawler.normalize import to_float

    assert to_float("除息") is None
    assert to_float("除權") is None
    assert to_float("N/A") is None
    assert to_float("abc") is None


def test_normalize_tpex_record_with_ex_dividend_change() -> None:
    """TPEX 除息當日的完整 normalize 流程不應拋出例外，change_sign 與 change 應為 None。"""
    record = {
        "symbol": "1234",
        "name": "測試股",
        "volume": "100,000",
        "open": "50.00",
        "high": "52.00",
        "low": "49.00",
        "close": "51.00",
        "change": "除息",
        "turnover": "5,000,000",
        "trade_count": "500",
        "avg_price": "50.50",
        "source_file": "tpex_2026-03-17",
        "source_line_no": 5,
    }
    normalized = normalize_record(
        market_code="TPEX",
        trade_date="115/03/17",
        record=record,
        source_encoding="utf-8",
        ingest_run_id="run-1",
        ingested_at="2026-03-17T00:00:00Z",
    )
    assert normalized["change_sign"] is None
    assert normalized["change_value"] is None
    assert normalized["close_price"] == 51.00


# ---------------------------------------------------------------------------
# TWSE JSON format (OpenAPI STOCK_DAY_ALL — 含 "change" 鍵而非 change_sign)
# ---------------------------------------------------------------------------


def test_normalize_twse_json_change_field_parsed() -> None:
    """TWSE JSON 格式的 record 含有 'change' 鍵時，應使用 parse_change_field 解析。"""
    record = {
        "symbol": "0050",
        "name": "元大台灣50",
        "volume": "72638922",
        "turnover": "5517624557",
        "open": "76.20",
        "high": "76.40",
        "low": "75.50",
        "close": "75.90",
        "change": "-0.1000",
        "trade_count": "101109",
        "source_file": "https://openapi.twse.com.tw/v1/exchangeReport/STOCK_DAY_ALL",
        "source_line_no": 1,
    }
    normalized = normalize_record(
        market_code="TWSE",
        trade_date="2026-03-20",
        record=record,
        source_encoding="utf-8",
        ingest_run_id="run-1",
        ingested_at="2026-03-20T00:00:00Z",
    )
    assert normalized["change_sign"] == "-"
    assert normalized["change_value"] == pytest.approx(0.1)
    assert normalized["symbol_clean"] == "0050"
    assert normalized["close_price"] == pytest.approx(75.90)


def test_normalize_twse_json_positive_change() -> None:
    """TWSE JSON 正漲無加號時，change_sign 應為 None，change_value 應為正值。"""
    record = {
        "symbol": "2330",
        "name": "台積電",
        "volume": "1000",
        "turnover": "1860000",
        "open": "1860.00",
        "high": "1870.00",
        "low": "1855.00",
        "close": "1865.00",
        "change": "5.0000",
        "trade_count": "500",
        "source_file": "url",
        "source_line_no": 1,
    }
    normalized = normalize_record(
        market_code="TWSE",
        trade_date="2026-03-20",
        record=record,
        source_encoding="utf-8",
        ingest_run_id="run-1",
        ingested_at="2026-03-20T00:00:00Z",
    )
    assert normalized["change_sign"] is None
    assert normalized["change_value"] == pytest.approx(5.0)


def test_normalize_twse_json_zero_change() -> None:
    """TWSE JSON change 為 '0.0000' 時，change_sign 應為 None，change_value 應為 0.0。"""
    record = {
        "symbol": "1101",
        "name": "台泥",
        "volume": "1000",
        "turnover": "100000",
        "open": "50.00",
        "high": "50.50",
        "low": "49.50",
        "close": "50.00",
        "change": "0.0000",
        "trade_count": "100",
        "source_file": "url",
        "source_line_no": 1,
    }
    normalized = normalize_record(
        market_code="TWSE",
        trade_date="2026-03-20",
        record=record,
        source_encoding="utf-8",
        ingest_run_id="run-1",
        ingested_at="2026-03-20T00:00:00Z",
    )
    assert normalized["change_sign"] is None
    assert normalized["change_value"] == pytest.approx(0.0)


def test_normalize_twse_json_empty_fields_become_none() -> None:
    """未成交時空字串欄位應 normalize 為 None。"""
    record = {
        "symbol": "00707R",
        "name": "期元大S&P日圓反1",
        "volume": "",
        "turnover": "",
        "open": "",
        "high": "",
        "low": "",
        "close": "",
        "change": "0.0000",
        "trade_count": "",
        "source_file": "url",
        "source_line_no": 1,
    }
    normalized = normalize_record(
        market_code="TWSE",
        trade_date="2026-03-20",
        record=record,
        source_encoding="utf-8",
        ingest_run_id="run-1",
        ingested_at="2026-03-20T00:00:00Z",
    )
    assert normalized["volume_shares"] is None
    assert normalized["close_price"] is None
    assert normalized["open_price"] is None
