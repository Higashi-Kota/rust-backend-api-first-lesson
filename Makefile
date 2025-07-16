.PHONY: help build test clean run migrate docker-build docker-run fmt clippy docker-pull-ghcr run-ghcr ghcr-login generate-password-hash

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
	@echo "  dev              - Run development environment with MailHog"
	@echo "  ci-check         - Run CI checks locally"
	@echo "  ci-check-fast    - Run CI checks with optimized profile"
	@echo "  build-ci         - Build with CI profile"
	@echo "  build-dev        - Fast development build"
	@echo "  test-integration - Run specific integration test group (GROUP=...)"
	@echo "  generate-password-hash - Generate Argon2 password hash for admin user"

# Build the entire workspace
build:
	cargo build --release --workspace

# Build only the application
build-app:
	cargo build --release --package task-backend

# Build only the migration
build-migration:
	cargo build --release --package migration

# Build with CI profile (optimized for CI/CD)
build-ci:
	cargo build --profile ci --workspace

# Fast development build
build-dev:
	cargo build --workspace

# Run tests for the entire workspace
test:
	cargo test --workspace --verbose

# Run the application
run:
	RUSTC_WRAPPER="" cargo run --package task-backend --bin task-backend

# Run database migrations
migrate:
	RUSTC_WRAPPER="" cargo run --package migration -- up

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
	docker run -p 5000:5000 \
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
		echo "SERVER_ADDR=0.0.0.0:5000" >> .env; \
		echo "RUST_LOG=info" >> .env; \
		echo "RUST_BACKTRACE=1" >> .env; \
		echo "Please edit .env file with your configuration"; \
	else \
		echo ".env file already exists"; \
	fi

# Run development environment with MailHog and MinIO
dev:
	docker-compose up postgres mailhog minio minio-mc -d
	@echo "Waiting for services to be ready..."
	@sleep 10
	RUSTC_WRAPPER="" $(MAKE) migrate
	@echo "📧 MailHog Web UI: http://localhost:8025"
	@echo "🗄️  MinIO Console: http://localhost:9001"
	RUSTC_WRAPPER="" $(MAKE) run

# Run CI checks locally
ci-check:
	$(MAKE) fmt
	$(MAKE) clippy
	$(MAKE) test

# Run CI checks with optimized profile
ci-check-fast:
	RUSTC_WRAPPER="" cargo fmt --all -- --check
	RUSTC_WRAPPER="" cargo clippy --workspace --all-targets --all-features -- -D warnings
	RUSTC_WRAPPER="" cargo test --profile ci --workspace --verbose

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

# Run specific integration test group
test-integration:
	@echo "Usage: make test-integration GROUP=integration::auth"
	@if [ -z "$(GROUP)" ]; then \
		echo "Error: GROUP parameter is required"; \
		exit 1; \
	fi
	cargo test --test main $(GROUP) --verbose

# Payment test helpers
test-payment-mock:
	@echo "🧪 Running payment tests with mock mode..."
	@cp .env.test .env.test.backup 2>/dev/null || true
	@echo "PAYMENT_DEVELOPMENT_MODE=true" > .env.test.tmp
	@grep -v "PAYMENT_DEVELOPMENT_MODE\|STRIPE_" .env.test >> .env.test.tmp || true
	@mv .env.test.tmp .env.test
	@make test-integration GROUP=integration::payment
	@mv .env.test.backup .env.test 2>/dev/null || true

test-payment-stripe:
	@echo "🧪 Running payment tests with Stripe test mode..."
	@echo "⚠️  Make sure your Stripe test credentials are in .env.test"
	@make test-integration GROUP=integration::payment

stripe-listen:
	@echo "🎧 Starting Stripe webhook forwarding..."
	@echo "📝 Copy the webhook secret to your .env file"
	stripe listen --forward-to localhost:5000/webhooks/stripe

# Generate Argon2 password hash for admin user
generate-password-hash:
	@echo "Building password hash generator..."
	@cargo build --package task-backend --bin generate-password-hash --release
	@echo ""
	@echo "=== Password Hash Generator ==="
	@echo "Usage: Enter password when prompted or pass as argument"
	@echo "Example: make generate-password-hash PASSWORD='MySecurePass123!'"
	@echo ""
	@if [ -z "$(PASSWORD)" ]; then \
		./target/release/generate-password-hash; \
	else \
		./target/release/generate-password-hash "$(PASSWORD)"; \
	fi