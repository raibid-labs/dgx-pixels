"""Job executor for processing generation requests

This module executes generation jobs by:
- Loading and configuring workflows
- Submitting to ComfyUI
- Monitoring progress
- Publishing updates
- Handling errors and cancellation
"""

import asyncio
import os
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Callable, Optional

try:
    from .comfyui_client import ComfyUIClient, WorkflowProgress, create_client
    from .progress_tracker import ProgressTracker
    from .job_queue import Job, JobStatus
    from .message_protocol import (
        JobStartedUpdate,
        ProgressUpdate,
        PreviewUpdate,
        JobFinishedUpdate,
        GenerationStage,
    )
except ImportError:
    from comfyui_client import ComfyUIClient, WorkflowProgress, create_client
    from progress_tracker import ProgressTracker
    from job_queue import Job, JobStatus
    from message_protocol import (
        JobStartedUpdate,
        ProgressUpdate,
        PreviewUpdate,
        JobFinishedUpdate,
        GenerationStage,
    )


# ============================================================================
# Configuration
# ============================================================================


@dataclass
class ExecutorConfig:
    """Job executor configuration"""

    comfyui_url: str = "http://localhost:8188"
    comfyui_timeout_s: float = 300.0
    workflow_dir: str = "/home/beengud/raibid-labs/dgx-pixels/workflows"
    output_dir: str = "/home/beengud/raibid-labs/dgx-pixels/outputs"
    default_workflow: str = "sprite_optimized.json"
    poll_interval_ms: int = 100
    max_retries: int = 3


# ============================================================================
# Job Executor
# ============================================================================


