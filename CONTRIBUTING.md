# 貢獻指南

感謝您對本專案的興趣！本指南將幫助您在 dev container 開發環境中進行貢獻。

## 開發環境設置

### 使用 Dev Container (推薦)

1. 安裝 VS Code 和 Docker Desktop
2. 安裝 "Dev Containers" 擴充套件
3. 開啟此專案資料夾
4. 執行命令: `Dev Containers: Reopen in Container`
5. 等待環境設置完成

### 本機開發 (替代方案)

```bash
# 安裝 uv
pip install uv

# 同步依賴
uv sync --all-extras

# 驗證安裝
ruff --version
pytest --version
```

## 開發工作流程

### 1. 建立分支

```bash
git checkout -b feature/功能名稱
# 或
git checkout -b fix/bug名稱
```

### 2. 進行變更

在編輯程式碼時:
- 儲存時會自動使用 ruff 格式化
- VS Code 會顯示任何 linting 問題

### 3. 執行測試

```bash
# 執行所有測試
make test

# 執行特定測試檔案
uv run pytest tests/test_file.py

# 執行測試並查看覆蓋率
make test-cov
```

### 4. 程式碼審查

```bash
# 檢查程式碼風格
make check

# 自動修復風格問題
make fix

# 執行 pre-commit 檢查
make pre-commit
```

### 5. 提交變更

```bash
# Pre-commit 會自動執行檢查
git add .
git commit -m "說明性的提交訊息"
```

提交訊息應遵循以下格式:
- `feat: 新功能說明`
- `fix: bug 修復說明`
- `docs: 文件更新`
- `test: 測試相關變更`
- `refactor: 程式碼重構`

## 代碼風格

本專案使用 **ruff** 進行程式碼格式化:

### 配置 (pyproject.toml)
- **行長度**: 88 字元 (PEP 8)
- **Python 版本**: 3.10+
- **Lint 規則**: E, W, F, I, C, B

### 常見風格規則

```python
# ❌ 不推薦
import os,sys
from module import a,b,c
x = 1 ;

# ✅ 推薦
import os
import sys
from module import a, b, c
x = 1
```

## 使用 uv 管理依賴

### 新增依賴

```bash
# 執行時依賴
uv add 套件名稱

# 開發用依賴
uv add --dev 套件名稱
```

### 查看依賴

```bash
# 顯示已安裝套件
uv pip list

# 顯示依賴樹
uv tree
```

## Docker-in-Docker 開發

Dev Container 內建 Docker-in-Docker 支援，可以直接構建和執行 Docker 映像。

### 常見 Docker 操作

```bash
# 檢查 Docker 安裝
docker --version

# 構建映像
docker build -t myapp:latest .

# 執行容器
docker run -d -p 8000:8000 myapp:latest

# 查看執行中的容器
docker ps

# 檢查日誌
docker logs <container_id>

# 停止容器
docker stop <container_id>

# 使用 Docker Compose
docker-compose up -d
docker-compose down
```

### Docker 在 CI/CD 中的應用

- 本地測試容器化應用
- 構建和推送映像到 Registry
- 使用 Docker Compose 管理相依服務
- 測試多容器應用部署

## 常用命令速查

| 命令               | 說明                     |
| ------------------ | ------------------------ |
| `make help`        | 顯示所有可用命令         |
| `make install`     | 安裝依賴                 |
| `make dev-install` | 安裝所有依賴(含開發)     |
| `make format`      | 格式化程式碼             |
| `make check`       | 檢查程式碼風格           |
| `make fix`         | 自動修復程式碼           |
| `make test`        | 執行測試                 |
| `make test-cov`    | 執行測試並產生覆蓋率報告 |
| `make clean`       | 清理快取檔案             |
| `make jupyter`     | 啟動 Jupyter Lab         |

## Jupyter Notebook 開發

對於互動式開發:

```bash
# 啟動 Jupyter Lab
make jupyter

# 或
jupyter lab --ip=0.0.0.0 --allow-root
```

然後在 VS Code 中存取 `http://localhost:8888`

### Notebook 格式化

VS Code 會自動格式化 Notebook 中的程式碼 (在儲存時使用 ruff)。

## 疑難排解

### 問題: ruff 無法找到
```bash
# 檢查 ruff 是否安裝
which ruff
uv pip list | grep ruff

# 重裝依賴
uv sync --all-extras --force
```

### 問題: pytest 無法執行
```bash
# 驗證 pytest 安裝
uv run pytest --version

# 檢查測試檔案結構
# tests/ 目錄下應有 test_*.py 的檔案
```

### 問題: 容器無法啟動
- 確保 Docker 正在執行
- 執行 "Dev Containers: Rebuild Container"
- 檢查系統資源 (磁碟空間、記憶體)

## 提交 Pull Request

1. 推送您的分支到遠端
2. 開啟 Pull Request
3. 填寫 PR 描述:
   - 說明變更內容
   - 列舉相關的 issue
   - 描述測試方法

4. 等待程式碼審查
5. 根據反饋進行調整並推送新提交

## 資源連結

- [uv GitHub](https://github.com/astral-sh/uv)
- [ruff 文檔](https://docs.astral.sh/ruff/)
- [pytest 文檔](https://docs.pytest.org/)
- [Python PEP 8](https://pep8.org/)

---

感謝您的貢獻！如有任何問題，請開啟 issue 或聯繫維護者。
