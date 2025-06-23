#!/bin/bash
# QuDAG startup script - selects binary based on NODE_TYPE environment variable

set -e

# Default to node type if not specified
NODE_TYPE="${NODE_TYPE:-node}"

# Function to handle signals
trap_handler() {
    echo "Received shutdown signal..."
    exit 0
}

# Set up signal handlers
trap trap_handler SIGTERM SIGINT

echo "Starting QuDAG with NODE_TYPE: $NODE_TYPE"

case "$NODE_TYPE" in
    "exchange")
        echo "Starting QuDAG Exchange Server..."
        exec qudag-exchange-server "$@"
        ;;
    "node"|"bootstrap"|*)
        echo "Starting QuDAG Node..."
        exec qudag "$@"
        ;;
esac