from __future__ import annotations

import pytest

from market_crawler.normalize import clean_numeric, clean_symbol
from market_crawler.parsers import parse_tpex_daily_close, parse_twse_daily_close


@pytest.mark.parametrize(
    ("content", "expected_errors"),
    [
        (b"", 1),
        (b"random text", 1),
    ],
)
def test_empty_or_invalid_twse_file(content: bytes, expected_errors: int) -> None:
    result = parse_twse_daily_close(content, source_file="x.csv")
    assert len(result.errors) >= expected_errors


def test_empty_or_invalid_tpex_file() -> None:
    result = parse_tpex_daily_close(b"", source_file="x.csv")
    assert result.errors


def test_missing_columns_twse_is_reported() -> None:
    raw = (
        '115年03月18日 每日收盤行情(全部)\n證券代號,證券名稱,成交股數\n="0050",元大台灣50,100\n'
    ).encode("cp950")
    result = parse_twse_daily_close(raw, source_file="x.csv")
    assert result.errors


def test_missing_columns_tpex_is_reported() -> None:
    raw = ("代號,名稱,收盤\n0050,元大台灣50,77.8\n").encode("cp950")
    result = parse_tpex_daily_close(raw, source_file="x.csv")
    assert result.errors


@pytest.mark.parametrize("value", ["--", "", "   ", None])
def test_guardrail_numeric_empty(value: str | None) -> None:
    assert clean_numeric(value) is None


def test_regression_symbol_not_lost_when_prefixed_equals() -> None:
    raw = '="0050"'
    assert clean_symbol(raw) == "0050"


def test_guardrail_symbol_plain() -> None:
    assert clean_symbol("0050") == "0050"


def test_guardrail_symbol_with_spaces() -> None:
    assert clean_symbol(' "0050" ') == "0050"


def test_guardrail_symbol_otc_bond_etf_code() -> None:
    assert clean_symbol("00679B") == "00679B"


def test_guardrail_symbol_structured_product() -> None:
    assert clean_symbol("03456P") == "03456P"


def test_guardrail_twse_double_dash_price_not_crash() -> None:
    raw = (
        "115年03月18日 每日收盤行情(全部)\n"
        "證券代號,證券名稱,成交股數,成交筆數,成交金額,開盤價,最高價,最低價,收盤價,漲跌(+/-),漲跌價差,最後揭示買價,最後揭示買量,最後揭示賣價,最後揭示賣量,本益比\n"
        '="00625K",富邦上証+R,0,0,0,--,--,--,--, ,0.00,8.73,5,8.83,5,0.00\n'
    ).encode("cp950")
    result = parse_twse_daily_close(raw, source_file="x.csv")
    assert len(result.records) == 1
