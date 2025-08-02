# Suggested Commands for Development

## Essential Development Commands

### Quick Start
```bash
# Setup environment
cp .env.example .env
make dev                    # Start PostgreSQL, MailHog, MinIO and run app

# Or step by step:
docker-compose up postgres mailhog minio minio-mc -d
make migrate               # Run database migrations
make run                   # Start the application
```

### Build & Test Commands
```bash
# Building
make build                 # Build entire workspace (release)
make build-dev            # Fast development build
make build-ci             # CI optimized build

# Testing
make test                 # Run all tests
make test-app            # Test application only
cargo test --lib         # Unit tests only (fast)
cargo test integration::tasks::crud_tests  # Specific integration test
make test-integration GROUP=integration::auth  # Test group

# Code Quality - MUST PASS BEFORE COMMIT
make fmt                  # Format code
make clippy              # Run linter
make ci-check-fast       # Run all CI checks locally
```

### Database Operations
```bash
make migrate             # Run pending migrations
make migrate-status      # Check migration status
make migrate-down        # Rollback last migration
```

### Development Workflow
```bash
# Auto-reload on changes
cargo watch -x "run --package task-backend"

# Check for issues
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check

# Generate password hash for admin
make generate-password-hash
```

### Docker Commands
```bash
make docker-build        # Build Docker image
make docker-run         # Run with Docker Compose
docker-compose logs -f app  # View application logs
```

### System Commands (Linux)
```bash
# File search (prefer ripgrep over grep)
rg "pattern" --type rust   # Search in Rust files
rg -i "todo"               # Case-insensitive search

# Standard commands
git status
git diff
git log --oneline -10
ls -la
cd /path/to/dir
find . -name "*.rs"
```

### Service URLs (when running locally)
- **API**: http://localhost:5000
- **MailHog UI**: http://localhost:8025
- **MinIO Console**: http://localhost:9001
  - Username: minioadmin
  - Password: minioadmin

## Important CI Requirements
Before pushing code, ensure:
1. `make fmt` - Code is formatted
2. `make clippy` - No clippy warnings
3. `make test` - All tests pass
4. No `#[allow(dead_code)]` added
5. API changes include 3 test cases (success, forbidden, unauthorized)