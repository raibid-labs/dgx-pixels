#!/usr/bin/env python3
"""
Unit tests for ComfyUI Client
"""

import pytest
import json
import requests
from pathlib import Path
from unittest.mock import Mock, patch, MagicMock
import sys

# Add project root to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent.parent))

from python.batch.comfyui_client import (
    ComfyUIClient,
    ComfyUIJob,
    ComfyUIJobStatus,
)


class TestComfyUIClient:
    """Test ComfyUI client functionality"""

    def test_client_initialization(self):
        """Test client initialization with default parameters"""
        client = ComfyUIClient()

        assert client.host == "localhost"
        assert client.port == 8188
        assert client.base_url == "http://localhost:8188"
        assert client.timeout == 300
        assert client.poll_interval == 1.0

    def test_client_initialization_custom(self):
        """Test client initialization with custom parameters"""
        client = ComfyUIClient(
            host="192.168.1.100",
            port=9000,
            timeout=600,
            poll_interval=0.5,
        )

        assert client.host == "192.168.1.100"
        assert client.port == 9000
        assert client.base_url == "http://192.168.1.100:9000"
        assert client.timeout == 600
        assert client.poll_interval == 0.5

    @patch("requests.get")
    def test_check_health_success(self, mock_get):
        """Test health check when server is healthy"""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_get.return_value = mock_response

        client = ComfyUIClient()
        assert client.check_health() is True

        mock_get.assert_called_once_with("http://localhost:8188/system_stats", timeout=5)

    @patch("requests.get")
    def test_check_health_failure(self, mock_get):
        """Test health check when server is down"""
        mock_get.side_effect = Exception("Connection refused")

        client = ComfyUIClient()
        assert client.check_health() is False

    @patch("requests.get")
    def test_get_queue_status(self, mock_get):
        """Test getting queue status"""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "queue_running": [],
            "queue_pending": [["1", "prompt_123"]],
        }
        mock_get.return_value = mock_response

        client = ComfyUIClient()
        queue = client.get_queue_status()

        assert "queue_running" in queue
        assert "queue_pending" in queue
        assert len(queue["queue_pending"]) == 1

    @patch("requests.post")
    def test_submit_workflow(self, mock_post):
        """Test workflow submission"""
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {"prompt_id": "test_prompt_id"}
        mock_post.return_value = mock_response

        client = ComfyUIClient()
        workflow = {"1": {"inputs": {}}}

        prompt_id = client.submit_workflow(workflow)

        assert prompt_id == "test_prompt_id"
        mock_post.assert_called_once()

    @patch("python.batch.comfyui_client.requests.post")
    def test_submit_workflow_error(self, mock_post):
        """Test workflow submission error handling"""
        mock_post.side_effect = requests.exceptions.RequestException("Network error")

        client = ComfyUIClient()
        workflow = {"1": {"inputs": {}}}

        with pytest.raises(RuntimeError, match="Failed to submit workflow"):
            client.submit_workflow(workflow)

    def test_inject_parameters_prompt(self):
        """Test parameter injection - prompt"""
        client = ComfyUIClient()

        workflow = {
            "2": {"inputs": {"text": "old prompt"}},
        }

        modified = client.inject_parameters(workflow, prompt="new prompt")

        assert modified["2"]["inputs"]["text"] == "new prompt"

    def test_inject_parameters_negative_prompt(self):
        """Test parameter injection - negative prompt"""
        client = ComfyUIClient()

        workflow = {
            "3": {"inputs": {"text": "old negative"}},
        }

        modified = client.inject_parameters(workflow, negative_prompt="new negative")

        assert modified["3"]["inputs"]["text"] == "new negative"

    def test_inject_parameters_batch_size(self):
        """Test parameter injection - batch size"""
        client = ComfyUIClient()

        workflow = {
            "4": {"inputs": {"batch_size": 1}},
        }

        modified = client.inject_parameters(workflow, batch_size=8)

        assert modified["4"]["inputs"]["batch_size"] == 8

    def test_inject_parameters_seed(self):
        """Test parameter injection - seed"""
        client = ComfyUIClient()

        workflow = {
            "5": {"inputs": {"seed": 0}},
        }

        modified = client.inject_parameters(workflow, seed=42)

        assert modified["5"]["inputs"]["seed"] == 42

    def test_inject_parameters_steps(self):
        """Test parameter injection - steps"""
        client = ComfyUIClient()

        workflow = {
            "5": {"inputs": {"steps": 20}},
        }

        modified = client.inject_parameters(workflow, steps=30)

        assert modified["5"]["inputs"]["steps"] == 30

    def test_inject_parameters_cfg_scale(self):
        """Test parameter injection - CFG scale"""
        client = ComfyUIClient()

        workflow = {
            "5": {"inputs": {"cfg": 7.0}},
        }

        modified = client.inject_parameters(workflow, cfg_scale=8.5)

        assert modified["5"]["inputs"]["cfg"] == 8.5

    def test_inject_parameters_model(self):
        """Test parameter injection - model"""
        client = ComfyUIClient()

        workflow = {
            "1": {"inputs": {"ckpt_name": "old_model.safetensors"}},
        }

        modified = client.inject_parameters(workflow, model="new_model.safetensors")

        assert modified["1"]["inputs"]["ckpt_name"] == "new_model.safetensors"

    def test_inject_parameters_multiple(self):
        """Test parameter injection - multiple parameters"""
        client = ComfyUIClient()

        workflow = {
            "2": {"inputs": {"text": "old prompt"}},
            "4": {"inputs": {"batch_size": 1}},
            "5": {"inputs": {"seed": 0, "steps": 20, "cfg": 7.0}},
        }

        modified = client.inject_parameters(
            workflow,
            prompt="new prompt",
            batch_size=4,
            seed=123,
            steps=25,
            cfg_scale=8.0,
        )

        assert modified["2"]["inputs"]["text"] == "new prompt"
        assert modified["4"]["inputs"]["batch_size"] == 4
        assert modified["5"]["inputs"]["seed"] == 123
        assert modified["5"]["inputs"]["steps"] == 25
        assert modified["5"]["inputs"]["cfg"] == 8.0

    def test_inject_parameters_no_modification(self):
        """Test parameter injection with no parameters (should not crash)"""
        client = ComfyUIClient()

        workflow = {
            "1": {"inputs": {"ckpt_name": "model.safetensors"}},
        }

        modified = client.inject_parameters(workflow)

        assert modified == workflow

    def test_extract_output_paths(self):
        """Test extracting output paths from history"""
        client = ComfyUIClient()

        outputs = {
            "7": {
                "images": [
                    {"filename": "image1.png", "subfolder": "output"},
                    {"filename": "image2.png", "subfolder": "output"},
                ]
            }
        }

        paths = client._extract_output_paths(outputs)

        assert len(paths) == 2
        assert "output/image1.png" in paths
        assert "output/image2.png" in paths

    def test_extract_output_paths_no_subfolder(self):
        """Test extracting output paths with empty subfolder"""
        client = ComfyUIClient()

        outputs = {
            "7": {
                "images": [
                    {"filename": "image1.png", "subfolder": ""},
                ]
            }
        }

        paths = client._extract_output_paths(outputs)

        assert len(paths) == 1
        assert paths[0] == "image1.png"

    def test_comfyui_job_creation(self):
        """Test ComfyUIJob dataclass creation"""
        job = ComfyUIJob(
            prompt_id="test_id",
            workflow={"1": {}},
        )

        assert job.prompt_id == "test_id"
        assert job.status == ComfyUIJobStatus.PENDING
        assert job.progress == 0.0
        assert job.error is None
        assert job.output_images == []

    def test_comfyui_job_status_enum(self):
        """Test ComfyUIJobStatus enum values"""
        assert ComfyUIJobStatus.PENDING == "pending"
        assert ComfyUIJobStatus.RUNNING == "running"
        assert ComfyUIJobStatus.COMPLETED == "completed"
        assert ComfyUIJobStatus.FAILED == "failed"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
