# Build stage
FROM rust:alpine AS builder
WORKDIR /app

# Better build environment for Rust + musl
RUN apk add --no-cache \
    build-base \
    cmake \
    musl-dev \
    pkgconfig \
    perl \
    sqlite-dev \
    openssl-dev \
    openssl-libs-static

# Optimize Rust compilation for musl
ENV RUSTFLAGS="-C target-feature=-crt-static -C link-arg=-s"
ENV CARGO_BUILD_TARGET="x86_64-unknown-linux-musl"
RUN rustup target add x86_64-unknown-linux-musl

COPY . .
RUN cargo build -p router-cli --release --target x86_64-unknown-linux-musl
RUN cargo build -p router-core --release --target x86_64-unknown-linux-musl
RUN cargo build -p router-api --release --target x86_64-unknown-linux-musl

# Runtime image stage
FROM alpine:latest
WORKDIR /app

# Install minimal runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    bash \
    sqlite-libs

# Copy binaries from builder
COPY --from=builder /app/target/release/router-core /usr/local/bin/router-core
COPY --from=builder /app/target/release/router-api /usr/local/bin/router-api
COPY --from=builder /app/target/release/router-cli /usr/local/bin/gwrs

COPY build-docker/alpine/entrypoint.sh /usr/local/bin/entrypoint.sh

# Make everything executable
RUN chmod +x /usr/local/bin/*

# Set entrypoint
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]