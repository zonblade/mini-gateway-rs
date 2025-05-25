# Gateway Router CLI Tool

A command-line interface for interacting with the Mini-Gateway Router API. This tool allows you to upload configuration files to the router service.

## Installation

```bash
# Build the CLI tool
cargo build --release
# The binary will be available at target/release/router-cli
```

## Usage

### Upload Configuration

Upload a YAML configuration file to the router:

```bash
# Using username and password
gwrs -u USERNAME -p PASSWORD --config config.yaml

# Using environment variables
export GWRS_USER=admin
export GWRS_PASS=password
gwrs --osenv --config config.yaml

# Specify custom API URL
gwrs -u USERNAME -p PASSWORD --api-url http://router-api:3000 --config config.yaml
```

### Configuration File Format

The configuration file should be in YAML format with the following structure:

```yaml
proxy:
  - name: "proxy1"
    listen: "127.0.0.1:8080"
    domains:
      - domain: "example.com"
        tls: false
        tls_cert: |
          -----BEGIN CERTIFICATE-----
          cert
          -----END CERTIFICATE-----
        tls_key: |
          -----BEGIN PRIVATE KEY-----
          key
          -----END PRIVATE KEY-----
    highspeed:
      enabled: true
      target: "gateway1"
    gateway:
      - name: "gateway1"
        domain: "example.com"
        target: "127.0.0.1:8080"
        path:
          - priority: 1
            pattern: "^(.*)$"
            target: "/$1"
```

## Environment Variables

- `GWRS_USER`: Username for API authentication
- `GWRS_PASS`: Password for API authentication

## Options

- `-u, --user`: Username for API authentication
- `-p, --pass`: Password for API authentication
- `--osenv`: Use credentials from environment variables
- `--api-url`: API base URL (default: http://localhost:3000)
- `--config`: Path to the configuration file

## Examples

```bash
# Basic usage
gwrs -u admin -p password --config my-gateway-config.yaml

# Use environment variables for credentials
gwrs --osenv --config my-gateway-config.yaml

# Specify custom API URL
gwrs -u admin -p password --api-url http://router-api:8080 --config my-gateway-config.yaml
```

## Debug Logging

To enable debug logging, set the `RUST_LOG` environment variable:

```bash
export RUST_LOG=info
# or for more detailed logs
export RUST_LOG=debug
```
