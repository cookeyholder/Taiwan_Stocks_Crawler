# README.md 更新規格

## 前置說明（檔案開頭）

SHALL 保留現有「臺股成交資訊收集」標題與子彈說明。

## 技術架構概覽

SHALL 在快速開始之前新增一節「技術架構」，說明：
- Rust workspace（stock-core / stock-cli / stock-tauri）
- Tauri v2 桌面殼層
- Vue 3 + TypeScript 前端（ECharts 圖表）
- SQLite 資料庫
- TWSE / TPEX HTTP 爬蟲（含 Big5 解碼）

## 建置指南

SHALL 新增一節「建置指南」，包含：

### Linux 前置需求
```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev
```

### Windows 前置需求
- Rust (rustup)
- Node.js >= 18
- Visual Studio Build Tools（勾選 C++ 桌面開發）
- WebView2（Win10 以上內建）

### 建置指令
```bash
cd stock-tauri && npm install && npm run tauri build
```

### CLI 建置
```bash
cargo build -p stock-cli --release
```

SHALL 說明輸出路徑：
- Linux: `deb/`, `rpm/`
- Windows: `msi/`, `nsis/`

## 快速開始

SHALL 在既有 Python 安裝步驟前新增 Tauri 開發環境：
```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node 依賴
cd stock-tauri && npm install

# 啟動開發模式
cd stock-tauri && npm run tauri dev
```
