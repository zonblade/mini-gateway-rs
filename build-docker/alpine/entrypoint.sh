#!/bin/bash
set -e

# Prepare OpenRC for container environment
if [ ! -d /run/openrc ]; then
    mkdir -p /run/openrc
    touch /run/openrc/softlevel
fi

# Setup necessary directories for OpenRC
mkdir -p /var/tmp/gwrs/log
mkdir -p /run
mkdir -p /var/run

# Prevent init scripts from running during install
echo 'rc_provide="loopback net"' >> /etc/rc.conf

# Setup basic mounts expected by OpenRC
mount -t proc none /proc
mount -t sysfs none /sys
mount -t tmpfs none /run

# Start OpenRC
if [ "$1" = "/sbin/init" ]; then
    # Start services directly
    /sbin/rc-service gwrs-core start
    /sbin/rc-service gwrs-api start
    
    # Keep container running
    echo "Services started. Container is now running..."
    exec tail -f /var/tmp/gwrs/log/core.log /var/tmp/gwrs/log/api.log
else
    # Run command as specified
    exec "$@"
fi