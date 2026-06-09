from __future__ import annotations

import pytest


def test_gui_imports() -> None:
    """驗證 GUI 模組可以成功導入，並檢查關鍵類別與屬性是否定義正確。"""
    try:
        import tkinter
        _ = tkinter
    except ImportError:
        pytest.skip("此環境未安裝 tkinter，跳過 GUI 模組導入測試。")

    try:
        import market_crawler.gui as gui

        # 檢查主要類別與名單是否存在
        assert hasattr(gui, "StockScreenerApp")
        assert hasattr(gui, "TAIWAN_50_SYMBOLS")
        assert len(gui.TAIWAN_50_SYMBOLS) > 0
        assert "2330" in gui.TAIWAN_50_SYMBOLS
    except Exception as e:
        pytest.fail(f"無法成功導入 GUI 模組或其關鍵組件缺失：{e}")
