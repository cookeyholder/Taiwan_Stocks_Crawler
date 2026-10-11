#!/usr/bin/env bash
# 臺灣股市資料爬蟲與分析系統 - Linux 平臺發行建置腳本
# 嚴格遵循臺灣繁體中文（zh-TW）規範

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${PROJECT_ROOT}"

echo "=========================================================="
echo " 開始建置 stock-gpui Linux 發行版本 (Release)"
echo "=========================================================="

# 1. 檢查 Rust 與 Cargo 環境
if ! command -v cargo &> /dev/null; then
    echo "錯誤：找不到 cargo 指令，請先安裝 Rust 工具鏈 (https://rustup.rs)。" >&2
    exit 1
fi

# 2. 檢查並處理 Linux 原生相依函式庫鏈結
# GPUI 需要 libxkbcommon-x11。若系統缺少 .so 符號連結，確保 .cargo_lib 提供相容性支援。
LOCAL_LIB_DIR="${PROJECT_ROOT}/.cargo_lib"
if [ ! -f "${LOCAL_LIB_DIR}/libxkbcommon-x11.so" ]; then
    mkdir -p "${LOCAL_LIB_DIR}"
    if [ -f "/usr/lib/x86_64-linux-gnu/libxkbcommon-x11.so.0" ]; then
        ln -sf /usr/lib/x86_64-linux-gnu/libxkbcommon-x11.so.0 "${LOCAL_LIB_DIR}/libxkbcommon-x11.so"
    fi
fi

# 3. 執行發行模式編譯
echo "--> 正在編譯 stock-gpui (Profile: Release)..."
cargo build --release -p stock-gpui

BINARY_PATH="${PROJECT_ROOT}/target/release/stock-gpui"

if [ ! -f "${BINARY_PATH}" ]; then
    echo "錯誤：建置完成但未找到產出檔案：${BINARY_PATH}" >&2
    exit 1
fi

chmod +x "${BINARY_PATH}"

# 4. 驗證二進位檔案依賴性 (ldd 檢查未解析符號)
echo "--> 正在檢驗 Linux 動態鏈結相依性..."
if command -v ldd &> /dev/null; then
    MISSING_LIBS=$(ldd "${BINARY_PATH}" | grep "not found" || true)
    if [ -n "${MISSING_LIBS}" ]; then
        echo "錯誤：偵測到缺失的共享函式庫：" >&2
        echo "${MISSING_LIBS}" >&2
        echo "請確保已安裝相應的系統套件（如 libxkbcommon-x11-0, libvulkan1 等）。" >&2
        exit 1
    else
        echo "相依性驗證通過：無缺失的共享函式庫。"
    fi
fi

# 5. 輸出打包成果至 dist 目錄
DIST_DIR="${PROJECT_ROOT}/dist"
mkdir -p "${DIST_DIR}"
cp "${BINARY_PATH}" "${DIST_DIR}/stock-gpui-linux"

echo "=========================================================="
echo " 建置成功！"
echo " 產出執行檔路徑：${DIST_DIR}/stock-gpui-linux"
echo "=========================================================="
