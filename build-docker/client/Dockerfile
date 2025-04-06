# Build stage
FROM rust:bookworm AS builderDebian
WORKDIR /app
# Install build dependencies
RUN apt-get update && apt-get install -y \
    cmake \
    && rm -rf /var/lib/apt/lists/*
COPY . .
RUN cargo build -p router-gui --release

# Build stage
FROM rust:alpine AS builderAlpine
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
RUN cargo build -p router-gui --release

# Final stage for output to host
FROM alpine:latest
WORKDIR /output
# Copy from Debian build and rename to gwrc-deb
COPY --from=builderDebian /app/target/release/router-gui /output/gwrc-deb
# Copy from Alpine build and rename to gwrc-alp
COPY --from=builderAlpine /app/target/release/router-gui /output/gwrc-alp

# Create volume mount point
VOLUME /host-output

# Command to copy builds to host mount
CMD cp -v /output/gwrc-deb /host-output/ && cp -v /output/gwrc-alp /host-output/
