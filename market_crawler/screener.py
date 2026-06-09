from __future__ import annotations

import sqlite3

import pandas as pd
import pandas_ta as ta


def screen_bollinger_squeeze(  # noqa: C901
    conn: sqlite3.Connection,
    bandwidth_limit: float = 0.08,
    consecutive_days: int = 5,
) -> pd.DataFrame:
    """從本地 SQLite 載入歷史股價，並篩選出布林通道帶寬小於門檻值且持續 N 天的股票。

    計算公式：
        Bandwidth = (UpperBand - LowerBand) / MiddleBand
    門檻條件：
        最近 N 天內，最大帶寬 <= bandwidth_limit
    """
    # 為了計算 20 日布林通道，我們需要至少 (consecutive_days + 30) 個交易日。
    # 我們多抓一些天數以確保能完整計算滾動窗口。
    required_days = consecutive_days + 40

    # 1. 取得資料庫中最新的交易日列表
    cur = conn.execute(
        "SELECT DISTINCT trade_date FROM market_quote_core ORDER BY trade_date DESC LIMIT ?",
        (required_days,)
    )
    dates = [r[0] for r in cur.fetchall()]
    if not dates:
        return pd.DataFrame()

    # 最早的交易日
    start_date = dates[-1]

    # 2. 載入資料
    df = pd.read_sql_query(
        """
        SELECT symbol_clean, trade_date, close_price, volume_shares, security_name
        FROM market_quote_core
        WHERE trade_date >= ?
        ORDER BY symbol_clean, trade_date ASC
        """,
        conn,
        params=(start_date,)
    )

    if df.empty:
        return pd.DataFrame()

    # 3. 分組計算布林頻寬
    def _calc_bandwidth(group: pd.DataFrame) -> pd.DataFrame:
        if len(group) < 20:
            group["bandwidth"] = None
            return group

        # 確保依日期排序
        group = group.sort_values("trade_date")

        try:
            # 呼叫 pandas_ta 計算布林通道
            bb = ta.bbands(group["close_price"], length=20, std=2)
            if bb is not None and not bb.empty:
                bbl = bb.iloc[:, 0]  # Lower
                bbm = bb.iloc[:, 1]  # Middle
                bbu = bb.iloc[:, 2]  # Upper
                # 防止分母為 0
                group["bandwidth"] = (bbu - bbl) / bbm.replace(0, float("nan"))
            else:
                group["bandwidth"] = None
        except Exception:
            group["bandwidth"] = None

        return group

    # 套用分組運算
    groups = []
    for _, group in df.groupby("symbol_clean"):
        groups.append(_calc_bandwidth(group.copy()))
    df = pd.concat(groups, ignore_index=True) if groups else pd.DataFrame()

    # 4. 篩選符合 Squeeze 條件的股票
    results = []
    for symbol, group in df.groupby("symbol_clean"):
        # 排除無效的帶寬欄位
        group_valid = group.dropna(subset=["bandwidth"])
        if len(group_valid) < consecutive_days:
            continue

        # 取得最近 N 天的資料
        recent = group_valid.tail(consecutive_days)
        max_bw = recent["bandwidth"].max()

        # 判斷是否符合帶寬限制
        if pd.notna(max_bw) and max_bw <= bandwidth_limit:
            latest = recent.iloc[-1]

            # 取得最新且非空的股票名稱
            names_non_null = group["security_name"].dropna()
            security_name = names_non_null.iloc[-1] if not names_non_null.empty else None

            results.append({
                "symbol": symbol,
                "name": security_name,
                "latest_close": latest["close_price"],
                "latest_volume": latest["volume_shares"],
                "max_bandwidth": float(max_bw),
                "latest_bandwidth": float(latest["bandwidth"]),
            })

    return pd.DataFrame(results)
