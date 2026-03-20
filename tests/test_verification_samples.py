from __future__ import annotations

from pathlib import Path

from market_crawler.normalize import normalize_record
from market_crawler.parsers import parse_tpex_daily_close, parse_twse_daily_close


def test_verify_twse_mapping_with_real_sample() -> None:
    sample = Path("MI_INDEX_ALL_20260318.csv").read_bytes()
    parsed = parse_twse_daily_close(sample, source_file="MI_INDEX_ALL_20260318.csv")
    assert parsed.records
    assert any(row["symbol"] == '="0050"' for row in parsed.records)


def test_verify_tpex_mapping_with_real_sample() -> None:
    sample = Path("SQUOTE_AL_1150318.csv").read_bytes()
    parsed = parse_tpex_daily_close(sample, source_file="SQUOTE_AL_1150318.csv")
    assert parsed.records
    assert any(row["symbol"] == "00679B" for row in parsed.records)


def test_verify_db_date_format_is_ad() -> None:
    sample = Path("SQUOTE_AL_1150318.csv").read_bytes()
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
