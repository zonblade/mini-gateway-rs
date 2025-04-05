# GWRS Mini-Gateway APT Installation

This directory contains files for installing GWRS Mini-Gateway components as Debian/Ubuntu packages via APT.

## Installation

### Quick Install

```bash
# Add the repository and install
curl -s https://raw.githubusercontent.com/zonblade/mini-gateway-rs/main/build-apt/install.sh | sudo bash
```

### Manual Installation

1. Download the installation script:
   ```bash
   wget https://raw.githubusercontent.com/zonblade/mini-gateway-rs/main/build-apt/install.sh
   ```

2. Make it executable:
   ```bash
   chmod +x install.sh
   ```

3. Run the installation:
   ```bash
   sudo ./install.sh
   ```

## Configuration Options

You can configure the IP address and port during installation:

- Interactive mode: You'll be prompted to enter values
- Non-interactive mode: Use environment variables or command line arguments

### Environment Variables

```bash
export GWRS_HOST=192.168.1.100  # Default: 0.0.0.0
export GWRS_PORT=8080           # Default: 24042
sudo ./install.sh
```

### Command Line Arguments

```bash
sudo ./install.sh --host 192.168.1.100 --port 8080
```

## Managing Services

After installation, the services can be managed with systemctl:

```bash
# Start services
sudo systemctl start gwrs-core
sudo systemctl start gwrs-api

# Stop services
sudo systemctl stop gwrs-core
sudo systemctl stop gwrs-api

# Check status
sudo systemctl status gwrs-core
sudo systemctl status gwrs-api

# Enable at boot
sudo systemctl enable gwrs-core
sudo systemctl enable gwrs-api
```

## Uninstallation

```bash
sudo apt remove gateway
```