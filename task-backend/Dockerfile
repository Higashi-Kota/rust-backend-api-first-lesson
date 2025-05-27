# Multi-stage build for Rust application
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./
COPY migration/Cargo.toml ./migration/

# Create dummy src files to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > migration/src/main.rs && \
    cargo build --release && \
    rm -rf src migration/src

# Copy source code
COPY src ./src
COPY migration ./migration

# Build application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false appuser

# Copy binary from builder stage
COPY --from=builder /app/target/release/task-backend /usr/local/bin/task-backend
COPY --from=builder /app/target/release/migration /usr/local/bin/migration

# Set ownership
RUN chown appuser:appuser /usr/local/bin/task-backend /usr/local/bin/migration

# Switch to non-root user
USER appuser

# Set environment variables
ENV RUST_LOG=info
ENV SERVER_ADDR=0.0.0.0:3000

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Default command
CMD ["task-backend"]