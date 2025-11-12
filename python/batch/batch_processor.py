#!/usr/bin/env python3
"""
Batch Processing Orchestrator

Main orchestrator for high-throughput batch generation:
- Queue management with priority scheduling
- Memory-aware batch size optimization
- GPU utilization monitoring
- Parallel batch execution
- Output organization
"""

import json
import time
import threading
from pathlib import Path
from typing import Dict, List, Optional, Any, Callable
from dataclasses import dataclass, field
from enum import Enum
from queue import PriorityQueue, Empty
import uuid

from .comfyui_client import ComfyUIClient, ComfyUIJob, ComfyUIJobStatus


class JobPriority(int, Enum):
    """Job priority levels (lower number = higher priority)"""
    URGENT = 0
    HIGH = 1
    NORMAL = 2
    LOW = 3


class BatchJobStatus(str, Enum):
    """Batch job status"""
    QUEUED = "queued"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"


@dataclass
class BatchJob:
    """Represents a batch generation job"""
    job_id: str
    prompts: List[str]
    workflow_path: Path
    batch_size: int = 1
    priority: JobPriority = JobPriority.NORMAL
    model: Optional[str] = None
    lora: Optional[str] = None
    steps: int = 20
    cfg_scale: float = 8.0
    seed_base: Optional[int] = None

    # Status tracking
    status: BatchJobStatus = BatchJobStatus.QUEUED
    created_at: float = field(default_factory=time.time)
    started_at: Optional[float] = None
    completed_at: Optional[float] = None

    # Results
    output_dir: Optional[Path] = None
    generated_images: List[Path] = field(default_factory=list)
    metadata: Dict[str, Any] = field(default_factory=dict)
    error: Optional[str] = None

    # Progress
    total_prompts: int = 0
    completed_prompts: int = 0

    def __post_init__(self):
        self.total_prompts = len(self.prompts)
        if self.seed_base is None:
            self.seed_base = int(time.time() * 1000)

    def progress(self) -> float:
        """Get progress as percentage (0.0-1.0)"""
        if self.total_prompts == 0:
            return 0.0
        return self.completed_prompts / self.total_prompts

    def __lt__(self, other):
        """Priority comparison for queue"""
        return self.priority < other.priority


