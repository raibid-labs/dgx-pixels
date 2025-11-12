#!/usr/bin/env python3
"""
Integration Tests for ComfyUI Workflows

Tests optimized workflows can be loaded and executed via ComfyUI HTTP API.
"""

import pytest
import json
import requests
from pathlib import Path
from time import sleep


# ComfyUI URL (can be overridden with env var)
import os
COMFYUI_URL = os.environ.get("COMFYUI_URL", "http://localhost:8188")

# Workflow paths
PROJECT_ROOT = Path(__file__).parent.parent.parent
WORKFLOW_DIR = PROJECT_ROOT / "workflows"


def check_comfyui_available():
    """Check if ComfyUI is running"""
    try:
        response = requests.get(f"{COMFYUI_URL}/system_stats", timeout=5)
        return response.status_code == 200
    except Exception:
        return False


pytestmark = pytest.mark.skipif(
    not check_comfyui_available(),
    reason="ComfyUI not available (start with: cd comfyui && python main.py)"
)


class TestWorkflowValidity:
    """Test that workflows are valid JSON and well-formed"""

    def test_sprite_optimized_workflow_valid(self):
        """Test sprite_optimized.json is valid"""
        workflow_path = WORKFLOW_DIR / "sprite_optimized.json"
        assert workflow_path.exists(), f"Workflow not found: {workflow_path}"

        with open(workflow_path, 'r') as f:
            workflow = json.load(f)

        # Check required nodes
        assert "1" in workflow  # CheckpointLoader
        assert "5" in workflow  # KSampler
        assert "7" in workflow  # SaveImage

        # Check checkpoint name
        assert workflow["1"]["inputs"]["ckpt_name"] == "sd_xl_base_1.0.safetensors"

    def test_batch_optimized_workflow_valid(self):
        """Test batch_optimized.json is valid"""
        workflow_path = WORKFLOW_DIR / "batch_optimized.json"
        assert workflow_path.exists()

        with open(workflow_path, 'r') as f:
            workflow = json.load(f)

        # Check batch size is 8
        assert workflow["4"]["inputs"]["batch_size"] == 8

    def test_pixel_art_workflow_valid(self):
        """Test pixel_art_workflow.json is valid"""
        workflow_path = WORKFLOW_DIR / "pixel_art_workflow.json"
        assert workflow_path.exists()

        with open(workflow_path, 'r') as f:
            workflow = json.load(f)

        # Check pixel art specific settings
        sampler = workflow["5"]["inputs"]
        assert sampler["sampler_name"] == "euler_ancestral"
        assert sampler["scheduler"] == "karras"


class TestComfyUIAPI:
    """Test ComfyUI HTTP API endpoints"""

    def test_system_stats_endpoint(self):
        """Test /system_stats endpoint"""
        response = requests.get(f"{COMFYUI_URL}/system_stats", timeout=10)
        assert response.status_code == 200
        stats = response.json()
        assert "system" in stats

    def test_queue_endpoint(self):
        """Test /queue endpoint"""
        response = requests.get(f"{COMFYUI_URL}/queue", timeout=10)
        assert response.status_code == 200
        queue = response.json()
        assert "queue_running" in queue
        assert "queue_pending" in queue

    def test_object_info_endpoint(self):
        """Test /object_info endpoint (node definitions)"""
        response = requests.get(f"{COMFYUI_URL}/object_info", timeout=10)
        assert response.status_code == 200
        object_info = response.json()

        # Check required node types exist
        assert "CheckpointLoaderSimple" in object_info
        assert "KSampler" in object_info
        assert "SaveImage" in object_info


class TestWorkflowExecution:
    """Test workflow execution via ComfyUI API"""

    def test_queue_workflow_sprite_optimized(self):
        """Test queuing sprite_optimized workflow"""
        workflow_path = WORKFLOW_DIR / "sprite_optimized.json"
        with open(workflow_path, 'r') as f:
            workflow = json.load(f)

        # Queue workflow
        response = requests.post(
            f"{COMFYUI_URL}/prompt",
            json={"prompt": workflow},
            timeout=10,
        )

        assert response.status_code == 200
        result = response.json()
        assert "prompt_id" in result

    def test_queue_workflow_batch_optimized(self):
        """Test queuing batch_optimized workflow"""
        workflow_path = WORKFLOW_DIR / "batch_optimized.json"
        with open(workflow_path, 'r') as f:
            workflow = json.load(f)

        response = requests.post(
            f"{COMFYUI_URL}/prompt",
            json={"prompt": workflow},
            timeout=10,
        )

        assert response.status_code == 200
        result = response.json()
        assert "prompt_id" in result

    def test_queue_workflow_pixel_art(self):
        """Test queuing pixel_art_workflow"""
        workflow_path = WORKFLOW_DIR / "pixel_art_workflow.json"
        with open(workflow_path, 'r') as f:
            workflow = json.load(f)

        response = requests.post(
            f"{COMFYUI_URL}/prompt",
            json={"prompt": workflow},
            timeout=10,
        )

        assert response.status_code == 200
        result = response.json()
        assert "prompt_id" in result


class TestWorkflowParameters:
    """Test workflow parameter validation"""

    def test_sprite_optimized_parameters(self):
        """Test sprite_optimized.json has correct parameters"""
        workflow_path = WORKFLOW_DIR / "sprite_optimized.json"
        with open(workflow_path, 'r') as f:
            workflow = json.load(f)

        # Check image dimensions
        latent = workflow["4"]["inputs"]
        assert latent["width"] == 1024
        assert latent["height"] == 1024
        assert latent["batch_size"] == 1

        # Check sampler settings
        sampler = workflow["5"]["inputs"]
        assert sampler["steps"] == 20
        assert sampler["cfg"] == 8.0
        assert sampler["sampler_name"] == "euler_ancestral"

    def test_batch_optimized_batch_size(self):
        """Test batch_optimized.json has batch size 8"""
        workflow_path = WORKFLOW_DIR / "batch_optimized.json"
        with open(workflow_path, 'r') as f:
            workflow = json.load(f)

        latent = workflow["4"]["inputs"]
        assert latent["batch_size"] == 8

    def test_pixel_art_workflow_quality_settings(self):
        """Test pixel_art_workflow has quality settings"""
        workflow_path = WORKFLOW_DIR / "pixel_art_workflow.json"
        with open(workflow_path, 'r') as f:
            workflow = json.load(f)

        # Check higher quality settings for pixel art
        sampler = workflow["5"]["inputs"]
        assert sampler["steps"] == 25  # More steps for quality
        assert sampler["cfg"] == 9.0   # Higher CFG for sharper results


if __name__ == "__main__":
    # Run tests
    pytest.main([__file__, "-v", "--tb=short"])
