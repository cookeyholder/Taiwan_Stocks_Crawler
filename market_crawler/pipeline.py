"""market_crawler/pipeline.py — 端到端每日行情抓取與入庫流程。

整合 http_client、parsers、normalize、classification、repository，
提供可直接呼叫的 `fetch_and_ingest_day()` 與供 orchestrator 使用的
`make_fetcher()` 包裝函式。

資料來源：
- TWSE （今日）：https://openapi.twse.com.tw/v1/exchangeReport/STOCK_DAY_ALL（JSON）
  注意：此 API 不接受日期參數，固定回傳最新交易日資料。
- TWSE （歷史）：https://www.twse.com.tw/rwd/zh/afterTrading/MI_INDEX?response=csv&date=YYYYMMDD&type=ALL
- TPEX：https://www.tpex.org.tw/www/zh-tw/afterTrading/dailyQuotes?response=json（JSON API，含日期參數）

命令列用法：
    twse-crawler --today
    twse-crawler --date 2026-03-17
    twse-crawler --backfill 30
    twse-crawler --start 2026-03-16 --end 2026-03-20
    python -m market_crawler.pipeline --today
"""

from __future__ import annotations

import argparse
import datetime as _dt
import re as _re
import sqlite3
import sys
import uuid
import warnings
from collections.abc import Callable

import urllib3

from market_crawler.classification import classify_instrument, load_security_master
from market_crawler.http_client import HttpPolicy, get_with_retry
from market_crawler.normalize import normalize_record
from market_crawler.orchestrator import run_date_range_ingestion
from market_crawler.parsers import (
    parse_tpex_daily_close_json,
    parse_twse_daily_close,
    parse_twse_daily_close_json,
)
from market_crawler.repository import SQLiteMarketRepository

# ---------------------------------------------------------------------------
# 抑制 InsecureRequestWarning（TWSE/TPEX SSL 憑證問題）
# ---------------------------------------------------------------------------
urllib3.disable_warnings(urllib3.exceptions.InsecureRequestWarning)

# ---------------------------------------------------------------------------
# 輔助：取得今日日期（可被 monkeypatch 覆蓋）
# ---------------------------------------------------------------------------


def _get_today() -> _dt.date:
    """回傳今日日期（測試時可 monkeypatch 此函式以控制「今天」）。"""
    return _dt.date.today()


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

# TWSE OpenAPI（今日）：無日期參數，固定回傳最新交易日資料
_TWSE_OPENAPI_URL = "https://openapi.twse.com.tw/v1/exchangeReport/STOCK_DAY_ALL"

# TWSE 歷史 CSV API（有日期參數）
_TWSE_CSV_URL_TEMPLATE = "https://www.twse.com.tw/rwd/zh/afterTrading/MI_INDEX?response=csv&date={yyyymmdd}&type=ALL"


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
    """建立 TPEX 每日收盤行情 JSON 下載 URL（新版 API，民國年格式）。"""
    year, month, day = date_str.split("-")
    roc_year = int(year) - 1911
    return (
        f"https://www.tpex.org.tw/www/zh-tw/afterTrading/dailyQuotes"
        f"?date={roc_year}/{month}/{day}&type=S&response=json"
    )


def _build_twse_csv_url(date_str: str) -> str:
    """建立 TWSE 歷史 CSV API URL。

    Args:
        date_str: ISO 格式日期字串，例如 ``"2026-03-17"``。

    Returns:
        完整 URL，日期以 YYYYMMDD 格式嵌入，例如
        ``"https://www.twse.com.tw/rwd/zh/afterTrading/MI_INDEX?response=csv&date=20260317&type=ALL"``
    """
    yyyymmdd = date_str.replace("-", "")
    return _TWSE_CSV_URL_TEMPLATE.format(yyyymmdd=yyyymmdd)


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
    """抓取 TWSE OpenAPI（今日 JSON）並驗證日期；日期不符時回傳 ``None``。"""
    resp = get_with_retry(_TWSE_OPENAPI_URL, policy)
    response_json = resp.json()

    # 驗證 API 回傳的日期與請求日期是否一致
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


