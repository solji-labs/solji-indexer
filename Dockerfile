FROM rust:latest AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    ca-certificates \
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
RUN rm -rf target/release/solji-indexer target/release/deps/solji_indexer* && \
    cargo build --release --locked

RUN ls -lh target/release/solji-indexer && \
    SIZE=$(stat -c%s target/release/solji-indexer) && \
    if [ $SIZE -lt 1000000 ]; then \
    echo "ERROR: Binary too small ($SIZE bytes)" && exit 1; \
    fi
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
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
