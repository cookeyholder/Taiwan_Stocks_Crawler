from __future__ import annotations

from pathlib import Path

import pytest

from market_crawler.normalize import normalize_record
from market_crawler.parsers import parse_tpex_daily_close, parse_twse_daily_close


def _read_required_sample(file_name: str) -> bytes:
    path = Path(file_name)
    if not path.exists():
        pytest.skip(f"required sample not found: {file_name}")
    return path.read_bytes()


def test_verify_twse_mapping_with_real_sample() -> None:
    sample = _read_required_sample("MI_INDEX_ALL_20260318.csv")
    parsed = parse_twse_daily_close(sample, source_file="MI_INDEX_ALL_20260318.csv")
    assert parsed.records
    assert any(row["symbol"] == '="0050"' for row in parsed.records)


def test_verify_tpex_mapping_with_real_sample() -> None:
    sample = _read_required_sample("SQUOTE_AL_1150318.csv")
    parsed = parse_tpex_daily_close(sample, source_file="SQUOTE_AL_1150318.csv")
    assert parsed.records
    assert any(row["symbol"] == "00679B" for row in parsed.records)


def test_verify_db_date_format_is_ad() -> None:
    sample = _read_required_sample("SQUOTE_AL_1150318.csv")
    parsed = parse_tpex_daily_close(sample, source_file="SQUOTE_AL_1150318.csv")
    normalized = normalize_record(
        market_code="TPEX",
        trade_date="115/03/18",
        record=parsed.records[0],
        source_encoding=parsed.source_encoding,
        ingest_run_id="verify-run",
        ingested_at="2026-03-19T00:00:00Z",
    )
    assert normalized["trade_date"].startswith("2026-")
