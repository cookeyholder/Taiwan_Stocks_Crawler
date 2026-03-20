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
