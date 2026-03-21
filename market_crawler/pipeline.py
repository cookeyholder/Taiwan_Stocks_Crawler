"""market_crawler/pipeline.py — 端到端每日行情抓取與入庫流程。

整合 http_client、parsers、normalize、classification、repository，
提供可直接呼叫的 `fetch_and_ingest_day()` 與供 orchestrator 使用的
`make_fetcher()` 包裝函式。

資料來源：
- TWSE：https://openapi.twse.com.tw/v1/exchangeReport/STOCK_DAY_ALL（JSON，UTF-8）
  注意：此 API 不接受日期參數，固定回傳最新交易日資料。
  若請求的日期與 API 回傳的日期不符，TWSE 資料將被略過。
- TPEX：https://www.tpex.org.tw/www/zh-tw/afterTrading/dailyQuotes（JSON，UTF-8）

注意事項：
- TWSE/TPEX 伺服器有時會因 User-Agent 不對而拒絕請求，故預設使用
  fake_useragent 隨機產生 User-Agent；若未安裝則退回靜態字串。
- TWSE/TPEX 的 SSL 憑證在部分環境下會通不過驗證，故 ssl_verify 預設為 False。
  使用者應評估自身安全需求。

命令列用法：
    python -m market_crawler.pipeline --start 2026-03-16 --end 2026-03-20
    python -m market_crawler.pipeline --start 2026-03-16 --end 2026-03-20 \\
        --db market_data.db --holidays 2026-04-04
"""
from __future__ import annotations

import sqlite3
import uuid
import warnings
from datetime import datetime, timezone
from typing import Callable

import urllib3

from market_crawler.classification import classify_instrument, load_security_master
from market_crawler.http_client import HttpPolicy, get_with_retry
from market_crawler.normalize import normalize_record
from market_crawler.orchestrator import run_date_range_ingestion
from market_crawler.parsers import (
    parse_tpex_daily_close_json,
    parse_twse_daily_close_json,
)
from market_crawler.repository import SQLiteMarketRepository

# ---------------------------------------------------------------------------
# 抑制 InsecureRequestWarning（TWSE/TPEX SSL 憑證問題）
# ---------------------------------------------------------------------------
urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)

# ---------------------------------------------------------------------------
# User-Agent
# ---------------------------------------------------------------------------

def _make_user_agent() -> str:
    """嘗試使用 fake_useragent，不可用時退回靜態 UA 字串。"""
    try:
        from fake_useragent import UserAgent  # noqa: PLC0415
        return UserAgent().random
    except Exception:  # noqa: BLE001
        return (
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
            "AppleWebKit/537.36 (KHTML, like Gecko) "
            "Chrome/123.0.0.0 Safari/537.36"
        )


# ---------------------------------------------------------------------------
# URL 建構
# ---------------------------------------------------------------------------

# TWSE OpenAPI 無日期參數，固定回傳最新交易日資料
_TWSE_OPENAPI_URL = "https://openapi.twse.com.tw/v1/exchangeReport/STOCK_DAY_ALL"


def _roc_compact_to_iso(roc_compact: str) -> str:
    """將 TWSE JSON 中的民國緊湊日期格式轉為 ISO 格式。

    例如 ``"1150320"`` → ``"2026-03-20"``。
    """
    roc_compact = roc_compact.strip()
    roc_year = int(roc_compact[:-4])
    month = int(roc_compact[-4:-2])
    day = int(roc_compact[-2:])
    return f"{roc_year + 1911:04d}-{month:02d}-{day:02d}"


def _build_tpex_url(date_str: str) -> str:
    """建立 TPEX 每日收盤行情 JSON 下載 URL（新版 API，民國年格式）。

    Args:
        date_str: ISO 格式日期字串，例如 ``"2026-03-17"``。

    Returns:
        完整 URL 字串，例如
        ``"https://www.tpex.org.tw/www/zh-tw/afterTrading/dailyQuotes?date=115/03/17&type=S&response=json"``
    """
    year, month, day = date_str.split("-")
    roc_year = int(year) - 1911
    return (
        f"https://www.tpex.org.tw/www/zh-tw/afterTrading/dailyQuotes"
        f"?date={roc_year}/{month}/{day}&type=S&response=json"
    )


# ---------------------------------------------------------------------------
# 內部：單一市場寫入邏輯
# ---------------------------------------------------------------------------

