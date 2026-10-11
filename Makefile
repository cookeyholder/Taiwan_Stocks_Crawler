.PHONY: help gpui-run gpui-build stock-cli stock-cli-help cargo-test clean \
        docker-help docker-version docker-ps docker-clean

help:
	@echo "Stock Trading System - Available commands:"
	@echo ""
	@echo "GPUI / Rust:"
	@echo "  make gpui-run     - Start GPUI desktop app"
	@echo "  make gpui-build   - Build GPUI desktop app for production"
	@echo "  make stock-cli    - Run Rust CLI (pass args via ARGS=)"
	@echo "  make cargo-test   - Run Rust unit tests"
	@echo ""
	@echo "Docker:"
	@echo "  make docker-help  - Show Docker help"
	@echo "  make docker-version - Check Docker version"
	@echo "  make docker-ps    - List running containers"
	@echo "  make docker-clean - Clean up unused Docker resources"
	@echo ""
	@echo "Maintenance:"
	@echo "  make clean        - Remove cache and temporary files"

gpui-run:
	cargo run -p stock-gpui

gpui-build:
	./scripts/build-linux.sh

stock-cli:
	cargo run -p stock-cli -- $(ARGS)

stock-cli-help: ARGS=--help
stock-cli-help: stock-cli

cargo-test:
	cargo test --workspace

clean:
	cargo clean

docker-help:
	@echo "Docker Commands:"
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
	docker --version
	docker-compose --version 2>/dev/null || echo "docker-compose not found"

docker-ps:
	docker ps

docker-clean:
	docker system prune -f

.DEFAULT_GOAL := help
