"""ComfyUI HTTP API client for workflow execution and progress monitoring

This module provides a client for interacting with the ComfyUI API to:
- Submit workflows for execution
- Monitor progress in real-time
- Retrieve generated images
- Manage workflow history
"""

import json
import os
import time
import uuid
from dataclasses import dataclass
from enum import Enum
from typing import Any, Dict, List, Optional, Tuple
from urllib.parse import urljoin

import requests
from requests.exceptions import ConnectionError, RequestException, Timeout


# ============================================================================
# Constants
# ============================================================================

DEFAULT_COMFYUI_URL = "http://localhost:8188"
DEFAULT_TIMEOUT_S = 300
DEFAULT_POLL_INTERVAL_MS = 100


# ============================================================================
# Data Structures
# ============================================================================


class ExecutionStatus(str, Enum):
    """Workflow execution status"""

    PENDING = "pending"
    RUNNING = "running"
    SUCCESS = "success"
    ERROR = "error"


@dataclass
class WorkflowProgress:
    """Real-time workflow progress information"""

    prompt_id: str
    status: ExecutionStatus
    current_node: Optional[str] = None
    node_name: Optional[str] = None
    step: int = 0
    total_steps: int = 0
    percent: float = 0.0
    eta_s: float = 0.0


@dataclass
class WorkflowOutput:
    """Workflow execution output"""

    prompt_id: str
    images: List[str]  # List of file paths
    duration_s: float
    success: bool
    error: Optional[str] = None


# ============================================================================
# ComfyUI Client
# ============================================================================


