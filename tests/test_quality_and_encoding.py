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
        "failure_reasons": {"parse_error": 1},
    }
