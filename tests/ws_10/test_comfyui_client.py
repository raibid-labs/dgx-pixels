"""Tests for ComfyUI client integration"""

import json
import pytest
import responses
from unittest.mock import Mock, patch

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '../../python/workers'))

from comfyui_client import (
    ComfyUIClient,
    ExecutionStatus,
    WorkflowProgress,
    WorkflowOutput,
    create_client,
)


@pytest.fixture
def client():
    """Create a test client"""
    return ComfyUIClient(base_url="http://localhost:8188")


@pytest.fixture
def sample_workflow():
    """Sample workflow JSON"""
    return {
        "1": {
            "inputs": {"ckpt_name": "sd_xl_base_1.0.safetensors"},
            "class_type": "CheckpointLoaderSimple",
        },
        "2": {
            "inputs": {"text": "test prompt", "clip": ["1", 1]},
            "class_type": "CLIPTextEncode",
            "_meta": {"title": "Positive Prompt"},
        },
    }


class TestComfyUIClient:
    """Test ComfyUI client functionality"""

    @responses.activate
    def test_health_check_success(self, client):
        """Test successful health check"""
        responses.add(
            responses.GET,
            "http://localhost:8188/system_stats",
            json={"system": {"ram": 128000}},
            status=200,
        )

        assert client.health_check() is True

    def test_health_check_failure(self, client):
        """Test failed health check"""
        # No response mocked - should fail
        assert client.health_check() is False

    @responses.activate
    def test_queue_prompt_success(self, client, sample_workflow):
        """Test successful prompt queuing"""
        prompt_id = "test-prompt-123"

        responses.add(
            responses.POST,
            "http://localhost:8188/prompt",
            json={"prompt_id": prompt_id},
            status=200,
        )

        result = client.queue_prompt(sample_workflow)
        assert result == prompt_id

    @responses.activate
    def test_queue_prompt_no_id(self, client, sample_workflow):
        """Test prompt queue without ID returned"""
        responses.add(
            responses.POST,
            "http://localhost:8188/prompt",
            json={"status": "ok"},
            status=200,
        )

        with pytest.raises(ValueError, match="No prompt_id"):
            client.queue_prompt(sample_workflow)

    @responses.activate
    def test_get_queue_status(self, client):
        """Test queue status retrieval"""
        queue_data = {
            "queue_running": [["1", "prompt-123"]],
            "queue_pending": [],
        }

        responses.add(
            responses.GET,
            "http://localhost:8188/queue",
            json=queue_data,
            status=200,
        )

        result = client.get_queue_status()
        assert result == queue_data

    @responses.activate
    def test_get_history_found(self, client):
        """Test history retrieval when prompt exists"""
        prompt_id = "test-123"
        history_data = {
            prompt_id: {
                "outputs": {"7": {"images": [{"filename": "test.png"}]}},
                "status": {"completed": True},
            }
        }

        responses.add(
            responses.GET,
            f"http://localhost:8188/history/{prompt_id}",
            json=history_data,
            status=200,
        )

        result = client.get_history(prompt_id)
        assert result == history_data[prompt_id]

    @responses.activate
    def test_get_history_not_found(self, client):
        """Test history retrieval when prompt doesn't exist"""
        prompt_id = "nonexistent"

        responses.add(
            responses.GET,
            f"http://localhost:8188/history/{prompt_id}",
            status=404,
        )

        result = client.get_history(prompt_id)
        assert result is None

    @responses.activate
    def test_interrupt(self, client):
        """Test execution interruption"""
        responses.add(
            responses.POST,
            "http://localhost:8188/interrupt",
            status=200,
        )

        # Should not raise
        client.interrupt()

    @responses.activate
    def test_poll_progress_running(self, client):
        """Test progress polling for running job"""
        prompt_id = "test-123"

        responses.add(
            responses.GET,
            "http://localhost:8188/queue",
            json={"queue_running": [[1, prompt_id]], "queue_pending": []},
            status=200,
        )

        progress = client.poll_progress(prompt_id)
        assert progress.prompt_id == prompt_id
        assert progress.status == ExecutionStatus.RUNNING

    @responses.activate
    def test_poll_progress_pending(self, client):
        """Test progress polling for pending job"""
        prompt_id = "test-123"

        responses.add(
            responses.GET,
            "http://localhost:8188/queue",
            json={"queue_running": [], "queue_pending": [[1, prompt_id]]},
            status=200,
        )

        progress = client.poll_progress(prompt_id)
        assert progress.prompt_id == prompt_id
        assert progress.status == ExecutionStatus.PENDING

    @responses.activate
    def test_poll_progress_completed(self, client):
        """Test progress polling for completed job"""
        prompt_id = "test-123"

        # Not in queue
        responses.add(
            responses.GET,
            "http://localhost:8188/queue",
            json={"queue_running": [], "queue_pending": []},
            status=200,
        )

        # Found in history
        responses.add(
            responses.GET,
            f"http://localhost:8188/history/{prompt_id}",
            json={
                prompt_id: {
                    "outputs": {"7": {"images": [{"filename": "test.png"}]}},
                }
            },
            status=200,
        )

        progress = client.poll_progress(prompt_id)
        assert progress.prompt_id == prompt_id
        assert progress.status == ExecutionStatus.SUCCESS

    def test_inject_parameters_prompt(self, client, sample_workflow):
        """Test parameter injection - prompt"""
        result = client.inject_parameters(
            sample_workflow,
            prompt="new prompt",
        )

        assert result["2"]["inputs"]["text"] == "new prompt"

    def test_inject_parameters_steps(self, client):
        """Test parameter injection - steps"""
        workflow = {
            "5": {
                "inputs": {"steps": 20, "cfg": 8.0},
                "class_type": "KSampler",
            }
        }

        result = client.inject_parameters(workflow, steps=30, cfg_scale=10.0)

        assert result["5"]["inputs"]["steps"] == 30
        assert result["5"]["inputs"]["cfg"] == 10.0

    def test_inject_parameters_resolution(self, client):
        """Test parameter injection - resolution"""
        workflow = {
            "4": {
                "inputs": {"width": 512, "height": 512},
                "class_type": "EmptyLatentImage",
            }
        }

        result = client.inject_parameters(workflow, width=1024, height=768)

        assert result["4"]["inputs"]["width"] == 1024
        assert result["4"]["inputs"]["height"] == 768

    @responses.activate
    def test_download_image(self, client, tmp_path):
        """Test image download"""
        image_data = b"fake-image-data"
        image_url = "http://localhost:8188/view?filename=test.png"

        responses.add(
            responses.GET,
            image_url,
            body=image_data,
            status=200,
        )

        output_path = tmp_path / "test.png"
        result = client.download_image(image_url, str(output_path))

        assert output_path.exists()
        assert output_path.read_bytes() == image_data
        assert result == str(output_path)

    def test_load_workflow(self, client, tmp_path, sample_workflow):
        """Test workflow loading from file"""
        workflow_file = tmp_path / "test_workflow.json"
        workflow_file.write_text(json.dumps(sample_workflow))

        result = client.load_workflow(str(workflow_file))
        assert result == sample_workflow


