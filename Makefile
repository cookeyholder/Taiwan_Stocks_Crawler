.PHONY: help install dev-install format check test lint clean jupyter docker-help docker-version docker-ps docker-clean

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
	@echo "Docker:"
	@echo "  make docker-help  - Show Docker help"
	@echo "  make docker-version - Check Docker version"
	@echo "  make docker-ps    - List running containers"
	@echo "  make docker-clean - Clean up unused Docker resources"
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

gui:
	@echo "🖥️ Starting GUI Application..."
	.venv/bin/python -m market_crawler.gui

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

docker-help:
	@echo "🐳 Docker-in-Docker Commands:"
	@echo ""
	@echo "  docker --version       - Check Docker version"
	@echo "  docker ps              - List running containers"
	@echo "  docker images          - List Docker images"
	@echo "  docker build -t <name> - Build an image"
	@echo "  docker run <image>     - Run a container"
	@echo "  docker-compose up      - Start services"
	@echo "  docker-compose down    - Stop services"
	@echo "  docker system df       - Show disk usage"
	@echo "  docker system prune    - Clean up resources"
	@echo ""

docker-version:
	@echo "🐳 Docker version:"
	docker --version
	docker-compose --version 2>/dev/null || echo "docker-compose not found"

docker-ps:
	@echo "🐳 Running Docker containers:"
	docker ps

docker-clean:
	@echo "🐳 Cleaning up Docker resources..."
	docker system prune -f
	@echo "✅ Docker cleanup complete!"

.DEFAULT_GOAL := help
