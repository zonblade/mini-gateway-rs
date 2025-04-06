# Build stage
FROM rust:bookworm AS builder
WORKDIR /app
# Install build dependencies
RUN apt-get update && apt-get install -y \
    cmake \
    && rm -rf /var/lib/apt/lists/*

COPY . .
RUN cargo build -p router-core --release
RUN cargo build -p router-api --release

# Runtime image stage
FROM debian:bookworm-slim
WORKDIR /app

# Install systemd and other dependencies
RUN apt-get update && apt-get install -y \
    systemd \
    systemd-sysv \
    ca-certificates \
    procps \
    && rm -rf /var/lib/apt/lists/*

# Copy binaries from builder
COPY --from=builder /app/target/release/router-core /usr/local/bin/router-core
COPY --from=builder /app/target/release/router-api /usr/local/bin/router-api

# Create necessary directories
RUN mkdir -p /opt/gwrs/bin && \
    mkdir -p /opt/gwrs/conf && \
    mkdir -p /var/log/gwrs

# Create symlinks to binaries
RUN ln -sf /usr/local/bin/router-core /opt/gwrs/bin/router-core && \
    ln -sf /usr/local/bin/router-api /opt/gwrs/bin/router-api

# Create systemd service files
RUN mkdir -p /etc/systemd/system
COPY build-docker/debian/router-core.service /etc/systemd/system/gwrs-core.service
COPY build-docker/debian/router-api.service /etc/systemd/system/gwrs-api.service

# Enable services
RUN systemctl enable gwrs-core.service
RUN systemctl enable gwrs-api.service

# Set up entrypoint script
COPY build-docker/debian/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Use systemd as command, with a fallback path
ENTRYPOINT ["/entrypoint.sh"]
CMD ["/lib/systemd/systemd"]

# Expose all ports since this is a base image
EXPOSE 1-65535