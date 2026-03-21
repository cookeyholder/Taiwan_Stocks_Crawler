from __future__ import annotations

from market_crawler.encoding import decode_with_fallback
from market_crawler.quality import BatchQualityReport


def test_decode_with_fallback_to_replace() -> None:
    text, encoding = decode_with_fallback(b"\xff\xfe", encodings=["ascii"])
    assert encoding == "utf-8-replace"
    assert isinstance(text, str)


def test_quality_report_as_dict() -> None:
    report = BatchQualityReport()
    report.record_success()
    report.record_failure("parse_error")
    assert report.as_dict() == {
        "success_count": 1,
        "failed_count": 1,
        "source_error_count": 0,
        "market_closed_count": 0,
        "failure_reasons": {"parse_error": 1},
    }


# ---------------------------------------------------------------------------
# market_closed_count 測試（task 1.3）
# ---------------------------------------------------------------------------


def test_quality_report_market_closed_count_field_exists() -> None:
    """BatchQualityReport 有 market_closed_count 欄位，預設值為 0"""
    report = BatchQualityReport()
    assert hasattr(report, "market_closed_count")
    assert report.market_closed_count == 0


def test_quality_report_record_market_closed_increments() -> None:
    """record_market_closed() 正確累加 market_closed_count"""
    report = BatchQualityReport()
    report.record_market_closed()
    report.record_market_closed()
    assert report.market_closed_count == 2


def test_quality_report_market_closed_not_counted_in_failed() -> None:
    """market_closed 不計入 failed_count 或 source_error_count"""
    report = BatchQualityReport()
    report.record_market_closed()
    assert report.failed_count == 0
    assert report.source_error_count == 0


def test_quality_report_market_closed_and_source_error_independent() -> None:
    """market_closed 與 source_error 各自獨立計數"""
    report = BatchQualityReport()
    report.record_market_closed()
    report.record_failure("source_error")
    assert report.market_closed_count == 1
    assert report.source_error_count == 1
    assert report.failed_count == 1


def test_quality_report_source_error_count_via_record_failure() -> None:
    """record_failure('source_error') 會累加 source_error_count"""
    report = BatchQualityReport()
    report.record_failure("source_error")
    report.record_failure("source_error")
    assert report.source_error_count == 2


def test_quality_report_as_dict_includes_market_closed_count() -> None:
    """as_dict() 包含 market_closed_count 欄位"""
    report = BatchQualityReport()
    report.record_market_closed()
    d = report.as_dict()
    assert "market_closed_count" in d
    assert d["market_closed_count"] == 1
