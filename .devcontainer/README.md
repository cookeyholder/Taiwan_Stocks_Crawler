# Stock Trading System - Dev Container 指南

這個專案使用 VS Code Dev Container 提供完整的開發環境。

## 環境配置

- **Python**: 3.13
- **作業系統**: Debian Trixie
- **套件管理器**: [uv](https://github.com/astral-sh/uv) - 快速、安全的 Python 套件管理器
- **程式碼格式化**: [ruff](https://github.com/astral-sh/ruff) - 超快速的 Python linter/formatter
- **Docker**: 內建 Docker-in-Docker 支援，可在容器內執行 Docker 命令
- **Docker Compose**: 已安裝最新版本
- **容器映像**: `mcr.microsoft.com/devcontainers/python:3.13-trixie`

### 建構效能模式

目前預設採用「快速模式」以縮短 Dev Container 首次可用時間：

- 不等待整段 `postCreateCommand` 才 attach（可更快進入編輯環境）
- 使用 Docker volume 快取 `uv`/`pip`/`npm` 下載內容
- `post-create-command.sh` 預設跳過耗時的可選安裝與 smoke check

可用環境變數切換行為：

- `DEVCONTAINER_FAST_SETUP=1`（預設）：快速模式
- `DEVCONTAINER_FAST_SETUP=0`：完整初始化模式
- `DEVCONTAINER_SMOKE_CHECK=1`：啟用 `pre-commit/pytest/ruff` smoke check
- `DEVCONTAINER_INSTALL_OPENSPEC=1`：安裝 OpenSpec CLI
- `DEVCONTAINER_INSTALL_PRECOMMIT=1`：安裝 pre-commit 與 hooks

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
   - 自動安裝 uv 和所有依賴套件（快速模式下採可重入策略，降低重複耗時）
   - VS Code 擴充套件會自動安裝

4. **需要完整初始化時**（例如你要做 CI 前本機驗證）:

```bash
DEVCONTAINER_FAST_SETUP=0 \
DEVCONTAINER_SMOKE_CHECK=1 \
DEVCONTAINER_INSTALL_OPENSPEC=1 \
DEVCONTAINER_INSTALL_PRECOMMIT=1
```

設定後可執行「Dev Containers: Rebuild Container」。

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

### Docker-in-Docker (🐳)

Dev Container 已內建 Docker-in-Docker 支援，可以直接在容器內執行 Docker 命令。

```bash
# 檢查 Docker 版本
docker --version

# 列出執行中的容器
docker ps

# 構建 Docker 映像
docker build -t myapp:latest .

# 執行容器
docker run -d myapp:latest

# 查看 Docker 磁碟空間
docker system df

# 清理未使用的資源
docker system prune
```

**注意**:
- Docker socket 通過掛載自動連接
- `vscode` 使用者已新增到 `docker` group
- 無需使用 `sudo docker` 命令

### 建構過久時的優先檢查

若你感覺建構時間過長，建議先檢查：

1. 是否誤開完整模式（`DEVCONTAINER_FAST_SETUP=0`）
2. 是否啟用 smoke check（`DEVCONTAINER_SMOKE_CHECK=1`）
3. Docker volume 快取是否存在（`tw-stock-uv-cache`、`tw-stock-pip-cache`、`tw-stock-npm-cache`）
4. 是否需要 Docker-in-Docker（若暫時不需要可先移除對應 feature）

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

### Docker 相關問題

#### Docker 命令無法執行
```bash
# 檢查 Docker daemon 是否運行
docker ps

# 檢查 docker socket 權限
ls -la /var/run/docker.sock

# 重建容器
# Dev Containers: Rebuild Container (F1)
```

#### "permission denied while trying to connect to Docker daemon"
- 確保 `vscode` 使用者在 `docker` group 中
- 重新開啟終端或 Dev Container
- 如需要，執行: `newgrp docker`

#### Docker 映像無法構建
- 檢查磁碟空間: `docker system df`
- 清理未使用的資源: `docker system prune -a`
- 確保有足夠的系統資源

## 更新機制

使用 uv 的優勢:
- **詳鎖定版本** (`uv.lock`): 確保再現性環境
- **平行安裝**: 比 pip 快很多倍
- **安全性**: 默認檢查依賴安全性

## 資源連結

- [uv 文檔](https://github.com/astral-sh/uv)
- [ruff 文檔](https://docs.astral.sh/ruff/)
- [Docker 官方文檔](https://docs.docker.com/)
- [Docker-in-Docker 指南](https://github.com/devcontainers/features/tree/main/src/docker-in-docker)
- [VS Code Dev Containers](https://code.visualstudio.com/docs/devcontainers/containers)
- [Python 3.10 文檔](https://docs.python.org/3.10/)

---

有任何問題，請查看日誌或執行 `docker logs <container_id>`。
