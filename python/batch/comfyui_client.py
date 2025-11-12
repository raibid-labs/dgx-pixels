#!/usr/bin/env python3
"""
ComfyUI API Client for Batch Processing

Provides a Python wrapper around ComfyUI's HTTP API for:
- Submitting workflow generation jobs
- Polling job status
- Retrieving generated images
- Managing workflows with parameter injection
"""

import json
import time
import requests
import uuid
from pathlib import Path
from typing import Dict, List, Optional, Any, Tuple
from dataclasses import dataclass
from enum import Enum


class ComfyUIJobStatus(str, Enum):
    """ComfyUI job status"""
    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"


@dataclass
class ComfyUIJob:
    """Represents a ComfyUI generation job"""
    prompt_id: str
    workflow: Dict[str, Any]
    status: ComfyUIJobStatus = ComfyUIJobStatus.PENDING
    progress: float = 0.0
    error: Optional[str] = None
    output_images: List[str] = None

    def __post_init__(self):
        if self.output_images is None:
            self.output_images = []


class ComfyUIClient:
    """
    Client for ComfyUI HTTP API

    Handles workflow submission, status polling, and output retrieval.
    Supports batch generation with parameter injection.
    """

    def __init__(
        self,
        host: str = "localhost",
        port: int = 8188,
        timeout: int = 300,
        poll_interval: float = 1.0,
    ):
        """
        Initialize ComfyUI client

        Args:
            host: ComfyUI server host
            port: ComfyUI server port
            timeout: Maximum time to wait for job completion (seconds)
            poll_interval: Time between status polls (seconds)
        """
        self.host = host
        self.port = port
        self.base_url = f"http://{host}:{port}"
        self.timeout = timeout
        self.poll_interval = poll_interval
        self.client_id = str(uuid.uuid4())

    def check_health(self) -> bool:
        """Check if ComfyUI server is healthy"""
        try:
            response = requests.get(f"{self.base_url}/system_stats", timeout=5)
            return response.status_code == 200
        except Exception as e:
            return False

    def get_queue_status(self) -> Dict[str, Any]:
        """Get current queue status"""
        try:
            response = requests.get(f"{self.base_url}/queue", timeout=5)
            response.raise_for_status()
            return response.json()
        except Exception as e:
            raise RuntimeError(f"Failed to get queue status: {e}")

    def submit_workflow(
        self,
        workflow: Dict[str, Any],
        client_id: Optional[str] = None,
    ) -> str:
        """
        Submit a workflow to ComfyUI

        Args:
            workflow: ComfyUI workflow JSON
            client_id: Optional client ID for tracking

        Returns:
            prompt_id: Unique ID for this generation job
        """
        if client_id is None:
            client_id = self.client_id

        payload = {
            "prompt": workflow,
            "client_id": client_id,
        }

        try:
            response = requests.post(
                f"{self.base_url}/prompt",
                json=payload,
                timeout=10,
            )
            response.raise_for_status()
            result = response.json()

            if "prompt_id" not in result:
                raise RuntimeError(f"Invalid response from ComfyUI: {result}")

            return result["prompt_id"]

        except requests.exceptions.RequestException as e:
            raise RuntimeError(f"Failed to submit workflow: {e}")

    def get_job_status(self, prompt_id: str) -> Tuple[ComfyUIJobStatus, float]:
        """
        Get status of a job

        Args:
            prompt_id: Job ID returned from submit_workflow

        Returns:
            (status, progress): Job status and progress (0.0-1.0)
        """
        try:
            # Check queue status
            queue_data = self.get_queue_status()

            # Check if in running queue
            if "queue_running" in queue_data:
                for item in queue_data["queue_running"]:
                    if len(item) >= 2 and item[1] == prompt_id:
                        return ComfyUIJobStatus.RUNNING, 0.5

            # Check if in pending queue
            if "queue_pending" in queue_data:
                for item in queue_data["queue_pending"]:
                    if len(item) >= 2 and item[1] == prompt_id:
                        return ComfyUIJobStatus.PENDING, 0.0

            # Not in queue - check history
            history = self.get_history(prompt_id)
            if history:
                # Job completed
                if "outputs" in history:
                    return ComfyUIJobStatus.COMPLETED, 1.0
                elif "error" in history or "exception" in history:
                    return ComfyUIJobStatus.FAILED, 0.0

            # If not found anywhere, assume completed (might have been cleared)
            return ComfyUIJobStatus.COMPLETED, 1.0

        except Exception as e:
            raise RuntimeError(f"Failed to get job status: {e}")

    def get_history(self, prompt_id: str) -> Optional[Dict[str, Any]]:
        """Get job history from ComfyUI"""
        try:
            response = requests.get(
                f"{self.base_url}/history/{prompt_id}",
                timeout=5,
            )
            response.raise_for_status()
            data = response.json()

            if prompt_id in data:
                return data[prompt_id]
            return None

        except Exception as e:
            return None

    def wait_for_completion(
        self,
        prompt_id: str,
        callback: Optional[callable] = None,
    ) -> ComfyUIJob:
        """
        Wait for a job to complete

        Args:
            prompt_id: Job ID
            callback: Optional callback for progress updates (status, progress)

        Returns:
            ComfyUIJob with results

        Raises:
            TimeoutError: If job doesn't complete within timeout
            RuntimeError: If job fails
        """
        job = ComfyUIJob(prompt_id=prompt_id, workflow={})
        start_time = time.time()

        while True:
            # Check timeout
            elapsed = time.time() - start_time
            if elapsed > self.timeout:
                raise TimeoutError(f"Job {prompt_id} timed out after {elapsed:.1f}s")

            # Get status
            try:
                status, progress = self.get_job_status(prompt_id)
                job.status = status
                job.progress = progress

                if callback:
                    callback(status, progress)

                # Check if complete
                if status == ComfyUIJobStatus.COMPLETED:
                    # Get outputs
                    history = self.get_history(prompt_id)
                    if history and "outputs" in history:
                        job.output_images = self._extract_output_paths(history["outputs"])
                    return job

                elif status == ComfyUIJobStatus.FAILED:
                    history = self.get_history(prompt_id)
                    error_msg = "Unknown error"
                    if history:
                        if "error" in history:
                            error_msg = str(history["error"])
                        elif "exception" in history:
                            error_msg = str(history["exception"])
                    job.error = error_msg
                    raise RuntimeError(f"Job {prompt_id} failed: {error_msg}")

            except (TimeoutError, RuntimeError):
                raise
            except Exception as e:
                print(f"[WARNING] Error checking status: {e}")

            # Wait before next poll
            time.sleep(self.poll_interval)

    def _extract_output_paths(self, outputs: Dict[str, Any]) -> List[str]:
        """Extract output image paths from history"""
        paths = []

        for node_id, node_output in outputs.items():
            if "images" in node_output:
                for img in node_output["images"]:
                    if "filename" in img and "subfolder" in img:
                        # Construct path relative to ComfyUI output directory
                        subfolder = img["subfolder"]
                        filename = img["filename"]
                        if subfolder:
                            paths.append(f"{subfolder}/{filename}")
                        else:
                            paths.append(filename)

        return paths

    def download_image(
        self,
        filename: str,
        subfolder: str = "",
        output_path: Optional[Path] = None,
    ) -> Path:
        """
        Download an output image from ComfyUI

        Args:
            filename: Image filename
            subfolder: Subfolder in ComfyUI output directory
            output_path: Where to save the image (defaults to ./outputs/)

        Returns:
            Path to downloaded image
        """
        if output_path is None:
            output_path = Path("outputs") / filename

        output_path.parent.mkdir(parents=True, exist_ok=True)

        # Build URL
        params = {"filename": filename, "subfolder": subfolder, "type": "output"}

        try:
            response = requests.get(
                f"{self.base_url}/view",
                params=params,
                timeout=30,
            )
            response.raise_for_status()

            # Save image
            with open(output_path, "wb") as f:
                f.write(response.content)

            return output_path

        except Exception as e:
            raise RuntimeError(f"Failed to download image: {e}")

    def inject_parameters(
        self,
        workflow: Dict[str, Any],
        prompt: Optional[str] = None,
        negative_prompt: Optional[str] = None,
        seed: Optional[int] = None,
        batch_size: Optional[int] = None,
        steps: Optional[int] = None,
        cfg_scale: Optional[float] = None,
        model: Optional[str] = None,
    ) -> Dict[str, Any]:
        """
        Inject parameters into a workflow template

        This modifies the workflow JSON by updating common parameters.
        Node IDs must match the template structure.

        Args:
            workflow: Base workflow to modify
            prompt: Positive prompt text
            negative_prompt: Negative prompt text
            seed: Random seed
            batch_size: Number of images to generate
            steps: Number of sampling steps
            cfg_scale: CFG scale
            model: Model checkpoint name

        Returns:
            Modified workflow
        """
        # Deep copy to avoid modifying original
        workflow = json.loads(json.dumps(workflow))

        # Update parameters based on common node IDs
        # Note: This assumes standard workflow structure from batch_optimized.json

        # Node 1: Checkpoint loader
        if model is not None and "1" in workflow:
            if "inputs" in workflow["1"]:
                workflow["1"]["inputs"]["ckpt_name"] = model

        # Node 2: Positive prompt
        if prompt is not None and "2" in workflow:
            if "inputs" in workflow["2"]:
                workflow["2"]["inputs"]["text"] = prompt

        # Node 3: Negative prompt
        if negative_prompt is not None and "3" in workflow:
            if "inputs" in workflow["3"]:
                workflow["3"]["inputs"]["text"] = negative_prompt

        # Node 4: Empty latent (batch size)
        if batch_size is not None and "4" in workflow:
            if "inputs" in workflow["4"]:
                workflow["4"]["inputs"]["batch_size"] = batch_size

        # Node 5: KSampler
        if "5" in workflow and "inputs" in workflow["5"]:
            if seed is not None:
                workflow["5"]["inputs"]["seed"] = seed
            if steps is not None:
                workflow["5"]["inputs"]["steps"] = steps
            if cfg_scale is not None:
                workflow["5"]["inputs"]["cfg"] = cfg_scale

        return workflow

    def generate_batch(
        self,
        workflow: Dict[str, Any],
        prompts: List[str],
        batch_size: int = 1,
        wait: bool = True,
        progress_callback: Optional[callable] = None,
    ) -> List[ComfyUIJob]:
        """
        Generate multiple images from a list of prompts

        Args:
            workflow: Base workflow template
            prompts: List of prompts to generate
            batch_size: Batch size per workflow execution
            wait: Wait for all jobs to complete
            progress_callback: Progress callback

        Returns:
            List of completed jobs
        """
        jobs = []

        for i, prompt in enumerate(prompts):
            # Inject parameters
            modified_workflow = self.inject_parameters(
                workflow,
                prompt=prompt,
                batch_size=batch_size,
                seed=int(time.time() * 1000) + i,  # Unique seed
            )

            # Submit job
            prompt_id = self.submit_workflow(modified_workflow)
            print(f"[BATCH] Submitted job {i+1}/{len(prompts)}: {prompt_id}")

            if wait:
                # Wait for completion
                try:
                    job = self.wait_for_completion(prompt_id, progress_callback)
                    jobs.append(job)
                    print(f"[BATCH] Completed job {i+1}/{len(prompts)}: {len(job.output_images)} images")
                except Exception as e:
                    print(f"[BATCH] Job {i+1} failed: {e}")
                    jobs.append(ComfyUIJob(
                        prompt_id=prompt_id,
                        workflow=modified_workflow,
                        status=ComfyUIJobStatus.FAILED,
                        error=str(e),
                    ))
            else:
                jobs.append(ComfyUIJob(
                    prompt_id=prompt_id,
                    workflow=modified_workflow,
                ))

        return jobs