class TestWorkflowExecution:
    """Test full workflow execution flow"""

    @responses.activate
    def test_wait_for_completion_success(self, client):
        """Test successful workflow completion"""
        prompt_id = "test-123"

        # Queue status - running
        responses.add(
            responses.GET,
            "http://localhost:8188/queue",
            json={"queue_running": [[1, prompt_id]], "queue_pending": []},
            status=200,
        )

        # Queue status - completed (not in queue)
        responses.add(
            responses.GET,
            "http://localhost:8188/queue",
            json={"queue_running": [], "queue_pending": []},
            status=200,
        )

        # History - success
        responses.add(
            responses.GET,
            f"http://localhost:8188/history/{prompt_id}",
            json={
                prompt_id: {
                    "outputs": {
                        "7": {
                            "images": [
                                {
                                    "filename": "test.png",
                                    "subfolder": "",
                                    "type": "output",
                                }
                            ]
                        }
                    }
                }
            },
            status=200,
        )

        # Callback tracking
        callback_calls = []

        def callback(progress):
            callback_calls.append(progress)

        output = client.wait_for_completion(prompt_id, callback=callback)

        assert output.success is True
        assert len(output.images) == 1
        assert "test.png" in output.images[0]
        assert len(callback_calls) > 0

    @responses.activate
    def test_wait_for_completion_timeout(self, client):
        """Test workflow timeout"""
        client.timeout_s = 0.5  # Very short timeout
        prompt_id = "test-123"

        # Always return running
        responses.add(
            responses.GET,
            "http://localhost:8188/queue",
            json={"queue_running": [[1, prompt_id]], "queue_pending": []},
            status=200,
        )

        # Mock interrupt
        responses.add(
            responses.POST,
            "http://localhost:8188/interrupt",
            status=200,
        )

        with pytest.raises(TimeoutError):
            client.wait_for_completion(prompt_id)

    @responses.activate
    def test_wait_for_completion_error(self, client):
        """Test workflow execution error"""
        prompt_id = "test-123"

        # Not in queue
        responses.add(
            responses.GET,
            "http://localhost:8188/queue",
            json={"queue_running": [], "queue_pending": []},
            status=200,
        )

        # History shows error
        responses.add(
            responses.GET,
            f"http://localhost:8188/history/{prompt_id}",
            json={
                prompt_id: {
                    "outputs": {},
                    "status": {"messages": [["error", "Test error"]]},
                }
            },
            status=200,
        )

        with pytest.raises(RuntimeError, match="execution failed"):
            client.wait_for_completion(prompt_id)


class TestClientCreation:
    """Test client creation utilities"""

    @responses.activate
    def test_create_client_success(self):
        """Test successful client creation"""
        responses.add(
            responses.GET,
            "http://localhost:8188/system_stats",
            json={},
            status=200,
        )

        client = create_client()
        assert client is not None

    def test_create_client_failure(self):
        """Test client creation failure"""
        with pytest.raises(ConnectionError):
            create_client(url="http://localhost:9999")


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
