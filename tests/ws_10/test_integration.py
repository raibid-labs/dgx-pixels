"""Integration tests for backend worker

These tests verify end-to-end functionality by testing the full stack:
- ZeroMQ communication
- Job execution
- ComfyUI integration
- Progress updates
"""

import pytest
import time
import threading
import zmq
import msgpack
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '../../python/workers'))

from generation_worker import GenerationWorker
from job_executor import ExecutorConfig
from message_protocol import (
    GenerateRequest,
    PingRequest,
    StatusRequest,
    ListModelsRequest,
    serialize,
    deserialize_response,
    deserialize_update,
)


# Skip integration tests if ComfyUI is not available
COMFYUI_AVAILABLE = False
try:
    import requests
    response = requests.get("http://localhost:8188/system_stats", timeout=2)
    COMFYUI_AVAILABLE = response.status_code == 200
except:
    pass

pytestmark = pytest.mark.skipif(
    not COMFYUI_AVAILABLE,
    reason="ComfyUI not available at localhost:8188"
)


@pytest.fixture
def zmq_client():
    """Create a ZeroMQ client for testing"""
    context = zmq.Context()

    # REQ socket
    req_socket = context.socket(zmq.REQ)
    req_socket.connect("tcp://127.0.0.1:5555")
    req_socket.setsockopt(zmq.RCVTIMEO, 5000)  # 5 second timeout

    # SUB socket
    sub_socket = context.socket(zmq.SUB)
    sub_socket.connect("tcp://127.0.0.1:5556")
    sub_socket.subscribe(b"")  # Subscribe to all messages
    sub_socket.setsockopt(zmq.RCVTIMEO, 1000)  # 1 second timeout

    yield {
        "context": context,
        "req": req_socket,
        "sub": sub_socket,
    }

    # Cleanup
    req_socket.close()
    sub_socket.close()
    context.term()


@pytest.fixture(scope="module")
def worker():
    """Start worker in background for testing"""
    config = ExecutorConfig(
        comfyui_url="http://localhost:8188",
        workflow_dir="/home/beengud/raibid-labs/dgx-pixels/workflows",
        output_dir="/tmp/dgx-pixels-test-output",
    )

    worker = GenerationWorker(
        req_addr="tcp://127.0.0.1:5555",
        pub_addr="tcp://127.0.0.1:5556",
        executor_config=config,
    )

    # Start in thread
    thread = threading.Thread(target=worker.start, daemon=True)
    thread.start()

    # Wait for worker to start
    time.sleep(2.0)

    yield worker

    # Shutdown
    worker.shutdown()


class TestBasicCommunication:
    """Test basic ZeroMQ communication"""

    def test_ping_pong(self, worker, zmq_client):
        """Test ping/pong health check"""
        request = PingRequest()
        zmq_client["req"].send(serialize(request))

        response_data = zmq_client["req"].recv()
        response = deserialize_response(response_data)

        assert response.to_dict()["type"] == "pong"

    def test_status_request(self, worker, zmq_client):
        """Test status request"""
        request = StatusRequest()
        zmq_client["req"].send(serialize(request))

        response_data = zmq_client["req"].recv()
        response = deserialize_response(response_data)

        status = response.to_dict()
        assert status["type"] == "status_info"
        assert "queue_size" in status
        assert "active_jobs" in status
        assert "uptime_s" in status

    def test_list_models(self, worker, zmq_client):
        """Test model listing"""
        request = ListModelsRequest()
        zmq_client["req"].send(serialize(request))

        response_data = zmq_client["req"].recv()
        response = deserialize_response(response_data)

        models = response.to_dict()
        assert models["type"] == "model_list"
        assert "models" in models


