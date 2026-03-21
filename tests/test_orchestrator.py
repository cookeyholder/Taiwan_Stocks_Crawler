from __future__ import annotations

from market_crawler.orchestrator import (
    classify_day,
    iter_date_strings,
    run_date_range_ingestion,
)


def test_iter_date_strings_inclusive_range() -> None:
    assert iter_date_strings("2026-03-18", "2026-03-20") == [
        "2026-03-18",
        "2026-03-19",
        "2026-03-20",
    ]


def test_classify_day_trading() -> None:
    assert classify_day("2026-03-18", holidays=set()) == "trading"


def test_classify_day_weekend() -> None:
    assert classify_day("2026-03-21", holidays=set()) == "weekend"


def test_classify_day_holiday_overrides_weekday() -> None:
    assert classify_day("2026-03-18", holidays={"2026-03-18"}) == "holiday"


def test_run_date_range_handles_source_failure() -> None:
    def fetcher(day: str) -> bool:
        if day == "2026-03-18":
            raise RuntimeError("upstream 500")
        return True

    outcomes, report = run_date_range_ingestion(
        start_date="2026-03-18",
        end_date="2026-03-21",
        holidays={"2026-03-20"},
        fetcher=fetcher,
    )
    assert len(outcomes) == 4
    assert any(o.day_type == "source_error" for o in outcomes)
    assert any(o.day_type == "holiday" for o in outcomes)
    assert any(o.day_type == "weekend" for o in outcomes)
    assert report.failed_count == 1


def test_run_date_range_handles_false_fetch_result() -> None:
    def fetcher(_day: str) -> bool:
        return False

    outcomes, report = run_date_range_ingestion(
        start_date="2026-03-18",
        end_date="2026-03-18",
        holidays=set(),
        fetcher=fetcher,
    )
    assert outcomes[0].day_type == "source_error"
    assert report.failed_count == 1


# ---------------------------------------------------------------------------
# market_closed 分類測試（task 1.1）
# ---------------------------------------------------------------------------


def test_classify_day_market_closed_when_api_date_mismatch() -> None:
    """工作日 + API 有回應 + 日期不符 → market_closed"""
    assert classify_day("2026-03-18", holidays=set(), api_date="2026-03-17") == "market_closed"


def test_classify_day_trading_when_api_date_matches() -> None:
    """工作日 + API 回傳日期相符 → trading"""
    assert classify_day("2026-03-18", holidays=set(), api_date="2026-03-18") == "trading"


def test_classify_day_trading_when_no_api_date() -> None:
    """工作日 + 無 API 日期（不做 API date 檢查）→ trading"""
    assert classify_day("2026-03-18", holidays=set()) == "trading"


def test_classify_day_weekend_not_affected_by_api_date() -> None:
    """週末即使有 api_date 也維持 weekend，不進入 market_closed 判斷"""
    assert classify_day("2026-03-21", holidays=set(), api_date="2026-03-20") == "weekend"


def test_classify_day_holiday_not_affected_by_api_date() -> None:
    """已知假日即使 api_date 不符也維持 holiday"""
    assert classify_day("2026-03-18", holidays={"2026-03-18"}, api_date="2026-03-17") == "holiday"


def test_run_date_range_market_closed_when_fetcher_returns_none() -> None:
    """fetcher 回傳 None → market_closed，不計入 source_error_count"""

    def fetcher(_day: str) -> None:
        return None

    outcomes, report = run_date_range_ingestion(
        start_date="2026-03-18",
        end_date="2026-03-18",
        holidays=set(),
        fetcher=fetcher,
    )
    assert outcomes[0].day_type == "market_closed"
    assert report.market_closed_count == 1
    assert report.failed_count == 0
    assert report.source_error_count == 0


def test_run_date_range_network_error_stays_source_error() -> None:
    """網路錯誤例外 → source_error，不受 market_closed 影響"""

    def fetcher(_day: str) -> bool:
        raise ConnectionError("upstream unreachable")

    outcomes, report = run_date_range_ingestion(
        start_date="2026-03-18",
        end_date="2026-03-18",
        holidays=set(),
        fetcher=fetcher,
    )
    assert outcomes[0].day_type == "source_error"
    assert report.source_error_count == 1
    assert report.market_closed_count == 0


def test_run_date_range_all_five_day_types() -> None:
    """混合場景：trading、holiday、market_closed、source_error、weekend 各一天"""
    call_count = {"n": 0}

    def fetcher(day: str) -> bool | None:
        call_count["n"] += 1
        if day == "2026-03-18":
            return True  # trading
        if day == "2026-03-19":
            return None  # market_closed
        if day == "2026-03-23":
            raise RuntimeError("network")  # source_error
        return True  # shouldn't reach

    outcomes, report = run_date_range_ingestion(
        start_date="2026-03-18",
        end_date="2026-03-23",
        holidays={"2026-03-20"},  # 週五假日
        fetcher=fetcher,
    )
    day_types = [o.day_type for o in outcomes]
    assert "trading" in day_types
    assert "market_closed" in day_types
    assert "holiday" in day_types
    assert "weekend" in day_types
    assert "source_error" in day_types
    assert report.success_count == 1
    assert report.market_closed_count == 1
    assert report.source_error_count == 1
    assert report.failure_reasons["source_error"] == 1
