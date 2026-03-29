# Gateway Router CLI Tool

A command-line interface for managing and monitoring the Mini-Gateway Router. Supports configuration management, full CRUD operations for all resources, and a real-time TUI monitoring dashboard.

## Installation

```bash
# Build the CLI tool
cargo build --release -p router-cli
# The binary will be available at target/release/router-cli
```

## Authentication

All commands (except `init`) require authentication. Provide credentials via:

```bash
# Command-line flags
gwrs proxy list -u USERNAME -p PASSWORD

# Environment variables
export GWRS_USER=admin
export GWRS_PASS=password
gwrs proxy list --osenv
```

Tokens are cached to `~/.config/gwrs/token.json` (60 min expiry). Use `--no-cache` to force fresh authentication.

## Global Options

| Option | Description |
|--------|-------------|
| `-u, --user` | Username for API authentication |
| `-p, --pass` | Password for API authentication |
| `--osenv` | Use GWRS_USER/GWRS_PASS environment variables |
| `--url` | API base URL (default: `http://localhost:24042`) |
| `--no-cache` | Skip token cache, force fresh login |
| `--format` | Output format: `table` (default), `json`, `yaml` |

## Commands

### Configuration Management

```bash
# Initialize a new configuration template
gwrs init [LOCATION]

# Upload configuration to the router
gwrs config <PATH>

# Export current configuration
gwrs export [OUTPUT]
```

### Proxy Management

```bash
gwrs proxy list
gwrs proxy get <ID>
gwrs proxy create --title "web-proxy" --listen "0.0.0.0:443" [--target "127.0.0.1:8080"] [--high-speed]
gwrs proxy delete <ID> [-y]
```

### Proxy Domain Management

```bash
gwrs domain list [--proxy-id <ID>] [--gwnode-id <ID>]
gwrs domain get <ID>
gwrs domain create --proxy-id <ID> [--tls] [--sni "example.com"]
gwrs domain delete <ID> [-y]
```

### Gateway Node Management

```bash
gwrs gwnode list [--proxy-id <ID>]
gwrs gwnode get <ID>
gwrs gwnode create --proxy-id <ID> --title "node1" --target "127.0.0.1:3000" [--priority 100]
gwrs gwnode delete <ID> [-y]
```

### Gateway Routing Rules

```bash
gwrs gateway list [--gwnode-id <ID>]
gwrs gateway get <ID>
gwrs gateway create --gwnode-id <ID> --pattern "^/api/(.*)$" --target "/v1/$1" --priority 1
gwrs gateway delete <ID> [-y]
```

### User Management (Admin)

```bash
gwrs user list
gwrs user get <ID>
gwrs user create --username "staff1" --email "staff@example.com" --password "..." [--role staff]
gwrs user update <ID> [--username "..."] [--email "..."] [--role admin|staff|user]
gwrs user delete <ID> [-y]
```

### Certificate Management

```bash
gwrs cert generate --domain "example.com" --proxy-id <ID> [--email "admin@example.com"] [--staging]
gwrs cert renew-all
gwrs cert due-for-renewal
```

### Sync

```bash
gwrs sync proxy
gwrs sync gateway
```

### Live Monitoring

```bash
gwrs monitor
```

Opens a real-time TUI dashboard showing:
- Overview panel with aggregate totals and success/error rate
- Per-second rates (req/s, bytes/s) with min/max/avg
- Traffic sparklines (requests, bytes_in, bytes_out) over 30 min history
- Status code distribution bar chart
- Connection status and uptime

Keybindings:
| Key | Action |
|-----|--------|
| `q` / `Ctrl+C` | Quit |
| `Tab` | Switch panel focus |
| `1` | Focus Gateway |
| `2` | Focus Proxy |
| `?` | Toggle help overlay |
| `Esc` | Close help |

## Output Formats

```bash
# Human-readable table (default)
gwrs proxy list

# JSON for scripting
gwrs proxy list --format json

# YAML
gwrs proxy list --format yaml
```

## Testing with Mock Server

A mock API server is included for development and testing:

```bash
cd test-app
npm install
node mock-api.js
# Login: admin / admin
# SSE stats broadcast every 5s
```

Then test commands against it:

```bash
gwrs proxy list -u admin -p admin --url http://localhost:24042
gwrs monitor -u admin -p admin --url http://localhost:24042
```

## Debug Logging

```bash
export RUST_LOG=debug
gwrs proxy list -u admin -p admin
```
