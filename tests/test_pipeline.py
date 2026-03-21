"""tests/test_pipeline.py — pipeline.py 的單元測試。

涵蓋 TWSE 日期路由、CLI 引數解析、自動假日抓取等核心邏輯。
"""

from __future__ import annotations

import datetime
from typing import Any
from unittest.mock import MagicMock

import pytest

import market_crawler.pipeline as _pipeline
from market_crawler.http_client import HttpPolicy
from market_crawler.parsers import ParseResult

# ---------------------------------------------------------------------------
# 輔助函式
# ---------------------------------------------------------------------------


def _make_policy() -> HttpPolicy:
    return HttpPolicy(
        timeout_seconds=5.0,
        retry_count=1,
        backoff_seconds=0.0,
        headers={},
        ssl_verify=False,
    )


def _empty_parse_result() -> ParseResult:
    """回傳空的 ParseResult（無資料列、無錯誤）。"""
    return ParseResult(records=[], errors=[], source_encoding="utf-8")


def _mock_response(json_data: Any = None, content: bytes = b"") -> MagicMock:
    resp = MagicMock()
    resp.json.return_value = json_data if json_data is not None else []
    resp.content = content
    return resp


# ---------------------------------------------------------------------------
# 任務 2.1（& 2.4）: TWSE 日期路由測試
# ---------------------------------------------------------------------------


