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
    instrument, _, _, _ = classify_instrument(symbol_clean="00679B", security_master={})
    assert instrument == "etf"


def test_classification_ticker_fallback_four_digit_bond_etf() -> None:
    """0050B（四位數+B）應分類為 etf，而非 preferred_stock。"""
    instrument, _, _, _ = classify_instrument(symbol_clean="0050B", security_master={})
    assert instrument == "etf"


def test_classification_6digit_etf() -> None:
    """006208（六碼純數字）應分類為 etf。"""
    instrument, _, _, _ = classify_instrument(symbol_clean="006208", security_master={})
    assert instrument == "etf"


def test_classification_leveraged_etf() -> None:
    """00631L（五碼+L）應分類為 etf。"""
    instrument, _, _, _ = classify_instrument(symbol_clean="00631L", security_master={})
    assert instrument == "etf"


def test_classification_inverse_etf() -> None:
    """00632R（五碼+R）應分類為 etf。"""
    instrument, _, _, _ = classify_instrument(symbol_clean="00632R", security_master={})
    assert instrument == "etf"


def test_classification_foreign_etf() -> None:
    """00643K（五碼+K）應分類為 etf。"""
    instrument, _, _, _ = classify_instrument(symbol_clean="00643K", security_master={})
    assert instrument == "etf"


def test_classification_active_etf() -> None:
    """00951A（五碼+A）應分類為 etf。"""
    instrument, _, _, _ = classify_instrument(symbol_clean="00951A", security_master={})
    assert instrument == "etf"


def test_classification_convertible_bond() -> None:
    """23301（五碼純數字）應分類為 convertible_bond。"""
    instrument, _, _, _ = classify_instrument(symbol_clean="23301", security_master={})
    assert instrument == "convertible_bond"


def test_classification_6digit_etf_before_warrant() -> None:
    """六碼純數字不應被權證規則捕獲。"""
    instrument, _, _, _ = classify_instrument(symbol_clean="006208", security_master={})
    assert instrument == "etf"


def test_classification_reclassify_database(tmp_path, stock_master_path) -> None:
    import sqlite3
    from market_crawler.classification import reclassify_database

    conn = sqlite3.connect(":memory:")
    conn.execute("CREATE TABLE market_quote_core (symbol_clean TEXT, instrument_type TEXT)")
    conn.execute("INSERT INTO market_quote_core VALUES ('006208', 'other')")
    conn.execute("INSERT INTO market_quote_core VALUES ('2330', 'other')")
    conn.execute("INSERT INTO market_quote_core VALUES ('00631L', 'other')")
    conn.execute("INSERT INTO market_quote_core VALUES ('23301', 'other')")
    conn.commit()

    updated = reclassify_database(conn, str(stock_master_path))
    assert updated == 4

    cur = conn.execute("SELECT instrument_type FROM market_quote_core WHERE symbol_clean = '006208'")
    assert cur.fetchone()[0] == "etf"
    cur = conn.execute("SELECT instrument_type FROM market_quote_core WHERE symbol_clean = '2330'")
    assert cur.fetchone()[0] == "stock"
    cur = conn.execute("SELECT instrument_type FROM market_quote_core WHERE symbol_clean = '00631L'")
    assert cur.fetchone()[0] == "etf"
    cur = conn.execute("SELECT instrument_type FROM market_quote_core WHERE symbol_clean = '23301'")
    assert cur.fetchone()[0] == "convertible_bond"
    conn.close()
