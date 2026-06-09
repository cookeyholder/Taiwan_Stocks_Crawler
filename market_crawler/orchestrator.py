from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass
from datetime import date, timedelta

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


def classify_day(
    day: str,
    holidays: set[str],
    *,
    api_date: str | None = None,
) -> str:
    """分類交易日類型。

    Args:
        day: ISO 格式日期，例如 ``"2026-03-18"``。
        holidays: 已知假日集合（ISO 格式日期字串）。
        api_date: API 實際回傳的交易日期。若提供且與 ``day`` 不符，
            表示當日未開市（颱風假等），回傳 ``"market_closed"``。

    Returns:
        ``"holiday"``、``"weekend"``、``"market_closed"``、或 ``"trading"``。
    """
    d = date.fromisoformat(day)
    if day in holidays:
        return "holiday"
    if d.weekday() >= 5:
        return "weekend"
    if api_date is not None and api_date != day:
        return "market_closed"
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
    fetcher: Callable[[str], bool | None],
) -> tuple[list[IngestionOutcome], BatchQualityReport]:
    """執行日期區間批次匯入。

    fetcher 回傳值語意：
    - ``True``  — 成功（data written）
    - ``False`` — 失敗（網路錯誤、格式解析失敗）→ source_error
    - ``None``  — API 有回應但交易日期不符請求日期 → market_closed
    """
    outcomes: list[IngestionOutcome] = []
    report = BatchQualityReport()

    try:
        for day in iter_date_strings(start_date, end_date):
            day_type = classify_day(day, holidays)
            if day_type != "trading":
                outcomes.append(IngestionOutcome(day_type=day_type, ok=True, message="skip"))
                continue

            try:
                result = fetcher(day)
            except Exception as exc:  # noqa: BLE001
                outcomes.append(IngestionOutcome(day_type="source_error", ok=False, message=str(exc)))
                report.record_failure("source_error")
                continue

            if result is None:
                # API 有回應但交易日期不符請求日期（颱風假等臨時休市）
                outcomes.append(
                    IngestionOutcome(day_type="market_closed", ok=True, message="market_closed")
                )
                report.record_market_closed()
            elif result:
                outcomes.append(IngestionOutcome(day_type="trading", ok=True, message="ok"))
                report.record_success()
            else:
                outcomes.append(IngestionOutcome(day_type="source_error", ok=False, message="failed"))
                report.record_failure("source_error")
    finally:
        if hasattr(fetcher, "close") and callable(fetcher.close):
            fetcher.close()

    return outcomes, report
