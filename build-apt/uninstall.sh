#!/bin/bash

# GWRS Mini-Gateway Uninstallation Script
# This script removes GWRS Mini-Gateway components (router-core and router-api)

set -e

# Check if running as root
if [ "$EUID" -ne 0 ]; then
  echo "Please run as root or using sudo"
  exit 1
fi

echo "=================================================="
echo "GWRS Mini-Gateway Uninstallation"
echo "=================================================="
echo "This will completely remove GWRS Mini-Gateway from your system."
echo "All configuration and log files will be deleted."
echo ""

# Confirm uninstallation
read -p "Are you sure you want to proceed? (y/N): " confirm
if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
  echo "Uninstallation cancelled."
  exit 0
fi

# Stop services
echo "Stopping services..."
systemctl stop gwrs-api.service 2>/dev/null || true
systemctl stop gwrs-core.service 2>/dev/null || true

# Disable services
echo "Disabling services..."
systemctl disable gwrs-api.service 2>/dev/null || true
systemctl disable gwrs-core.service 2>/dev/null || true

# Remove service files
echo "Removing service files..."
rm -f /etc/systemd/system/gwrs-core.service
rm -f /etc/systemd/system/gwrs-api.service
systemctl daemon-reload

# Remove installed files
echo "Removing installed files..."
rm -rf /opt/gwrs

# Remove logs
echo "Removing log files..."
rm -rf /var/tmp/gwrs/log

echo "Uninstallation complete. GWRS Mini-Gateway has been removed from your system."
exit 0