from __future__ import annotations

import json
import sqlite3
from abc import ABC, abstractmethod
from typing import Any, Mapping


class MarketRepository(ABC):
    @abstractmethod
    def init_schema(self) -> None:
        raise NotImplementedError  # pragma: no cover

    @abstractmethod
    def upsert_core(self, quote: dict[str, Any]) -> None:
        raise NotImplementedError  # pragma: no cover

    @abstractmethod
    def upsert_extension(self, quote: dict[str, Any]) -> None:
        raise NotImplementedError  # pragma: no cover

    @abstractmethod
    def fetch_by_trade_date(self, trade_date: str) -> list[dict[str, Any]]:
        raise NotImplementedError  # pragma: no cover


class SQLiteMarketRepository(MarketRepository):
    def __init__(self, connection: sqlite3.Connection) -> None:
        self.connection = connection

    def init_schema(self) -> None:
        # Enable FK enforcement; must be set per-connection in SQLite.
        self.connection.execute("PRAGMA foreign_keys = ON")
        self.connection.execute(
            """
            CREATE TABLE IF NOT EXISTS market_quote_core (
                trade_date TEXT NOT NULL,
                market_code TEXT NOT NULL,
                symbol_raw TEXT NOT NULL,
                symbol_clean TEXT NOT NULL,
                security_name TEXT,
                instrument_type TEXT NOT NULL,
                instrument_subtype TEXT,
                isin_code TEXT,
                cfi_code TEXT,
                section_label TEXT,
                open_price REAL,
                high_price REAL,
                low_price REAL,
                close_price REAL,
                volume_shares INTEGER,
                turnover_value REAL,
                trade_count INTEGER,
                change_sign TEXT,
                change_value REAL,
                last_bid_price REAL,
                last_bid_qty_raw TEXT,
                last_ask_price REAL,
                last_ask_qty_raw TEXT,
                pe_ratio REAL,
                issued_shares INTEGER,
                next_limit_up REAL,
                next_limit_down REAL,
                source_file TEXT NOT NULL,
                source_encoding TEXT NOT NULL,
                source_line_no INTEGER NOT NULL,
                ingest_run_id TEXT NOT NULL,
                ingested_at TEXT NOT NULL,
                PRIMARY KEY (trade_date, market_code, symbol_clean)
            )
            """
        )
        self.connection.execute(
            """
            CREATE TABLE IF NOT EXISTS market_quote_extension (
                trade_date TEXT NOT NULL,
                market_code TEXT NOT NULL,
                symbol_clean TEXT NOT NULL,
                raw_change TEXT,
                raw_payload TEXT,
                PRIMARY KEY (trade_date, market_code, symbol_clean),
                FOREIGN KEY (trade_date, market_code, symbol_clean)
                REFERENCES market_quote_core(trade_date, market_code, symbol_clean)
            )
            """
        )
        self.connection.commit()

    @staticmethod
    def _normalize_text(value: Any) -> Any:
        if isinstance(value, str):
            return value.encode("utf-8", errors="replace").decode("utf-8")
        return value

    def _normalize_value(self, value: Any) -> Any:
        if isinstance(value, Mapping):
            return {
                self._normalize_text(key): self._normalize_value(inner_value)
                for key, inner_value in value.items()
            }
        if isinstance(value, list):
            return [self._normalize_value(item) for item in value]
        return self._normalize_text(value)

    def _normalize_quote_payload(self, quote: dict[str, Any]) -> dict[str, Any]:
        return {key: self._normalize_value(value) for key, value in quote.items()}

    def upsert_core(self, quote: dict[str, Any]) -> None:
        normalized_quote = self._normalize_quote_payload(quote)
        self.connection.execute(
            """
            INSERT INTO market_quote_core (
                trade_date, market_code, symbol_raw, symbol_clean, security_name,
                instrument_type, instrument_subtype, isin_code, cfi_code, section_label,
                open_price, high_price, low_price, close_price, volume_shares,
                turnover_value, trade_count, change_sign, change_value, last_bid_price,
                last_bid_qty_raw, last_ask_price, last_ask_qty_raw, pe_ratio,
                issued_shares, next_limit_up, next_limit_down, source_file,
                source_encoding, source_line_no, ingest_run_id, ingested_at
            ) VALUES (
                :trade_date, :market_code, :symbol_raw, :symbol_clean, :security_name,
                :instrument_type, :instrument_subtype, :isin_code, :cfi_code, :section_label,
                :open_price, :high_price, :low_price, :close_price, :volume_shares,
                :turnover_value, :trade_count, :change_sign, :change_value, :last_bid_price,
                :last_bid_qty_raw, :last_ask_price, :last_ask_qty_raw, :pe_ratio,
                :issued_shares, :next_limit_up, :next_limit_down, :source_file,
                :source_encoding, :source_line_no, :ingest_run_id, :ingested_at
            )
            ON CONFLICT(trade_date, market_code, symbol_clean) DO UPDATE SET
                symbol_raw=excluded.symbol_raw,
                security_name=excluded.security_name,
                instrument_type=excluded.instrument_type,
                instrument_subtype=excluded.instrument_subtype,
                isin_code=excluded.isin_code,
                cfi_code=excluded.cfi_code,
                section_label=excluded.section_label,
                open_price=excluded.open_price,
                high_price=excluded.high_price,
                low_price=excluded.low_price,
                close_price=excluded.close_price,
                volume_shares=excluded.volume_shares,
                turnover_value=excluded.turnover_value,
                trade_count=excluded.trade_count,
                change_sign=excluded.change_sign,
                change_value=excluded.change_value,
                last_bid_price=excluded.last_bid_price,
                last_bid_qty_raw=excluded.last_bid_qty_raw,
                last_ask_price=excluded.last_ask_price,
                last_ask_qty_raw=excluded.last_ask_qty_raw,
                pe_ratio=excluded.pe_ratio,
                issued_shares=excluded.issued_shares,
                next_limit_up=excluded.next_limit_up,
                next_limit_down=excluded.next_limit_down,
                source_file=excluded.source_file,
                source_encoding=excluded.source_encoding,
                source_line_no=excluded.source_line_no,
                ingest_run_id=excluded.ingest_run_id,
                ingested_at=excluded.ingested_at
            """,
            normalized_quote,
        )

    def upsert_extension(self, quote: dict[str, Any]) -> None:
        normalized_quote = self._normalize_quote_payload(quote)
        self.connection.execute(
            """
            INSERT INTO market_quote_extension (
                trade_date, market_code, symbol_clean, raw_change, raw_payload
            ) VALUES (
                :trade_date, :market_code, :symbol_clean, :raw_change, :raw_payload
            )
            ON CONFLICT(trade_date, market_code, symbol_clean) DO UPDATE SET
                raw_change=excluded.raw_change,
                raw_payload=excluded.raw_payload
            """,
            {
                "trade_date": normalized_quote["trade_date"],
                "market_code": normalized_quote["market_code"],
                "symbol_clean": normalized_quote["symbol_clean"],
                "raw_change": normalized_quote.get("raw_change"),
                "raw_payload": json.dumps(normalized_quote.get("raw_payload"), ensure_ascii=False),
            },
        )

    def fetch_by_trade_date(self, trade_date: str) -> list[dict[str, Any]]:
        cur = self.connection.execute(
            """
            SELECT trade_date, market_code, symbol_clean, instrument_type
            FROM market_quote_core
            WHERE trade_date = ?
            ORDER BY market_code, symbol_clean
            """,
            (trade_date,),
        )
        rows = cur.fetchall()
        return [
            {
                "trade_date": row[0],
                "market_code": row[1],
                "symbol_clean": row[2],
                "instrument_type": row[3],
            }
            for row in rows
        ]
