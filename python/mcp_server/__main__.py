"""Entry point for running MCP server as a module

Usage:
    python -m python.mcp_server
"""

from .server import start_server

if __name__ == "__main__":
    start_server()
