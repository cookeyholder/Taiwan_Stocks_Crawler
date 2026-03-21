from __future__ import annotations

import datetime
import warnings
from typing import Any
from unittest.mock import MagicMock

import pytest

import market_crawler.pipeline as _pipeline
from market_crawler.normalize import clean_numeric, clean_symbol
from market_crawler.orchestrator import classify_day
from market_crawler.parsers import parse_tpex_daily_close, parse_twse_daily_close
from market_crawler.quality import BatchQualityReport


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


# ---------------------------------------------------------------------------
# Task 6 防呆測試：CLI 引數 / 路由 / market_closed / source_error 語意
# ---------------------------------------------------------------------------


def test_guardrail_backfill_negative_raises() -> None:
    """--backfill 負數 → 報錯退出"""
    with pytest.raises(SystemExit) as exc:
        _pipeline.main(["--backfill", "-1", "--db", ":memory:"])
    assert exc.value.code != 0


def test_guardrail_backfill_zero_raises() -> None:
    """--backfill 0 → 報錯退出"""
    with pytest.raises(SystemExit) as exc:
        _pipeline.main(["--backfill", "0", "--db", ":memory:"])
    assert exc.value.code != 0


def test_guardrail_today_and_start_are_mutually_exclusive() -> None:
    """--today 與 --start 互斥 → 報錯退出"""
    with pytest.raises(SystemExit) as exc:
        _pipeline.main(["--today", "--start", "2026-03-18", "--db", ":memory:"])
    assert exc.value.code != 0


def test_guardrail_date_wrong_format_yyyymmdd_raises() -> None:
    """--date 20260317 格式不含連字號 → 報錯退出"""
    with pytest.raises(SystemExit) as exc:
        _pipeline.main(["--date", "20260317", "--db", ":memory:"])
    assert exc.value.code != 0


def test_guardrail_twse_csv_url_has_yyyymmdd() -> None:
    """TWSE CSV API URL 中的日期格式必須是 YYYYMMDD（無連字號）"""
    url = _pipeline._build_twse_csv_url("2026-03-17")
    assert "20260317" in url
    assert "2026-03-17" not in url


def test_guardrail_market_closed_not_in_source_error_count() -> None:
    """market_closed 不計入 source_error_count"""
    report = BatchQualityReport()
    report.record_market_closed()
    report.record_market_closed()
    assert report.source_error_count == 0
    assert report.market_closed_count == 2


def test_guardrail_classify_day_market_closed_when_api_date_mismatch() -> None:
    """工作日 + API 回傳日期不符 → market_closed，不為 source_error"""
    result = classify_day("2026-03-20", set(), api_date="2026-03-19")  # Friday
    assert result == "market_closed"


def test_guardrail_classify_day_holiday_short_circuits_api_date() -> None:
    """已知假日 → holiday，即使 api_date 不符也不會被標 market_closed"""
    result = classify_day("2026-04-04", {"2026-04-04"}, api_date="2026-04-03")
    assert result == "holiday"


def test_guardrail_classify_day_weekend_not_affected_by_api_date() -> None:
    """週末 → weekend，api_date 不影響"""
    result = classify_day("2026-03-22", set(), api_date="2026-03-20")  # Sunday
    assert result == "weekend"


def test_guardrail_fetch_and_ingest_returns_none_for_market_closed(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """TWSE JSON API 回傳日期不符（今日路由）→ fetch_and_ingest_day 回傳 None"""
    today = datetime.date(2026, 3, 21)
    monkeypatch.setattr("market_crawler.pipeline._get_today", lambda: today)

    def mock_get(url: str, policy: Any) -> MagicMock:
        resp = MagicMock()
        # JSON API 回傳不同日期（前一交易日）
        resp.json.return_value = [{"Date": "1150318", "Code": "0050"}]
        resp.content = b""
        return resp

    monkeypatch.setattr("market_crawler.pipeline.get_with_retry", mock_get)

    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        result = _pipeline.fetch_and_ingest_day(
            "2026-03-21", db_path=":memory:", security_master_path=None
        )

    assert result is None, f"Expected None (market_closed signal), got {result!r}"


# ---------------------------------------------------------------------------
# Task 6.2 整合測試：--today 完整 mock 流程
# ---------------------------------------------------------------------------


def test_integration_today_full_flow_with_mocks(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """--today 完整流程：CLI 解析 → 路由判斷 → 假日抓取 → run() → BatchQualityReport 正確"""
    today = datetime.date(2026, 3, 21)
    monkeypatch.setattr("market_crawler.pipeline._get_today", lambda: today)

    # 自動假日抓取 mock（成功，添加一個假日）
    monkeypatch.setattr(
        "market_crawler.pipeline._fetch_twse_holidays",
        lambda _: {datetime.date(2026, 4, 4)},
    )

    called_with: dict[str, Any] = {}

    def mock_run(**kwargs: Any) -> tuple[list, BatchQualityReport]:
        called_with.update(kwargs)
        report = BatchQualityReport()
        report.record_success()
        return [], report

    monkeypatch.setattr("market_crawler.pipeline.run_date_range_ingestion", mock_run)
    monkeypatch.setattr("market_crawler.pipeline.make_fetcher", lambda **_: lambda _d: True)

    _pipeline.main(["--today", "--db", ":memory:"])

    # 驗證 run_date_range_ingestion 呼叫參數
    assert called_with.get("start_date") == "2026-03-21"
    assert called_with.get("end_date") == "2026-03-21"
    holidays = called_with.get("holidays", set())
    holidays_str = {d.isoformat() if isinstance(d, datetime.date) else d for d in holidays}
    assert "2026-04-04" in holidays_str, f"Auto holiday not merged: {holidays_str}"
