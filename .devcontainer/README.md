# Stock Trading System - Dev Container 指南

這個專案使用 VS Code Dev Container 提供完整的開發環境。

## 環境配置

- **Python**: 3.10
- **套件管理器**: [uv](https://github.com/astral-sh/uv) - 快速、安全的 Python 套件管理器
- **程式碼格式化**: [ruff](https://github.com/astral-sh/ruff) - 超快速的 Python linter/formatter
- **容器映像**: `mcr.microsoft.com/devcontainers/python:3.10-bullseye`

## 快速開始

### 使用 VS Code Dev Container

1. **需要的工具**:
   - VS Code
   - Docker Desktop 或相容的容器執行時
   - VS Code "Dev Containers" 擴充套件

2. **開啟 Dev Container**:
   - 在 VS Code 中開啟專案資料夾
   - 按 `F1` 並搜尋 "Dev Containers: Reopen in Container"
   - 等待容器建立完成

3. **自動設置**:
   - 容器會自動執行 `post-create-command.sh`
   - 自動安裝 uv 和所有依賴套件
   - VS Code 擴充套件會自動安裝

## 常用命令

### 套件管理 (uv)

```bash
# 同步所有依賴 (包含開發相依性)
uv sync --all-extras

# 執行檔案
uv run python script.py

# 執行 pytest
uv run pytest

# 新增套件
uv add pandas

# 新增開發用套件
uv add --dev pytest-cov
```

### 程式碼格式化 (ruff)

```bash
# 檢查程式碼風格
ruff check .

# 自動修復風格問題
ruff check . --fix

# 格式化程式碼
ruff format .

# 只檢查特定檔案
ruff check path/to/file.py
```

### Jupyter Notebook

```bash
# 啟動 Jupyter Lab (在容器內)
jupyter lab --ip=0.0.0.0 --allow-root

# 然後在 VS Code 中存取 http://localhost:8888
```

### Git 與 Pre-commit

```bash
# 初始化 pre-commit
pre-commit install

# 手動執行 pre-commit 檢查
pre-commit run --all-files
```

## 專案結構

```
.
├── .devcontainer/
│   ├── devcontainer.json      # Dev Container 配置
│   ├── post-create-command.sh # 容器建立後執行的腳本
│   └── Dockerfile             # (可選) 自訂 Docker 映像
├── pyproject.toml             # 專案配置 (uv, ruff, pytest)
├── .gitignore                 # Git 忽略規則
├── .pre-commit-config.yaml    # Pre-commit hooks 配置
└── src/                       # 原始程式碼
```

## pyproject.toml 配置說明

### 主要依賴 (dependencies)
- pandas: 資料處理
- numpy: 數值運算
- requests: HTTP 網路請求

### 開發依賴 (dev)
- ruff: Linter 和 Formatter
- pytest: 測試框架
- pytest-cov: 測試覆蓋率

### Jupyter 依賴 (jupyter)
- jupyter: Jupyter Notebook 核心
- jupyterlab: Jupyter Lab 介面
- ipykernel: IPython 核心

## Ruff 配置

在 `pyproject.toml` 中的 `[tool.ruff]` 部分設定:
- **line-length**: 88 字元 (PEP 8 建議)
- **target-version**: Python 3.10
- **lint rules**: 包含 E, W, F, I, C, B 等檢查

## VS Code 設定

Dev Container 會自動配置:
- Python 格式化器: ruff
- 儲存時自動格式化
- 儲存時自動整理 imports
- Jupyter Notebook 支援

## 疑難排解

### 容器無法啟動
- 確保 Docker Desktop 正在執行
- 嘗試執行 "Dev Containers: Rebuild Container"
- 檢查 `.devcontainer/devcontainer.json` 配置

### uv 無法找到
- 確保 Python 路徑正確: `/usr/local/bin/python`
- 重新建立容器

### Ruff 無法格式化
- 確保 ruff 已安裝: `uv pip list | grep ruff`
- 檢查 VS Code 中的 ruff 擴充套件狀態

## 更新機制

使用 uv 的優勢:
- **詳鎖定版本** (`uv.lock`): 確保再現性環境
- **平行安裝**: 比 pip 快很多倍
- **安全性**: 默認檢查依賴安全性

## 資源連結

- [uv 文檔](https://github.com/astral-sh/uv)
- [ruff 文檔](https://docs.astral.sh/ruff/)
- [VS Code Dev Containers](https://code.visualstudio.com/docs/devcontainers/containers)
- [Python 3.10 文檔](https://docs.python.org/3.10/)

---

有任何問題，請查看日誌或執行 `docker logs <container_id>`。
