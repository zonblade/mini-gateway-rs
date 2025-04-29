#!/bin/bash

# GWRS Mini-Gateway Installation Script
# This script installs GWRS Mini-Gateway components (router-core and router-api)

set -e

# Default configuration
HOST=${GWRS_HOST:-"0.0.0.0"}
PORT=${GWRS_PORT:-24042}
INTERACTIVE=true
SCRIPT_VERSION="0.0.1"
DOWNLOAD_BASE_URL="https://gateway.rs/release/latest"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --host)
      HOST="$2"
      shift 2
      ;;
    --port)
      PORT="$2"
      shift 2
      ;;
    --non-interactive)
      INTERACTIVE=false
      shift
      ;;
    --help)
      echo "Usage: $0 [options]"
      echo "Options:"
      echo "  --host HOST          Set the host IP (default: 0.0.0.0)"
      echo "  --port PORT          Set the port (default: 24042)"
      echo "  --non-interactive    Run in non-interactive mode"
      echo "  --help               Show this help message"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

# Check if running as root
if [ "$EUID" -ne 0 ]; then
  echo "Please run as root or using sudo"
  exit 1
fi

# Banner
echo "=================================================="
echo "GWRS Mini-Gateway Installation Script v${SCRIPT_VERSION}"
echo "=================================================="

# Prompt for configuration if in interactive mode
if [ "$INTERACTIVE" = true ]; then
  read -p "Enter host IP address [${HOST}]: " input_host
  HOST=${input_host:-$HOST}
  
  read -p "Enter port number [${PORT}]: " input_port
  PORT=${input_port:-$PORT}
fi

echo "Installing with configuration:"
echo "  - Host: ${HOST}"
echo "  - Port: ${PORT}"
echo ""

# Install dependencies
echo "Installing dependencies..."
apt-get update
apt-get install -y curl wget gnupg ca-certificates

# Create directory for GWRS
mkdir -p /opt/gwrs/bin
mkdir -p /opt/gwrs/conf
mkdir -p /tmp/gwrs/log

# Download and install binaries
echo "Downloading and installing binaries..."
wget -q "${DOWNLOAD_BASE_URL}/router-core" -O /opt/gwrs/bin/router-core
wget -q "${DOWNLOAD_BASE_URL}/router-api" -O /opt/gwrs/bin/router-api

# Verify download was successful
if [ ! -s /opt/gwrs/bin/router-core ] || [ ! -s /opt/gwrs/bin/router-api ]; then
  echo "Error: Failed to download binaries from ${DOWNLOAD_BASE_URL}"
  echo "Please check your internet connection and try again."
  exit 1
fi

# Set permissions
chmod +x /opt/gwrs/bin/router-core
chmod +x /opt/gwrs/bin/router-api

# Create configuration files
cat > /opt/gwrs/conf/core.conf <<EOF
HOST=${HOST}
PORT=${PORT}
LOG_LEVEL=info
EOF

cat > /opt/gwrs/conf/api.conf <<EOF
HOST=${HOST}
PORT=$((PORT + 1))
CORE_HOST=${HOST}
CORE_PORT=${PORT}
LOG_LEVEL=info
EOF

# Create systemd service files
cat > /etc/systemd/system/gwrs-core.service <<EOF
[Unit]
Description=GWRS Mini-Gateway Core
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=root
Group=root
ExecStart=/opt/gwrs/bin/router-core --config /opt/gwrs/conf/core.conf
Restart=on-failure
RestartSec=5
StandardOutput=append:/tmp/gwrs/log/core.log
StandardError=append:/tmp/gwrs/log/core.error.log

[Install]
WantedBy=multi-user.target
EOF

cat > /etc/systemd/system/gwrs-api.service <<EOF
[Unit]
Description=GWRS Mini-Gateway API
After=network.target gwrs-core.service
Wants=network-online.target gwrs-core.service

[Service]
Type=simple
User=root
Group=root
ExecStart=/opt/gwrs/bin/router-api --config /opt/gwrs/conf/api.conf
Restart=on-failure
RestartSec=5
StandardOutput=append:/tmp/gwrs/log/api.log
StandardError=append:/tmp/gwrs/log/api.error.log

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd and enable services
systemctl daemon-reload
systemctl enable gwrs-core.service
systemctl enable gwrs-api.service

# Start services
echo "Starting services..."
systemctl start gwrs-core.service
systemctl start gwrs-api.service

# Check if services are running
sleep 2
core_status=$(systemctl is-active gwrs-core.service)
api_status=$(systemctl is-active gwrs-api.service)

if [ "$core_status" = "active" ] && [ "$api_status" = "active" ]; then
  echo "Installation successful!"
  echo "GWRS Mini-Gateway is now running."
  echo "  - Core is listening on ${HOST}:${PORT}"
  echo "  - API is listening on ${HOST}:$((PORT + 1))"
  echo ""
  echo "You can manage the services with:"
  echo "  sudo systemctl {start|stop|restart|status} gwrs-core"
  echo "  sudo systemctl {start|stop|restart|status} gwrs-api"
else
  echo "Warning: Services did not start properly."
  echo "Please check the logs in /tmp/gwrs/log/"
  echo "  - Core status: ${core_status}"
  echo "  - API status: ${api_status}"
fi

exit 0