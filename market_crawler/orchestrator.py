from __future__ import annotations

from dataclasses import dataclass
from datetime import date, timedelta
from typing import Callable

from market_crawler.quality import BatchQualityReport


def iter_date_strings(start_date: str, end_date: str) -> list[str]:
    start = date.fromisoformat(start_date)
    end = date.fromisoformat(end_date)
    days: list[str] = []
    cursor = start
    while cursor <= end:
        days.append(cursor.isoformat())
        cursor += timedelta(days=1)
    return days


def classify_day(day: str, holidays: set[str]) -> str:
    d = date.fromisoformat(day)
    if day in holidays:
        return "holiday"
    if d.weekday() >= 5:
        return "weekend"
    return "trading"


@dataclass(slots=True)
class IngestionOutcome:
    day_type: str
    ok: bool
    message: str


def run_date_range_ingestion(
    *,
    start_date: str,
    end_date: str,
    holidays: set[str],
    fetcher: Callable[[str], bool],
) -> tuple[list[IngestionOutcome], BatchQualityReport]:
    outcomes: list[IngestionOutcome] = []
    report = BatchQualityReport()

    for day in iter_date_strings(start_date, end_date):
        day_type = classify_day(day, holidays)
        if day_type != "trading":
            outcomes.append(IngestionOutcome(day_type=day_type, ok=True, message="skip"))
            continue

        try:
            ok = fetcher(day)
        except Exception as exc:  # noqa: BLE001
            outcomes.append(IngestionOutcome(day_type="source_error", ok=False, message=str(exc)))
            report.record_failure("source_error")
            continue

        if ok:
            outcomes.append(IngestionOutcome(day_type="trading", ok=True, message="ok"))
            report.record_success()
        else:
            outcomes.append(IngestionOutcome(day_type="source_error", ok=False, message="failed"))
            report.record_failure("source_error")

    return outcomes, report
