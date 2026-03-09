# ============================================================================
# LinkBit Rust Microservice - Dockerfile
# ============================================================================
# Multi-stage build for Rust application
# ============================================================================

# Stage 1: Build
FROM rust:alpine AS build

WORKDIR /app

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev

# Copy Cargo files and download dependencies (cached layer)
COPY cargo.toml Cargo.toml
RUN mkdir src && \
  echo "fn main() {}" > src/main.rs && \
  cargo build --release && \
  rm -rf src

# Copy source code and build
COPY src ./src
RUN touch src/main.rs
RUN cargo build --release

# Stage 2: Runtime
FROM alpine:3.19

WORKDIR /app

# Install runtime dependencies
RUN apk add --no-cache libgcc openssl ca-certificates

# Create non-root user
RUN addgroup -S linkbit && adduser -S linkbit -G linkbit

# Copy binary from build stage
COPY --from=build /app/target/release/linkbit-bitcoin-escrow /app/linkbit-oracle

# Change ownership
RUN chown -R linkbit:linkbit /app

# Switch to non-root user
USER linkbit

# Expose port
EXPOSE 9000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://127.0.0.1:9000/health || exit 1

# Run application
ENTRYPOINT ["./linkbit-oracle"]