class TestJobExecution:
    """Test job execution flow"""

    def test_generate_job_accepted(self, worker, zmq_client):
        """Test that generation request is accepted"""
        request = GenerateRequest(
            id="test-job-1",
            prompt="pixel art character sprite",
            model="sd_xl_base_1.0.safetensors",
            size=[1024, 1024],
            steps=5,  # Minimal steps for fast test
            cfg_scale=7.0,
        )

        zmq_client["req"].send(serialize(request))

        response_data = zmq_client["req"].recv()
        response = deserialize_response(response_data)

        result = response.to_dict()
        assert result["type"] == "job_accepted"
        assert result["job_id"] == "test-job-1"
        assert "estimated_time_s" in result

    def test_progress_updates_received(self, worker, zmq_client):
        """Test that progress updates are received"""
        # Submit job
        request = GenerateRequest(
            id="test-job-2",
            prompt="pixel art tree sprite",
            model="sd_xl_base_1.0.safetensors",
            size=[512, 512],
            steps=3,
            cfg_scale=7.0,
        )

        zmq_client["req"].send(serialize(request))
        zmq_client["req"].recv()  # Receive acceptance

        # Listen for progress updates
        updates_received = []
        timeout_start = time.time()

        while time.time() - timeout_start < 30.0:  # 30 second timeout
            try:
                update_data = zmq_client["sub"].recv()
                update = deserialize_update(update_data)
                updates_received.append(update.to_dict())

                # Check for completion
                if update.to_dict()["type"] == "job_finished":
                    break
            except zmq.Again:
                continue

        # Should have received multiple updates
        assert len(updates_received) > 0

        # Should have job_started
        assert any(u["type"] == "job_started" for u in updates_received)

        # Should have progress updates
        assert any(u["type"] == "progress" for u in updates_received)

    @pytest.mark.slow
    def test_full_generation_flow(self, worker, zmq_client):
        """Test complete generation flow from request to output"""
        # Submit job
        request = GenerateRequest(
            id="test-job-3",
            prompt="pixel art sword item, 32x32",
            model="sd_xl_base_1.0.safetensors",
            size=[1024, 1024],
            steps=5,
            cfg_scale=7.0,
        )

        zmq_client["req"].send(serialize(request))
        response_data = zmq_client["req"].recv()
        response = deserialize_response(response_data)

        assert response.to_dict()["type"] == "job_accepted"

        # Wait for completion
        job_complete = False
        timeout_start = time.time()

        while time.time() - timeout_start < 60.0:  # 60 second timeout
            try:
                update_data = zmq_client["sub"].recv()
                update = deserialize_update(update_data)

                if update.to_dict()["type"] == "job_finished":
                    job_complete = True
                    break
            except zmq.Again:
                continue

        assert job_complete, "Job did not complete within timeout"


class TestErrorHandling:
    """Test error handling and edge cases"""

    def test_invalid_request(self, worker, zmq_client):
        """Test handling of invalid request"""
        # Send malformed data
        zmq_client["req"].send(b"invalid-data")

        response_data = zmq_client["req"].recv()
        response = deserialize_response(response_data)

        assert response.to_dict()["type"] == "error"

    def test_missing_workflow(self, worker, zmq_client):
        """Test handling of missing workflow"""
        # This would require custom workflow specification
        # For now, using default workflow should work
        pass


class TestConcurrentJobs:
    """Test concurrent job handling"""

    def test_multiple_jobs_queued(self, worker, zmq_client):
        """Test that multiple jobs can be queued"""
        job_ids = []

        # Submit 3 jobs
        for i in range(3):
            request = GenerateRequest(
                id=f"concurrent-job-{i}",
                prompt=f"test prompt {i}",
                model="sd_xl_base_1.0.safetensors",
                size=[512, 512],
                steps=3,
                cfg_scale=7.0,
            )

            zmq_client["req"].send(serialize(request))
            response_data = zmq_client["req"].recv()
            response = deserialize_response(response_data)

            assert response.to_dict()["type"] == "job_accepted"
            job_ids.append(response.to_dict()["job_id"])

        # All jobs should be accepted
        assert len(job_ids) == 3


class TestPerformance:
    """Test performance metrics"""

    def test_message_latency(self, worker, zmq_client):
        """Test request/response latency"""
        latencies = []

        for _ in range(10):
            start = time.time()

            request = PingRequest()
            zmq_client["req"].send(serialize(request))
            zmq_client["req"].recv()

            latency = (time.time() - start) * 1000  # ms
            latencies.append(latency)

        avg_latency = sum(latencies) / len(latencies)
        max_latency = max(latencies)

        print(f"\nMessage latency: avg={avg_latency:.2f}ms, max={max_latency:.2f}ms")

        # Should be fast
        assert avg_latency < 10.0  # <10ms average
        assert max_latency < 50.0  # <50ms max

    def test_progress_update_rate(self, worker, zmq_client):
        """Test progress update frequency"""
        # Submit job
        request = GenerateRequest(
            id="perf-test-1",
            prompt="test",
            model="sd_xl_base_1.0.safetensors",
            size=[512, 512],
            steps=10,
            cfg_scale=7.0,
        )

        zmq_client["req"].send(serialize(request))
        zmq_client["req"].recv()

        # Count progress updates
        progress_count = 0
        start_time = None
        end_time = None

        timeout_start = time.time()

        while time.time() - timeout_start < 30.0:
            try:
                update_data = zmq_client["sub"].recv()
                update = deserialize_update(update_data)
                update_dict = update.to_dict()

                if update_dict["type"] == "progress":
                    if start_time is None:
                        start_time = time.time()
                    progress_count += 1
                    end_time = time.time()

                if update_dict["type"] == "job_finished":
                    break
            except zmq.Again:
                continue

        if start_time and end_time and progress_count > 0:
            duration = end_time - start_time
            update_rate = progress_count / duration  # Hz

            print(f"\nProgress update rate: {update_rate:.1f} Hz ({progress_count} updates in {duration:.2f}s)")

            # Should be >10 Hz as per spec
            assert update_rate >= 10.0


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])
