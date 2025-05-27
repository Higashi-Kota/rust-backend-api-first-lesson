.PHONY: help build test clean run migrate docker-build docker-run fmt clippy docker-pull-ghcr run-ghcr ghcr-login

# Default target
help:
	@echo "Available targets:"
	@echo "  build            - Build the application"
	@echo "  test             - Run tests"
	@echo "  run              - Run the application"
	@echo "  migrate          - Run database migrations"
	@echo "  docker-build     - Build Docker image"
	@echo "  docker-run       - Run with Docker Compose"
	@echo "  docker-pull-ghcr - Pull image from GitHub Container Registry"
	@echo "  run-ghcr         - Run with GitHub Container Registry image"
	@echo "  ghcr-login       - Login to GitHub Container Registry"
	@echo "  fmt              - Format code"
	@echo "  clippy           - Run clippy linter"
	@echo "  clean            - Clean build artifacts"
	@echo "  dev-setup        - Setup development environment"
	@echo "  dev              - Run development environment"
	@echo "  ci-check         - Run CI checks locally"

# Build the application
build:
	cargo build --release

# Run tests
test:
	cargo test --verbose

# Run the application
run:
	cargo run

# Run database migrations
migrate:
	cargo run --bin migration -- up

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

# Format code
fmt:
	cargo fmt --all

# Run clippy
clippy:
	cargo clippy --all-targets --all-features -- -D warnings

# Clean build artifacts
clean:
	cargo clean
	docker-compose down -v
	docker system prune -f

# Development setup
dev-setup:
	cp .env.example .env
	@echo "Please edit .env file with your configuration"

# Run development environment
dev:
	docker-compose up postgres -d
	sleep 3
	$(MAKE) migrate
	$(MAKE) run

# Run CI checks locally
ci-check:
	$(MAKE) fmt
	$(MAKE) clippy
	$(MAKE) test