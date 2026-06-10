# 專案文件更新

## 背景

專案已從 Python Tkinter 遷移至 Tauri v2 + Rust + Vue 3（PR #8），
但所有文件仍停留在 Python 時代，未反映目前的技術架構。

## 目標

將所有專案文件以 **zh-tw** 更新，涵蓋：
- 技術架構說明（Rust workspace、Tauri 桌面殼層、Vue 前端）
- 建置指南（Linux / Windows）
- CLI 使用說明（stock-cli）
- 測試矩陣（Rust 單元測試）
- 貢獻指南（Rust 開發流程）

## 非目標

- 不刪除 Python 歷史文件（歸檔即可）
- 不修改 openspec 規格文件
- 不修改實際程式碼
