# Build stage
FROM rust:alpine AS builder
WORKDIR /app

# Install build dependencies
RUN apk add --no-cache \
    build-base \
    cmake \
    musl-dev\
    perl \
    perl-utils \
    perl-dev

COPY . .
RUN cargo build -p router-core --release
RUN cargo build -p router-api --release

# Runtime image stage
FROM alpine:latest
WORKDIR /app

# Install minimal runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    bash

# Copy binaries from builder
COPY --from=builder /app/target/release/router-core /usr/local/bin/router-core
COPY --from=builder /app/target/release/router-api /usr/local/bin/router-api

COPY build-docker/alpine/entrypoint.sh /usr/local/bin/entrypoint.sh

# Make everything executable
RUN chmod +x /usr/local/bin/*

# Expose API port
EXPOSE 24042

# Set entrypoint
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]