def _fetch_and_parse_twse_csv(date_str: str, policy: HttpPolicy) -> object:
    """抓取 TWSE 歷史 CSV API（MI_INDEX）並解析。

    使用具備日期參數的 CSV API，適用於歷史日期（date < today）。

    Args:
        date_str: ISO 格式日期字串，例如 ``"2026-03-17"``。
        policy: HTTP 請求策略。

    Returns:
        ``ParseResult`` 物件（records + errors）。
    """
    url = _build_twse_csv_url(date_str)
    resp = get_with_retry(url, policy)
    return parse_twse_daily_close(resp.content, source_file=url)


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
) -> bool | None:
    """抓取指定日期的 TWSE 與 TPEX 每日收盤行情並寫入 SQLite。

    **TWSE 路由邏輯**:
    - ``date_str == today`` → 使用 TWSE JSON API（STOCK_DAY_ALL）
    - ``date_str < today`` → 使用 TWSE 歷史 CSV API（MI_INDEX?date=YYYYMMDD）

    Args:
        date_str: ISO 格式交易日，例如 ``"2026-03-17"``。
        db_path: SQLite 資料庫路徑。
        security_master_path: StockID.csv 路徑；``None`` 則跳過分類擴充。
        policy: HTTP 請求策略。
        ingest_run_id: 本次匯入的唯一識別碼（留空時自動產生 UUID）。

    Returns:
        - ``True``  — 至少有一個市場成功寫入資料
        - ``False`` — 兩個市場均寫入失敗（網路/格式錯誤）
        - ``None``  — TWSE 回傳日期不符（market_closed 訊號）
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

    ingested_at = _dt.datetime.now(_dt.timezone.utc).isoformat()

    # 載入有價證券主檔
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
    twse_market_closed = False  # TWSE 日期不符旗標
    today = _get_today()
    requested_date = _dt.date.fromisoformat(date_str)

    with conn:
        # --- TWSE ---
        try:
            if requested_date < today:
                # 歷史日期：使用 CSV API
                twse_result = _fetch_and_parse_twse_csv(date_str, policy)
            else:
                # 今日：使用 JSON API（date_str == today 或 future）
                twse_result = _fetch_and_parse_twse(date_str, policy)

            if twse_result is None:
                # JSON API 路徑：日期不符 → market_closed 訊號
                twse_market_closed = True
            else:
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

    if total_written == 0 and twse_market_closed:
        return None  # market_closed 訊號
    return total_written > 0


# ---------------------------------------------------------------------------
# make_fetcher：供 orchestrator 使用的 Callable
# ---------------------------------------------------------------------------


def make_fetcher(
    db_path: str = "market_data.db",
    security_master_path: str | None = "StockID.csv",
    policy: HttpPolicy | None = None,
) -> Callable[[str], bool | None]:
    """回傳一個 ``(date_str) -> bool | None`` 的可呼叫物件，供 `run_date_range_ingestion` 使用。

    Args:
        db_path: SQLite 資料庫路徑。
        security_master_path: StockID.csv 路徑。
        policy: HTTP 請求策略（``None`` 使用預設值）。

    Returns:
        符合 orchestrator 介面的 fetcher callable：
        - ``True``  — 成功寫入
        - ``False`` — 失敗（網路/格式錯誤）
        - ``None``  — market_closed 訊號
    """
    if policy is None:
        policy = HttpPolicy(
            timeout_seconds=30.0,
            retry_count=3,
            backoff_seconds=1.5,
            headers={"user-agent": _make_user_agent()},
            ssl_verify=False,
        )

    def _fetcher(date_str: str) -> bool | None:
        return fetch_and_ingest_day(
            date_str,
            db_path=db_path,
            security_master_path=security_master_path,
            policy=policy,
        )

    return _fetcher


# ---------------------------------------------------------------------------
# 假日抓取：best-effort TWSE holidaySchedule API
# ---------------------------------------------------------------------------

_TWSE_HOLIDAY_URL = "https://www.twse.com.tw/rwd/zh/holidaySchedule/holidaySchedule?response=json&queryYear={year}"


def _fetch_twse_holidays(policy: HttpPolicy) -> set[_dt.date]:
    """嘗試從 TWSE holidaySchedule API 抓取當年國定假日清單。

    Args:
        policy: HTTP 請求策略。

    Returns:
        當年國定假日的 ``date`` 物件集合。若 API 失敗則拋出例外（由呼叫端 try/except 處理）。
    """
    year = _get_today().year
    url = _TWSE_HOLIDAY_URL.format(year=year)
    resp = get_with_retry(url, policy)
    data = resp.json()

    holidays: set[_dt.date] = set()
    # API 回傳格式：{"stat": "OK", "data": [[...], ...]}
    # 第一欄通常為日期字串（西元 YYYY/MM/DD 或民國 YYY/MM/DD）
    rows = data.get("data", []) if isinstance(data, dict) else []
    for row in rows:
        if not row:
            continue
        raw_date = str(row[0]).strip()
        # 支援 YYYY/MM/DD 格式
        try:
            parts = (
                raw_date.replace("年", "/")
                .replace("月", "/")
                .replace("日", "")
                .split("/")
            )
            if len(parts) == 3:
                year_part = int(parts[0])
                month_part = int(parts[1])
                day_part = int(parts[2])
                # 民國年轉西元 (< 200 視為民國年)
                if year_part < 200:
                    year_part += 1911
                holidays.add(_dt.date(year_part, month_part, day_part))
        except (ValueError, IndexError):
            continue

    return holidays


# ---------------------------------------------------------------------------
# 命令列入口
# ---------------------------------------------------------------------------

_ISO_DATE_RE = _re.compile(r"^\d{4}-\d{2}-\d{2}$")


def _validate_iso_date(
    value: str, flag: str, parser: argparse.ArgumentParser
) -> _dt.date:
    """驗證字串為合法 ISO 日期，否則呼叫 parser.error() 報錯。"""
    if not _ISO_DATE_RE.match(value):
        parser.error(f"{flag} 格式錯誤（須為 YYYY-MM-DD）：{value!r}")
    try:
        return _dt.date.fromisoformat(value)
    except ValueError:
        parser.error(f"{flag} 必須為有效日期（YYYY-MM-DD）：{value!r}")
        return _dt.date.min  # unreachable


def _resolve_date_range(
    args: object,
    parser: argparse.ArgumentParser,
    today: _dt.date,
) -> tuple[str, str]:
    """將 argparse Namespace 解析為 ``(start_date, end_date)`` ISO 字串。"""
    assert isinstance(args, argparse.Namespace)

    if (args.today or args.date or args.backfill is not None) and (
        args.start or args.end
    ):
        parser.error("--today/--date/--backfill 與 --start/--end 不能同時使用")

    if args.today:
        return today.isoformat(), today.isoformat()

    if args.date:
        iso = _validate_iso_date(args.date, "--date", parser).isoformat()
        return iso, iso

    if args.backfill is not None:
        if args.backfill <= 0:
            parser.error(f"--backfill 值必須大於 0，得到：{args.backfill}")
        return (
            today - _dt.timedelta(days=args.backfill)
        ).isoformat(), today.isoformat()

    if args.start and args.end:
        start_date = _validate_iso_date(args.start, "--start", parser)
        end_date = _validate_iso_date(args.end, "--end", parser)
        if start_date > end_date:
            parser.error(f"--start 不得晚於 --end：{args.start} > {args.end}")
        return args.start, args.end
        return args.start, args.end

    parser.error("請指定 --today、--date、--backfill 或 --start/--end 其中一種模式")
    return "", ""  # unreachable


def main(argv: list[str] | None = None) -> None:
    """twse-crawler CLI 主程式。

    支援四種執行模式（互斥）：

    - ``--today``          — 抓取今日資料
    - ``--date YYYY-MM-DD`` — 抓取單一指定日期
    - ``--backfill N``     — 從今日往前 N 天（含今日）
    - ``--start ... --end ...`` — 指定區間

    其他引數：

    - ``--db PATH``         — SQLite 資料庫路徑（預設 ``market_data.db``）
    - ``--holidays DATE ...`` — 手動指定假日清單（ISO 格式，可多個）
    - ``--no-auto-holidays``  — 停用自動抓取 TWSE holidaySchedule API
    - ``--timeout``、``--retries`` — HTTP 參數
    """
    parser = argparse.ArgumentParser(
        description="抓取 TWSE/TPEX 每日收盤行情並寫入 SQLite",
    )

    # 執行模式（互斥）
    mode_group = parser.add_mutually_exclusive_group()
    mode_group.add_argument("--today", action="store_true", help="抓取今日資料")
    mode_group.add_argument("--date", metavar="YYYY-MM-DD", help="抓取指定單日")
    mode_group.add_argument(
        "--backfill", type=int, metavar="N", help="從今日往前 N 天（含今日）"
    )

    # 區間引數（與 --today/--date/--backfill 互斥，不加入同一 group 以允許 --start/--end 搭配使用）
    parser.add_argument("--start", metavar="YYYY-MM-DD", help="起始日期（ISO 格式）")
    parser.add_argument("--end", metavar="YYYY-MM-DD", help="結束日期（ISO 格式）")

    # 其他引數
    parser.add_argument(
        "--db",
        default="market_data.db",
        help="SQLite 資料庫路徑（預設：market_data.db）",
    )
    parser.add_argument(
        "--holidays",
        nargs="*",
        default=[],
        metavar="DATE",
        help="假日清單（ISO 格式，多個日期以空格分隔）",
    )
    parser.add_argument(
        "--no-auto-holidays",
        action="store_true",
        help="停用自動抓取 TWSE holidaySchedule",
    )
    parser.add_argument(
        "--timeout", type=float, default=30.0, help="HTTP 逾時秒數（預設：30）"
    )
    parser.add_argument(
        "--retries", type=int, default=3, help="HTTP 重試次數（預設：3）"
    )

    args = parser.parse_args(argv)

    # --- 解析日期區間 ---
    start_date, end_date = _resolve_date_range(args, parser, _get_today())

    # --- HTTP 策略 ---
    policy = HttpPolicy(
        timeout_seconds=args.timeout,
        retry_count=args.retries,
        backoff_seconds=1.5,
        headers={"user-agent": _make_user_agent()},
        ssl_verify=False,
    )

    # --- 假日集合 ---
    holidays: set[str] = set(args.holidays)

    if not args.no_auto_holidays:
        try:
            auto_holidays = _fetch_twse_holidays(policy)
            holidays |= {d.isoformat() for d in auto_holidays}
        except Exception as exc:  # noqa: BLE001
            warnings.warn(f"自動抓取 TWSE 假日失敗，繼續執行（{exc}）", stacklevel=1)

    # --- 執行 ---
    fetcher = make_fetcher(db_path=args.db, policy=policy)
    outcomes, report = run_date_range_ingestion(
        start_date=start_date,
        end_date=end_date,
        holidays=holidays,
        fetcher=fetcher,
    )

    # 摘要輸出
    for outcome in outcomes:
        status = "✓" if outcome.ok else "✗"
        print(f"  {status} {outcome.day_type:14s} {outcome.message}")

    market_closed = report.as_dict().get("market_closed_count", 0)
    print(
        f"\n完成：成功 {report.success_count} 天，"
        f"休市 {market_closed} 天，"
        f"失敗 {report.failed_count} 天"
    )
    if report.failed_count > 0:
        sys.exit(1)


if __name__ == "__main__":
    main()
