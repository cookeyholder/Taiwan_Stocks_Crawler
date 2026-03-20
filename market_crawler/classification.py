from __future__ import annotations

import re
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


def _from_ticker_pattern(symbol: str) -> str:
    # Based on Taiwan ticker coding principles fallback rules.
    if re.fullmatch(r"\d{4}", symbol):
        return "stock"
    if re.fullmatch(r"\d{4}[A-Y]", symbol):
        return "preferred_stock"
    if re.fullmatch(r"\d{4}Z[1-9]", symbol):
        return "exchangeable_preferred_stock"
    if re.fullmatch(r"\d{4}G[1-9]", symbol):
        return "warrant_certificate"
    if re.fullmatch(r"\d{2}\d{3}[PUTFQCBXY]", symbol):
        return "warrant"
    if re.fullmatch(r"\d{4}[A-Z]", symbol) and symbol.endswith("B"):
        return "etf"
    if re.fullmatch(r"[BGFAHPI]\w{5}", symbol):
        return "bond"
    if re.fullmatch(r"T\w{5}", symbol):
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