def _ingest_records(
    *,
    market_code: str,
    date_str: str,
    parse_result: object,
    security_master: dict,
    ingest_run_id: str,
    ingested_at: str,
    repo: SQLiteMarketRepository,
) -> int:
    """將 parse_result 中的 records 進行 normalize / classify 後寫入 repo。"""
    written = 0
    for rec in parse_result.records:  # type: ignore[attr-defined]
        quote = normalize_record(
            market_code=market_code,
            trade_date=date_str,
            record=rec,
            source_encoding=parse_result.source_encoding,  # type: ignore[attr-defined]
            ingest_run_id=ingest_run_id,
            ingested_at=ingested_at,
        )
        i_type, i_subtype, sec_label, cfi = classify_instrument(
            symbol_clean=quote["symbol_clean"],
            security_master=security_master,
        )
        quote["instrument_type"] = i_type
        quote["instrument_subtype"] = i_subtype
        if cfi:
            quote["cfi_code"] = cfi
        if sec_label:
            quote["section_label"] = sec_label
        repo.upsert_core(quote)
        repo.upsert_extension(quote)
        written += 1
    if parse_result.errors:  # type: ignore[attr-defined]
        warnings.warn(
            f"{market_code} {date_str}：{len(parse_result.errors)} 筆解析錯誤",  # type: ignore[arg-type]
            stacklevel=3,
        )
    return written


# ---------------------------------------------------------------------------
# 內部：TWSE fetch + 日期驗證
# ---------------------------------------------------------------------------

def _fetch_and_parse_twse(date_str: str, policy: HttpPolicy) -> object | None:
    """抓取 TWSE OpenAPI 並驗證日期；日期不符時回傳 ``None``。"""
    resp = get_with_retry(_TWSE_OPENAPI_URL, policy)
    response_json = resp.json()

    # 驗證 API 回傳的日期與請求日期是否一致
    # （STOCK_DAY_ALL 不接受日期參數，固定回傳最新交易日）
    returned_date: str | None = None
    if isinstance(response_json, list) and response_json:
        raw_date = str(response_json[0].get("Date", ""))
        if raw_date:
            try:
                returned_date = _roc_compact_to_iso(raw_date)
            except (ValueError, IndexError):
                pass

    if returned_date and returned_date != date_str:
        warnings.warn(
            f"TWSE API 回傳日期 {returned_date} 與請求日期 {date_str} 不符，略過 TWSE 資料",
            stacklevel=3,
        )
        return None

    return parse_twse_daily_close_json(response_json, source_file=_TWSE_OPENAPI_URL)


# ---------------------------------------------------------------------------
# 核心：單日抓取與入庫
# ---------------------------------------------------------------------------

def fetch_and_ingest_day(
    date_str: str,
    *,
    db_path: str = "market_data.db",
    security_master_path: str | None = "StockID.csv",
    policy: HttpPolicy | None = None,
    ingest_run_id: str | None = None,
) -> bool:
    """抓取指定日期的 TWSE 與 TPEX 每日收盤行情並寫入 SQLite。

    Args:
        date_str: ISO 格式交易日，例如 ``"2026-03-17"``。
        db_path: SQLite 資料庫路徑。
        security_master_path: StockID.csv 路徑；``None`` 則跳過分類擴充。
        policy: HTTP 請求策略。
        ingest_run_id: 本次匯入的唯一識別碼（留空時自動產生 UUID）。

    Returns:
        ``True`` 表示至少有一個市場成功寫入資料；``False`` 表示兩個市場均無資料。
    """
    if policy is None:
        policy = HttpPolicy(
            timeout_seconds=30.0,
            retry_count=3,
            backoff_seconds=1.5,
            headers={"user-agent": _make_user_agent()},
            ssl_verify=False,
        )
    if ingest_run_id is None:
        ingest_run_id = str(uuid.uuid4())

    ingested_at = datetime.now(timezone.utc).isoformat()

    # 載入有價證券主檔（提供分類輔助資訊）
    security_master = {}
    if security_master_path:
        try:
            security_master = load_security_master(security_master_path)
        except Exception as exc:  # noqa: BLE001
            warnings.warn(f"無法載入 {security_master_path}：{exc}", stacklevel=2)

    conn = sqlite3.connect(db_path)
    conn.row_factory = sqlite3.Row
    repo = SQLiteMarketRepository(conn)
    repo.init_schema()

    total_written = 0

    with conn:
        # --- TWSE ---
        try:
            twse_result = _fetch_and_parse_twse(date_str, policy)
            if twse_result is not None:
                total_written += _ingest_records(
                    market_code="TWSE",
                    date_str=date_str,
                    parse_result=twse_result,
                    security_master=security_master,
                    ingest_run_id=ingest_run_id,
                    ingested_at=ingested_at,
                    repo=repo,
                )
        except Exception as exc:  # noqa: BLE001
            warnings.warn(f"TWSE {date_str} 抓取/入庫失敗：{exc}", stacklevel=2)

        # --- TPEX ---
        try:
            tpex_url = _build_tpex_url(date_str)
            resp = get_with_retry(tpex_url, policy)
            response_json = resp.json()
            result = parse_tpex_daily_close_json(response_json, source_file=tpex_url)
            total_written += _ingest_records(
                market_code="TPEX",
                date_str=date_str,
                parse_result=result,
                security_master=security_master,
                ingest_run_id=ingest_run_id,
                ingested_at=ingested_at,
                repo=repo,
            )
        except Exception as exc:  # noqa: BLE001
            warnings.warn(f"TPEX {date_str} 抓取/入庫失敗：{exc}", stacklevel=2)

    conn.close()
    return total_written > 0


