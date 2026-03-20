from __future__ import annotations

import csv
from dataclasses import dataclass
from io import StringIO
from typing import Any

from market_crawler.encoding import decode_with_fallback
from market_crawler.models import ParseError


@dataclass(slots=True)
class ParseResult:
    market_code: str
    source_encoding: str
    records: list[dict[str, Any]]
    errors: list[ParseError]


TWSE_HEADER = [
    "證券代號",
    "證券名稱",
    "成交股數",
    "成交筆數",
    "成交金額",
    "開盤價",
    "最高價",
    "最低價",
    "收盤價",
    "漲跌(+/-)",
    "漲跌價差",
    "最後揭示買價",
    "最後揭示買量",
    "最後揭示賣價",
    "最後揭示賣量",
    "本益比",
]

TPEX_HEADER = [
    "代號",
    "名稱",
    "收盤",
    "漲跌",
    "開盤",
    "最高",
    "最低",
    "成交股數",
    "成交金額(元)",
    "成交筆數",
    "最後買價",
    "最後買量(張數)",
    "最後賣價",
    "最後賣量(張數)",
    "發行股數",
    "次日漲停價",
    "次日跌停價",
]


def _normalize_header_cell(value: str) -> str:
    return value.replace(" ", "").strip()


def parse_twse_daily_close(content: bytes, source_file: str) -> ParseResult:
    text, encoding = decode_with_fallback(content)
    rows = list(csv.reader(StringIO(text)))

    section_idx = -1
    header_idx = -1
    for i, row in enumerate(rows):
        if row and "每日收盤行情(全部)" in row[0]:
            section_idx = i
        if section_idx >= 0 and row:
            normalized = [_normalize_header_cell(cell) for cell in row]
            if normalized[: len(TWSE_HEADER)] == TWSE_HEADER:
                header_idx = i
                break

    if header_idx < 0:
        return ParseResult(
            market_code="TWSE",
            source_encoding=encoding,
            records=[],
            errors=[
                ParseError(
                    market_code="TWSE",
                    source_line_no=0,
                    reason="TWSE daily close section/header not found",
                )
            ],
        )

    records: list[dict[str, Any]] = []
    errors: list[ParseError] = []

    for line_no, row in enumerate(rows[header_idx + 1 :], start=header_idx + 2):
        if not row or not row[0].strip():
            break
        if row[0].startswith("說明") or row[0].startswith("備註"):
            break
        if len(row) < len(TWSE_HEADER):
            errors.append(
                ParseError(
                    market_code="TWSE",
                    source_line_no=line_no,
                    reason=f"Expected >=16 columns, got {len(row)}",
                )
            )
            continue

        records.append(
            {
                "symbol": row[0].strip(),
                "name": row[1].strip(),
                "volume": row[2].strip(),
                "trade_count": row[3].strip(),
                "turnover": row[4].strip(),
                "open": row[5].strip(),
                "high": row[6].strip(),
                "low": row[7].strip(),
                "close": row[8].strip(),
                "change_sign": row[9].strip(),
                "change_value": row[10].strip(),
                "last_bid": row[11].strip(),
                "last_bid_qty": row[12].strip(),
                "last_ask": row[13].strip(),
                "last_ask_qty": row[14].strip(),
                "pe_ratio": row[15].strip(),
                "source_file": source_file,
                "source_line_no": line_no,
            }
        )

    return ParseResult(
        market_code="TWSE",
        source_encoding=encoding,
        records=records,
        errors=errors,
    )


def parse_tpex_daily_close(content: bytes, source_file: str) -> ParseResult:
    text, encoding = decode_with_fallback(content)
    rows = list(csv.reader(StringIO(text)))

    header_idx = -1
    for i, row in enumerate(rows):
        normalized = [_normalize_header_cell(cell) for cell in row]
        if normalized[: len(TPEX_HEADER)] == TPEX_HEADER:
            header_idx = i
            break

    if header_idx < 0:
        return ParseResult(
            market_code="TPEX",
            source_encoding=encoding,
            records=[],
            errors=[
                ParseError(
                    market_code="TPEX",
                    source_line_no=0,
                    reason="TPEX header not found",
                )
            ],
        )

    records: list[dict[str, Any]] = []
    errors: list[ParseError] = []

    for line_no, row in enumerate(rows[header_idx + 1 :], start=header_idx + 2):
        if not row or not row[0].strip():
            continue
        if len(row) < len(TPEX_HEADER):
            errors.append(
                ParseError(
                    market_code="TPEX",
                    source_line_no=line_no,
                    reason=f"Expected >=17 columns, got {len(row)}",
                )
            )
            continue

        records.append(
            {
                "symbol": row[0].strip(),
                "name": row[1].strip(),
                "close": row[2].strip(),
                "change": row[3].strip(),
                "open": row[4].strip(),
                "high": row[5].strip(),
                "low": row[6].strip(),
                "volume": row[7].strip(),
                "turnover": row[8].strip(),
                "trade_count": row[9].strip(),
                "last_bid": row[10].strip(),
                "last_bid_qty": row[11].strip(),
                "last_ask": row[12].strip(),
                "last_ask_qty": row[13].strip(),
                "issued_shares": row[14].strip(),
                "next_limit_up": row[15].strip(),
                "next_limit_down": row[16].strip(),
                "source_file": source_file,
                "source_line_no": line_no,
            }
        )

    return ParseResult(
        market_code="TPEX",
        source_encoding=encoding,
        records=records,
        errors=errors,
    )
