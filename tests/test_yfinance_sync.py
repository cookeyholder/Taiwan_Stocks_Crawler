from __future__ import annotations

from typing import Any
from unittest.mock import MagicMock, patch

import pandas as pd
import pytest

from market_crawler.yfinance_sync import (
    fetch_yfinance_history,
    map_yfinance_to_canonical,
    sync_yfinance_to_db,
)


def test_fetch_yfinance_history_twse() -> None:
    # 測試 TWSE 代碼是否被正確轉換為 .TW 後端後呼叫 yf.download
    mock_df = pd.DataFrame(
        data={
            "Open": [100.0],
            "High": [101.0],
            "Low": [99.0],
            "Close": [100.5],
            "Volume": [1000],
        },
        index=pd.DatetimeIndex(["2026-06-09"]),
    )

    with patch("yfinance.download", return_value=mock_df) as mock_download:
        df = fetch_yfinance_history("2330", "TWSE")
        mock_download.assert_called_once_with("2330.TW", period="3mo", progress=False)
        assert not df.empty
        assert df.iloc[0]["Close"] == 100.5


def test_fetch_yfinance_history_tpex() -> None:
    # 測試 TPEX 代碼是否被正確轉換為 .TWO 後端後呼叫 yf.download
    mock_df = pd.DataFrame(
        data={
            "Open": [50.0],
            "High": [51.0],
            "Low": [49.0],
            "Close": [50.5],
            "Volume": [500],
        },
        index=pd.DatetimeIndex(["2026-06-09"]),
    )

    with patch("yfinance.download", return_value=mock_df) as mock_download:
        df = fetch_yfinance_history("6023", "TPEX", period="1mo")
        mock_download.assert_called_once_with("6023.TWO", period="1mo", progress=False)
        assert not df.empty
        assert df.iloc[0]["Close"] == 50.5


def test_map_yfinance_to_canonical() -> None:
    row = pd.Series(
        data={
            "Open": 100.0,
            "High": 101.0,
            "Low": 99.0,
            "Close": 100.5,
            "Volume": 1000,
            "Adj Close": 100.5,
        }
    )
    quote = map_yfinance_to_canonical(
        symbol="2330",
        market_code="TWSE",
        date_str="2026-06-09",
        row=row,
        ingest_run_id="test-run",
        ingested_at="2026-06-09T12:00:00Z",
    )

    assert quote["trade_date"] == "2026-06-09"
    assert quote["market_code"] == "TWSE"
    assert quote["symbol_clean"] == "2330"
    assert quote["close_price"] == 100.5
    assert quote["volume_shares"] == 1000
    assert quote["source_file"] == "yfinance"
    assert quote["raw_payload"]["Adj Close"] == 100.5


def test_sync_yfinance_to_db(sqlite_repo: Any) -> None:
    # 測試完整的寫入資料庫流程
    mock_df = pd.DataFrame(
        data={
            "Open": [100.0, 102.0],
            "High": [101.0, 103.0],
            "Low": [99.0, 101.0],
            "Close": [100.5, 102.5],
            "Volume": [1000, 2000],
        },
        index=pd.DatetimeIndex(["2026-06-08", "2026-06-09"]),
    )

    with patch("yfinance.download", return_value=mock_df):
        count = sync_yfinance_to_db(sqlite_repo, "2330", "TWSE", period="1mo")
        assert count == 2

        # 驗證 SQLite 中的寫入紀錄
        import sqlite3
        sqlite_repo.connection.row_factory = sqlite3.Row
        rows_core = sqlite_repo.connection.execute(
            "SELECT * FROM market_quote_core WHERE symbol_clean='2330' ORDER BY trade_date"
        ).fetchall()
        assert len(rows_core) == 2
        assert rows_core[0]["close_price"] == 100.5
        assert rows_core[1]["close_price"] == 102.5

        rows_ext = sqlite_repo.connection.execute(
            "SELECT * FROM market_quote_extension WHERE symbol_clean='2330' ORDER BY trade_date"
        ).fetchall()
        assert len(rows_ext) == 2
