.PHONY: help install dev-install format check test lint clean jupyter

help:
	@echo "🚀 Stock Trading System - Available commands:"
	@echo ""
	@echo "Setup:"
	@echo "  make install      - Install all dependencies with uv"
	@echo "  make dev-install  - Install with dev and jupyter dependencies"
	@echo ""
	@echo "Development:"
	@echo "  make format       - Format code with ruff"
	@echo "  make check        - Check code style with ruff"
	@echo "  make lint         - Run all linters and formatters"
	@echo "  make fix          - Auto-fix code style issues"
	@echo ""
	@echo "Testing:"
	@echo "  make test         - Run pytest"
	@echo "  make test-cov     - Run tests with coverage report"
	@echo ""
	@echo "Jupyter:"
	@echo "  make jupyter      - Start Jupyter Lab"
	@echo ""
	@echo "Maintenance:"
	@echo "  make clean        - Remove cache and temporary files"
	@echo "  make pre-commit   - Run pre-commit checks"
	@echo ""

install:
	@echo "📦 Installing dependencies with uv..."
	uv sync

dev-install:
	@echo "📦 Installing all dependencies including dev and jupyter..."
	uv sync --all-extras

format:
	@echo "✨ Formatting code with ruff..."
	ruff format .

check:
	@echo "🔍 Checking code style with ruff..."
	ruff check .

lint: check format
	@echo "✅ Linting complete!"

fix:
	@echo "🔧 Auto-fixing code style..."
	ruff check . --fix
	ruff format .

test:
	@echo "🧪 Running pytest..."
	uv run pytest

test-cov:
	@echo "🧪 Running pytest with coverage..."
	uv run pytest --cov=. --cov-report=html

clean:
	@echo "🧹 Cleaning up..."
	find . -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null || true
	find . -type f -name "*.pyc" -delete
	find . -type d -name ".pytest_cache" -exec rm -rf {} + 2>/dev/null || true
	find . -type d -name ".ruff_cache" -exec rm -rf {} + 2>/dev/null || true
	find . -type d -name "*.egg-info" -exec rm -rf {} + 2>/dev/null || true
	find . -type d -name ".coverage" -exec rm -rf {} + 2>/dev/null || true
	find . -type d -name "htmlcov" -exec rm -rf {} + 2>/dev/null || true
	@echo "✅ Cleanup complete!"

jupyter:
	@echo "📓 Starting Jupyter Lab..."
	jupyter lab --ip=0.0.0.0 --allow-root

pre-commit:
	@echo "🔍 Running pre-commit checks..."
	pre-commit run --all-files

.DEFAULT_GOAL := help
