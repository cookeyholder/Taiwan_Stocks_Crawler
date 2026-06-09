from __future__ import annotations

import re
import sqlite3
from dataclasses import dataclass


@dataclass(slots=True)
class SecurityMasterEntry:
    symbol_clean: str
    name: str
    market: str
    section_label: str
    cfi_code: str


def load_security_master(path: str) -> dict[str, SecurityMasterEntry]:
    import csv

    master: dict[str, SecurityMasterEntry] = {}
    with open(path, encoding="utf-8", newline="") as file_obj:
        for row in csv.DictReader(file_obj):
            symbol = (row.get("代號") or "").strip()
            if not symbol:
                continue
            master[symbol] = SecurityMasterEntry(
                symbol_clean=symbol,
                name=(row.get("名稱") or "").strip(),
                market=(row.get("市場別") or "").strip(),
                section_label=(row.get("產業別") or "").strip(),
                cfi_code=(row.get("CFICode") or "").strip(),
            )
    return master


def _from_section(section_label: str | None) -> str | None:
    if not section_label:
        return None
    label = section_label.lower()
    if "etf" in label:
        return "etf"
    if "權證" in section_label:
        return "warrant"
    if "債" in section_label:
        return "bond"
    return "stock"


def _from_cfi(cfi_code: str | None) -> str | None:
    if not cfi_code:
        return None
    first = cfi_code[:1].upper()
    if first == "E":
        return "stock"
    if first == "D":
        return "bond"
    if first == "R":
        return "warrant"
    return None


_RE_STOCK = re.compile(r"\d{4}")
_RE_ETF_BOND = re.compile(r"\d{4,5}B")
_RE_ETF_6DIGIT = re.compile(r"\d{6}")
_RE_PREFERRED = re.compile(r"\d{4}[A-Y]")
_RE_EXCHANGEABLE = re.compile(r"\d{4}Z[1-9]")
_RE_WARRANT_CERT = re.compile(r"\d{4}G[1-9]")
_RE_WARRANT = re.compile(r"\d{2}\d{3}[PUTFQCBXY]")
_RE_ETF_LETTER = re.compile(r"\d{5}[KLMRSAD]")
_RE_CONVERTIBLE_BOND = re.compile(r"\d{5}")
_RE_BOND = re.compile(r"[BGFAHPI]\w{5}")
_RE_FUND = re.compile(r"T\w{5}")


def _from_ticker_pattern(symbol: str) -> str:
    if _RE_STOCK.fullmatch(symbol):
        return "stock"
    if _RE_ETF_BOND.fullmatch(symbol):
        return "etf"
    if _RE_ETF_6DIGIT.fullmatch(symbol):
        return "etf"
    if _RE_PREFERRED.fullmatch(symbol):
        return "preferred_stock"
    if _RE_EXCHANGEABLE.fullmatch(symbol):
        return "exchangeable_preferred_stock"
    if _RE_WARRANT_CERT.fullmatch(symbol):
        return "warrant_certificate"
    if _RE_WARRANT.fullmatch(symbol):
        return "warrant"
    if _RE_ETF_LETTER.fullmatch(symbol):
        return "etf"
    if _RE_CONVERTIBLE_BOND.fullmatch(symbol):
        return "convertible_bond"
    if _RE_BOND.fullmatch(symbol):
        return "bond"
    if _RE_FUND.fullmatch(symbol):
        return "fund"
    return "other"


def classify_instrument(
    *,
    symbol_clean: str,
    security_master: dict[str, SecurityMasterEntry],
    override: dict[str, str] | None = None,
) -> tuple[str, str | None, str | None, str | None]:
    override = override or {}
    if symbol_clean in override:
        return override[symbol_clean], None, None, None

    entry = security_master.get(symbol_clean)
    if entry:
        from_section = _from_section(entry.section_label)
        if from_section:
            return from_section, None, entry.section_label, entry.cfi_code
        from_cfi = _from_cfi(entry.cfi_code)
        if from_cfi:
            return from_cfi, None, entry.section_label, entry.cfi_code

    return _from_ticker_pattern(symbol_clean), None, None, None


def reclassify_database(conn: sqlite3.Connection, stock_master_path: str | None = None) -> int:
    """對 market_quote_core 全部記錄重新分類，回傳更新的筆數。"""
    master: dict[str, SecurityMasterEntry] = {}
    if stock_master_path:
        try:
            master = load_security_master(stock_master_path)
        except Exception:
            pass
    cur = conn.execute("SELECT DISTINCT symbol_clean FROM market_quote_core")
    updates = 0
    for (symbol,) in cur.fetchall():
        instrument, _, _, _ = classify_instrument(
            symbol_clean=symbol, security_master=master
        )
        conn.execute(
            "UPDATE market_quote_core SET instrument_type = ? WHERE symbol_clean = ?",
            (instrument, symbol),
        )
        updates += 1
    conn.commit()
    return updates
