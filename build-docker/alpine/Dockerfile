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

# Install OpenRC and other dependencies
RUN apk add --no-cache \
    openrc \
    ca-certificates \
    procps \
    bash

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

# Create service files directory
RUN mkdir -p /etc/init.d

# Copy service files
COPY build-docker/alpine/router-core.initd /etc/init.d/gwrs-core
COPY build-docker/alpine/router-api.initd /etc/init.d/gwrs-api

# Make the init scripts executable
RUN chmod +x /etc/init.d/gwrs-core
RUN chmod +x /etc/init.d/gwrs-api

# Add services to default runlevel
RUN rc-update add gwrs-core default
RUN rc-update add gwrs-api default

# Setup entrypoint
COPY build-docker/alpine/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Expose API
EXPOSE 24042

ENTRYPOINT ["/entrypoint.sh"]
CMD ["/sbin/init"]