# ---------------------------------------------------------------------------
# make_fetcher：供 orchestrator 使用的 Callable
# ---------------------------------------------------------------------------

def make_fetcher(
    db_path: str = "market_data.db",
    security_master_path: str | None = "StockID.csv",
    policy: HttpPolicy | None = None,
) -> Callable[[str], bool]:
    """回傳一個 ``(date_str) -> bool`` 的可呼叫物件，供 `run_date_range_ingestion` 使用。

    Args:
        db_path: SQLite 資料庫路徑。
        security_master_path: StockID.csv 路徑。
        policy: HTTP 請求策略（``None`` 使用預設值）。

    Returns:
        符合 orchestrator 介面的 fetcher callable。
    """
    if policy is None:
        policy = HttpPolicy(
            timeout_seconds=30.0,
            retry_count=3,
            backoff_seconds=1.5,
            headers={"user-agent": _make_user_agent()},
            ssl_verify=False,
        )

    def _fetcher(date_str: str) -> bool:
        return fetch_and_ingest_day(
            date_str,
            db_path=db_path,
            security_master_path=security_master_path,
            policy=policy,
        )

    return _fetcher


# ---------------------------------------------------------------------------
# 命令列入口
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    import argparse
    import sys

    _parser = argparse.ArgumentParser(
        description="抓取 TWSE/TPEX 每日收盤行情並寫入 SQLite"
    )
    _parser.add_argument("--start", required=True, help="起始日期（ISO 格式，如 2026-03-16）")
    _parser.add_argument("--end", required=True, help="結束日期（ISO 格式，如 2026-03-20）")
    _parser.add_argument("--db", default="market_data.db", help="SQLite 資料庫路徑（預設：market_data.db）")
    _parser.add_argument(
        "--holidays", nargs="*", default=[],
        help="假日清單（ISO 格式，多個日期以空格分隔）",
    )
    _parser.add_argument("--timeout", type=float, default=30.0, help="HTTP 逾時秒數（預設：30）")
    _parser.add_argument("--retries", type=int, default=3, help="HTTP 重試次數（預設：3）")
    _args = _parser.parse_args()

    _policy = HttpPolicy(
        timeout_seconds=_args.timeout,
        retry_count=_args.retries,
        backoff_seconds=1.5,
        headers={"user-agent": _make_user_agent()},
        ssl_verify=False,
    )
    _fetcher = make_fetcher(db_path=_args.db, policy=_policy)
    _outcomes, _report = run_date_range_ingestion(
        start_date=_args.start,
        end_date=_args.end,
        holidays=set(_args.holidays),
        fetcher=_fetcher,
    )

    # 摘要輸出
    for _o in _outcomes:
        status = "✓" if _o.ok else "✗"
        print(f"  {status} {_o.day_type:12s} {_o.message}")

    print(f"\n完成：成功 {_report.success_count} 天，失敗 {_report.failed_count} 天")
    sys.exit(0 if _report.failed_count == 0 else 1)
