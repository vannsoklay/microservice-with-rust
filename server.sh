#!/bin/bash
#
# Note: intentionally avoids `declare -A` (associative arrays) so this also
# works on macOS's stock /bin/bash (3.2, no associative array support), not
# just Linux/newer bash. Services are tracked as parallel indexed arrays
# instead, keyed by position.

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

ROOT_DIR="src/services"
ROOT_DIR_SERVICE_BUILD="target/release"

# Services started by start/stop/restart/status, their release binary path,
# and the directory holding their .env (the crate root, NOT the build output
# dir — those differ, which used to make start_services look for .env in
# target/release/.env and silently load nothing for every non-gateway service).
SERVICE_NAMES=(gateway user authentication post comment vote follow)
SERVICE_PATHS=(
    "${ROOT_DIR_SERVICE_BUILD}/gateway"
    "${ROOT_DIR}/user/${ROOT_DIR_SERVICE_BUILD}/user"
    "${ROOT_DIR}/authentication/${ROOT_DIR_SERVICE_BUILD}/authentication"
    "${ROOT_DIR}/post/${ROOT_DIR_SERVICE_BUILD}/post"
    "${ROOT_DIR}/comment/${ROOT_DIR_SERVICE_BUILD}/comment"
    "${ROOT_DIR}/vote/${ROOT_DIR_SERVICE_BUILD}/vote"
    "${ROOT_DIR}/follow/${ROOT_DIR_SERVICE_BUILD}/follow"
)
SERVICE_ENV_DIRS=(
    "."
    "${ROOT_DIR}/user"
    "${ROOT_DIR}/authentication"
    "${ROOT_DIR}/post"
    "${ROOT_DIR}/comment"
    "${ROOT_DIR}/vote"
    "${ROOT_DIR}/follow"
)
# "storage" is deliberately excluded from start/stop: it isn't routed through
# the gateway today. It's still built by `build`/`dev` below.

# Crates to build for `build`/`dev`: same order as SERVICE_NAMES, plus storage.
CRATE_NAMES=(gateway user authentication post comment vote follow storage)
CRATE_DIRS=(
    "."
    "src/services/user"
    "src/services/authentication"
    "src/services/post"
    "src/services/comment"
    "src/services/vote"
    "src/services/follow"
    "src/services/storage"
)

LOG_DIR="$HOME/log/my_services"
mkdir -p "$LOG_DIR"

# Function to print error in red
print_error() {
    echo -e "${RED}$1${NC}"
}

print_success() {
    echo -e "${GREEN}$1${NC}"
}

# Build every crate in release mode so start_services finds fresh binaries
build_services() {
    for i in "${!CRATE_NAMES[@]}"; do
        CRATE="${CRATE_NAMES[$i]}"
        CRATE_DIR="${CRATE_DIRS[$i]}"
        echo "Building $CRATE ($CRATE_DIR)..."
        if (cd "$CRATE_DIR" && cargo build --release); then
            print_success "Built $CRATE"
        else
            print_error "Build failed for $CRATE"
            exit 1
        fi
    done
    print_success "All crates built."
}

# Function to copy Linux build binaries into service root dirs
copy_binaries() {
    for i in "${!SERVICE_NAMES[@]}"; do
        SERVICE="${SERVICE_NAMES[$i]}"
        SERVICE_BUILD_PATH="${SERVICE_PATHS[$i]}"
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
    for i in "${!SERVICE_NAMES[@]}"; do
        SERVICE="${SERVICE_NAMES[$i]}"
        SERVICE_PATH="${SERVICE_PATHS[$i]}"
        ENV_FILE="${SERVICE_ENV_DIRS[$i]}/.env"
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
    for SERVICE in "${SERVICE_NAMES[@]}"; do
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
    for SERVICE in "${SERVICE_NAMES[@]}"; do
        if [ -f "/tmp/$SERVICE.pid" ] && ps -p "$(cat /tmp/$SERVICE.pid)" > /dev/null; then
            print_success "$SERVICE is running (PID: $(cat /tmp/$SERVICE.pid))"
        else
            print_error "$SERVICE is not running."
        fi
    done
}

usage() {
    echo "Usage: $0 {dev|build|start|stop|restart|status|remove-pid <service-name>|copy-binaries}"
    echo "  dev            Build every service and start them all (one-command local run)"
    echo "  build          Build every service in release mode, don't start anything"
    echo "  start          Start all services from existing release binaries"
    echo "  stop           Stop all running services"
    echo "  restart        Stop then start all services"
    echo "  status         Show which services are running"
    echo "  remove-pid     Remove a stale PID file for one service"
    echo "  copy-binaries  Copy built binaries into their service directories"
}

case "$1" in
    dev)
        build_services
        start_services
        ;;
    build)
        build_services
        ;;
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
