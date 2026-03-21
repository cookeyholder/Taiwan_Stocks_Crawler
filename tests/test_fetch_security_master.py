"""tests/test_fetch_security_master.py — 有價證券主檔抓取模組的完整測試套件。

涵蓋：
- _decode_response：CP950/UTF-8/錯誤位元組 等編碼處理
- _parse_isin_page：正常解析、段落標頭過濾、短列過濾、非英數代號過濾、空表格
- fetch_and_save：成功寫入、欄位完整性、去重、部分市場失敗、資料列數不足防呆、
                  網路錯誤、原子寫入、FetchResult.success 屬性
"""

from __future__ import annotations

import csv
import os
from types import SimpleNamespace

import pytest

import market_crawler.fetch_security_master as fsm
from market_crawler.fetch_security_master import (
    FetchResult,
    _decode_response,
    _parse_isin_page,
    fetch_and_save,
)
from market_crawler.http_client import HttpPolicy

# ---------------------------------------------------------------------------
# 輔助函式
# ---------------------------------------------------------------------------

_DEFAULT_POLICY = HttpPolicy(timeout_seconds=0.1, retry_count=1, backoff_seconds=0)


def _make_html(*rows: tuple[str, ...]) -> str:
    """產生包含標頭列（6 欄）加上若干資料列的假 ISIN HTML 輸入。

    rows 中每一個 tuple 都是 (代號 名稱, col1, 上市日, 市場別, 產業別, CFICode)。
    注意：第一欄必須是「代號 名稱」（有空白）或區段標頭（無空白）。
    """
    header = (
        "<tr>"
        "<td>有價證券代號及名稱</td><td>ISIN</td>"
        "<td>上市日</td><td>市場別</td><td>產業別</td><td>CFICode</td>"
        "</tr>"
    )
    data_rows = []
    for row in rows:
        cells = "".join(f"<td>{c}</td>" for c in row)
        data_rows.append(f"<tr>{cells}</tr>")
    return "<table>" + header + "".join(data_rows) + "</table>"


def _make_response(html: str, encoding: str = "utf-8") -> SimpleNamespace:
    return SimpleNamespace(content=html.encode(encoding))


def _make_cp950_response(html: str) -> SimpleNamespace:
    return SimpleNamespace(content=html.encode("cp950"))


def _stock_row(
    symbol: str = "2330",
    name: str = "台積電",
    listing_date: str = "1994/09/05",
    market: str = "上市",
    section: str = "半導體",
    cfi: str = "ESVUFR",
) -> tuple[str, ...]:
    return (f"{symbol} {name}", "TW0002330005", listing_date, market, section, cfi)


# ---------------------------------------------------------------------------
# _decode_response 測試群組
# ---------------------------------------------------------------------------


class TestDecodeResponse:
    def test_decode_cp950_bytes(self):
        """CP950 編碼的繁體中文應正確解碼。"""
        original = "台積電"
        encoded = original.encode("cp950")
        assert _decode_response(encoded) == original

    def test_decode_big5hkscs_bytes(self):
        """Big5HKSCS 編碼（ISIN 頁面常用）應正確解碼。"""
        original = "上市股票"
        encoded = original.encode("big5hkscs")
        assert _decode_response(encoded) == original

    def test_decode_utf8_bytes(self):
        """純 UTF-8 位元組也應正確解碼（fallback 路徑）。"""
        original = "ETF測試資料"
        encoded = original.encode("utf-8")
        assert _decode_response(encoded) == original

    def test_decode_utf8_sig_bytes_does_not_raise(self):
        """帶有 BOM 的 UTF-8-SIG 位元組應能被解碼（可能透過 big5hkscs 或 utf-8-sig），
        重點是函式不應拋出例外，且回傳型別必須為 str。
        注意：ISIN 頁面在實務上使用 Big5/CP950，UTF-8-SIG 為邊界案例。
        """
        encoded = "BOM測試".encode("utf-8-sig")
        result = _decode_response(encoded)
        assert isinstance(result, str)  # 只保證不拋出，不保證內容正確（encoding priority 問題）

    def test_decode_invalid_bytes_uses_replace(self):
        """完全無法解碼的位元組應使用 replace 模式，不應拋出例外。"""
        invalid = b"\xff\xfe\xfd\xfc"  # 對 Big5/CP950/UTF-8 皆為無效序列
        result = _decode_response(invalid)
        assert isinstance(result, str)  # 不應 raise，應回傳字串


