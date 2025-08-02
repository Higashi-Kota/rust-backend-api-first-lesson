# Technology Stack

## Core Technologies
- **Language**: Rust 1.86.0
- **Web Framework**: Axum 0.8 (async web framework)
- **Async Runtime**: Tokio (full features)
- **Database**: PostgreSQL
- **ORM**: SeaORM 1.1.12 with migration support
- **Authentication**: JWT (jsonwebtoken crate)
- **Password Hashing**: Argon2

## Key Dependencies
- **Serialization**: serde, serde_json
- **Validation**: validator crate with custom validators
- **Date/Time**: chrono with serde support
- **UUID**: uuid v4 for IDs
- **Error Handling**: thiserror for custom error types
- **Logging**: tracing, tracing-subscriber with env-filter
- **HTTP Middleware**: tower-http (CORS, tracing, timeout, limit)

## External Services
- **Email**: 
  - Development: MailHog (local SMTP)
  - Production: Mailgun support
  - lettre for SMTP transport
- **Storage**: 
  - MinIO (S3-compatible) for development
  - AWS S3 SDK for production
  - Image processing with image crate (JPEG, PNG, WebP)
- **Payment**: 
  - Stripe integration (async-stripe)
  - Webhook support
  - Development mode with mocking

## Development Tools
- **Testing**: 
  - testcontainers for integration tests
  - PostgreSQL test containers
  - reqwest for HTTP testing
- **Code Quality**:
  - rustfmt for formatting
  - clippy for linting
  - cargo-audit for security
- **Build Tools**:
  - Cargo workspaces (task-backend, migration)
  - Make for common tasks
  - Docker support

## Environment
- Configuration via .env files (dotenvy)
- Multiple build profiles (dev, test, ci, release)
- Docker Compose for local services