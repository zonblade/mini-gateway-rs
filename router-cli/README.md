# Gateway Router CLI Tool

A command-line interface for interacting with the Mini-Gateway Router API. This tool allows you to upload configuration files to the router service.

## Installation

```bash
# Build the CLI tool
cargo build --release
# The binary will be available at target/release/router-cli
```

## Usage

### Initialize Configuration

Create a new configuration file with default template:

```bash
# Create in current directory
gwrs init

# Create in specific directory
gwrs init /path/to/directory
```

### Upload Configuration

Upload a YAML configuration file to the router. You can use either the direct flag or the config subcommand:

```bash
# Using direct flag
gwrs --config config.yaml -u USERNAME -p PASSWORD

# Using config subcommand
gwrs config config.yaml -u USERNAME -p PASSWORD

# Using environment variables
export GWRS_USER=admin
export GWRS_PASS=password
gwrs --osenv --config config.yaml
# or
gwrs config config.yaml --osenv

# Specify custom API URL
gwrs --config config.yaml -u USERNAME -p PASSWORD --url http://router-api:3000
# or
gwrs config config.yaml -u USERNAME -p PASSWORD --url http://router-api:3000
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

## Global Options

These options can be used with any command:

- `-u, --user`: Username for API authentication
- `-p, --pass`: Password for API authentication
- `--osenv`: Use credentials from environment variables
- `--url`: API base URL (default: http://localhost:24042)

## Commands

### init [LOCATION]
Initialize a new configuration file. If LOCATION is not specified, creates in the current directory.

### config CONFIG
Upload a configuration file to the router. CONFIG is the path to your configuration file.

## Examples

```bash
# Initialize new configuration
gwrs init
gwrs init /path/to/directory

# Upload configuration (direct flag)
gwrs --config my-gateway-config.yaml -u admin -p password

# Upload configuration (subcommand)
gwrs config my-gateway-config.yaml -u admin -p password

# Use environment variables for credentials
gwrs --osenv --config my-gateway-config.yaml
# or
gwrs config my-gateway-config.yaml --osenv

# Specify custom API URL
gwrs --config my-gateway-config.yaml -u admin -p password --url http://router-api:8080
# or
gwrs config my-gateway-config.yaml -u admin -p password --url http://router-api:8080
```

## Debug Logging

To enable debug logging, set the `RUST_LOG` environment variable:

```bash
export RUST_LOG=info
# or for more detailed logs
export RUST_LOG=debug
```
