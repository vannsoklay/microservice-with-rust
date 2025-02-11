#!/bin/bash

# Define the root directory where all services are located
ROOT_DIR="src/services"
ROOT_DIR_SERVICE="target/release"

# Define services and their locations
declare -A SERVICES
SERVICES["gateway"]="${ROOT_DIR_SERVICE}/root-microservice.exe"
SERVICES["service1"]="${ROOT_DIR}/user/${ROOT_DIR_SERVICE}/user.exe"
SERVICES["service2"]="${ROOT_DIR}/accommodation/${ROOT_DIR_SERVICE}/accommodation.exe"
# SERVICES["service3"]="${ROOT_DIR}/storage/${ROOT_DIR_SERVICE}/storage.exe"

LOG_DIR="$HOME/log/my_services"
mkdir -p "$LOG_DIR"  # Ensure log directory exists

start_services() {
    for SERVICE in "${!SERVICES[@]}"; do
        if [ -f "${SERVICES[$SERVICE]}" ]; then  # Check if the file exists
            chmod +x "${SERVICES[$SERVICE]}"    # Set execute permissions
            echo "Starting $SERVICE..."
            nohup "${SERVICES[$SERVICE]}" > "$LOG_DIR/$SERVICE.log" 2>&1 &
            echo $! > "/tmp/$SERVICE.pid"  # Save process ID
        else
            echo "Error: ${SERVICES[$SERVICE]} not found!"
        fi
    done
    echo "All services started."
}


stop_services() {
    for SERVICE in "${!SERVICES[@]}"; do
        echo "Stopping $SERVICE..."
        if [ -f "/tmp/$SERVICE.pid" ]; then
            kill "$(cat /tmp/$SERVICE.pid)" && rm "/tmp/$SERVICE.pid"
        else
            echo "$SERVICE is not running."
        fi
    done
    echo "All services stopped."
}

restart_services() {
    stop_services
    start_services
}

status_services() {
    for SERVICE in "${!SERVICES[@]}"; do
        if [ -f "/tmp/$SERVICE.pid" ] && ps -p "$(cat /tmp/$SERVICE.pid)" > /dev/null; then
            echo "$SERVICE is running (PID: $(cat /tmp/$SERVICE.pid))"
        else
            echo "$SERVICE is not running."
        fi
    done
}

case "$1" in
    start)
        start_services
        ;;
    stop)
        stop_services
        ;;
    restart)
        restart_services
        ;;
    status)
        status_services
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|status}"
        exit 1
        ;;
esac