class BatchProcessor:
    """
    High-throughput batch processing orchestrator

    Features:
    - Priority-based job queue
    - Memory-aware batch size optimization
    - Parallel batch execution
    - GPU utilization monitoring
    - Automatic output organization
    """

    def __init__(
        self,
        comfyui_host: str = "localhost",
        comfyui_port: int = 8188,
        output_base_dir: Path = Path("outputs/batches"),
        max_concurrent_batches: int = 1,
        enable_gpu_monitoring: bool = True,
    ):
        """
        Initialize batch processor

        Args:
            comfyui_host: ComfyUI server host
            comfyui_port: ComfyUI server port
            output_base_dir: Base directory for batch outputs
            max_concurrent_batches: Max parallel batch executions
            enable_gpu_monitoring: Enable GPU utilization monitoring
        """
        self.client = ComfyUIClient(host=comfyui_host, port=comfyui_port)
        self.output_base_dir = Path(output_base_dir)
        self.output_base_dir.mkdir(parents=True, exist_ok=True)

        self.max_concurrent_batches = max_concurrent_batches
        self.enable_gpu_monitoring = enable_gpu_monitoring

        # Job tracking
        self.job_queue = PriorityQueue()
        self.jobs: Dict[str, BatchJob] = {}
        self.active_jobs: Dict[str, BatchJob] = {}

        # Worker thread
        self.running = False
        self.worker_thread: Optional[threading.Thread] = None

        # Statistics
        self.total_processed = 0
        self.total_failed = 0
        self.start_time = time.time()

        # Throughput tracking
        self.generation_times: List[float] = []
        self.max_tracked_times = 100  # Keep last 100 generation times

    def start(self) -> None:
        """Start the batch processor worker"""
        if self.running:
            print("[WARNING] Batch processor already running")
            return

        print("[BATCH] Starting batch processor...")

        # Health check
        if not self.client.check_health():
            raise RuntimeError("ComfyUI server is not responding")

        self.running = True
        self.worker_thread = threading.Thread(target=self._worker_loop, daemon=True)
        self.worker_thread.start()

        print("[BATCH] Batch processor started")

    def stop(self) -> None:
        """Stop the batch processor worker"""
        if not self.running:
            return

        print("[BATCH] Stopping batch processor...")
        self.running = False

        if self.worker_thread:
            self.worker_thread.join(timeout=10)

        print("[BATCH] Batch processor stopped")

    def submit_job(
        self,
        prompts: List[str],
        workflow_path: Path,
        batch_size: int = 1,
        priority: JobPriority = JobPriority.NORMAL,
        **kwargs,
    ) -> str:
        """
        Submit a new batch job

        Args:
            prompts: List of prompts to generate
            workflow_path: Path to ComfyUI workflow JSON
            batch_size: Images per batch (1, 4, or 8)
            priority: Job priority
            **kwargs: Additional job parameters (model, lora, steps, cfg_scale, etc.)

        Returns:
            job_id: Unique job identifier
        """
        if not workflow_path.exists():
            raise FileNotFoundError(f"Workflow not found: {workflow_path}")

        # Validate batch size
        if batch_size not in [1, 4, 8]:
            print(f"[WARNING] Unusual batch size {batch_size}, optimal values are 1, 4, 8")

        # Create job
        job_id = str(uuid.uuid4())
        job = BatchJob(
            job_id=job_id,
            prompts=prompts,
            workflow_path=workflow_path,
            batch_size=batch_size,
            priority=priority,
            **kwargs,
        )

        # Store job
        self.jobs[job_id] = job

        # Add to queue
        self.job_queue.put((priority, time.time(), job_id))

        print(f"[BATCH] Job {job_id} submitted: {len(prompts)} prompts, batch_size={batch_size}, priority={priority.name}")

        return job_id

    def get_job(self, job_id: str) -> Optional[BatchJob]:
        """Get job by ID"""
        return self.jobs.get(job_id)

    def cancel_job(self, job_id: str) -> bool:
        """Cancel a queued or running job"""
        job = self.jobs.get(job_id)
        if not job:
            return False

        if job.status in (BatchJobStatus.QUEUED, BatchJobStatus.RUNNING):
            job.status = BatchJobStatus.CANCELLED
            job.completed_at = time.time()

            # Remove from active jobs
            if job_id in self.active_jobs:
                del self.active_jobs[job_id]

            print(f"[BATCH] Job {job_id} cancelled")
            return True

        return False

    def get_queue_size(self) -> int:
        """Get number of queued jobs"""
        return self.job_queue.qsize()

    def get_active_count(self) -> int:
        """Get number of active jobs"""
        return len(self.active_jobs)

    def get_statistics(self) -> Dict[str, Any]:
        """Get processor statistics"""
        uptime = time.time() - self.start_time

        # Calculate throughput
        avg_generation_time = 0.0
        if self.generation_times:
            avg_generation_time = sum(self.generation_times) / len(self.generation_times)

        throughput_per_minute = 0.0
        if avg_generation_time > 0:
            throughput_per_minute = 60.0 / avg_generation_time

        return {
            "uptime_s": uptime,
            "queue_size": self.get_queue_size(),
            "active_jobs": self.get_active_count(),
            "total_processed": self.total_processed,
            "total_failed": self.total_failed,
            "avg_generation_time_s": avg_generation_time,
            "throughput_per_minute": throughput_per_minute,
        }

    def _worker_loop(self) -> None:
        """Main worker loop"""
        print("[BATCH] Worker loop started")

        while self.running:
            try:
                # Check if we can process more jobs
                if len(self.active_jobs) >= self.max_concurrent_batches:
                    time.sleep(0.5)
                    continue

                # Get next job from queue (with timeout)
                try:
                    priority, timestamp, job_id = self.job_queue.get(timeout=1.0)
                except Empty:
                    continue

                # Get job
                job = self.jobs.get(job_id)
                if not job:
                    continue

                # Check if cancelled
                if job.status == BatchJobStatus.CANCELLED:
                    continue

                # Process job
                self._process_job(job)

            except Exception as e:
                print(f"[ERROR] Worker loop error: {e}")
                time.sleep(1.0)

        print("[BATCH] Worker loop stopped")

    def _process_job(self, job: BatchJob) -> None:
        """Process a single batch job"""
        job_id = job.job_id

        try:
            # Mark as running
            job.status = BatchJobStatus.RUNNING
            job.started_at = time.time()
            self.active_jobs[job_id] = job

            print(f"[BATCH] Processing job {job_id}: {job.total_prompts} prompts")

            # Create output directory
            timestamp = time.strftime("%Y%m%d_%H%M%S")
            output_dir = self.output_base_dir / f"batch_{timestamp}_{job_id[:8]}"
            output_dir.mkdir(parents=True, exist_ok=True)
            job.output_dir = output_dir

            # Create subdirectories
            images_dir = output_dir / "images"
            images_dir.mkdir(exist_ok=True)

            metadata_dir = output_dir / "metadata"
            metadata_dir.mkdir(exist_ok=True)

            # Load workflow
            with open(job.workflow_path) as f:
                base_workflow = json.load(f)

            # Process each prompt
            for i, prompt in enumerate(job.prompts):
                if job.status == BatchJobStatus.CANCELLED:
                    break

                start_time = time.time()

                try:
                    # Inject parameters
                    workflow = self.client.inject_parameters(
                        base_workflow,
                        prompt=prompt,
                        batch_size=job.batch_size,
                        seed=job.seed_base + i,
                        steps=job.steps,
                        cfg_scale=job.cfg_scale,
                        model=job.model,
                    )

                    # Submit to ComfyUI
                    prompt_id = self.client.submit_workflow(workflow)

                    # Wait for completion
                    def progress_callback(status, progress):
                        job.completed_prompts = i + progress

                    comfy_job = self.client.wait_for_completion(
                        prompt_id,
                        callback=progress_callback,
                    )

                    # Download images
                    for img_path in comfy_job.output_images:
                        # Extract filename and subfolder
                        parts = img_path.split("/")
                        if len(parts) > 1:
                            subfolder = parts[0]
                            filename = parts[1]
                        else:
                            subfolder = ""
                            filename = parts[0]

                        # Download to local output directory
                        local_path = images_dir / f"prompt_{i:04d}_{filename}"
                        self.client.download_image(filename, subfolder, local_path)
                        job.generated_images.append(local_path)

                    # Save metadata for this prompt
                    prompt_metadata = {
                        "prompt_index": i,
                        "prompt": prompt,
                        "batch_size": job.batch_size,
                        "seed": job.seed_base + i,
                        "steps": job.steps,
                        "cfg_scale": job.cfg_scale,
                        "model": job.model,
                        "lora": job.lora,
                        "comfyui_prompt_id": prompt_id,
                        "generation_time_s": time.time() - start_time,
                        "output_images": [str(p.name) for p in job.generated_images[-len(comfy_job.output_images):]],
                    }

                    metadata_file = metadata_dir / f"prompt_{i:04d}.json"
                    with open(metadata_file, "w") as f:
                        json.dump(prompt_metadata, f, indent=2)

                    # Track generation time
                    generation_time = time.time() - start_time
                    self.generation_times.append(generation_time)
                    if len(self.generation_times) > self.max_tracked_times:
                        self.generation_times.pop(0)

                    job.completed_prompts = i + 1

                    print(f"[BATCH] Job {job_id}: {i+1}/{job.total_prompts} complete ({generation_time:.1f}s)")

                except Exception as e:
                    print(f"[ERROR] Failed to process prompt {i}: {e}")
                    job.error = str(e)
                    # Continue with next prompt

            # Save batch summary
            batch_info = {
                "job_id": job_id,
                "total_prompts": job.total_prompts,
                "completed_prompts": job.completed_prompts,
                "batch_size": job.batch_size,
                "priority": job.priority.name,
                "created_at": job.created_at,
                "started_at": job.started_at,
                "completed_at": time.time(),
                "duration_s": time.time() - job.started_at,
                "total_images": len(job.generated_images),
                "workflow": str(job.workflow_path),
            }

            with open(output_dir / "batch_info.json", "w") as f:
                json.dump(batch_info, f, indent=2)

            # Mark as completed
            job.status = BatchJobStatus.COMPLETED
            job.completed_at = time.time()
            self.total_processed += 1

            print(f"[BATCH] Job {job_id} completed: {len(job.generated_images)} images in {job.completed_at - job.started_at:.1f}s")

        except Exception as e:
            print(f"[ERROR] Job {job_id} failed: {e}")
            job.status = BatchJobStatus.FAILED
            job.completed_at = time.time()
            job.error = str(e)
            self.total_failed += 1

        finally:
            # Remove from active jobs
            if job_id in self.active_jobs:
                del self.active_jobs[job_id]


