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
log_info "Installing system tools (Node.js 24 LTS, npm, rg, jq, yq, fzf, shellcheck, ...)..."
ROOT_CMD=""
APT_GET="apt-get"
if command -v sudo > /dev/null 2>&1; then
    ROOT_CMD="sudo"
    APT_GET="sudo apt-get"
fi

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

node_major=$(node --version 2>/dev/null | sed -E 's/^v([0-9]+).*/\1/')
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
log_info "Installing OpenSpec CLI..."
# 以使用者層級 prefix 安裝，避免 non-root 寫入 /usr/lib/node_modules 失敗
NPM_GLOBAL_PREFIX="$HOME/.local"
mkdir -p "$NPM_GLOBAL_PREFIX"
if ! npm config set prefix "$NPM_GLOBAL_PREFIX"; then
    log_error "Failed to configure npm global prefix"
    exit 1
fi
export PATH="$HOME/.local/bin:$PATH"

if ! npm install -g @fission-ai/openspec; then
    log_error "Failed to install OpenSpec CLI"
    exit 1
fi

if command -v openspec > /dev/null 2>&1; then
    log_success "OpenSpec installed: $(openspec --version)"
else
    log_error "OpenSpec installation completed but command is unavailable"
    exit 1
fi

# 安裝 uv
log_info "Installing uv package manager..."
# 以使用者層級安裝，避免 non-root 寫入系統 site-packages 失敗
if ! python3 -m pip install --user --upgrade pip uv; then
    log_error "Failed to install uv package manager"
    exit 1
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

# 安裝項目依賴
log_info "Installing project dependencies with uv..."
if ! $UV_CMD sync --all-extras; then
    log_error "Failed to install project dependencies"
    exit 1
fi
log_success "Project dependencies installed"

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

# 首次啟動 smoke check
log_info "Running first startup smoke checks (pre-commit, pytest, ruff)..."
smoke_failed=0

if ! run_smoke_check "pre-commit" bash -lc "pre-commit --version || python3 -m pre_commit --version"; then
    smoke_failed=1
fi

if ! run_smoke_check "pytest" bash -lc "$UV_CMD run pytest --version"; then
    smoke_failed=1
fi

if ! run_smoke_check "ruff" bash -lc "$UV_CMD run ruff --version"; then
    smoke_failed=1
fi

if [ "$smoke_failed" -ne 0 ]; then
    log_error "One or more smoke checks failed"
    exit 1
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
