#!/bin/bash
# Engram MCP Server startup script

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"

HOST="${HOST:-127.0.0.1}"
PORT="${PORT:-8080}"
DB_PATH="${DB_PATH:-$REPO_ROOT/.vibe/engram-memory/memory.db}"

# Check if stop command
if [ "$1" = "stop" ]; then
    echo "Stopping Engram MCP Server..."
    # Try multiple methods to find the PID
    PID=""
    
    # Method 1: Use ss (available on most Linux systems)
    if command -v ss &>/dev/null; then
        PID=$(ss -tlnp | grep ":$PORT " | grep -oP 'pid=\K[0-9]+' | head -1)
    fi
    
    # Method 2: Use netstat
    if [ -z "$PID" ] && command -v netstat &>/dev/null; then
        PID=$(netstat -tlnp 2>/dev/null | grep ":$PORT " | awk '{print $7}' | cut -d'/' -f1 | head -1)
    fi
    
    # Method 3: Use ps with port grep
    if [ -z "$PID" ]; then
        PID=$(ps aux | grep "start_server.py.*--port $PORT" | grep -v grep | awk '{print $2}' | head -1)
    fi
    
    if [ -n "$PID" ]; then
        echo "Found server on port $PORT (PID: $PID)"
        kill $PID 2>/dev/null
        sleep 1
        if ps -p $PID > /dev/null 2>&1; then
            echo "Server still running, force killing..."
            kill -9 $PID 2>/dev/null
            sleep 1
        fi
        echo "✅ Server stopped"
    else
        echo "No server found on port $PORT"
    fi
    exit 0
fi

# Create log directory
mkdir -p "$REPO_ROOT/.vibe/engram-memory/logs"
LOG_FILE="$REPO_ROOT/.vibe/engram-memory/logs/server_$(date +%Y%m%d_%H%M%S).log"

# Kill any existing server on the same port
PID=""
if command -v ss &>/dev/null; then
    PID=$(ss -tlnp | grep ":$PORT " | grep -oP 'pid=\K[0-9]+' | head -1)
elif command -v netstat &>/dev/null; then
    PID=$(netstat -tlnp 2>/dev/null | grep ":$PORT " | awk '{print $7}' | cut -d'/' -f1 | head -1)
else
    PID=$(ps aux | grep "start_server.py.*--port $PORT" | grep -v grep | awk '{print $2}' | head -1)
fi

if [ -n "$PID" ]; then
    echo "Killing existing server on port $PORT (PID: $PID)"
    kill $PID 2>/dev/null
    sleep 1
    if ps -p $PID > /dev/null 2>&1; then
        kill -9 $PID 2>/dev/null
        sleep 1
    fi
fi

echo "Starting Engram MCP Server..."
echo "  Host: $HOST"
echo "  Port: $PORT"
echo "  Database: $DB_PATH"
echo "  Log: $LOG_FILE"
echo ""

# Start the server in the background
cd "$SCRIPT_DIR"
nohup python3 start_server.py --host "$HOST" --port "$PORT" --db "$DB_PATH" >> "$LOG_FILE" 2>&1 &

# Get the PID
SERVER_PID=$!
echo "  PID: $SERVER_PID"
echo ""

# Wait for server to start
sleep 2

# Check if server is running
if ps -p $SERVER_PID > /dev/null; then
    echo "✅ Engram MCP Server started successfully"
    echo ""
    echo "Server is running on: http://$HOST:$PORT"
    echo ""
    echo "Available endpoints:"
    echo "  GET  /memory/list              - List all memories"
    echo "  GET  /memory/search?type=X&limit=N  - Search memories"
    echo "  GET  /memory/<id>               - Retrieve memory"
    echo "  POST /memory/store             - Store new memory"
    echo ""
    echo "To stop the server:"
    echo "  ./engram_server.sh stop"
    echo ""
    echo "To view logs:"
    echo "  tail -f $LOG_FILE"
else
    echo "❌ Failed to start server"
    echo "Check log file: $LOG_FILE"
    tail -20 "$LOG_FILE"
    exit 1
fi
