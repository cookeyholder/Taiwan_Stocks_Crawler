# -*- mode: python ; coding: utf-8 -*-

import os
import sys
import matplotlib


block_cipher = None

datas = []

# Bundle matplotlib's mpl-data (required for font/rc config)
import matplotlib as mpl
mpl_data_dir = os.path.join(os.path.dirname(mpl.__file__), "mpl-data")
datas.append((mpl_data_dir, "matplotlib/mpl-data"))

a = Analysis(
    ["market_crawler/gui.py"],
    pathex=[],
    binaries=[],
    datas=datas,
    hiddenimports=[
        "pandas",
        "pandas_ta",
        "mplfinance",
        "matplotlib",
        "matplotlib.backends.backend_tkagg",
        "matplotlib.backends.backend_agg",
        "requests",
        "numpy",
        "yfinance",
        "sqlite3",
    ],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[
        "tkinter.test",
        "unittest",
        "pytest",
        "test",
    ],
    noarchive=False,
    optimize=0,
)

pyz = PYZ(a.pure)

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.datas,
    [],
    name="stock_screener",
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    upx_exclude=[],
    runtime_tmpdir=None,
    console=False,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
)