# ---------------------------------------------------------------------------
# _parse_isin_page 測試群組
# ---------------------------------------------------------------------------


class TestParseIsinPage:
    def test_extracts_all_six_columns(self):
        """正常資料列應能解析出「代號、名稱、上市日、市場別、產業別、CFICode」。"""
        html = _make_html(_stock_row())
        records = _parse_isin_page(html)
        assert len(records) == 1
        r = records[0]
        assert r["代號"] == "2330"
        assert r["名稱"] == "台積電"
        assert r["上市日"] == "1994/09/05"
        assert r["市場別"] == "上市"
        assert r["產業別"] == "半導體"
        assert r["CFICode"] == "ESVUFR"

    def test_filters_section_header_rows(self):
        """段落標頭列（第一欄只有單一詞）應被過濾，不進入輸出。"""
        section_header = ("水泥工業", "", "", "", "", "")
        html = _make_html(section_header, _stock_row())
        records = _parse_isin_page(html)
        assert len(records) == 1
        assert records[0]["代號"] == "2330"

    def test_filters_rows_with_too_few_cells(self):
        """欄位數不足 6 的資料列應被略過。"""
        html = (
            "<table><tr><td>有價證券代號及名稱</td><td>ISIN</td><td>上市日</td><td>市場別</td><td>產業別</td><td>CFICode</td></tr>"
            "<tr><td>2330 台積電</td><td>只有兩欄</td></tr>"
            "</table>"
        )
        records = _parse_isin_page(html)
        assert records == []

    def test_filters_non_alnum_starting_codes(self):
        """代號以非英數字元開頭者應被略過（例如 HTML 特殊字元殘留）。"""
        bad_row = ("＊2330 台積電", "", "1994/09/05", "上市", "半導體", "ESVUFR")
        html = _make_html(
            bad_row, _stock_row("0050", "元大台灣50", "2003/06/30", "上市", "ETF", "EMXXXX")
        )
        records = _parse_isin_page(html)
        # 只有 0050 應被保留（代號以 0 開頭，isalnum OK）
        assert len(records) == 1
        assert records[0]["代號"] == "0050"

    def test_empty_html_returns_empty_list(self):
        """完全空白的 HTML 應回傳空清單，不應拋出例外。"""
        assert _parse_isin_page("") == []

    def test_header_only_table_returns_empty_list(self):
        """只有標頭列，無資料列時，應回傳空清單。"""
        html = "<table><tr><td>有價證券代號及名稱</td><td>ISIN</td><td>上市日</td><td>市場別</td><td>產業別</td><td>CFICode</td></tr></table>"
        assert _parse_isin_page(html) == []

    def test_multiple_rows_all_extracted(self):
        """多筆不同類型商品（股票、ETF、債券 ETF）均應進入輸出。"""
        html = _make_html(
            _stock_row("2330", "台積電", cfi="ESVUFR"),
            _stock_row("0050", "元大台灣50", cfi="EMXXXX"),
            _stock_row("00679B", "元大美債20年", cfi="DBFTFR"),
        )
        records = _parse_isin_page(html)
        codes = {r["代號"] for r in records}
        assert codes == {"2330", "0050", "00679B"}

    def test_no_cficode_filter(self):
        """解析函式不應依 CFICode 過濾，所有類型均應保留。"""
        html = _make_html(
            _stock_row("2330", "台積電", cfi="ESVUFR"),
            _stock_row("00631L", "元大台灣50正2", cfi="EMABVR"),  # 槓桿 ETF
            _stock_row("028011", "某公司債", cfi="DBXXXX"),  # 債券
        )
        records = _parse_isin_page(html)
        assert len(records) == 3


