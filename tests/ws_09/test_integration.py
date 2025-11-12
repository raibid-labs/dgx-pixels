"""Integration tests for ZeroMQ IPC between Rust and Python"""

import sys
import os
import time
import threading

# Add python directory to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "../../python"))

import zmq
from workers.message_protocol import (
    GenerateRequest,
    PingRequest,
    StatusRequest,
    ListModelsRequest,
    serialize,
    deserialize_response,
    DEFAULT_REQ_REP_ADDR,
    DEFAULT_PUB_SUB_ADDR,
)
from workers.zmq_server import ZmqServer


def test_server_ping():
    """Test simple ping/pong communication"""
    # Start server in background thread
    server = ZmqServer(
        req_addr="tcp://127.0.0.1:15555", pub_addr="tcp://127.0.0.1:15556"
    )

    def run_server():
        try:
            server.start()
        except:
            pass

    server_thread = threading.Thread(target=run_server, daemon=True)
    server_thread.start()

    # Give server time to bind
    time.sleep(1.5)

    try:
        # Create client
        context = zmq.Context()
        socket = context.socket(zmq.REQ)
        socket.connect("tcp://127.0.0.1:15555")
        socket.setsockopt(zmq.RCVTIMEO, 3000)  # 3 second timeout

        # Send ping
        request = PingRequest()
        socket.send(serialize(request))

        # Receive pong
        response_data = socket.recv()
        response = deserialize_response(response_data)

        assert response.to_dict()["type"] == "pong"

        socket.close()
        context.term()

    finally:
        server.running = False
        time.sleep(0.2)


def test_server_status():
    """Test status request"""
    server = ZmqServer(
        req_addr="tcp://127.0.0.1:15557", pub_addr="tcp://127.0.0.1:15558"
    )

    def run_server():
        try:
            server.start()
        except:
            pass

    server_thread = threading.Thread(target=run_server, daemon=True)
    server_thread.start()
    time.sleep(1.5)

    try:
        context = zmq.Context()
        socket = context.socket(zmq.REQ)
        socket.connect("tcp://127.0.0.1:15557")
        socket.setsockopt(zmq.RCVTIMEO, 3000)

        # Send status request
        request = StatusRequest()
        socket.send(serialize(request))

        # Receive status
        response_data = socket.recv()
        response = deserialize_response(response_data)

        assert response.to_dict()["type"] == "status_info"
        assert "version" in response.to_dict()
        assert "queue_size" in response.to_dict()

        socket.close()
        context.term()

    finally:
        server.running = False
        time.sleep(0.2)


def test_server_generate_request():
    """Test generation request queueing"""
    server = ZmqServer(
        req_addr="tcp://127.0.0.1:15559", pub_addr="tcp://127.0.0.1:15560"
    )

    def run_server():
        try:
            server.start()
        except:
            pass

    server_thread = threading.Thread(target=run_server, daemon=True)
    server_thread.start()
    time.sleep(1.5)

    try:
        context = zmq.Context()
        socket = context.socket(zmq.REQ)
        socket.connect("tcp://127.0.0.1:15559")
        socket.setsockopt(zmq.RCVTIMEO, 3000)

        # Send generate request
        request = GenerateRequest(
            id="test-job-001",
            prompt="16-bit knight sprite",
            model="sdxl-base",
            size=[1024, 1024],
            steps=30,
            cfg_scale=7.5,
        )
        socket.send(serialize(request))

        # Receive job accepted response
        response_data = socket.recv()
        response = deserialize_response(response_data)

        response_dict = response.to_dict()
        assert response_dict["type"] == "job_accepted"
        assert response_dict["job_id"] == "test-job-001"
        assert "estimated_time_s" in response_dict

        socket.close()
        context.term()

    finally:
        server.running = False
        time.sleep(0.2)


def test_pub_sub_updates():
    """Test PUB-SUB progress updates"""
    server = ZmqServer(
        req_addr="tcp://127.0.0.1:15561", pub_addr="tcp://127.0.0.1:15562"
    )

    updates_received = []

    def run_server():
        try:
            server.start()
        except:
            pass

    def receive_updates():
        time.sleep(1.0)  # Wait for server to start
        context = zmq.Context()
        socket = context.socket(zmq.SUB)
        socket.connect("tcp://127.0.0.1:15562")
        socket.setsockopt(zmq.SUBSCRIBE, b"")
        socket.setsockopt(zmq.RCVTIMEO, 2000)

        for _ in range(5):  # Try to receive up to 5 updates
            try:
                data = socket.recv()
                updates_received.append(data)
            except zmq.Again:
                break

        socket.close()
        context.term()

    server_thread = threading.Thread(target=run_server, daemon=True)
    server_thread.start()
    time.sleep(1.5)

    # Start subscriber
    sub_thread = threading.Thread(target=receive_updates, daemon=True)
    sub_thread.start()

    try:
        # Send a generate request which triggers an update
        context = zmq.Context()
        socket = context.socket(zmq.REQ)
        socket.connect("tcp://127.0.0.1:15561")
        socket.setsockopt(zmq.RCVTIMEO, 3000)

        request = GenerateRequest(
            id="test-job-002",
            prompt="pixel art warrior",
            model="sdxl-base",
            size=[512, 512],
            steps=20,
            cfg_scale=8.0,
        )
        socket.send(serialize(request))
        socket.recv()  # Receive response

        socket.close()
        context.term()

        # Wait for subscriber to finish
        sub_thread.join(timeout=3)

        # Should have received at least one update (job started)
        print(f"Received {len(updates_received)} updates")
        assert len(updates_received) > 0

    finally:
        server.running = False
        time.sleep(0.2)


def test_message_latency():
    """Test message latency (should be <10ms p95 for Python)"""
    server = ZmqServer(
        req_addr="tcp://127.0.0.1:15563", pub_addr="tcp://127.0.0.1:15564"
    )

    def run_server():
        try:
            server.start()
        except:
            pass

    server_thread = threading.Thread(target=run_server, daemon=True)
    server_thread.start()
    time.sleep(1.5)

    latencies = []

    try:
        context = zmq.Context()
        socket = context.socket(zmq.REQ)
        socket.connect("tcp://127.0.0.1:15563")
        socket.setsockopt(zmq.RCVTIMEO, 3000)

        # Send 50 ping requests (reduced from 100 for faster tests)
        for _ in range(50):
            start = time.time()

            request = PingRequest()
            socket.send(serialize(request))
            socket.recv()

            latency_ms = (time.time() - start) * 1000
            latencies.append(latency_ms)

        socket.close()
        context.term()

        # Calculate p95 latency
        latencies.sort()
        p95_latency = latencies[int(len(latencies) * 0.95)]

        print(f"\nLatency stats:")
        print(f"  Min: {min(latencies):.2f}ms")
        print(f"  Median: {latencies[len(latencies)//2]:.2f}ms")
        print(f"  P95: {p95_latency:.2f}ms")
        print(f"  Max: {max(latencies):.2f}ms")

        # P95 should be well under 10ms for local IPC
        assert p95_latency < 10.0, f"P95 latency {p95_latency:.2f}ms exceeds 10ms"

    finally:
        server.running = False
        time.sleep(0.2)


if __name__ == "__main__":
    import pytest

    pytest.main([__file__, "-v", "-s"])
