# 臺灣股市資料爬蟲與分析系統 - Windows 平臺發行建置腳本
# 嚴格遵循臺灣繁體中文（zh-TW）規範

[CmdletBinding()]
param (
    [switch]$SkipCheck = $false
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir

Set-Location $ProjectRoot

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " 開始建置 stock-gpui Windows 發行版本 (Release)" -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. 檢查 Rust 與 Cargo 環境
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error "錯誤：找不到 cargo 指令，請先安裝 Rust 工具鏈 (https://rustup.rs)。"
    exit 1
}

# 2. 說明 Direct3D 加速渲染與字型排版
Write-Host "--> Windows 圖形環境說明：" -ForegroundColor Green
Write-Host "    - GPUI 於 Windows 底層透過 WGPU/Direct3D 12 進行 GPU 硬體加速渲染。"
Write-Host "    - 系統字型支援：自動回退至微軟正黑體 (Microsoft JhengHei) 與 Segoe UI。"

# 3. 執行發行模式編譯
Write-Host "--> 正在編譯 stock-gpui (Profile: Release)..." -ForegroundColor Green
cargo build --release -p stock-gpui

$BinaryPath = Join-Path $ProjectRoot "target\release\stock-gpui.exe"

if (-not (Test-Path $BinaryPath)) {
    Write-Error "錯誤：建置完成但未找到產出檔案：$BinaryPath"
    exit 1
}

# 4. 輸出打包成果至 dist 目錄
$DistDir = Join-Path $ProjectRoot "dist"
if (-not (Test-Path $DistDir)) {
    New-Item -ItemType Directory -Path $DistDir | Out-Null
}

$DestPath = Join-Path $DistDir "stock-gpui-windows.exe"
Copy-Item -Path $BinaryPath -Destination $DestPath -Force

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " 建置成功！" -ForegroundColor Cyan
Write-Host " 產出執行檔路徑：$DestPath" -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan
