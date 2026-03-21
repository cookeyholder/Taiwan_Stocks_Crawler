#!/bin/bash
set -e

# 日誌函式
log_info() {
    echo "ℹ️  [$(date '+%H:%M:%S')] $1"
}

log_success() {
    echo "✅ [$(date '+%H:%M:%S')] $1"
}

log_warn() {
    echo "⚠️  [$(date '+%H:%M:%S')] $1"
}

log_error() {
    echo "❌ [$(date '+%H:%M:%S')] $1" >&2
}

FAST_SETUP="${DEVCONTAINER_FAST_SETUP:-1}"
SMOKE_CHECK="${DEVCONTAINER_SMOKE_CHECK:-0}"

if [ "$FAST_SETUP" = "1" ]; then
    DEFAULT_INSTALL_OPENSPEC="0"
    DEFAULT_INSTALL_PRECOMMIT="0"
else
    DEFAULT_INSTALL_OPENSPEC="1"
    DEFAULT_INSTALL_PRECOMMIT="1"
fi

INSTALL_OPENSPEC="${DEVCONTAINER_INSTALL_OPENSPEC:-$DEFAULT_INSTALL_OPENSPEC}"
INSTALL_PRECOMMIT="${DEVCONTAINER_INSTALL_PRECOMMIT:-$DEFAULT_INSTALL_PRECOMMIT}"

log_info "Setup mode: FAST_SETUP=$FAST_SETUP, SMOKE_CHECK=$SMOKE_CHECK"
log_info "Optional installs: OPENSpec=$INSTALL_OPENSPEC, PRECOMMIT=$INSTALL_PRECOMMIT"

run_smoke_check() {
    local name="$1"
    shift

    if "$@" > /dev/null 2>&1; then
        log_success "Smoke check passed: $name"
        return 0
    fi

    log_error "Smoke check failed: $name"
    return 1
}

# 檢查 Python 版本
log_info "Checking Python version..."
python_version=$(python3 --version 2>&1 | awk '{print $2}')
log_success "Python version: $python_version"

# 安裝系統工具（含 Node.js 24 LTS / npm / ripgrep 等）
log_info "Checking required system tools (Node.js 24 LTS, npm, rg, jq, yq, fzf, shellcheck, ...)..."
ROOT_CMD=""
APT_GET="apt-get"
if command -v sudo > /dev/null 2>&1; then
    ROOT_CMD="sudo"
    APT_GET="sudo apt-get"
fi

need_system_packages=0

node_major=$(node --version 2>/dev/null | sed -E 's/^v([0-9]+).*/\1/' || true)
if [ "$node_major" != "24" ]; then
    need_system_packages=1
fi

for cmd in npm rg jq fdfind yq fzf shellcheck tree sqlite3; do
    if ! command -v "$cmd" > /dev/null 2>&1; then
        need_system_packages=1
        break
    fi
done

if [ "$need_system_packages" -eq 1 ]; then
    log_info "Missing tools detected. Installing system packages..."

    if ! $APT_GET -o DPkg::Lock::Timeout=600 update; then
        log_error "Failed to update apt index"
        exit 1
    fi

    # 設定 Node.js 24 LTS 官方來源（NodeSource）
    log_info "Configuring Node.js 24 LTS repository..."
    if ! $APT_GET -o DPkg::Lock::Timeout=600 install -y ca-certificates curl gnupg; then
        log_error "Failed to install Node.js repository prerequisites"
        exit 1
    fi

    $ROOT_CMD mkdir -p /etc/apt/keyrings
    if ! curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | $ROOT_CMD gpg --dearmor --batch --yes -o /etc/apt/keyrings/nodesource.gpg; then
        log_error "Failed to configure NodeSource key"
        exit 1
    fi

    echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_24.x nodistro main" | $ROOT_CMD tee /etc/apt/sources.list.d/nodesource.list > /dev/null

    if ! $APT_GET -o DPkg::Lock::Timeout=600 update; then
        log_error "Failed to refresh apt index after adding Node.js repository"
        exit 1
    fi

    if ! $APT_GET -o DPkg::Lock::Timeout=600 install -y \
        nodejs ripgrep jq fd-find yq fzf bat shellcheck tree sqlite3; then
        log_error "Failed to install required system tools"
        exit 1
    fi
fi

node_major=$(node --version 2>/dev/null | sed -E 's/^v([0-9]+).*/\1/' || true)
if [ "$node_major" != "24" ]; then
    log_error "Expected Node.js 24 LTS, but got: $(node --version 2>/dev/null || echo unavailable)"
    exit 1
fi

if ! npm --version > /dev/null 2>&1; then
    log_error "npm is unavailable after Node.js installation"
    exit 1
fi

log_success "Node.js version: $(node --version), npm version: $(npm --version)"

# Debian 套件名為 batcat，補上 bat 命令相容
if command -v batcat > /dev/null 2>&1 && ! command -v bat > /dev/null 2>&1; then
    if command -v sudo > /dev/null 2>&1; then
        sudo ln -sf "$(command -v batcat)" /usr/local/bin/bat || true
    else
        ln -sf "$(command -v batcat)" /usr/local/bin/bat || true
    fi
fi

log_success "System tools installed"

# 安裝 OpenSpec CLI
if [ "$INSTALL_OPENSPEC" = "1" ]; then
    log_info "Installing OpenSpec CLI..."
    # 以使用者層級 prefix 安裝，避免 non-root 寫入 /usr/lib/node_modules 失敗
    NPM_GLOBAL_PREFIX="$HOME/.local"
    mkdir -p "$NPM_GLOBAL_PREFIX"
    if ! npm config set prefix "$NPM_GLOBAL_PREFIX"; then
        log_error "Failed to configure npm global prefix"
        exit 1
    fi
    export PATH="$HOME/.local/bin:$PATH"

    if ! command -v openspec > /dev/null 2>&1; then
        if ! npm install -g @fission-ai/openspec; then
            log_error "Failed to install OpenSpec CLI"
            exit 1
        fi
    fi

    if command -v openspec > /dev/null 2>&1; then
        log_success "OpenSpec installed: $(openspec --version)"
    else
        log_error "OpenSpec installation completed but command is unavailable"
        exit 1
    fi
else
    log_info "Skipping OpenSpec CLI install in fast mode"
fi

# 安裝 uv
log_info "Installing uv package manager..."
if ! python3 -m uv --version > /dev/null 2>&1 && ! command -v uv > /dev/null 2>&1; then
    # 以使用者層級安裝，避免 non-root 寫入系統 site-packages 失敗
    if ! python3 -m pip install --user --upgrade pip uv; then
        log_error "Failed to install uv package manager"
        exit 1
    fi
fi

export PATH="$HOME/.local/bin:$PATH"

# 優先使用 module 方式呼叫 uv，避免 PATH 尚未更新
if python3 -m uv --version > /dev/null 2>&1; then
    UV_CMD="python3 -m uv"
elif command -v uv > /dev/null 2>&1; then
    UV_CMD="uv"
else
    log_error "uv installation completed but command is unavailable"
    exit 1
fi

log_success "uv installed successfully"

run_uv() {
    # 避免 shell 內殘留的 VIRTUAL_ENV 干擾 uv 對專案 .venv 的判斷
    env -u VIRTUAL_ENV $UV_CMD "$@"
}

# 確保快取掛載目錄可寫，避免 volume 預設 root 擁有造成 uv 初始化失敗
ensure_cache_writable() {
    local cache_dir="$1"

    mkdir -p "$cache_dir"

    if [ -w "$cache_dir" ]; then
        return 0
    fi

    log_warn "Cache directory is not writable: $cache_dir; attempting to fix ownership"

    if [ "$(id -u)" -eq 0 ]; then
        chown -R "$(id -un):$(id -gn)" "$cache_dir" || true
    elif command -v sudo > /dev/null 2>&1; then
        sudo chown -R "$(id -un):$(id -gn)" "$cache_dir" || true
    fi

    if [ -w "$cache_dir" ]; then
        return 0
    fi

    log_error "Cache directory remains non-writable after fix attempt: $cache_dir"
    return 1
}

if ! ensure_cache_writable "$HOME/.cache"; then
    exit 1
fi

if ! ensure_cache_writable "$HOME/.cache/uv"; then
    exit 1
fi

# 安裝項目依賴
log_info "Installing project dependencies with uv..."
sync_required=1

if [ -f "uv.lock" ] && [ -d ".venv" ]; then
    current_lock_hash="$(sha256sum uv.lock | awk '{print $1}')"
    sync_stamp_file=".venv/.last_uv_sync.lock.sha256"

    if [ "$FAST_SETUP" = "1" ] && [ -f "$sync_stamp_file" ] && [ "$(cat "$sync_stamp_file")" = "$current_lock_hash" ]; then
        sync_required=0
        log_info "Skipping uv sync (fast mode and lock hash unchanged)"
    fi
fi

