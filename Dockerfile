# Multi-stage build for Rust workspace application
FROM rust:1.86-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    lld \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Enable sccache for Docker builds
ENV RUSTC_WRAPPER=sccache
ENV SCCACHE_DIR=/sccache

# Install sccache
RUN cargo install sccache --version 0.8.2

# Copy workspace configuration first for better caching
COPY Cargo.toml Cargo.lock ./
COPY rust-toolchain.toml ./

# Create dummy projects to cache dependencies
RUN mkdir -p task-backend/src migration/src && \
    echo "fn main() {}" > task-backend/src/main.rs && \
    echo "fn main() {}" > migration/src/main.rs

# Copy workspace member Cargo.toml files
COPY task-backend/Cargo.toml ./task-backend/
COPY migration/Cargo.toml ./migration/

# Build dependencies only (this layer will be cached)
RUN --mount=type=cache,target=/sccache \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked --workspace

# Remove dummy source files
RUN rm -rf task-backend/src migration/src

# Copy actual source code
COPY task-backend/ ./task-backend/
COPY migration/ ./migration/

# Touch main.rs to ensure rebuild with actual code
RUN touch task-backend/src/main.rs migration/src/main.rs

# Build the entire workspace with actual code
RUN --mount=type=cache,target=/sccache \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked --workspace && \
    cp target/release/task-backend /usr/local/bin/ && \
    cp target/release/migration /usr/local/bin/

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libpq5 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user for security
RUN useradd -r -s /bin/false -m -d /app app

# Copy binaries from builder stage
COPY --from=builder /usr/local/bin/task-backend /usr/local/bin/task-backend
COPY --from=builder /usr/local/bin/migration /usr/local/bin/migration

# Set proper ownership
RUN chown app:app /usr/local/bin/task-backend /usr/local/bin/migration

# Switch to app user
USER app

# Set working directory
WORKDIR /app

# Expose port
EXPOSE 3000

# Add health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Default command
CMD ["task-backend"]