class ComfyUIClient:
    """HTTP client for ComfyUI API

    Provides methods for:
    - Workflow submission
    - Progress polling
    - Image retrieval
    - Prompt interruption
    """

    def __init__(
        self,
        base_url: str = DEFAULT_COMFYUI_URL,
        timeout_s: float = DEFAULT_TIMEOUT_S,
        poll_interval_ms: int = DEFAULT_POLL_INTERVAL_MS,
    ):
        self.base_url = base_url.rstrip("/")
        self.timeout_s = timeout_s
        self.poll_interval_s = poll_interval_ms / 1000.0
        self.session = requests.Session()

    def health_check(self) -> bool:
        """Check if ComfyUI is accessible"""
        try:
            response = self.session.get(
                f"{self.base_url}/system_stats", timeout=5.0
            )
            return response.status_code == 200
        except (ConnectionError, Timeout):
            return False

    def queue_prompt(self, workflow: Dict[str, Any], client_id: Optional[str] = None) -> str:
        """Queue a workflow for execution

        Args:
            workflow: ComfyUI workflow JSON
            client_id: Optional client identifier

        Returns:
            prompt_id: Unique identifier for this execution

        Raises:
            ConnectionError: If ComfyUI is not accessible
            RequestException: If the request fails
        """
        if client_id is None:
            client_id = str(uuid.uuid4())

        payload = {"prompt": workflow, "client_id": client_id}

        try:
            response = self.session.post(
                f"{self.base_url}/prompt",
                json=payload,
                timeout=10.0,
            )
            response.raise_for_status()

            result = response.json()
            prompt_id = result.get("prompt_id")

            if not prompt_id:
                raise ValueError("No prompt_id returned from ComfyUI")

            return prompt_id

        except ConnectionError as e:
            raise ConnectionError(f"Cannot connect to ComfyUI at {self.base_url}: {e}")
        except RequestException as e:
            raise RequestException(f"Failed to queue prompt: {e}")

    def get_queue_status(self) -> Dict[str, Any]:
        """Get current queue status

        Returns:
            Queue status with running and pending prompts
        """
        response = self.session.get(f"{self.base_url}/queue", timeout=5.0)
        response.raise_for_status()
        return response.json()

    def get_history(self, prompt_id: str) -> Optional[Dict[str, Any]]:
        """Get execution history for a prompt

        Args:
            prompt_id: The prompt ID to query

        Returns:
            History entry if found, None otherwise
        """
        response = self.session.get(f"{self.base_url}/history/{prompt_id}", timeout=5.0)

        if response.status_code == 404:
            return None

        response.raise_for_status()
        history = response.json()

        return history.get(prompt_id)

    def interrupt(self) -> None:
        """Interrupt the current execution"""
        try:
            response = self.session.post(f"{self.base_url}/interrupt", timeout=5.0)
            response.raise_for_status()
        except RequestException as e:
            print(f"Warning: Failed to interrupt: {e}")

    def poll_progress(self, prompt_id: str) -> WorkflowProgress:
        """Poll for current execution progress

        Args:
            prompt_id: The prompt ID to monitor

        Returns:
            Current progress information
        """
        # Check queue first
        queue = self.get_queue_status()

        # Check if in running queue
        running = queue.get("queue_running", [])
        for item in running:
            if len(item) >= 2 and item[1] == prompt_id:
                # Extract progress from item if available
                return WorkflowProgress(
                    prompt_id=prompt_id,
                    status=ExecutionStatus.RUNNING,
                    current_node=None,
                    node_name=None,
                    step=0,
                    total_steps=0,
                    percent=0.0,
                    eta_s=0.0,
                )

        # Check if in pending queue
        pending = queue.get("queue_pending", [])
        for item in pending:
            if len(item) >= 2 and item[1] == prompt_id:
                return WorkflowProgress(
                    prompt_id=prompt_id,
                    status=ExecutionStatus.PENDING,
                    step=0,
                    total_steps=0,
                    percent=0.0,
                    eta_s=0.0,
                )

        # Check history for completion
        history = self.get_history(prompt_id)
        if history:
            # Execution completed
            has_outputs = bool(history.get("outputs"))
            status = ExecutionStatus.SUCCESS if has_outputs else ExecutionStatus.ERROR

            return WorkflowProgress(
                prompt_id=prompt_id,
                status=status,
                step=100,
                total_steps=100,
                percent=100.0,
                eta_s=0.0,
            )

        # Not found anywhere - still pending
        return WorkflowProgress(
            prompt_id=prompt_id,
            status=ExecutionStatus.PENDING,
            step=0,
            total_steps=0,
            percent=0.0,
            eta_s=0.0,
        )

    def wait_for_completion(
        self,
        prompt_id: str,
        callback: Optional[callable] = None,
    ) -> WorkflowOutput:
        """Wait for workflow execution to complete

        Args:
            prompt_id: The prompt ID to wait for
            callback: Optional progress callback function(progress: WorkflowProgress)

        Returns:
            Workflow output with images and metadata

        Raises:
            TimeoutError: If execution exceeds timeout
            RuntimeError: If execution fails
        """
        start_time = time.time()

        while True:
            elapsed = time.time() - start_time

            # Check timeout
            if elapsed > self.timeout_s:
                self.interrupt()
                raise TimeoutError(
                    f"Workflow execution exceeded timeout ({self.timeout_s}s)"
                )

            # Poll progress
            progress = self.poll_progress(prompt_id)

            # Call progress callback
            if callback:
                callback(progress)

            # Check completion
            if progress.status == ExecutionStatus.SUCCESS:
                return self._extract_output(prompt_id, elapsed)
            elif progress.status == ExecutionStatus.ERROR:
                history = self.get_history(prompt_id)
                error_msg = "Unknown error"
                if history and "status" in history:
                    error_msg = str(history.get("status", {}).get("messages", error_msg))
                raise RuntimeError(f"Workflow execution failed: {error_msg}")

            # Wait before next poll
            time.sleep(self.poll_interval_s)

    def _extract_output(self, prompt_id: str, duration_s: float) -> WorkflowOutput:
        """Extract output images from completed workflow

        Args:
            prompt_id: The prompt ID
            duration_s: Execution duration

        Returns:
            WorkflowOutput with image paths
        """
        history = self.get_history(prompt_id)

        if not history:
            return WorkflowOutput(
                prompt_id=prompt_id,
                images=[],
                duration_s=duration_s,
                success=False,
                error="No history found",
            )

        outputs = history.get("outputs", {})
        image_paths = []

        # Extract images from all output nodes
        for node_id, node_output in outputs.items():
            images = node_output.get("images", [])

            for img in images:
                filename = img.get("filename")
                subfolder = img.get("subfolder", "")
                img_type = img.get("type", "output")

                # Construct URL to retrieve image
                if filename:
                    # For now, store URL - actual download handled by caller
                    params = f"filename={filename}&subfolder={subfolder}&type={img_type}"
                    image_url = f"{self.base_url}/view?{params}"
                    image_paths.append(image_url)

        return WorkflowOutput(
            prompt_id=prompt_id,
            images=image_paths,
            duration_s=duration_s,
            success=True,
        )

    def download_image(self, image_url: str, output_path: str) -> str:
        """Download an image from ComfyUI

        Args:
            image_url: URL to the image (from _extract_output)
            output_path: Local path to save image

        Returns:
            Path to saved image
        """
        response = self.session.get(image_url, timeout=30.0)
        response.raise_for_status()

        # Ensure directory exists
        os.makedirs(os.path.dirname(output_path), exist_ok=True)

        with open(output_path, "wb") as f:
            f.write(response.content)

        return output_path

    def load_workflow(self, workflow_path: str) -> Dict[str, Any]:
        """Load a workflow from JSON file

        Args:
            workflow_path: Path to workflow JSON file

        Returns:
            Workflow dictionary
        """
        with open(workflow_path, "r") as f:
            return json.load(f)

    def inject_parameters(
        self,
        workflow: Dict[str, Any],
        prompt: Optional[str] = None,
        negative_prompt: Optional[str] = None,
        steps: Optional[int] = None,
        cfg_scale: Optional[float] = None,
        seed: Optional[int] = None,
        width: Optional[int] = None,
        height: Optional[int] = None,
    ) -> Dict[str, Any]:
        """Inject parameters into workflow

        This modifies common workflow parameters based on standard node IDs.
        For custom workflows, you may need to modify this method.

        Args:
            workflow: Workflow dictionary
            prompt: Positive prompt text
            negative_prompt: Negative prompt text
            steps: Number of sampling steps
            cfg_scale: CFG scale value
            seed: Random seed
            width: Image width
            height: Image height

        Returns:
            Modified workflow
        """
        # Deep copy to avoid modifying original
        import copy
        workflow = copy.deepcopy(workflow)

        # Common node ID mappings (may need adjustment per workflow)
        for node_id, node in workflow.items():
            class_type = node.get("class_type")

            # Positive prompt
            if class_type == "CLIPTextEncode" and prompt:
                if node.get("_meta", {}).get("title") == "Positive Prompt":
                    node["inputs"]["text"] = prompt

            # Negative prompt
            if class_type == "CLIPTextEncode" and negative_prompt:
                if node.get("_meta", {}).get("title") == "Negative Prompt":
                    node["inputs"]["text"] = negative_prompt

            # KSampler parameters
            if class_type == "KSampler":
                if steps is not None:
                    node["inputs"]["steps"] = steps
                if cfg_scale is not None:
                    node["inputs"]["cfg"] = cfg_scale
                if seed is not None:
                    node["inputs"]["seed"] = seed

            # Empty latent (resolution)
            if class_type == "EmptyLatentImage":
                if width is not None:
                    node["inputs"]["width"] = width
                if height is not None:
                    node["inputs"]["height"] = height

        return workflow

    def close(self) -> None:
        """Close the HTTP session"""
        self.session.close()


# ============================================================================
# Convenience Functions
# ============================================================================


def create_client(
    url: str = DEFAULT_COMFYUI_URL,
    timeout_s: float = DEFAULT_TIMEOUT_S,
) -> ComfyUIClient:
    """Create and validate a ComfyUI client

    Args:
        url: ComfyUI server URL
        timeout_s: Request timeout

    Returns:
        Initialized client

    Raises:
        ConnectionError: If ComfyUI is not accessible
    """
    client = ComfyUIClient(base_url=url, timeout_s=timeout_s)

    if not client.health_check():
        raise ConnectionError(f"ComfyUI is not accessible at {url}")

    return client
