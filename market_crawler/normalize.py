from __future__ import annotations

import re
from datetime import datetime
from typing import Any

CANONICAL_CORE_FIELDS = [
    "trade_date",
    "market_code",
    "symbol_raw",
    "symbol_clean",
    "security_name",
    "instrument_type",
    "instrument_subtype",
    "isin_code",
    "cfi_code",
    "section_label",
    "open_price",
    "high_price",
    "low_price",
    "close_price",
    "volume_shares",
    "turnover_value",
    "trade_count",
    "change_sign",
    "change_value",
    "last_bid_price",
    "last_bid_qty_raw",
    "last_ask_price",
    "last_ask_qty_raw",
    "pe_ratio",
    "issued_shares",
    "next_limit_up",
    "next_limit_down",
    "source_file",
    "source_encoding",
    "source_line_no",
    "ingest_run_id",
    "ingested_at",
]

CANONICAL_EXTENSION_FIELDS = ["raw_change", "raw_payload"]


def clean_symbol(symbol_raw: str) -> str:
    value = symbol_raw.strip()
    # Keep the original symbol untouched elsewhere; this returns canonical clean key.
    match = re.fullmatch(r"=\"([^\"]+)\"", value)
    if match:
        return match.group(1).strip()
    value = value.strip('"')
    return value.strip()


def clean_numeric(value: str | None) -> str | None:
    if value is None:
        return None
    cleaned = value.strip().replace(",", "")
    if cleaned in {"", "--", "---"}:
        return None
    return cleaned


def to_int(value: str | None) -> int | None:
    cleaned = clean_numeric(value)
    return int(cleaned) if cleaned is not None else None


def to_float(value: str | None) -> float | None:
    cleaned = clean_numeric(value)
    return float(cleaned) if cleaned is not None else None


def parse_change_field(change: str | None) -> tuple[str | None, float | None]:
    if not change or change.strip() in {"", "--"}:
        return None, None
    value = change.strip()
    if value[0] in {"+", "-"}:
        sign = value[0]
        magnitude = to_float(value[1:])
        return sign, magnitude
    return None, to_float(value)


def roc_to_ad_date(value: str) -> str:
    value = value.strip()
    if "-" in value:
        dt = datetime.strptime(value, "%Y-%m-%d")
        return dt.strftime("%Y-%m-%d")
    parts = value.split("/")
    if len(parts) != 3:
        raise ValueError(f"Unsupported date format: {value}")
    year = int(parts[0])
    month = int(parts[1])
    day = int(parts[2])
    if year < 1911:
        year += 1911
    dt = datetime(year=year, month=month, day=day)
    return dt.strftime("%Y-%m-%d")


def normalize_record(
    *,
    market_code: str,
    trade_date: str,
    record: dict[str, Any],
    source_encoding: str,
    ingest_run_id: str,
    ingested_at: str,
) -> dict[str, Any]:
    symbol_raw = record["symbol"]
    symbol_clean = clean_symbol(symbol_raw)

    if market_code == "TPEX":
        change_sign, change_value = parse_change_field(record.get("change"))
    else:
        _raw_sign = record.get("change_sign")
        change_sign = (_raw_sign.strip() if _raw_sign else None) or None
        change_value = to_float(record.get("change_value"))

    return {
        "trade_date": roc_to_ad_date(trade_date),
        "market_code": market_code,
        "symbol_raw": symbol_raw,
        "symbol_clean": symbol_clean,
        "security_name": record.get("name"),
        "instrument_type": "other",
        "instrument_subtype": None,
        "isin_code": None,
        "cfi_code": None,
        "section_label": None,
        "open_price": to_float(record.get("open")),
        "high_price": to_float(record.get("high")),
        "low_price": to_float(record.get("low")),
        "close_price": to_float(record.get("close")),
        "volume_shares": to_int(record.get("volume")),
        "turnover_value": to_float(record.get("turnover")),
        "trade_count": to_int(record.get("trade_count")),
        "change_sign": change_sign,
        "change_value": change_value,
        "last_bid_price": to_float(record.get("last_bid")),
        "last_bid_qty_raw": record.get("last_bid_qty"),
        "last_ask_price": to_float(record.get("last_ask")),
        "last_ask_qty_raw": record.get("last_ask_qty"),
        "pe_ratio": to_float(record.get("pe_ratio")),
        "issued_shares": to_int(record.get("issued_shares")),
        "next_limit_up": to_float(record.get("next_limit_up")),
        "next_limit_down": to_float(record.get("next_limit_down")),
        "source_file": record.get("source_file", ""),
        "source_encoding": source_encoding,
        "source_line_no": record.get("source_line_no", 0),
        "ingest_run_id": ingest_run_id,
        "ingested_at": ingested_at,
        "raw_change": record.get("change") or record.get("change_value"),
        "raw_payload": record,
    }