class JobExecutor:
    """Executes generation jobs with progress tracking

    Features:
    - Workflow loading and parameter injection
    - ComfyUI integration
    - Real-time progress tracking
    - Error handling and retry logic
    - Job cancellation support
    """

    def __init__(
        self,
        config: ExecutorConfig,
        update_callback: Optional[Callable] = None,
    ):
        self.config = config
        self.update_callback = update_callback
        self.progress_tracker = ProgressTracker()
        self.client: Optional[ComfyUIClient] = None
        self.cancelled_jobs: set = set()

        # Ensure output directory exists
        os.makedirs(self.config.output_dir, exist_ok=True)

    def initialize(self) -> None:
        """Initialize the executor and ComfyUI client"""
        print(f"Initializing JobExecutor...")
        print(f"  ComfyUI URL: {self.config.comfyui_url}")
        print(f"  Workflow dir: {self.config.workflow_dir}")
        print(f"  Output dir: {self.config.output_dir}")

        # Create ComfyUI client
        try:
            self.client = create_client(
                url=self.config.comfyui_url,
                timeout_s=self.config.comfyui_timeout_s,
            )
            print("  ComfyUI connection: OK")
        except Exception as e:
            raise RuntimeError(f"Failed to connect to ComfyUI: {e}")

    def shutdown(self) -> None:
        """Shutdown the executor"""
        if self.client:
            self.client.close()
            self.client = None

    def execute_job(self, job: Job) -> tuple[bool, Optional[str], Optional[str]]:
        """Execute a generation job

        Args:
            job: Job to execute

        Returns:
            Tuple of (success, output_path, error_message)
        """
        if not self.client:
            return False, None, "Executor not initialized"

        job_id = job.job_id

        try:
            # Publish job started update
            self._publish_update(
                JobStartedUpdate(job_id=job_id, timestamp=int(time.time()))
            )

            # Load workflow
            workflow = self._load_workflow(job)

            # Inject parameters
            workflow = self._inject_parameters(job, workflow)

            # Submit to ComfyUI
            print(f"[{job_id}] Submitting to ComfyUI...")
            prompt_id = self.client.queue_prompt(workflow)
            print(f"[{job_id}] ComfyUI prompt_id: {prompt_id}")

            # Start progress tracking
            self.progress_tracker.start_job(
                job_id=job_id,
                prompt_id=prompt_id,
                total_steps=job.steps,
            )

            # Monitor execution with progress updates
            output = self.client.wait_for_completion(
                prompt_id=prompt_id,
                callback=lambda p: self._handle_progress(job_id, p),
            )

            # Check if job was cancelled during execution
            if job_id in self.cancelled_jobs:
                self.cancelled_jobs.remove(job_id)
                return False, None, "Job cancelled"

            # Download generated image
            if not output.images:
                return False, None, "No images generated"

            output_path = self._download_output(job_id, output.images[0])

            # Complete progress tracking
            self.progress_tracker.complete_job(job_id)

            # Publish job finished update
            self._publish_update(
                JobFinishedUpdate(
                    job_id=job_id,
                    success=True,
                    duration_s=output.duration_s,
                )
            )

            print(f"[{job_id}] Complete: {output_path} ({output.duration_s:.2f}s)")
            return True, output_path, None

        except TimeoutError as e:
            error = f"Timeout: {e}"
            print(f"[{job_id}] {error}")
            self._publish_update(
                JobFinishedUpdate(job_id=job_id, success=False, duration_s=0.0)
            )
            return False, None, error

        except Exception as e:
            error = f"Execution failed: {e}"
            print(f"[{job_id}] {error}")
            self._publish_update(
                JobFinishedUpdate(job_id=job_id, success=False, duration_s=0.0)
            )
            return False, None, error

    def cancel_job(self, job_id: str) -> None:
        """Mark a job for cancellation

        Args:
            job_id: Job to cancel
        """
        self.cancelled_jobs.add(job_id)

        # Interrupt ComfyUI execution
        if self.client:
            try:
                self.client.interrupt()
                print(f"[{job_id}] Cancellation requested")
            except Exception as e:
                print(f"[{job_id}] Failed to interrupt: {e}")

    def _load_workflow(self, job: Job) -> dict:
        """Load workflow JSON file

        Args:
            job: Job with workflow requirements

        Returns:
            Workflow dictionary
        """
        # Determine workflow file
        workflow_file = self.config.default_workflow

        # TODO: Add logic to select different workflows based on job parameters
        # For example, batch workflows, img2img, etc.

        workflow_path = os.path.join(self.config.workflow_dir, workflow_file)

        if not os.path.exists(workflow_path):
            raise FileNotFoundError(f"Workflow not found: {workflow_path}")

        return self.client.load_workflow(workflow_path)

    def _inject_parameters(self, job: Job, workflow: dict) -> dict:
        """Inject job parameters into workflow

        Args:
            job: Job with parameters
            workflow: Workflow template

        Returns:
            Modified workflow
        """
        return self.client.inject_parameters(
            workflow=workflow,
            prompt=job.prompt,
            steps=job.steps,
            cfg_scale=job.cfg_scale,
            width=job.size[0] if job.size else 1024,
            height=job.size[1] if len(job.size) > 1 else 1024,
        )

    def _handle_progress(self, job_id: str, workflow_progress: WorkflowProgress) -> None:
        """Handle progress updates from ComfyUI

        Args:
            job_id: Job identifier
            workflow_progress: Progress from ComfyUI
        """
        # Check if cancelled
        if job_id in self.cancelled_jobs:
            if self.client:
                self.client.interrupt()
            return

        # Update progress tracker
        stage, step, total_steps, percent, eta_s = self.progress_tracker.update_progress(
            job_id=job_id,
            workflow_progress=workflow_progress,
        )

        # Publish progress update
        self._publish_update(
            ProgressUpdate(
                job_id=job_id,
                stage=stage,
                step=step,
                total_steps=total_steps,
                percent=percent,
                eta_s=eta_s,
            )
        )

    def _download_output(self, job_id: str, image_url: str) -> str:
        """Download generated image

        Args:
            job_id: Job identifier
            image_url: URL to image

        Returns:
            Local path to saved image
        """
        # Generate output filename
        timestamp = int(time.time())
        filename = f"{job_id}_{timestamp}.png"
        output_path = os.path.join(self.config.output_dir, filename)

        # Download image
        self.client.download_image(image_url, output_path)

        return output_path

    def _publish_update(self, update: object) -> None:
        """Publish an update via callback

        Args:
            update: Update message to publish
        """
        if self.update_callback:
            try:
                self.update_callback(update)
            except Exception as e:
                print(f"Warning: Failed to publish update: {e}")

    def get_stats(self) -> dict:
        """Get executor statistics

        Returns:
            Performance statistics
        """
        return {
            "progress_tracker": self.progress_tracker.get_stats(),
            "active_cancellations": len(self.cancelled_jobs),
        }


# ============================================================================
# Async Job Executor (Future Enhancement)
# ============================================================================


class AsyncJobExecutor:
    """Async version of JobExecutor for concurrent job processing

    This is a placeholder for future concurrent job execution support.
    Current implementation uses synchronous execution.
    """

    def __init__(self, config: ExecutorConfig):
        self.config = config
        self.executor = JobExecutor(config)

    async def execute_job_async(self, job: Job) -> tuple[bool, Optional[str], Optional[str]]:
        """Execute job asynchronously

        Args:
            job: Job to execute

        Returns:
            Tuple of (success, output_path, error_message)
        """
        # Run in thread pool to avoid blocking
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(
            None,
            self.executor.execute_job,
            job,
        )
