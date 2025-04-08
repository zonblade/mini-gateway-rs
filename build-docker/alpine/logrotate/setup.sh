#!/bin/sh

# Install logrotate if not already installed
if ! command -v logrotate >/dev/null 2>&1; then
    echo "Installing logrotate..."
    apk add --no-cache logrotate
fi

# Create logrotate config directory if it doesn't exist
mkdir -p /etc/logrotate.d

# Copy our logrotate configuration
cp /etc/gwrs/logrotate/gwrs /etc/logrotate.d/

# Create cron job for logrotate if it doesn't exist
if [ ! -f /etc/periodic/daily/logrotate ]; then
    echo "Setting up daily logrotate cron job..."
    echo '#!/bin/sh
/usr/sbin/logrotate /etc/logrotate.conf
EXITVALUE=$?
if [ $EXITVALUE != 0 ]; then
    /usr/bin/logger -t logrotate "ALERT exited abnormally with [$EXITVALUE]"
fi
exit 0' > /etc/periodic/daily/logrotate
    chmod +x /etc/periodic/daily/logrotate
fi

echo "Logrotate configuration for GWRS services has been installed."