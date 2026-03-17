#!/bin/bash
set -e

echo "📦 Installing uv package manager..."
pip install --upgrade pip
pip install uv

echo "🔧 Installing project dependencies with uv..."
uv sync --all-extras

echo "✨ Setting up ruff for code formatting..."
# ruff 已經通過 uv 的 dev dependencies 安裝

echo "� Setting up Docker-in-Docker..."
# 確保 docker socket 有正確的權限
if [ -S /var/run/docker.sock ]; then
    echo "✓ Docker socket 已連接"
else
    echo "⚠ Docker socket 未找到"
fi

# 新增 vscode 使用者到 docker group (如果 docker group 存在)
if getent group docker > /dev/null; then
    usermod -aG docker vscode || true
    echo "✓ vscode 使用者已新增到 docker group"
fi

echo "📝 Installing pre-commit hooks..."
uv pip install pre-commit || true

echo "✅ Dev container setup complete!"
echo ""
echo "🚀 Quick start commands:"
echo "   uv run pytest              - Run tests"
echo "   ruff check .               - Check code style"
echo "   ruff check . --fix         - Auto-fix code style"
echo "   jupyter lab --ip=0.0.0.0   - Start Jupyter Lab"
echo ""
echo "🐳 Docker commands:"
echo "   docker --version           - Check Docker version"
echo "   docker ps                  - List running containers"
echo "   docker build -t myapp .    - Build Docker image"
echo ""