# ---------------------------------------------------------------------------
# fetch_and_save 測試群組
# ---------------------------------------------------------------------------


def _minimal_html(n: int = 120) -> str:
    """產生至少 n 筆資料列的 HTML 假頁面（用於測試成功寫入路徑）。"""
    rows = [_stock_row(str(1000 + i), f"股票{i}") for i in range(n)]
    return _make_html(*rows)


class TestFetchAndSave:
    def test_success_writes_csv(self, tmp_path, monkeypatch):
        """正常抓取兩個市場 → CSV 應被寫入檔案。"""
        target = str(tmp_path / "StockID.csv")
        html = _minimal_html(120)

        def fake_get(url, policy):
            return _make_response(html)

        monkeypatch.setattr(fsm, "get_with_retry", fake_get)
        result = fetch_and_save(target, _DEFAULT_POLICY)

        assert result.success
        assert os.path.exists(target)

    def test_success_result_total_rows(self, tmp_path, monkeypatch):
        """成功時 FetchResult.total_rows 應等於實際寫入行數（去重後）。"""
        target = str(tmp_path / "StockID.csv")
        # 兩個市場各提供 120 列不同代號
        html_twse = _make_html(*[_stock_row(str(1000 + i), f"股票{i}") for i in range(120)])
        html_tpex = _make_html(*[_stock_row(str(2000 + i), f"上櫃股{i}") for i in range(120)])
        calls = [html_twse, html_tpex]

        def fake_get(url, policy):
            return _make_response(calls.pop(0))

        monkeypatch.setattr(fsm, "get_with_retry", fake_get)
        result = fetch_and_save(target, _DEFAULT_POLICY)

        assert result.total_rows == 240

    def test_csv_has_correct_columns(self, tmp_path, monkeypatch):
        """輸出 CSV 的第一列應包含六個指定欄位名稱。"""
        target = str(tmp_path / "StockID.csv")

        monkeypatch.setattr(
            fsm, "get_with_retry", lambda url, policy: _make_response(_minimal_html())
        )
        fetch_and_save(target, _DEFAULT_POLICY)

        with open(target, encoding="utf-8", newline="") as f:
            reader = csv.reader(f)
            header = next(reader)
        assert header == list(fsm._OUTPUT_COLS)

    def test_deduplicates_symbols_across_markets(self, tmp_path, monkeypatch):
        """同一代號出現在上市與上櫃頁面時，輸出僅保留一筆。"""
        target = str(tmp_path / "StockID.csv")
        shared_symbol = "9999"
        # 分別準備 120 筆，兩個市場都包含代號 9999
        rows_base = [_stock_row(str(1000 + i), f"股票{i}") for i in range(119)]
        shared = _stock_row(shared_symbol, "共用名稱")
        html = _make_html(shared, *rows_base)

        monkeypatch.setattr(fsm, "get_with_retry", lambda url, policy: _make_response(html))
        result = fetch_and_save(target, _DEFAULT_POLICY)

        with open(target, encoding="utf-8") as f:
            rows = list(csv.DictReader(f))

        symbols = [r["代號"] for r in rows]
        assert symbols.count(shared_symbol) == 1
        assert result.total_rows == len(symbols)

    def test_partial_failure_one_market_still_saves(self, tmp_path, monkeypatch):
        """一個市場網路錯誤時，另一個市場的資料仍應被寫入。"""
        target = str(tmp_path / "StockID.csv")
        good_html = _minimal_html(120)
        call_count = [0]

        def fake_get(url, policy):
            call_count[0] += 1
            if call_count[0] == 1:
                raise RuntimeError("HTTP 503")
            return _make_response(good_html)

        monkeypatch.setattr(fsm, "get_with_retry", fake_get)
        result = fetch_and_save(target, _DEFAULT_POLICY)

        assert result.success
        assert len(result.errors) == 1
        assert "抓取失敗" in result.errors[0]
        assert os.path.exists(target)

    def test_both_markets_fail_no_file_written(self, tmp_path, monkeypatch):
        """兩個市場均失敗時，不應寫入任何檔案。"""
        target = str(tmp_path / "StockID.csv")

        monkeypatch.setattr(
            fsm,
            "get_with_retry",
            lambda url, policy: (_ for _ in ()).throw(RuntimeError("timeout")),
        )
        result = fetch_and_save(target, _DEFAULT_POLICY)

        assert not result.success
        assert not os.path.exists(target)

    def test_too_few_rows_no_file_written(self, tmp_path, monkeypatch):
        """合計資料列數 < 100 時，不應寫入檔案。"""
        target = str(tmp_path / "StockID.csv")
        sparse_html = _make_html(*[_stock_row(str(i)) for i in range(10)])  # 只有 10 筆

        monkeypatch.setattr(fsm, "get_with_retry", lambda url, policy: _make_response(sparse_html))
        result = fetch_and_save(target, _DEFAULT_POLICY)

        assert not result.success
        assert not os.path.exists(target)

    def test_too_few_rows_error_message(self, tmp_path, monkeypatch):
        """列數不足時，errors 清單應包含明確的訊息。"""
        target = str(tmp_path / "StockID.csv")
        sparse_html = _make_html(*[_stock_row(str(i)) for i in range(5)])

        monkeypatch.setattr(fsm, "get_with_retry", lambda url, policy: _make_response(sparse_html))
        result = fetch_and_save(target, _DEFAULT_POLICY)

        assert any("不足" in e for e in result.errors)

    def test_empty_parse_result_recorded_as_error(self, tmp_path, monkeypatch):
        """解析到空結果（空表格）時，errors 應記錄，不應拋出例外。"""
        target = str(tmp_path / "StockID.csv")
        empty_html = "<html><body>無資料</body></html>"

        monkeypatch.setattr(fsm, "get_with_retry", lambda url, policy: _make_response(empty_html))
        result = fetch_and_save(target, _DEFAULT_POLICY)

        assert not result.success
        assert len(result.errors) >= 1  # 至少有解析失敗或列數不足的錯誤

    def test_atomic_write_no_partial_file_on_exception(self, tmp_path, monkeypatch):
        """寫入過程中拋出例外時，目標 CSV 不應存在（原子性保證）。"""
        target = str(tmp_path / "StockID.csv")
        good_html = _minimal_html(120)

        monkeypatch.setattr(fsm, "get_with_retry", lambda url, policy: _make_response(good_html))

        # 讓 os.replace 失敗，模擬寫入最後一步失敗
        def broken_replace(src, dst):
            raise OSError("磁碟空間不足")

        monkeypatch.setattr(os, "replace", broken_replace)

        with pytest.raises(OSError):
            fetch_and_save(target, _DEFAULT_POLICY)

        assert not os.path.exists(target)

    def test_market_counts_populated_on_success(self, tmp_path, monkeypatch):
        """成功時 FetchResult.market_counts 應以市場名稱為鍵、列數為值。"""
        target = str(tmp_path / "StockID.csv")
        html_twse = _make_html(*[_stock_row(str(1000 + i), f"股票{i}") for i in range(80)])
        html_tpex = _make_html(*[_stock_row(str(2000 + i), f"上櫃{i}") for i in range(60)])
        calls = [html_twse, html_tpex]

        monkeypatch.setattr(fsm, "get_with_retry", lambda url, policy: _make_response(calls.pop(0)))
        result = fetch_and_save(target, _DEFAULT_POLICY)

        assert result.success
        assert result.market_counts.get("上市") == 80
        assert result.market_counts.get("上櫃") == 60

    def test_output_includes_etf_and_bond_types(self, tmp_path, monkeypatch):
        """輸出 CSV 應包含 ETF（EMXXXX）與債券 ETF（DBXXXX）等不同 CFICode。"""
        target = str(tmp_path / "StockID.csv")
        rows = [
            _stock_row("2330", "台積電", cfi="ESVUFR"),
            _stock_row("0050", "元大台灣50", cfi="EMXXXX"),
            _stock_row("00679B", "元大美債20年", cfi="DBFTFR"),
        ] + [_stock_row(str(3000 + i), f"股票{i}") for i in range(120)]
        html = _make_html(*rows)

        monkeypatch.setattr(fsm, "get_with_retry", lambda url, policy: _make_response(html))
        fetch_and_save(target, _DEFAULT_POLICY)

        with open(target, encoding="utf-8") as f:
            rows_out = list(csv.DictReader(f))

        cfi_codes = {r["CFICode"] for r in rows_out}
        assert "ESVUFR" in cfi_codes
        assert "EMXXXX" in cfi_codes
        assert "DBFTFR" in cfi_codes


