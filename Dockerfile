# ----------------------------------------------------------------------
# Stage 1: Build Stage - Dynamic linking (default GNU target)
# ----------------------------------------------------------------------
FROM rust:latest AS builder

# Install build dependencies (especially libssl-dev for openssl-sys crate to find headers)
RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev build-essential \
    && rm -rf /var/lib/apt/lists/*

# Set container working directory to /app
WORKDIR /app

# 1. Cache dependencies - Copy only Cargo.toml/Cargo.lock
COPY Cargo.toml Cargo.lock ./

# 2. Try to cache dependencies (use default target, skip Musl step)
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release
RUN rm -rf src

# 3. Copy all source code
COPY . .

# 4. Final build (use default dynamic linking target)
RUN cargo build --release

# ----------------------------------------------------------------------
# Stage 2: Runtime Stage - Ensure dynamic libraries exist
# ----------------------------------------------------------------------
FROM debian:bookworm-slim

# Install runtime dynamic OpenSSL libraries (libssl3 is the runtime required library)
RUN apt-get update && apt-get install -y \
    libssl3 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy compiled binary (dynamic linking target path is /app/target/release/)
COPY --from=builder /app/target/release/solji-indexer /usr/local/bin/solji-indexer

# Copy .env file
COPY .env /usr/local/bin/.env

# Set working directory to program directory
WORKDIR /usr/local/bin

# Expose application port
EXPOSE 8080

# Define command to execute when container starts
CMD ["/usr/local/bin/solji-indexer"]
