from __future__ import annotations

from dataclasses import dataclass
from typing import Any


@dataclass(slots=True)
class ParseError:
    market_code: str
    source_line_no: int
    reason: str


@dataclass(slots=True)
class CanonicalQuote:
    trade_date: str
    market_code: str
    symbol_raw: str
    symbol_clean: str
    security_name: str | None
    instrument_type: str
    instrument_subtype: str | None
    isin_code: str | None
    cfi_code: str | None
    section_label: str | None
    open_price: float | None
    high_price: float | None
    low_price: float | None
    close_price: float | None
    volume_shares: int | None
    turnover_value: float | None
    trade_count: int | None
    change_sign: str | None
    change_value: float | None
    last_bid_price: float | None
    last_bid_qty_raw: str | None
    last_ask_price: float | None
    last_ask_qty_raw: str | None
    pe_ratio: float | None
    issued_shares: int | None
    next_limit_up: float | None
    next_limit_down: float | None
    source_file: str
    source_encoding: str
    source_line_no: int
    ingest_run_id: str
    ingested_at: str
    extension: dict[str, Any]
