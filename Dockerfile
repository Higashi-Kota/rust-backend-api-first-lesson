# Multi-stage build for Rust workspace application
FROM rust:1.86-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy workspace configuration first for better caching
COPY Cargo.toml Cargo.lock ./
COPY rust-toolchain.toml ./

# Copy all workspace members
COPY task-backend/ ./task-backend/
COPY migration/ ./migration/

# Build the entire workspace (this will build both task-backend and migration)
RUN cargo build --release --locked --workspace

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user for security
RUN useradd -r -s /bin/false -m -d /app app

# Copy binary from builder stage
COPY --from=builder /app/target/release/task-backend /usr/local/bin/task-backend

# Copy migration binary as well (useful for running migrations in container)
COPY --from=builder /app/target/release/migration /usr/local/bin/migration

# Set proper ownership
RUN chown app:app /usr/local/bin/task-backend /usr/local/bin/migration

# Switch to app user
USER app

# Set working directory
WORKDIR /app

# Expose port
EXPOSE 5000

# Add health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:5000/health || exit 1

# Default command
CMD ["task-backend"]