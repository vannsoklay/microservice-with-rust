#!/bin/bash

# Define the root directory where all services are located
ROOT_DIR="src/services"
ROOT_DIR_SERVICE="target/release"

# Define services and their locations
declare -A SERVICES
SERVICES["gateway"]="${ROOT_DIR_SERVICE}/gateway.exe"
SERVICES["user"]="${ROOT_DIR}/user/${ROOT_DIR_SERVICE}/user.exe"
SERVICES["authentication"]="${ROOT_DIR}/authentication/${ROOT_DIR_SERVICE}/authentication.exe"
SERVICES["post"]="${ROOT_DIR}/post/${ROOT_DIR_SERVICE}/post.exe"
SERVICES["accommodation"]="${ROOT_DIR}/accommodation/${ROOT_DIR_SERVICE}/accommodation.exe"
SERVICES["booking"]="${ROOT_DIR}/booking/${ROOT_DIR_SERVICE}/accommodation.exe"
# SERVICES["storage"]="${ROOT_DIR}/storage/${ROOT_DIR_SERVICE}/storage.exe"

LOG_DIR="$HOME/log/my_services"
mkdir -p "$LOG_DIR"  # Ensure log directory exists

start_services() {
    for SERVICE in "${!SERVICES[@]}"; do
        SERVICE_PATH="${SERVICES[$SERVICE]}"
        SERVICE_DIR="$(dirname "$SERVICE_PATH")"
        ENV_FILE="${SERVICE_DIR}/.env"
        echo "env $ENV_FILE"

        if [ -f "$SERVICE_PATH" ]; then
            chmod +x "$SERVICE_PATH"

            echo "Starting $SERVICE..."

            # Load and export .env if it exists
            if [ -f "$ENV_FILE" ]; then
                echo "Loading environment from $ENV_FILE"
                export $(grep -v '^#' "$ENV_FILE" | xargs)
            fi

            nohup "$SERVICE_PATH" > "$LOG_DIR/$SERVICE.log" 2>&1 &
            echo $! > "/tmp/$SERVICE.pid"
        else
            echo "Error: $SERVICE_PATH not found!"
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

remove_pid() {
    SERVICE_NAME="$1"
    if [ -z "$SERVICE_NAME" ]; then
        echo "Please specify a service name."
        exit 1
    fi

    if [ -f "/tmp/$SERVICE_NAME.pid" ]; then
        rm "/tmp/$SERVICE_NAME.pid"
        echo "Removed PID file for $SERVICE_NAME."
    else
        echo "PID file for $SERVICE_NAME does not exist."
    fi
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
    remove-pid)
        remove_pid "$2"
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|status|remove-pid <service-name>}"
        exit 1
        ;;
esac
