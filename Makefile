.PHONY: help build test clean run migrate docker-build docker-run fmt clippy docker-pull-ghcr run-ghcr ghcr-login

# Default target
help:
	@echo "Available targets:"
	@echo "  build            - Build the workspace"
	@echo "  build-app        - Build only task-backend"
	@echo "  build-migration  - Build only migration"
	@echo "  test             - Run tests for entire workspace"
	@echo "  run              - Run the task-backend application"
	@echo "  migrate          - Run database migrations"
	@echo "  migrate-status   - Check migration status"
	@echo "  migrate-down     - Rollback last migration"
	@echo "  docker-build     - Build Docker image"
	@echo "  docker-run       - Run with Docker Compose"
	@echo "  docker-pull-ghcr - Pull image from GitHub Container Registry"
	@echo "  run-ghcr         - Run with GitHub Container Registry image"
	@echo "  ghcr-login       - Login to GitHub Container Registry"
	@echo "  fmt              - Format code in workspace"
	@echo "  clippy           - Run clippy linter on workspace"
	@echo "  clean            - Clean build artifacts"
	@echo "  dev-setup        - Setup development environment"
	@echo "  dev              - Run development environment"
	@echo "  ci-check         - Run CI checks locally"

# Build the entire workspace
build:
	cargo build --release --workspace

# Build only the application
build-app:
	cargo build --release --package task-backend

# Build only the migration
build-migration:
	cargo build --release --package migration

# Run tests for the entire workspace
test:
	cargo test --workspace --verbose

# Run the application
run:
	cargo run --package task-backend

# Run database migrations
migrate:
	cargo run --package migration -- up

# Check migration status
migrate-status:
	cargo run --package migration -- status

# Rollback last migration
migrate-down:
	cargo run --package migration -- down

# Build Docker image
docker-build:
	docker build -t task-backend .

# Run with Docker Compose
docker-run:
	docker-compose up

# Pull image from GitHub Container Registry
docker-pull-ghcr:
	@echo "Pulling image from GitHub Container Registry..."
	@read -p "Enter GitHub username: " username; \
	read -p "Enter repository name: " repo; \
	docker pull ghcr.io/$$username/$$repo:latest

# Run with GitHub Container Registry image
run-ghcr:
	@echo "Running with GitHub Container Registry image..."
	@read -p "Enter GitHub username: " username; \
	read -p "Enter repository name: " repo; \
	docker run -p 3000:3000 \
		-e DATABASE_URL=postgres://postgres:password@host.docker.internal:5432/taskdb \
		ghcr.io/$$username/$$repo:latest

# Login to GitHub Container Registry
ghcr-login:
	@echo "Logging in to GitHub Container Registry..."
	@read -p "Enter GitHub username: " username; \
	read -s -p "Enter GitHub Personal Access Token: " token; \
	echo; \
	echo $$token | docker login ghcr.io -u $$username --password-stdin

# Format code in workspace
fmt:
	cargo fmt --all

# Run clippy on workspace
clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

# Clean build artifacts
clean:
	cargo clean
	docker-compose down -v
	docker system prune -f

# Development setup
dev-setup:
	@if [ ! -f .env ]; then \
		echo "Creating .env file..."; \
		echo "DATABASE_URL=postgres://postgres:password@localhost:5432/taskdb" > .env; \
		echo "SERVER_ADDR=0.0.0.0:3000" >> .env; \
		echo "RUST_LOG=info" >> .env; \
		echo "RUST_BACKTRACE=1" >> .env; \
		echo "Please edit .env file with your configuration"; \
	else \
		echo ".env file already exists"; \
	fi

# Run development environment
dev:
	docker-compose up postgres -d
	@echo "Waiting for PostgreSQL to be ready..."
	@sleep 5
	$(MAKE) migrate
	$(MAKE) run

# Run CI checks locally
ci-check:
	$(MAKE) fmt
	$(MAKE) clippy
	$(MAKE) test

# Workspace-specific commands
workspace-info:
	@echo "Workspace members:"
	@cargo metadata --format-version 1 | jq -r '.workspace_members[]'

# Check all packages individually
check-all:
	cargo check --workspace --all-targets --all-features

# Run specific package tests
test-app:
	cargo test --package task-backend --verbose

test-migration:
	cargo test --package migration --verbose

# Install development tools
install-tools:
	cargo install cargo-audit
	cargo install cargo-tarpaulin
	cargo install sqlx-cli --features postgres

# Security audit
audit:
	cargo audit

# Generate documentation
docs:
	cargo doc --workspace --no-deps --open

# Profile build
profile:
	cargo build --workspace --release --timings