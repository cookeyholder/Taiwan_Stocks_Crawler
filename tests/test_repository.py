from __future__ import annotations

import pytest

from market_crawler.repository import SQLiteMarketRepository


def _quote(symbol: str = "0050", close: float = 77.8) -> dict[str, object]:
    symbol_raw = f'="{symbol}"' if symbol.isdigit() and len(symbol) == 4 else symbol
    return {
        "trade_date": "2026-03-18",
        "market_code": "TWSE",
        "symbol_raw": symbol_raw,
        "symbol_clean": symbol,
        "security_name": "元大台灣50",
        "instrument_type": "stock",
        "instrument_subtype": None,
        "isin_code": None,
        "cfi_code": "ESVUFR",
        "section_label": "ETF",
        "open_price": 77.75,
        "high_price": 78.05,
        "low_price": 77.45,
        "close_price": close,
        "volume_shares": 78000193,
        "turnover_value": 6064253100.0,
        "trade_count": 65062,
        "change_sign": "+",
        "change_value": 1.15,
        "last_bid_price": 77.75,
        "last_bid_qty_raw": "203",
        "last_ask_price": 77.80,
        "last_ask_qty_raw": "115",
        "pe_ratio": 0.0,
        "issued_shares": None,
        "next_limit_up": None,
        "next_limit_down": None,
        "source_file": "MI_INDEX_ALL_20260318.csv",
        "source_encoding": "cp950",
        "source_line_no": 314,
        "ingest_run_id": "run-1",
        "ingested_at": "2026-03-19T00:00:00Z",
        "raw_change": "1.15",
        "raw_payload": {"a": 1},
    }


def test_sqlite_schema_created(sqlite_repo: SQLiteMarketRepository) -> None:
    rows = sqlite_repo.connection.execute(
        "SELECT name FROM sqlite_master WHERE type='table'"
    ).fetchall()
    names = {row[0] for row in rows}
    assert "market_quote_core" in names
    assert "market_quote_extension" in names


def test_sqlite_idempotent_upsert(sqlite_repo: SQLiteMarketRepository) -> None:
    quote = _quote(symbol="0050", close=77.8)
    sqlite_repo.upsert_core(quote)
    sqlite_repo.upsert_core(quote)
    count = sqlite_repo.connection.execute("SELECT COUNT(*) FROM market_quote_core").fetchone()[0]
    assert count == 1


def test_sqlite_upsert_updates_existing_row(sqlite_repo: SQLiteMarketRepository) -> None:
    quote = _quote(symbol="0050", close=77.8)
    sqlite_repo.upsert_core(quote)
    updated = _quote(symbol="0050", close=78.0)
    sqlite_repo.upsert_core(updated)
    close = sqlite_repo.connection.execute(
        "SELECT close_price FROM market_quote_core WHERE symbol_clean='0050'"
    ).fetchone()[0]
    assert close == 78.0


def test_sqlite_extension_upsert(sqlite_repo: SQLiteMarketRepository) -> None:
    quote = _quote(symbol="0050", close=77.8)
    sqlite_repo.upsert_core(quote)
    sqlite_repo.upsert_extension(quote)
    sqlite_repo.upsert_extension({**quote, "raw_change": "2.00", "raw_payload": {"a": 2}})
    raw_change = sqlite_repo.connection.execute(
        "SELECT raw_change FROM market_quote_extension WHERE symbol_clean='0050'"
    ).fetchone()[0]
    assert raw_change == "2.00"


def test_sqlite_fetch_by_trade_date_consistency(sqlite_repo: SQLiteMarketRepository) -> None:
    sqlite_repo.upsert_core(_quote(symbol="0050", close=77.8))
    sqlite_repo.upsert_core(_quote(symbol="00679B", close=27.5) | {"market_code": "TPEX"})
    rows = sqlite_repo.fetch_by_trade_date("2026-03-18")
    assert len(rows) == 2
    assert {row["market_code"] for row in rows} == {"TWSE", "TPEX"}


def test_single_core_table_covers_stock_etf_warrant(sqlite_repo: SQLiteMarketRepository) -> None:
    sqlite_repo.upsert_core(_quote(symbol="2330", close=950.0) | {"instrument_type": "stock"})
    sqlite_repo.upsert_core(_quote(symbol="0050", close=77.8) | {"instrument_type": "etf"})
    sqlite_repo.upsert_core(_quote(symbol="03456P", close=1.2) | {"instrument_type": "warrant"})

    rows = sqlite_repo.fetch_by_trade_date("2026-03-18")
    assert {row["instrument_type"] for row in rows} == {"stock", "etf", "warrant"}


def test_upsert_core_normalizes_text_to_utf8(sqlite_repo: SQLiteMarketRepository) -> None:
    quote = _quote(symbol="0050", close=77.8)
    quote["security_name"] = "測試\udcff名稱"
    sqlite_repo.upsert_core(quote)
    saved_name = sqlite_repo.connection.execute(
        "SELECT security_name FROM market_quote_core WHERE symbol_clean='0050'"
    ).fetchone()[0]
    assert saved_name == "測試?名稱"


def test_upsert_extension_normalizes_text_to_utf8(sqlite_repo: SQLiteMarketRepository) -> None:
    quote = _quote(symbol="0050", close=77.8)
    quote["raw_payload"] = {"note": "測試\udcffpayload"}
    sqlite_repo.upsert_core(quote)
    sqlite_repo.upsert_extension(quote)
    saved_payload = sqlite_repo.connection.execute(
        "SELECT raw_payload FROM market_quote_extension WHERE symbol_clean='0050'"
    ).fetchone()[0]
    assert "?payload" in saved_payload


def test_foreign_key_constraint_enforced(sqlite_repo: SQLiteMarketRepository) -> None:
    """未在 market_quote_core 建立記錄，不得直接寫入 market_quote_extension。"""
    import sqlite3

    with pytest.raises(sqlite3.IntegrityError):
        sqlite_repo.connection.execute(
            """
            INSERT INTO market_quote_extension (trade_date, market_code, symbol_clean, raw_change, raw_payload)
            VALUES ('2026-03-21', 'TWSE', 'ORPHAN', NULL, NULL)
            """
        )
        sqlite_repo.connection.commit()


def test_pragma_foreign_keys_on(sqlite_repo: SQLiteMarketRepository) -> None:
    """init_schema 應啟用 PRAGMA foreign_keys = ON。"""
    result = sqlite_repo.connection.execute("PRAGMA foreign_keys").fetchone()
    assert result[0] == 1, "PRAGMA foreign_keys 應為 ON（1）"
