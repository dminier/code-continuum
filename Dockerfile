# ============================================================
# Multi-stage Dockerfile for Code Graph RAG Rust
# ============================================================
# Stage 1: Builder - Compile Rust application
# Stage 2: Runtime - Minimal image with binary only
# ============================================================

# ==================== STAGE 1: BUILD ====================
# Use same base as devcontainer (which works)
FROM mcr.microsoft.com/devcontainers/rust:2.0.3-1 AS builder
 

# Install build dependencies (same as devcontainer)
RUN apt-get update && apt-get install -y \
    cmake \
    build-essential \
    pkg-config \
    libssl-dev \
    python3 \
    python3-venv \
    python3-pip \
    git \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Configure certificate paths for all tools
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
ENV CURL_CA_BUNDLE=/etc/ssl/certs/ca-certificates.crt
ENV CARGO_HTTP_CAINFO=/etc/ssl/certs/ca-certificates.crt
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
ENV GIT_SSL_CAINFO=/etc/ssl/certs/ca-certificates.crt
ENV REQUESTS_CA_BUNDLE=/etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_DIR=/etc/ssl/certs

RUN curl -v https://static.crates.io/
# Create app directory
WORKDIR /app

# Minimal Cargo config (like devcontainer)
RUN mkdir -p .cargo && \
    echo '[net]' > .cargo/config.toml && \
    echo 'git-fetch-with-cli = true' >> .cargo/config.toml

# Match devcontainer env
ENV UV_NATIVE_TLS=true
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=git

# Copy manifests
COPY Cargo.toml ./

# Copy source code (tests not needed for production)
COPY src ./src

# Build release binary (optimized)
RUN cargo build --release --bin code-continuum

# Strip debug symbols to reduce size
RUN strip /app/target/release/code-continuum

# ==================== STAGE 2: RUNTIME ====================
FROM debian:bookworm-slim

# Install runtime dependencies only
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 appuser

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/code-continuum /usr/local/bin/code-continuum

# Create data directory for analysis results
RUN mkdir -p /app/data && chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Default Neo4j connection (override via environment variables)
ENV NEO4J_URI=bolt://neo4j:7687
ENV NEO4J_USER=neo4j
ENV NEO4J_PASSWORD=production_password_change_me
ENV RUST_LOG=info

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD test -f /usr/local/bin/code-continuum || exit 1

# Volume for code to analyze
VOLUME ["/app/data"]

# Entry point
ENTRYPOINT ["code-continuum"]
CMD ["--help"]