class TestTwseDateRouting:
    """TWSE 資料來源路由：今日 → JSON API；歷史 → CSV API。"""

    def test_today_route_calls_json_api(self, monkeypatch: pytest.MonkeyPatch) -> None:
        """date == today → 呼叫 TWSE JSON API（STOCK_DAY_ALL），不呼叫 CSV API"""
        today = datetime.date(2026, 3, 21)
        monkeypatch.setattr("market_crawler.pipeline._get_today", lambda: today)

        captured: list[str] = []

        def mock_get(url: str, policy: Any) -> MagicMock:
            captured.append(url)
            return _mock_response(json_data=[])

        monkeypatch.setattr("market_crawler.pipeline.get_with_retry", mock_get)
        monkeypatch.setattr(
            "market_crawler.pipeline._ingest_records",
            lambda **_: 0,
        )

        _pipeline.fetch_and_ingest_day(
            "2026-03-21", db_path=":memory:", security_master_path=None
        )

        assert any(
            "STOCK_DAY_ALL" in url for url in captured
        ), f"JSON API not called: {captured}"
        assert not any(
            "MI_INDEX" in url for url in captured
        ), "CSV API should NOT be called for today"

    def test_historical_route_calls_csv_api(
        self, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        """date < today → 呼叫 TWSE CSV API（MI_INDEX），不呼叫 JSON API"""
        today = datetime.date(2026, 3, 21)
        monkeypatch.setattr("market_crawler.pipeline._get_today", lambda: today)

        captured: list[str] = []

        def mock_get(url: str, policy: Any) -> MagicMock:
            captured.append(url)
            return _mock_response(content=b"")

        monkeypatch.setattr("market_crawler.pipeline.get_with_retry", mock_get)
        monkeypatch.setattr(
            "market_crawler.pipeline._ingest_records",
            lambda **_: 0,
        )

        _pipeline.fetch_and_ingest_day(
            "2026-03-18", db_path=":memory:", security_master_path=None
        )

        assert any(
            "MI_INDEX" in url for url in captured
        ), f"CSV API not called: {captured}"
        assert not any(
            "STOCK_DAY_ALL" in url for url in captured
        ), "JSON API should NOT be called for historical"

    def test_csv_api_url_has_yyyymmdd_format(
        self, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        """CSV API URL 中的日期參數為 YYYYMMDD（無連字號）"""
        today = datetime.date(2026, 3, 21)
        monkeypatch.setattr("market_crawler.pipeline._get_today", lambda: today)

        captured: list[str] = []

        def mock_get(url: str, policy: Any) -> MagicMock:
            captured.append(url)
            return _mock_response(content=b"")

        monkeypatch.setattr("market_crawler.pipeline.get_with_retry", mock_get)
        monkeypatch.setattr(
            "market_crawler.pipeline._ingest_records",
            lambda **_: 0,
        )

        _pipeline.fetch_and_ingest_day(
            "2026-01-02", db_path=":memory:", security_master_path=None
        )

        csv_urls = [u for u in captured if "MI_INDEX" in u]
        assert csv_urls, "MI_INDEX URL not found"
        assert "20260102" in csv_urls[0], f"YYYYMMDD format not found in {csv_urls[0]}"
        assert (
            "2026-01-02" not in csv_urls[0]
        ), "Hyphenated date should NOT appear in CSV URL"

    def test_tpex_routing_unchanged_for_historical(
        self, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        """TPEX 歷史日期仍走既有 CSV 路徑（tpex.org.tw），不走 MI_INDEX"""
        today = datetime.date(2026, 3, 21)
        monkeypatch.setattr("market_crawler.pipeline._get_today", lambda: today)

        captured: list[str] = []

        def mock_get(url: str, policy: Any) -> MagicMock:
            captured.append(url)
            return _mock_response(content=b"", json_data={})

        monkeypatch.setattr("market_crawler.pipeline.get_with_retry", mock_get)
        monkeypatch.setattr(
            "market_crawler.pipeline._ingest_records",
            lambda **_: 0,
        )

        _pipeline.fetch_and_ingest_day(
            "2026-01-02", db_path=":memory:", security_master_path=None
        )

        tpex_urls = [u for u in captured if "tpex.org.tw" in u]
        assert tpex_urls, "TPEX URL not found"
        assert not any(
            "MI_INDEX" in u for u in tpex_urls
        ), "TPEX should NOT use MI_INDEX"


# ---------------------------------------------------------------------------
# 任務 3.1: CLI 引數測試（--today / --date / --backfill / 互斥）
# ---------------------------------------------------------------------------


class TestCliArguments:
    """main() CLI 引數解析測試。"""

    def test_today_flag_runs_today(self, monkeypatch: pytest.MonkeyPatch) -> None:
        """--today 等同今日起訖區間"""
        today = datetime.date(2026, 3, 21)
        monkeypatch.setattr("market_crawler.pipeline._get_today", lambda: today)

        called_with: dict[str, Any] = {}

        def mock_run(**kwargs: Any) -> tuple[list, Any]:
            called_with.update(kwargs)
            from market_crawler.quality import BatchQualityReport

            return [], BatchQualityReport()

        monkeypatch.setattr(
            "market_crawler.pipeline.run_date_range_ingestion", mock_run
        )
        monkeypatch.setattr(
            "market_crawler.pipeline.make_fetcher", lambda **_: lambda _d: True
        )

        _pipeline.main(["--today", "--db", ":memory:"])

        assert called_with.get("start_date") == "2026-03-21"
        assert called_with.get("end_date") == "2026-03-21"

    def test_date_flag_single_day(self, monkeypatch: pytest.MonkeyPatch) -> None:
        """--date YYYY-MM-DD 等同單日起訖"""
        called_with: dict[str, Any] = {}

        def mock_run(**kwargs: Any) -> tuple[list, Any]:
            called_with.update(kwargs)
            from market_crawler.quality import BatchQualityReport

            return [], BatchQualityReport()

        monkeypatch.setattr(
            "market_crawler.pipeline.run_date_range_ingestion", mock_run
        )
        monkeypatch.setattr(
            "market_crawler.pipeline.make_fetcher", lambda **_: lambda _d: True
        )
        monkeypatch.setattr(
            "market_crawler.pipeline._get_today", lambda: datetime.date(2026, 3, 21)
        )

        _pipeline.main(["--date", "2026-03-17", "--db", ":memory:"])

        assert called_with.get("start_date") == "2026-03-17"
        assert called_with.get("end_date") == "2026-03-17"

    def test_backfill_flag_n_days(self, monkeypatch: pytest.MonkeyPatch) -> None:
        """--backfill 30 等同 today - 30 days 至 today"""
        today = datetime.date(2026, 3, 21)
        monkeypatch.setattr("market_crawler.pipeline._get_today", lambda: today)

        called_with: dict[str, Any] = {}

        def mock_run(**kwargs: Any) -> tuple[list, Any]:
            called_with.update(kwargs)
            from market_crawler.quality import BatchQualityReport

            return [], BatchQualityReport()

        monkeypatch.setattr(
            "market_crawler.pipeline.run_date_range_ingestion", mock_run
        )
        monkeypatch.setattr(
            "market_crawler.pipeline.make_fetcher", lambda **_: lambda _d: True
        )

        _pipeline.main(["--backfill", "30", "--db", ":memory:"])

        assert called_with.get("start_date") == "2026-02-19"
        assert called_with.get("end_date") == "2026-03-21"

    def test_today_and_start_are_mutually_exclusive(
        self, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        """--today 與 --start 同時指定 → 報錯退出"""
        monkeypatch.setattr(
            "market_crawler.pipeline._get_today", lambda: datetime.date(2026, 3, 21)
        )

        with pytest.raises(SystemExit) as exc_info:
            _pipeline.main(["--today", "--start", "2026-03-18", "--db", ":memory:"])
        assert exc_info.value.code != 0

    def test_date_wrong_format_raises(self, monkeypatch: pytest.MonkeyPatch) -> None:
        """--date 20260317（非 YYYY-MM-DD）→ 報錯退出"""
        monkeypatch.setattr(
            "market_crawler.pipeline._get_today", lambda: datetime.date(2026, 3, 21)
        )

        with pytest.raises(SystemExit) as exc_info:
            _pipeline.main(["--date", "20260317", "--db", ":memory:"])
        assert exc_info.value.code != 0

    def test_backfill_zero_raises(self, monkeypatch: pytest.MonkeyPatch) -> None:
        """--backfill 0 → 報錯退出"""
        monkeypatch.setattr(
            "market_crawler.pipeline._get_today", lambda: datetime.date(2026, 3, 21)
        )

        with pytest.raises(SystemExit) as exc_info:
            _pipeline.main(["--backfill", "0", "--db", ":memory:"])
        assert exc_info.value.code != 0


# ---------------------------------------------------------------------------
# 任務 5.1: 自動假日抓取測試
# ---------------------------------------------------------------------------


class TestAutoHolidayFetch:
    """自動抓取 TWSE holidaySchedule 假日測試。"""

    def test_auto_holiday_fetch_success_adds_holidays(
        self, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        """holidaySchedule API 正常回應時，假日加入集合"""
        monkeypatch.setattr(
            "market_crawler.pipeline._fetch_twse_holidays",
            lambda policy: {datetime.date(2026, 4, 4)},
        )

        called_with: dict[str, Any] = {}

        def mock_run(**kwargs: Any) -> tuple[list, Any]:
            called_with.update(kwargs)
            from market_crawler.quality import BatchQualityReport

            return [], BatchQualityReport()

        monkeypatch.setattr(
            "market_crawler.pipeline.run_date_range_ingestion", mock_run
        )
        monkeypatch.setattr(
            "market_crawler.pipeline.make_fetcher", lambda **_: lambda _d: True
        )
        monkeypatch.setattr(
            "market_crawler.pipeline._get_today", lambda: datetime.date(2026, 3, 21)
        )

        _pipeline.main(["--today", "--db", ":memory:"])

        holidays = called_with.get("holidays", set())
        assert (
            datetime.date(2026, 4, 4).isoformat() in holidays
            or datetime.date(2026, 4, 4) in holidays
        )

    def test_auto_holiday_fetch_failure_continues(
        self, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        """holidaySchedule API 失敗時 → warning，不中斷作業"""

        def mock_fetch_holidays(policy: Any) -> set:
            raise ConnectionError("API unreachable")

        monkeypatch.setattr(
            "market_crawler.pipeline._fetch_twse_holidays", mock_fetch_holidays
        )

        called_with: dict[str, Any] = {}

        def mock_run(**kwargs: Any) -> tuple[list, Any]:
            called_with.update(kwargs)
            from market_crawler.quality import BatchQualityReport

            return [], BatchQualityReport()

        monkeypatch.setattr(
            "market_crawler.pipeline.run_date_range_ingestion", mock_run
        )
        monkeypatch.setattr(
            "market_crawler.pipeline.make_fetcher", lambda **_: lambda _d: True
        )
        monkeypatch.setattr(
            "market_crawler.pipeline._get_today", lambda: datetime.date(2026, 3, 21)
        )

        import warnings

        with warnings.catch_warnings(record=True) as w:
            warnings.simplefilter("always")
            _pipeline.main(["--today", "--db", ":memory:"])

        # 應該有 warning 但不拋出例外
        assert any(
            "holiday" in str(warning.message).lower() or "假日" in str(warning.message)
            for warning in w
        ), f"Expected holiday warning, got: {[str(x.message) for x in w]}"

    def test_no_auto_holidays_flag_skips_fetch(
        self, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        """--no-auto-holidays 時不呼叫 holidaySchedule API"""
        fetch_called = {"n": 0}

        def mock_fetch_holidays(policy: Any) -> set:
            fetch_called["n"] += 1
            return set()

        monkeypatch.setattr(
            "market_crawler.pipeline._fetch_twse_holidays", mock_fetch_holidays
        )

        def mock_run(**kwargs: Any) -> tuple[list, Any]:
            from market_crawler.quality import BatchQualityReport

            return [], BatchQualityReport()

        monkeypatch.setattr(
            "market_crawler.pipeline.run_date_range_ingestion", mock_run
        )
        monkeypatch.setattr(
            "market_crawler.pipeline.make_fetcher", lambda **_: lambda _d: True
        )
        monkeypatch.setattr(
            "market_crawler.pipeline._get_today", lambda: datetime.date(2026, 3, 21)
        )

        _pipeline.main(["--today", "--no-auto-holidays", "--db", ":memory:"])

        assert (
            fetch_called["n"] == 0
        ), "Holiday fetch should NOT be called with --no-auto-holidays"

    def test_manual_holidays_merged_with_auto(
        self, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        """--holidays 與自動抓取結果正確合併（取聯集）"""
        monkeypatch.setattr(
            "market_crawler.pipeline._fetch_twse_holidays",
            lambda policy: {datetime.date(2026, 4, 4)},
        )

        called_with: dict[str, Any] = {}

        def mock_run(**kwargs: Any) -> tuple[list, Any]:
            called_with.update(kwargs)
            from market_crawler.quality import BatchQualityReport

            return [], BatchQualityReport()

        monkeypatch.setattr(
            "market_crawler.pipeline.run_date_range_ingestion", mock_run
        )
        monkeypatch.setattr(
            "market_crawler.pipeline.make_fetcher", lambda **_: lambda _d: True
        )
        monkeypatch.setattr(
            "market_crawler.pipeline._get_today", lambda: datetime.date(2026, 3, 21)
        )

        _pipeline.main(["--today", "--holidays", "2026-02-28", "--db", ":memory:"])

        holidays = called_with.get("holidays", set())
        # 兩個假日都應在集合中（ISO 格式或 date 物件）
        holidays_str = {
            d.isoformat() if isinstance(d, datetime.date) else d for d in holidays
        }
        assert (
            "2026-04-04" in holidays_str
        ), f"Auto-fetched holiday missing: {holidays_str}"
        assert "2026-02-28" in holidays_str, f"Manual holiday missing: {holidays_str}"