# ---------------------------------------------------------------------------
# FetchResult 屬性測試
# ---------------------------------------------------------------------------


class TestFetchResult:
    def test_success_true_when_total_rows_ge_min(self):
        """total_rows >= _MIN_ROWS 時 success 應為 True。"""
        result = FetchResult(total_rows=fsm._MIN_ROWS)
        assert result.success is True

    def test_success_false_when_total_rows_lt_min(self):
        """total_rows < _MIN_ROWS 時 success 應為 False。"""
        result = FetchResult(total_rows=fsm._MIN_ROWS - 1)
        assert result.success is False

    def test_success_exactly_at_min_threshold(self):
        """total_rows 恰好等於 _MIN_ROWS 時 success 應為 True（邊界值）。"""
        result = FetchResult(total_rows=100)
        assert result.success is True

    def test_errors_default_empty_list(self):
        """未傳入 errors 時，FetchResult.errors 應為空清單。"""
        result = FetchResult(total_rows=0)
        assert result.errors == []

    def test_market_counts_default_empty_dict(self):
        """未傳入 market_counts 時，應為空字典。"""
        result = FetchResult(total_rows=0)
        assert result.market_counts == {}


# ---------------------------------------------------------------------------
# 防呆測試（Guardrail Tests）
# ---------------------------------------------------------------------------


