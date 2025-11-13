"""FastMCP Server for DGX-Pixels

This module provides an MCP (Model Context Protocol) server that enables
Bevy game engine to communicate with the DGX-Pixels AI sprite generation backend.

Features:
- Generate single sprites via MCP tools
- Generate batches of sprites
- Deploy generated sprites to Bevy assets directory
- Integration with Python backend worker
- Support for stdio and SSE transports
"""

# Note: We don't import server.mcp at package level to avoid requiring
# fastmcp for all imports. Import directly when needed:
#
#   from python.mcp_server.server import mcp, start_server

__all__ = ["config_loader", "backend_client", "tools"]
