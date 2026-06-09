from __future__ import annotations

import datetime as _dt
import uuid
from typing import Any

import pandas as pd
import yfinance as yf

from market_crawler.repository import SQLiteMarketRepository


def fetch_yfinance_history(
    symbol: str,
    market_code: str,
    period: str = "3mo",
) -> pd.DataFrame:
    """從 yfinance 下載指定股票的歷史 K 線資料。

    將台灣代號格式化（TWSE ➔ .TW，TPEX ➔ .TWO）後發起請求。
    """
    suffix = ".TW" if market_code.upper() == "TWSE" else ".TWO"
    ticker_symbol = f"{symbol}{suffix}"
    
    # 進行歷史下載
    df = yf.download(ticker_symbol, period=period, progress=False)
    
    # 處理多層 Columns 指標（有些版本的 yfinance 會回傳 MultiIndex）
    if isinstance(df.columns, pd.MultiIndex):
        df.columns = df.columns.get_level_values(0)
        
    return df


def map_yfinance_to_canonical(
    symbol: str,
    market_code: str,
    date_str: str,
    row: pd.Series,
    ingest_run_id: str,
    ingested_at: str,
) -> dict[str, Any]:
    """將 yfinance 的單日 Series 資料對接轉換為本地符合 CanonicalQuote 格式的 dict。"""
    # 預防 row 中可能缺漏部分欄位
    open_p = float(row["Open"]) if pd.notna(row.get("Open")) else None
    high_p = float(row["High"]) if pd.notna(row.get("High")) else None
    low_p = float(row["Low"]) if pd.notna(row.get("Low")) else None
    close_p = float(row["Close"]) if pd.notna(row.get("Close")) else None
    volume = int(row["Volume"]) if pd.notna(row.get("Volume")) else 0

    raw_payload_dict = {
        "Open": open_p,
        "High": high_p,
        "Low": low_p,
        "Close": close_p,
        "Volume": volume,
    }
    if "Adj Close" in row:
        raw_payload_dict["Adj Close"] = float(row["Adj Close"])

    return {
        "trade_date": date_str,
        "market_code": market_code.upper(),
        "symbol_raw": symbol,
        "symbol_clean": symbol,
        "security_name": None,
        "instrument_type": "stock",
        "instrument_subtype": None,
        "isin_code": None,
        "cfi_code": None,
        "section_label": None,
        "open_price": open_p,
        "high_price": high_p,
        "low_price": low_p,
        "close_price": close_p,
        "volume_shares": volume,
        "turnover_value": None,
        "trade_count": None,
        "change_sign": None,
        "change_value": None,
        "last_bid_price": None,
        "last_bid_qty_raw": None,
        "last_ask_price": None,
        "last_ask_qty_raw": None,
        "pe_ratio": None,
        "issued_shares": None,
        "next_limit_up": None,
        "next_limit_down": None,
        "source_file": "yfinance",
        "source_encoding": "utf-8",
        "source_line_no": 0,
        "ingest_run_id": ingest_run_id,
        "ingested_at": ingested_at,
        "raw_change": None,
        "raw_payload": raw_payload_dict,
    }


def sync_yfinance_to_db(
    repo: SQLiteMarketRepository,
    symbol: str,
    market_code: str,
    period: str = "3mo",
    ingest_run_id: str | None = None,
) -> int:
    """下載 yfinance 歷史資料並寫入本地 SQLite 資料庫，回傳寫入的成功記錄筆數。"""
    df = fetch_yfinance_history(symbol, market_code, period)
    if df.empty:
        return 0

    if ingest_run_id is None:
        ingest_run_id = str(uuid.uuid4())
    
    ingested_at = _dt.datetime.now(_dt.timezone.utc).isoformat()
    
    quotes: list[dict[str, Any]] = []
    
    for date_idx, row in df.iterrows():
        # yfinance index 為 DatetimeIndex，需轉為 YYYY-MM-DD
        date_str = date_idx.strftime("%Y-%m-%d")
        
        quote = map_yfinance_to_canonical(
            symbol=symbol,
            market_code=market_code,
            date_str=date_str,
            row=row,
            ingest_run_id=ingest_run_id,
            ingested_at=ingested_at,
        )
        normalized_quote = repo._normalize_quote_payload(quote)
        quotes.append(normalized_quote)
        
    if quotes:
        repo.upsert_core_batch(quotes)
        repo.upsert_extension_batch(quotes)
        
    return len(quotes)
