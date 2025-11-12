#!/usr/bin/env python3
"""Example: Starting the ZeroMQ backend server

This example demonstrates how to start the backend server
and handle generation requests.

Usage:
    python3 examples/python_server_example.py
"""

import sys
import os

# Add python directory to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "../python"))

from workers.zmq_server import ZmqServer


def main():
    print("=" * 60)
    print("DGX-Pixels Backend Server Example")
    print("=" * 60)
    print()
    print("This server will:")
    print("  1. Listen for generation requests on tcp://127.0.0.1:5555")
    print("  2. Publish progress updates on tcp://127.0.0.1:5556")
    print("  3. Queue jobs and respond with estimated time")
    print()
    print("Press Ctrl+C to stop")
    print("=" * 60)
    print()

    # Create and start server with default addresses
    server = ZmqServer()
    server.start()


if __name__ == "__main__":
    main()
