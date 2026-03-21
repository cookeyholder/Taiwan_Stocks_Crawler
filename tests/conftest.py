from __future__ import annotations

import sqlite3
import sys
from pathlib import Path
from typing import Any

import pytest

ROOT_DIR = Path(__file__).resolve().parents[1]
if str(ROOT_DIR) not in sys.path:
    sys.path.insert(0, str(ROOT_DIR))


@pytest.fixture
def fixture_dir() -> Path:
    fixture_root = Path(__file__).parent / "fixtures"
    if not fixture_root.exists():
        pytest.skip(f"Fixture directory not found: {fixture_root}")
    return fixture_root


@pytest.fixture
def twse_fixture_bytes(fixture_dir: Path) -> bytes:
    return (fixture_dir / "twse_sample.csv").read_bytes()


@pytest.fixture
def tpex_fixture_bytes(fixture_dir: Path) -> bytes:
    return (fixture_dir / "tpex_sample.csv").read_bytes()


@pytest.fixture
def sqlite_repo() -> Any:
    from market_crawler.repository import SQLiteMarketRepository

    conn = sqlite3.connect(":memory:")
    repo = SQLiteMarketRepository(conn)
    repo.init_schema()
    try:
        yield repo
    finally:
        conn.close()


@pytest.fixture
def stock_master_path() -> str:
    # StockID.csv is a large data file not committed to the repo.
    # Tests that require it are skipped in CI / clean checkouts.
    for candidate in [ROOT_DIR / "StockID.csv", Path(__file__).parent / "fixtures" / "StockID.csv"]:
        if candidate.exists():
            return str(candidate)
    pytest.skip("StockID.csv not found; skipping tests that require the security master file")
