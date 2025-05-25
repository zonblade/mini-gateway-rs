#!/bin/bash
set -e

# Simple configuration
LOG_DIR="/tmp/gwrs/log"
PID_DIR="/tmp/gwrs/pids"
CHECK_INTERVAL=5

# Create directories
mkdir -p "$LOG_DIR" "$PID_DIR"

# Logging
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_DIR/manager.log"
}

# Start core service
start_core() {
    log "Starting router-core..."
    nohup /usr/local/bin/router-core > "$LOG_DIR/core.log" 2> "$LOG_DIR/core.error" &
    echo $! > "$PID_DIR/core.pid"
    log "router-core started (PID: $!)"
}

# Start API service
start_api() {
    log "Starting router-api..."
    nohup /usr/local/bin/router-api > "$LOG_DIR/api.log" 2> "$LOG_DIR/api.error" &
    echo $! > "$PID_DIR/api.pid"
    log "router-api started (PID: $!)"
}

# Stop a service
stop_service() {
    local service=$1
    local pid_file="$PID_DIR/${service}.pid"
    
    if [ -f "$pid_file" ]; then
        local pid=$(cat "$pid_file")
        if kill -0 $pid 2>/dev/null; then
            log "Stopping $service (PID: $pid)"
            kill $pid
            sleep 2
            # Force kill if still running
            if kill -0 $pid 2>/dev/null; then
                kill -9 $pid
            fi
        fi
        rm -f "$pid_file"
    fi
}

# Check if service is running
is_running() {
    local service=$1
    local pid_file="$PID_DIR/${service}.pid"
    
    if [ -f "$pid_file" ]; then
        local pid=$(cat "$pid_file")
        if kill -0 $pid 2>/dev/null; then
            return 0  # Running
        else
            rm -f "$pid_file"  # Clean up stale PID
        fi
    fi
    return 1  # Not running
}

# Restart core (and then API)
restart_core() {
    log "Core is down! Restarting core and API..."
    
    # Stop both services
    stop_service "api"
    stop_service "core"
    
    sleep 2
    
    # Start core first
    start_core
    sleep 3
    
    # Then start API
    start_api
}

# Restart API only
restart_api() {
    log "API is down! Restarting API..."
    stop_service "api"
    sleep 2
    start_api
}

# Monitor services
monitor() {
    log "Starting service monitor..."
    
    while true; do
        # Check core
        if ! is_running "core"; then
            restart_core
        # Check API (only if core is running)
        elif ! is_running "api"; then
            restart_api
        fi
        
        sleep $CHECK_INTERVAL
    done
}

# Cleanup on exit
cleanup() {
    log "Shutting down services..."
    stop_service "api"
    stop_service "core"
    exit 0
}

# Handle signals
trap cleanup SIGTERM SIGINT

# Main execution
log "=== Router Process Manager Starting ==="

# Start both services
start_core
sleep 3  # Give core time to start
start_api

# Monitor forever
monitor