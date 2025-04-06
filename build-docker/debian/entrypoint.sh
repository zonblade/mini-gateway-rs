#!/bin/bash
set -e

# Prepare systemd for container environment
if [ ! -d /run/systemd/system ]; then
    mkdir -p /run/systemd/system
fi

# Check for the correct init path
INIT_PATH=""
for path in /sbin/init /lib/systemd/systemd /usr/lib/systemd/systemd /bin/systemd; do
    if [ -x "$path" ]; then
        INIT_PATH="$path"
        break
    fi
done

# If we're supposed to run init but couldn't find it
if [ "$1" = "/sbin/init" ] && [ -z "$INIT_PATH" ]; then
    echo "Error: Could not find systemd init binary. Please check your installation."
    exit 1
elif [ "$1" = "/sbin/init" ] && [ -n "$INIT_PATH" ]; then
    # Use the found init path instead of the specified one
    shift
    exec "$INIT_PATH" "$@"
else
    # Run command as specified
    exec "$@"
fi