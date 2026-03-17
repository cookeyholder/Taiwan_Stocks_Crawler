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

# 檢查 Python 版本
log_info "Checking Python version..."
python_version=$(python3 --version 2>&1 | awk '{print $2}')
log_success "Python version: $python_version"

# 安裝 uv
log_info "Installing uv package manager..."
if ! pip install --upgrade pip uv; then
    log_error "Failed to install uv package manager"
    exit 1
fi

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

# 驗證 ruff 安裝
log_info "Verifying ruff installation..."
if $UV_CMD run ruff --version > /dev/null 2>&1; then
    log_success "ruff is available"
else
    log_warn "ruff verification skipped"
fi

# 設定 Docker-in-Docker
log_info "Setting up Docker-in-Docker..."
if [ -S /var/run/docker.sock ]; then
    log_success "Docker socket is connected"
else
    log_warn "Docker socket not found (optional)"
fi

# 新增 vscode 使用者到 docker group
if getent group docker > /dev/null 2>&1; then
    if usermod -aG docker vscode 2>/dev/null; then
        log_success "vscode user added to docker group"
    else
        log_warn "Could not add vscode to docker group"
    fi
fi

# 安裝 pre-commit hooks
log_info "Installing pre-commit hooks..."
if $UV_CMD pip install pre-commit > /dev/null 2>&1; then
    log_success "pre-commit installed"
else
    log_warn "pre-commit installation skipped"
fi

# 列出可用命令
log_success "Dev container setup complete!"
echo ""
echo "🚀 Quick start commands:"
echo "   uv run pytest                    - Run tests"
echo "   ruff check .                     - Check code style"
echo "   ruff check . --fix               - Auto-fix code style"
echo "   ruff format .                    - Format code"
echo "   uv run jupyter lab --ip=0.0.0.0  - Start Jupyter Lab"
echo ""
echo "🐳 Docker commands:"
echo "   docker --version                 - Check Docker version"
echo "   docker ps                        - List running containers"
echo ""
echo "📚 Useful links:"
echo "   - uv docs: https://docs.astral.sh/uv/"
echo "   - ruff docs: https://docs.astral.sh/ruff/"
echo ""