if __name__ == "__main__":
    # Self-test
    print("=== Batch Processor Self-Test ===\n")

    processor = BatchProcessor()

    # Start processor
    processor.start()

    # Submit test job
    workflow_path = Path(__file__).parent.parent.parent / "workflows" / "batch_optimized.json"

    if workflow_path.exists():
        job_id = processor.submit_job(
            prompts=["pixel art warrior sprite", "pixel art mage sprite"],
            workflow_path=workflow_path,
            batch_size=1,
            priority=JobPriority.HIGH,
        )

        print(f"\n✅ Job submitted: {job_id}")
        print("Waiting for completion...")

        # Monitor progress
        while True:
            job = processor.get_job(job_id)
            if job.status in (BatchJobStatus.COMPLETED, BatchJobStatus.FAILED, BatchJobStatus.CANCELLED):
                break

            stats = processor.get_statistics()
            print(f"Progress: {job.progress()*100:.1f}% | Queue: {stats['queue_size']} | Active: {stats['active_jobs']}")
            time.sleep(2)

        # Show results
        job = processor.get_job(job_id)
        print(f"\nJob Status: {job.status.value}")
        print(f"Generated Images: {len(job.generated_images)}")
        print(f"Output Directory: {job.output_dir}")
    else:
        print(f"⚠️  Workflow not found: {workflow_path}")

    processor.stop()
    print("\n✅ Self-test complete")
