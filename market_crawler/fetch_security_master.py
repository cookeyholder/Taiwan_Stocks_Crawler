"""從 TWSE/TPEX ISIN 查詢頁面抓取有價證券主檔，更新 StockID.csv。

使用方式（命令列）：
    python -m market_crawler.fetch_security_master
    python -m market_crawler.fetch_security_master --output /path/to/StockID.csv
    python -m market_crawler.fetch_security_master --timeout 60 --retries 5

功能說明：
- 以 Python 標準函式庫 html.parser 解析 HTML 表格，不依賴 lxml/beautifulsoup4。
- 支援 big5hkscs / cp950 / UTF-8 等多種編碼 fallback。
- 包含所有商品類型（股票、ETF、債券、上市基金等），不濾除 CFICode。
- 最低資料列數防呆（< 100 列不寫入），避免因抓取失敗覆蓋舊版資料。
- 原子寫入（先寫暫存檔再改名），避免寫入中斷造成 CSV 損毀。
- 每個市場獨立錯誤處理，一個市場失敗不影響另一個市場。
"""

from __future__ import annotations

import csv
import os
import tempfile
from dataclasses import dataclass, field
from html.parser import HTMLParser

from market_crawler.http_client import HttpPolicy, get_with_retry

# ISIN 公開查詢頁面
#   strMode=2：上市有價證券（TWSE）
#   strMode=4：上櫃有價證券（TPEX）
_ISIN_SOURCES: dict[str, str] = {
    "上市": "https://isin.twse.com.tw/isin/C_public.jsp?strMode=2",
    "上櫃": "https://isin.twse.com.tw/isin/C_public.jsp?strMode=4",
}

# 輸出欄位（與 market_crawler.classification.load_security_master 預期結構一致）
_OUTPUT_COLS: tuple[str, ...] = ("代號", "名稱", "上市日", "市場別", "產業別", "CFICode")

# 防呆閾值：低於此列數視為抓取失敗，不覆蓋舊檔
_MIN_ROWS: int = 100


# ---------------------------------------------------------------------------
# 資料結構
# ---------------------------------------------------------------------------


@dataclass(slots=True)
class FetchResult:
    """抓取結果摘要。"""

    total_rows: int
    market_counts: dict[str, int] = field(default_factory=dict)
    errors: list[str] = field(default_factory=list)

    @property
    def success(self) -> bool:
        """是否已成功寫入檔案（資料列數達到閾值）。"""
        return self.total_rows >= _MIN_ROWS


# ---------------------------------------------------------------------------
# HTML 表格解析（stdlib only）
# ---------------------------------------------------------------------------


class _TableParser(HTMLParser):
    """極簡 HTML 表格解析器，使用 Python 標準函式庫，不依賴外部套件。"""

    def __init__(self) -> None:
        super().__init__()
        self._in_cell: bool = False
        self._current_row: list[str] = []
        self._current_cell_parts: list[str] = []
        self.rows: list[list[str]] = []

    def handle_starttag(self, tag: str, attrs: list) -> None:  # noqa: ARG002
        if tag in ("td", "th"):
            self._in_cell = True
            self._current_cell_parts = []
        elif tag == "tr":
            self._current_row = []

    def handle_endtag(self, tag: str) -> None:
        if tag in ("td", "th"):
            self._in_cell = False
            self._current_row.append("".join(self._current_cell_parts).strip())
        elif tag == "tr":
            if self._current_row:
                self.rows.append(list(self._current_row))

    def handle_data(self, data: str) -> None:
        if self._in_cell:
            self._current_cell_parts.append(data)


# ---------------------------------------------------------------------------
# 內部工具函式
# ---------------------------------------------------------------------------


def _decode_response(content: bytes) -> str:
    """以多重編碼 fallback 解碼 ISIN 頁面回應（Big5 → CP950 → UTF-8）。"""
    for enc in ("big5hkscs", "cp950", "utf-8-sig", "utf-8"):
        try:
            return content.decode(enc)
        except (UnicodeDecodeError, LookupError):
            continue
    return content.decode("utf-8", errors="replace")


def _parse_isin_page(html_text: str) -> list[dict[str, str]]:
    """解析 ISIN 公開查詢頁面 HTML，回傳有效的有價證券 records 清單。

    過濾規則：
    - 第一欄「有價證券代號及名稱」必須能以空白拆成代號與名稱兩部分。
    - 代號第一個字元必須為英文或數字（排除段落標頭列，如「水泥工業」）。
    - 資料列欄位數不足 6 欄者略過。
    """
    parser = _TableParser()
    parser.feed(html_text)
    rows = parser.rows

    if len(rows) < 2:  # 至少要有標頭列 + 一筆資料列
        return []

    if len(rows[0]) < 6:  # 標頭欄位數不足，格式異常
        return []

    records: list[dict[str, str]] = []
    for row in rows[1:]:
        if len(row) < 6:
            continue

        # 第一欄格式：「代號 名稱」（以空白隔開）
        parts = row[0].strip().split(None, 1)
        if len(parts) < 2:
            # 段落標頭列（例如「水泥工業」、「上市」）或空列，略過
            continue

        symbol = parts[0].strip()
        name = parts[1].strip()

        # 代號必須以英文字母或數字開頭
        if not symbol or not symbol[0].isalnum():
            continue

        records.append(
            {
                "代號": symbol,
                "名稱": name,
                "上市日": row[2].strip(),
                "市場別": row[3].strip(),
                "產業別": row[4].strip(),
                "CFICode": row[5].strip(),
            }
        )

    return records


