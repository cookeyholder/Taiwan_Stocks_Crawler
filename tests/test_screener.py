from __future__ import annotations

import sqlite3

import pytest

from market_crawler.screener import screen_bollinger_squeeze


@pytest.fixture
def mock_db() -> sqlite3.Connection:
    # 建立內存資料庫並初始化報價 Schema 的簡化版，用來測試選股邏輯
    conn = sqlite3.connect(":memory:")
    conn.execute(
        """
        CREATE TABLE market_quote_core (
            trade_date TEXT,
            market_code TEXT,
            symbol_raw TEXT,
            symbol_clean TEXT,
            security_name TEXT,
            close_price REAL,
            volume_shares INTEGER,
            PRIMARY KEY (trade_date, market_code, symbol_clean)
        )
        """
    )
    return conn


def test_screen_bollinger_squeeze(mock_db: sqlite3.Connection) -> None:
    # 建立測試資料
    # 1. 股票 A: 價格極度穩定 (開盤/收盤都在 100 左右)，應該被篩選出來
    # 我們需要寫入 50 天的資料以進行 20MA 布林計算
    dates = [f"2026-06-{i:02d}" for i in range(1, 31)] + [f"2026-07-{i:02d}" for i in range(1, 26)]

    # 確保寫入 55 天以滿足 20 + 5 限制
    assert len(dates) == 55

    # 股票 A 價格都在 100 左右，bandwidth 應該極小
    records_a = [
        ("2026-06-01" if i == 0 else dates[i], "TWSE", "A", "A", "A公司", 100.0, 1000)
        for i in range(len(dates))
    ]

    # 股票 B 價格劇烈波動，不符合帶寬門檻值 0.08
    records_b = []
    for i, dt in enumerate(dates):
        price = 100.0 if i % 2 == 0 else 120.0  # 波動劇烈
        records_b.append((dt, "TWSE", "B", "B", "B公司", price, 2000))

    # 股票 C 只有 5 天資料，不足 20 日布林通道計算需求，不應被篩選
    records_c = [
        (dates[i], "TWSE", "C", "C", "C公司", 100.0, 500)
        for i in range(5)
    ]

    # 寫入資料庫
    cur = mock_db.cursor()
    cur.executemany(
        """
        INSERT INTO market_quote_core (trade_date, market_code, symbol_raw, symbol_clean, security_name, close_price, volume_shares)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        """,
        records_a + records_b + records_c
    )
    mock_db.commit()

    # 執行篩選
    df_res = screen_bollinger_squeeze(mock_db, bandwidth_limit=0.08, consecutive_days=5)

    assert not df_res.empty
    symbols = df_res["symbol"].tolist()

    # 應該只篩選出 A，不含 B (波動過大) 且不含 C (資料不足)
    assert "A" in symbols
    assert "B" not in symbols
    assert "C" not in symbols

    # 驗證結果屬性
    row_a = df_res[df_res["symbol"] == "A"].iloc[0]
    assert row_a["name"] == "A公司"
    assert row_a["latest_close"] == 100.0
    assert row_a["latest_volume"] == 1000
    assert row_a["latest_bandwidth"] < 0.05
