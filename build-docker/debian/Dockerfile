# Build stage
FROM rust:bookworm AS builder
WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    perl \
    libperl-dev \
    libsqlite3-dev \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY . .
RUN cargo build -p router-cli --release
RUN cargo build -p router-core --release
RUN cargo build -p router-api --release

# Runtime image stage
FROM debian:bookworm-slim
WORKDIR /app

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    bash \
    libsqlite3-0 \
    libssl3 \
    procps \
    util-linux \
    coreutils \
    && rm -rf /var/lib/apt/lists/*

# Copy binaries from builder
COPY --from=builder /app/target/release/router-core /usr/local/bin/router-core
COPY --from=builder /app/target/release/router-api /usr/local/bin/router-api
COPY --from=builder /app/target/release/router-cli /usr/local/bin/gwrs

COPY build-docker/debian/entrypoint.sh /usr/local/bin/entrypoint.sh

# Make everything executable
RUN chmod +x /usr/local/bin/*

# Set entrypoint
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]