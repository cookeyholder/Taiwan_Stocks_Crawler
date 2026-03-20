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
    assert report.failure_reasons["source_error"] == 1