class TestGuardrails:
    def test_does_not_overwrite_existing_file_on_too_few_rows(self, tmp_path, monkeypatch):
        """列數不足時，既有的 StockID.csv 不應被覆蓋。"""
        target = str(tmp_path / "StockID.csv")
        original_content = (
            "代號,名稱,上市日,市場別,產業別,CFICode\n2330,台積電,1994/09/05,上市,半導體,ESVUFR\n"
        )
        with open(target, "w", encoding="utf-8") as f:
            f.write(original_content)

        sparse_html = _make_html(*[_stock_row(str(i)) for i in range(3)])
        monkeypatch.setattr(fsm, "get_with_retry", lambda url, policy: _make_response(sparse_html))
        fetch_and_save(target, _DEFAULT_POLICY)

        with open(target, encoding="utf-8") as f:
            content = f.read()
        assert content == original_content

    def test_no_leftover_tmp_file_on_success(self, tmp_path, monkeypatch):
        """成功寫入後，暫存檔不應殘留在目錄中。"""
        target = str(tmp_path / "StockID.csv")
        monkeypatch.setattr(
            fsm, "get_with_retry", lambda url, policy: _make_response(_minimal_html())
        )
        fetch_and_save(target, _DEFAULT_POLICY)

        tmp_files = [f for f in os.listdir(tmp_path) if f.endswith(".tmp")]
        assert tmp_files == []

    def test_no_leftover_tmp_file_on_ioerror(self, tmp_path, monkeypatch):
        """OS 錯誤導致寫入失敗時，暫存檔不應殘留在目錄中。"""
        target = str(tmp_path / "StockID.csv")
        monkeypatch.setattr(
            fsm, "get_with_retry", lambda url, policy: _make_response(_minimal_html())
        )

        def broken_replace(src, dst):
            raise OSError("磁碟空間不足")

        monkeypatch.setattr(os, "replace", broken_replace)

        with pytest.raises(OSError):
            fetch_and_save(target, _DEFAULT_POLICY)

        tmp_files = [f for f in os.listdir(tmp_path) if f.endswith(".tmp")]
        assert tmp_files == []

    def test_runtime_error_from_get_with_retry_caught(self, tmp_path, monkeypatch):
        """get_with_retry 拋出 RuntimeError 時，不應 propagate，應記錄錯誤。"""
        target = str(tmp_path / "StockID.csv")

        def always_fail(url, policy):
            raise RuntimeError("three retries failed")

        monkeypatch.setattr(fsm, "get_with_retry", always_fail)
        result = fetch_and_save(target, _DEFAULT_POLICY)
        assert not result.success
        assert len(result.errors) >= 2  # 兩個市場均失敗

    def test_no_duplicate_symbols_in_output(self, tmp_path, monkeypatch):
        """輸出 CSV 中任何代號都不應重複出現。"""
        target = str(tmp_path / "StockID.csv")
        # 兩個市場中 500～619 代號重疊（各 120 筆，共 119 個唯一代號 + 1 個重疊）
        rows_a = [_stock_row(str(500 + i), f"股{i}") for i in range(120)]
        rows_b = [_stock_row(str(500 + i), f"股{i}_副") for i in range(120)]
        html_a = _make_html(*rows_a)
        html_b = _make_html(*rows_b)
        calls = [html_a, html_b]

        monkeypatch.setattr(fsm, "get_with_retry", lambda url, policy: _make_response(calls.pop(0)))
        fetch_and_save(target, _DEFAULT_POLICY)

        with open(target, encoding="utf-8") as f:
            rows_out = list(csv.DictReader(f))
        symbols = [r["代號"] for r in rows_out]
        assert len(symbols) == len(set(symbols))

    def test_cp950_encoded_content_parsed_correctly(self, tmp_path, monkeypatch):
        """CP950 編碼的頁面回應應能正確解析，不應出現亂碼代號。"""
        target = str(tmp_path / "StockID.csv")
        html = _make_html(*[_stock_row(str(1000 + i), f"股票{i}") for i in range(120)])

        def fake_get(url, policy):
            return _make_cp950_response(html)

        monkeypatch.setattr(fsm, "get_with_retry", fake_get)
        result = fetch_and_save(target, _DEFAULT_POLICY)
        assert result.success

        with open(target, encoding="utf-8") as f:
            rows_out = list(csv.DictReader(f))
        # 代號應全為純 ASCII 英數字
        for r in rows_out:
            assert r["代號"].isalnum(), f"代號含非英數字元：{r['代號']!r}"

    def test_default_policy_used_when_none_passed(self, tmp_path, monkeypatch):
        """policy=None 時，函式應使用預設策略而不崩潰。"""
        target = str(tmp_path / "StockID.csv")
        monkeypatch.setattr(
            fsm, "get_with_retry", lambda url, policy: _make_response(_minimal_html())
        )
        result = fetch_and_save(target, policy=None)
        assert result.success

    def test_second_market_skipped_when_first_returns_zero_rows(self, tmp_path, monkeypatch):
        """第一個市場解析出 0 筆時，錯誤訊息應明確，且不影響第二市場處理。"""
        target = str(tmp_path / "StockID.csv")
        empty_html = "<html><body></body></html>"
        good_html = _minimal_html(120)
        calls = [empty_html, good_html]

        monkeypatch.setattr(fsm, "get_with_retry", lambda url, policy: _make_response(calls.pop(0)))
        result = fetch_and_save(target, _DEFAULT_POLICY)

        assert result.success
        assert any("解析不到" in e for e in result.errors)
