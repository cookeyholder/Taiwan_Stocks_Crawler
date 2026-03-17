#!/bin/bash
set -e

echo "📦 Installing uv package manager..."
pip install --upgrade pip
pip install uv

echo "🔧 Installing project dependencies with uv..."
uv sync --all-extras

echo "✨ Setting up ruff for code formatting..."
# ruff 已經通過 uv 的 dev dependencies 安裝

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