# ---------------------------------------------------------------------------
# 主要公開函式
# ---------------------------------------------------------------------------


def fetch_and_save(
    output_path: str,
    policy: HttpPolicy | None = None,
) -> FetchResult:
    """從 TWSE/TPEX ISIN 頁面抓取有價證券主檔並寫入 CSV。

    - 上市（TWSE）和上櫃（TPEX）分開抓取，任一失敗不影響另一來源。
    - 同一代號若同時出現在兩個市場，以先遇到者（上市優先）為準，避免重複。
    - 合計資料列數 < 100 時視為抓取失敗，不覆蓋既有檔案。
    - 採原子寫入：先寫暫存檔再 os.replace，避免因中斷產生損毀 CSV。

    Args:
        output_path: 輸出 CSV 的路徑（例如 ``"StockID.csv"``）。
        policy: HTTP 請求策略。預設 timeout=30s、retry=3 次、backoff=1s。

    Returns:
        :class:`FetchResult` — 包含 total_rows、market_counts、errors 清單。
    """
    if policy is None:
        policy = HttpPolicy(timeout_seconds=30.0, retry_count=3, backoff_seconds=1.0)

    all_records: list[dict[str, str]] = []
    errors: list[str] = []
    market_counts: dict[str, int] = {}
    seen_symbols: set[str] = set()

    for market_label, url in _ISIN_SOURCES.items():
        try:
            response = get_with_retry(url, policy)
        except RuntimeError as exc:
            errors.append(f"{market_label}：抓取失敗 — {exc}")
            continue

        html_text = _decode_response(response.content)
        records = _parse_isin_page(html_text)

        if not records:
            errors.append(f"{market_label}：解析不到有效資料列")
            continue

        count = 0
        for rec in records:
            sym = rec["代號"]
            if sym not in seen_symbols:
                seen_symbols.add(sym)
                all_records.append(rec)
                count += 1
        market_counts[market_label] = count

    total = len(all_records)

    if total < _MIN_ROWS:
        errors.append(f"資料列數不足（{total} < {_MIN_ROWS}），不寫入檔案")
        return FetchResult(total_rows=total, market_counts=market_counts, errors=errors)

    # 原子寫入：先寫暫存檔，成功後才以 os.replace 替換目標檔
    abs_output = os.path.abspath(output_path)
    dir_name = os.path.dirname(abs_output)
    fd, tmp_path = tempfile.mkstemp(dir=dir_name, suffix=".tmp", prefix="StockID_")
    try:
        with os.fdopen(fd, "w", encoding="utf-8", newline="") as fout:
            writer = csv.DictWriter(fout, fieldnames=list(_OUTPUT_COLS))
            writer.writeheader()
            writer.writerows(all_records)
        os.replace(tmp_path, abs_output)
    except Exception:
        try:
            os.unlink(tmp_path)
        except OSError:
            pass
        raise

    return FetchResult(total_rows=total, market_counts=market_counts, errors=errors)


# ---------------------------------------------------------------------------
# 命令列入口
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    import argparse
    import sys

    _arg_parser = argparse.ArgumentParser(
        description="更新有價證券主檔 StockID.csv（從 TWSE/TPEX ISIN 公開查詢頁面抓取）"
    )
    _arg_parser.add_argument(
        "--output",
        default="StockID.csv",
        help="輸出 CSV 路徑（預設：StockID.csv）",
    )
    _arg_parser.add_argument(
        "--timeout",
        type=float,
        default=30.0,
        help="每次 HTTP 請求逾時秒數（預設：30）",
    )
    _arg_parser.add_argument(
        "--retries",
        type=int,
        default=3,
        help="HTTP 請求最大重試次數（預設：3）",
    )
    _args = _arg_parser.parse_args()

    _policy = HttpPolicy(
        timeout_seconds=_args.timeout,
        retry_count=_args.retries,
        backoff_seconds=1.0,
    )
    _result = fetch_and_save(output_path=_args.output, policy=_policy)

    if _result.success:
        print(f"✓ 已寫入 {_result.total_rows} 筆 → {_args.output}")
        for _market, _count in _result.market_counts.items():
            print(f"  {_market}：{_count} 筆")
    else:
        print(
            f"✗ 更新失敗（資料列數：{_result.total_rows}）",
            file=sys.stderr,
        )
    for _err in _result.errors:
        print(f"  警告：{_err}", file=sys.stderr)

    sys.exit(0 if _result.success else 1)
