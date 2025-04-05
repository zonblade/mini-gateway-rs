#!/bin/bash

# GWRS Mini-Gateway APT Repository Setup Script

set -e

# Check if running as root
if [ "$EUID" -ne 0 ]; then
  echo "Please run as root or using sudo"
  exit 1
fi

echo "=================================================="
echo "GWRS Mini-Gateway APT Repository Setup"
echo "=================================================="

# Install dependencies
apt-get update
apt-get install -y apt-transport-https ca-certificates curl gnupg

# Add the Gateway.rs GPG key 
echo "Adding Gateway.rs GPG key..."
curl -fsSL https://gateway.rs/gpg-key.asc | gpg --dearmor -o /usr/share/keyrings/gateway-rs-archive-keyring.gpg

# Add the repository to APT sources
echo "deb [signed-by=/usr/share/keyrings/gateway-rs-archive-keyring.gpg] https://gateway.rs/apt stable main" | tee /etc/apt/sources.list.d/gateway-rs.list > /dev/null

# Update package lists
apt-get update

echo "Repository setup complete."
echo "You can now install GWRS Mini-Gateway packages using:"
echo "  sudo apt install gateway"
echo ""
echo "This will install both router-core and router-api components."

# Optional: Install the package
read -p "Do you want to install GWRS Mini-Gateway now? (y/N): " install_now
if [[ "$install_now" =~ ^[Yy]$ ]]; then
  echo "Installing GWRS Mini-Gateway..."
  apt-get install -y gateway
  
  # Configure host and port
  read -p "Enter host IP address [0.0.0.0]: " host
  host=${host:-"0.0.0.0"}
  
  read -p "Enter port number [24042]: " port
  port=${port:-24042}
  
  # Set configuration
  echo "Configuring with:"
  echo "  - Host: ${host}"
  echo "  - Port: ${port}"
  
  # Update configuration files
  sed -i "s/HOST=.*/HOST=${host}/" /opt/gwrs/conf/core.conf
  sed -i "s/PORT=.*/PORT=${port}/" /opt/gwrs/conf/core.conf
  
  sed -i "s/HOST=.*/HOST=${host}/" /opt/gwrs/conf/api.conf
  sed -i "s/PORT=.*/PORT=$((port + 1))/" /opt/gwrs/conf/api.conf
  sed -i "s/CORE_HOST=.*/CORE_HOST=${host}/" /opt/gwrs/conf/api.conf
  sed -i "s/CORE_PORT=.*/CORE_PORT=${port}/" /opt/gwrs/conf/api.conf
  
  # Restart services
  systemctl restart gwrs-core.service
  systemctl restart gwrs-api.service
  
  echo "Installation complete! GWRS Mini-Gateway is now running."
fi

exit 0