if __name__ == "__main__":
    # Self-test
    print("=== ComfyUI Client Self-Test ===\n")

    client = ComfyUIClient()

    # Test 1: Health check
    print("Test 1: Health check...")
    if client.check_health():
        print("✅ ComfyUI server is healthy")
    else:
        print("❌ ComfyUI server is not responding")
        print("Make sure ComfyUI is running on http://localhost:8188")
        exit(1)

    # Test 2: Queue status
    print("\nTest 2: Queue status...")
    try:
        queue = client.get_queue_status()
        print(f"✅ Queue status: {json.dumps(queue, indent=2)}")
    except Exception as e:
        print(f"❌ Failed to get queue status: {e}")

    # Test 3: Load and inject workflow
    print("\nTest 3: Workflow parameter injection...")
    workflow_path = Path(__file__).parent.parent.parent / "workflows" / "batch_optimized.json"
    if workflow_path.exists():
        with open(workflow_path) as f:
            workflow = json.load(f)

        modified = client.inject_parameters(
            workflow,
            prompt="test pixel art sprite",
            batch_size=2,
            seed=42,
        )

        # Verify modifications
        assert modified["2"]["inputs"]["text"] == "test pixel art sprite"
        assert modified["4"]["inputs"]["batch_size"] == 2
        assert modified["5"]["inputs"]["seed"] == 42
        print("✅ Workflow parameters injected successfully")
    else:
        print(f"⚠️  Workflow not found: {workflow_path}")

    print("\n✅ Self-test complete")
