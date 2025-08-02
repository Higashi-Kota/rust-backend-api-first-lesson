# Project Structure

## Repository Layout
```
rust-backend-api-first-lesson/
├── task-backend/           # Main application
│   ├── src/
│   │   ├── api/           # HTTP handlers & routing
│   │   │   └── handlers/  # Endpoint implementations
│   │   ├── domain/        # Core business models
│   │   ├── service/       # Business logic layer
│   │   ├── repository/    # Data access layer
│   │   ├── middleware/    # Auth, logging, permissions
│   │   ├── config/        # Configuration management
│   │   ├── utils/         # Helpers (JWT, email, validation)
│   │   ├── types/         # Shared types (ApiResponse, pagination)
│   │   ├── extractors/    # Custom Axum extractors
│   │   ├── logging/       # Structured logging utilities
│   │   ├── shared/        # Cross-cutting concerns
│   │   ├── main.rs        # Application entry point
│   │   ├── db.rs          # Database connection
│   │   ├── error.rs       # Error types
│   │   └── lib.rs         # Library exports
│   ├── tests/
│   │   ├── integration/   # E2E tests by feature
│   │   ├── unit/          # Unit tests
│   │   └── common/        # Test utilities
│   └── Cargo.toml
├── migration/             # Database migrations
│   ├── src/
│   │   ├── lib.rs
│   │   └── m{date}_{seq}_{name}.rs files
│   └── Cargo.toml
├── .env.example           # Environment template
├── .env.test             # Test environment
├── CLAUDE.md             # Project guidelines
├── Cargo.toml            # Workspace configuration
├── Cargo.lock            # Dependency lock file
├── Makefile              # Development commands
├── docker-compose.yml    # Local services
├── Dockerfile            # Container build
└── rust-toolchain.toml   # Rust version config
```

## Key Directories

### API Layer (`api/handlers/`)
- Auth handlers (signup, signin, refresh)
- Task CRUD operations
- Team & organization management
- Admin endpoints
- Payment integration

### Service Layer (`service/`)
- AuthService: Authentication logic
- TaskService: Task business rules
- UserService: User management
- PermissionService: Dynamic permissions
- PaymentService: Stripe integration

### Repository Layer (`repository/`)
- Direct database operations via SeaORM
- One repository per domain entity
- Transaction support
- Query builders

### Middleware (`middleware/`)
- JWT authentication
- Role-based authorization  
- Request logging
- Permission checking
- Admin-only routes

### Domain Models (`domain/`)
- User, Role, Permission entities
- Task, Team, Organization models
- Subscription tiers
- Audit logs

### Test Organization (`tests/`)
Integration tests organized by feature:
- auth/: Authentication flows
- tasks/: Task operations
- team/: Team management
- permission/: Permission system
- payment/: Payment processing
- admin/: Admin functions
- gdpr/: GDPR compliance

## Configuration Files
- `.env`: Runtime configuration
- `Cargo.toml`: Dependencies & workspace
- `rust-toolchain.toml`: Rust version (1.86.0)
- `docker-compose.yml`: PostgreSQL, MailHog, MinIO
- `Makefile`: Developer commands