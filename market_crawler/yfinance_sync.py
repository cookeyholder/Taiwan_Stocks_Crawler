from __future__ import annotations

import datetime as _dt
import os
import uuid
from typing import Any

import pandas as pd
import yfinance as yf

from market_crawler.repository import SQLiteMarketRepository

_STATIC_SYMBOL_NAMES: dict[str, str] = {
    # 台灣 50 成份股
    "0050": "元大台灣50", "2330": "台積電", "2317": "鴻海", "2454": "聯發科", "2308": "台達電",
    "2881": "富邦金", "2882": "國泰金", "2303": "聯電", "2891": "中信金", "2886": "兆豐金",
    "2884": "玉山金", "1216": "統一", "2892": "第一金", "2885": "元大金", "3711": "日月光投控",
    "2382": "廣達", "2880": "華南金", "2883": "開發金", "2890": "永豐金", "2002": "中鋼",
    "5880": "合庫金", "2357": "華碩", "2301": "光寶科", "2603": "長榮", "2327": "國巨",
    "2409": "友達", "3045": "台灣大", "2609": "陽明", "2408": "南亞科", "3008": "大立光",
    "2615": "萬海", "5876": "上海商銀", "4938": "和碩", "2395": "研華", "2379": "瑞昱",
    "3231": "緯創", "2474": "可成", "1301": "台塑", "1303": "南亞", "1326": "台化",
    "6505": "台塑化", "9904": "寶成", "1101": "台泥", "1402": "遠東新", "2105": "建大",
    "2912": "統一超", "3034": "聯詠", "3037": "欣興", "4904": "遠傳", "9910": "豐泰",
    # 常見熱門 ETF
    "0056": "元大高股息", "00878": "國泰永續高股息", "00919": "群益台灣精選高息",
    "00929": "復華台灣科技優息", "00713": "元大台灣高息低波", "0050B": "元大美債20年"
}


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
    security_name: str | None = None,
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
        "security_name": security_name,
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

    # 1. 嘗試從 StockID.csv 載入股票名稱
    security_name = None
    csv_path = "StockID.csv"
    if os.path.exists(csv_path):
        try:
            from market_crawler.classification import load_security_master
            master = load_security_master(csv_path)
            if symbol in master:
                security_name = master[symbol].name
        except Exception:
            pass

    # 2. 嘗試從靜態常見表載入
    if not security_name:
        security_name = _STATIC_SYMBOL_NAMES.get(symbol)

    # 3. 嘗試從 yfinance API 動態取得名稱（作為單股下載的最後 fallback）
    if not security_name:
        try:
            suffix = ".TW" if market_code.upper() == "TWSE" else ".TWO"
            ticker_symbol = f"{symbol}{suffix}"
            ticker = yf.Ticker(ticker_symbol)
            security_name = ticker.info.get("shortName") or ticker.info.get("longName")
        except Exception:
            pass

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
            security_name=security_name,
        )
        normalized_quote = repo._normalize_quote_payload(quote)
        quotes.append(normalized_quote)

    if quotes:
        repo.upsert_core_batch(quotes)
        repo.upsert_extension_batch(quotes)
        try:
            repo.connection.commit()
        except Exception:
            pass

    return len(quotes)