if [ "$sync_required" -eq 1 ]; then
    if ! run_uv sync --all-extras; then
        log_error "Failed to install project dependencies"
        exit 1
    fi

    if [ -f "uv.lock" ] && [ -d ".venv" ]; then
        sha256sum uv.lock | awk '{print $1}' > .venv/.last_uv_sync.lock.sha256 || true
    fi
fi
log_success "Project dependencies installed"

# 驗證測試執行所需套件可在專案環境匯入
log_info "Verifying test dependencies (pytest, requests, urllib3)..."
if ! run_uv run python -c "import pytest, requests, urllib3"; then
    log_error "Test dependencies verification failed"
    exit 1
fi
log_success "Test dependencies are ready"

# 設定 Docker-in-Docker
log_info "Setting up Docker-in-Docker..."
if [ -S /var/run/docker.sock ]; then
    log_success "Docker socket is connected"
else
    log_warn "Docker socket not found (optional)"
fi

# 將目前使用者加入 docker group（若權限允許）
if getent group docker > /dev/null 2>&1; then
    target_user="${USER:-$(id -un)}"

    if id -nG "$target_user" 2>/dev/null | tr ' ' '\n' | grep -qx "docker"; then
        log_success "User $target_user is already in docker group"
    elif [ "$(id -u)" -eq 0 ]; then
        if usermod -aG docker "$target_user"; then
            log_success "User $target_user added to docker group"
        else
            log_warn "Could not add $target_user to docker group"
        fi
    elif command -v sudo > /dev/null 2>&1 && sudo -n true > /dev/null 2>&1; then
        if sudo usermod -aG docker "$target_user"; then
            log_success "User $target_user added to docker group"
        else
            log_warn "Could not add $target_user to docker group"
        fi
    else
        log_warn "Skipped adding $target_user to docker group (insufficient privileges in postCreate context)"
    fi
fi

if [ "$INSTALL_PRECOMMIT" = "1" ]; then
    # 安裝 pre-commit hooks
    log_info "Installing pre-commit hooks..."
    if python3 -m pip install --user pre-commit > /dev/null 2>&1; then
        if pre-commit install --install-hooks > /dev/null 2>&1 || python3 -m pre_commit install --install-hooks > /dev/null 2>&1; then
            log_success "pre-commit installed"
        else
            log_error "Failed to install pre-commit git hooks"
            exit 1
        fi
    else
        log_error "Failed to install pre-commit"
        exit 1
    fi
else
    log_info "Skipping pre-commit install in fast mode"
fi

# 首次啟動 smoke check
if [ "$SMOKE_CHECK" = "1" ]; then
    log_info "Running first startup smoke checks (pre-commit, pytest, ruff)..."
    smoke_failed=0

    if ! run_smoke_check "pre-commit" bash -lc "pre-commit --version || python3 -m pre_commit --version"; then
        smoke_failed=1
    fi

    if ! run_smoke_check "pytest" bash -lc "env -u VIRTUAL_ENV $UV_CMD run python -m pytest --version"; then
        smoke_failed=1
    fi

    if ! run_smoke_check "ruff" bash -lc "env -u VIRTUAL_ENV $UV_CMD run ruff --version"; then
        smoke_failed=1
    fi

    if [ "$smoke_failed" -ne 0 ]; then
        log_error "One or more smoke checks failed"
        exit 1
    fi
else
    log_info "Skipping smoke checks (set DEVCONTAINER_SMOKE_CHECK=1 to enable)"
fi

# 列出可用命令
log_success "Dev container setup complete!"
echo ""
echo "🚀 Quick start commands:"
echo "   uv run pytest                    - Run tests"
echo "   openspec list --json             - List OpenSpec changes"
echo "   ruff check .                     - Check code style"
echo "   ruff check . --fix               - Auto-fix code style"
echo "   ruff format .                    - Format code"
echo "   uv run jupyter lab --ip=0.0.0.0  - Start Jupyter Lab"
echo ""
echo "🐳 Docker commands:"
echo "   docker --version                 - Check Docker version"
echo "   docker ps                        - List running containers"
echo ""
echo "🧰 Utility commands:"
echo "   rg <pattern>                     - Fast code search"
echo "   jq . file.json                   - Pretty-print JSON"
echo "   yq . file.yaml                   - Query YAML"
echo "   tree -L 2                        - Show project structure"
echo ""
echo "📚 Useful links:"
echo "   - uv docs: https://docs.astral.sh/uv/"
echo "   - ruff docs: https://docs.astral.sh/ruff/"
echo "   - OpenSpec: https://github.com/fission-ai/openspec"
echo ""
