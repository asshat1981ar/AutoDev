#!/usr/bin/env python3
"""Start script for Engram MCP Server.

Usage:
    python start_server.py [--host HOST] [--port PORT] [--db DB_PATH]

    Default:
        HOST: 127.0.0.1
        PORT: 8080
        DB_PATH: .vibe/engram-memory/memory.db
"""

import argparse
import sys
import time

# Add parent directory to path for imports
sys.path.insert(0, sys.path[0] + '/../..')

from engram_mcp import EngramMemory


def main():
    parser = argparse.ArgumentParser(description='Start Engram MCP Server')
    parser.add_argument('--host', default='127.0.0.1', help='Host to bind to')
    parser.add_argument('--port', type=int, default=8080, help='Port to listen on')
    parser.add_argument('--db', default='.vibe/engram-memory/memory.db', 
                        help='Database path')
    args = parser.parse_args()

    print(f'Starting Engram MCP Server...')
    print(f'  Host: {args.host}')
    print(f'  Port: {args.port}')
    print(f'  Database: {args.db}')
    print()

    # Create and start the server
    engram = EngramMemory(db_path=args.db)
    engram.start_server(host=args.host, port=args.port)

    print('✅ Engram MCP Server started successfully')
    print()
    print('Endpoints:')
    print('  GET  http://{host}:{port}/memory/list              - List all memories')
    print('  GET  http://{host}:{port}/memory/search?type=X&limit=N  - Search memories')
    print('  GET  http://{host}:{port}/memory/<id>               - Retrieve memory')
    print('  POST http://{host}:{port}/memory/store             - Store new memory')
    print()
    print('Database location:', engram.db_path)
    print()
    print('Server running. Press Ctrl+C to stop.')
    print()

    # Keep the main thread alive
    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        print()
        print('Shutting down...')
        engram.stop_server()
        print('✅ Server stopped')


if __name__ == '__main__':
    main()
