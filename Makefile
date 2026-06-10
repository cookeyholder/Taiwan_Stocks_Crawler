.PHONY: help tauri-dev tauri-build stock-cli stock-cli-help cargo-test clean \
        docker-help docker-version docker-ps docker-clean

help:
	@echo "Stock Trading System - Available commands:"
	@echo ""
	@echo "Tauri / Rust:"
	@echo "  make tauri-dev    - Start Tauri desktop app (dev mode)"
	@echo "  make tauri-build  - Build Tauri desktop app for production"
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

tauri-dev:
	cd stock-tauri && npm run tauri dev

tauri-build:
	cd stock-tauri && npm run tauri build

stock-cli:
	cargo run -p stock-cli -- $(ARGS)

stock-cli-help: ARGS=--help
stock-cli-help: stock-cli

cargo-test:
	cargo test -p stock-core

clean:
	rm -rf stock-tauri/src-tauri/target

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
