from __future__ import annotations

from market_crawler.classification import (
    SecurityMasterEntry,
    classify_instrument,
    load_security_master,
)


def test_classification_priority_override(stock_master_path: str) -> None:
    master = load_security_master(stock_master_path)
    instrument, subtype, section, cfi = classify_instrument(
        symbol_clean="0050", security_master=master, override={"0050": "manual_etf"}
    )
    assert instrument == "manual_etf"
    assert subtype is None
    assert section is None
    assert cfi is None


def test_classification_priority_section_then_cfi(stock_master_path: str) -> None:
    master = load_security_master(stock_master_path)
    instrument, _, section, cfi = classify_instrument(symbol_clean="1101", security_master=master)
    assert instrument == "stock"
    assert section is not None
    assert cfi is not None


def test_classification_ticker_fallback_warrant() -> None:
    instrument, _, _, _ = classify_instrument(symbol_clean="03456P", security_master={})
    assert instrument == "warrant"


def test_classification_ticker_fallback_preferred() -> None:
    instrument, _, _, _ = classify_instrument(symbol_clean="1234A", security_master={})
    assert instrument == "preferred_stock"


def test_classification_ticker_fallback_bond_prefix() -> None:
    instrument, _, _, _ = classify_instrument(symbol_clean="B12AB3", security_master={})
    assert instrument == "bond"


def test_classification_ticker_fallback_other() -> None:
    instrument, _, _, _ = classify_instrument(symbol_clean="XYZ", security_master={})
    assert instrument == "other"


def test_classification_uses_cfi_when_section_missing() -> None:
    master = {
        "ABCD": SecurityMasterEntry(
            symbol_clean="ABCD",
            name="sample",
            market="TPEX",
            section_label="",
            cfi_code="RXXXX",
        )
    }
    instrument, _, section, cfi = classify_instrument(symbol_clean="ABCD", security_master=master)
    assert instrument == "warrant"
    assert section == ""
    assert cfi == "RXXXX"


def test_classification_cfi_unknown_falls_back_to_ticker() -> None:
    master = {
        "1234A": SecurityMasterEntry(
            symbol_clean="1234A",
            name="sample",
            market="TWSE",
            section_label="",
            cfi_code="XXXXXX",
        )
    }
    instrument, _, _, _ = classify_instrument(symbol_clean="1234A", security_master=master)
    assert instrument == "preferred_stock"


def test_classification_ticker_fallback_five_digit_bond_etf() -> None:
    """00679B（五位數+B）應分類為 etf，而非 warrant。"""
    instrument, _, _, _ = classify_instrument(
        symbol_clean="00679B", security_master={}
    )
    assert instrument == "etf"


def test_classification_ticker_fallback_four_digit_bond_etf() -> None:
    """0050B（四位數+B）應分類為 etf，而非 preferred_stock。"""
    instrument, _, _, _ = classify_instrument(
        symbol_clean="0050B", security_master={}
    )
    assert instrument == "etf"
