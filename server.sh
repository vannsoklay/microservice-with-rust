#!/bin/bash

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

ROOT_DIR="src/services"
ROOT_DIR_SERVICE_BUILD="target/release"

declare -A SERVICES
SERVICES["gateway"]="${ROOT_DIR_SERVICE_BUILD}/gateway"
SERVICES["user"]="${ROOT_DIR}/user/${ROOT_DIR_SERVICE_BUILD}/user"
SERVICES["authentication"]="${ROOT_DIR}/authentication/${ROOT_DIR_SERVICE_BUILD}/authentication"
SERVICES["post"]="${ROOT_DIR}/post/${ROOT_DIR_SERVICE_BUILD}/post"
SERVICES["comment"]="${ROOT_DIR}/comment/${ROOT_DIR_SERVICE_BUILD}/comment"
SERVICES["vote"]="${ROOT_DIR}/vote/${ROOT_DIR_SERVICE_BUILD}/vote"
SERVICES["follow"]="${ROOT_DIR}/follow/${ROOT_DIR_SERVICE_BUILD}/follow"
SERVICES["accommodation"]="${ROOT_DIR}/accommodation/${ROOT_DIR_SERVICE_BUILD}/accommodation"
SERVICES["booking"]="${ROOT_DIR}/booking/${ROOT_DIR_SERVICE_BUILD}/booking"
# SERVICES["storage"]="${ROOT_DIR}/storage/${ROOT_DIR_SERVICE_BUILD}/storage"

LOG_DIR="$HOME/log/my_services"
mkdir -p "$LOG_DIR"

# Function to print error in red
print_error() {
    echo -e "${RED}$1${NC}"
}

print_success() {
    echo -e "${GREEN}$1${NC}"
}

# Function to copy Linux build binaries into service root dirs
copy_binaries() {
    for SERVICE in "${!SERVICES[@]}"; do
        SERVICE_BUILD_PATH="${SERVICES[$SERVICE]}"
        SERVICE_DIR="$(dirname "$SERVICE_BUILD_PATH")"
        SERVICE_BIN_NAME="$(basename "$SERVICE_BUILD_PATH")"
        DEST_DIR="$SERVICE_DIR"
        DEST_PATH="${DEST_DIR}/${SERVICE_BIN_NAME}"

        mkdir -p "$DEST_DIR"

        if [ -f "$SERVICE_BUILD_PATH" ]; then
            cp "$SERVICE_BUILD_PATH" "$DEST_PATH"
            chmod +x "$DEST_PATH"
            print_success "Copied $SERVICE_BIN_NAME to $DEST_PATH"
        else
            print_error "Warning: $SERVICE_BUILD_PATH not found!"
        fi
    done
}

start_services() {
    copy_binaries
    for SERVICE in "${!SERVICES[@]}"; do
        SERVICE_PATH="${SERVICES[$SERVICE]}"
        SERVICE_DIR="$(dirname "$SERVICE_PATH")"
        ENV_FILE="${SERVICE_DIR}/.env"
        echo "Looking for env file at $ENV_FILE"

        if [ -f "$SERVICE_PATH" ]; then
            chmod +x "$SERVICE_PATH"

            echo "Starting $SERVICE..."

            # Load .env if it exists
            if [ -f "$ENV_FILE" ]; then
                echo "Loading environment from $ENV_FILE"
                set -a
                source "$ENV_FILE"
                set +a
            fi

            nohup "$SERVICE_PATH" > "$LOG_DIR/$SERVICE.log" 2>&1 &
            echo $! > "/tmp/$SERVICE.pid"
        else
            print_error "Error: $SERVICE_PATH not found!"
        fi
    done
    print_success "All services started."
}

stop_services() {
    for SERVICE in "${!SERVICES[@]}"; do
        echo "Stopping $SERVICE..."
        if [ -f "/tmp/$SERVICE.pid" ]; then
            kill "$(cat /tmp/$SERVICE.pid)" && rm "/tmp/$SERVICE.pid"
        else
            print_error "$SERVICE is not running."
        fi
    done
    print_success "All services stopped."
}

remove_pid() {
    SERVICE_NAME="$1"
    if [ -z "$SERVICE_NAME" ]; then
        print_error "Please specify a service name."
        exit 1
    fi

    if [ -f "/tmp/$SERVICE_NAME.pid" ]; then
        rm "/tmp/$SERVICE_NAME.pid"
        print_success "Removed PID file for $SERVICE_NAME."
    else
        print_error "PID file for $SERVICE_NAME does not exist."
    fi
}

restart_services() {
    stop_services
    start_services
}

status_services() {
    for SERVICE in "${!SERVICES[@]}"; do
        if [ -f "/tmp/$SERVICE.pid" ] && ps -p "$(cat /tmp/$SERVICE.pid)" > /dev/null; then
            print_success "$SERVICE is running (PID: $(cat /tmp/$SERVICE.pid))"
        else
            print_error "$SERVICE is not running."
        fi
    done
}

usage() {
    echo "Usage: $0 {start|stop|restart|status|remove-pid <service-name>|copy-binaries}"
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
    copy-binaries)
        copy_binaries
        ;;
    *)
        usage
        exit 1
        ;;
esac