"""
Example: Integrating Prometheus Metrics into DGX-Pixels Workers

This example demonstrates how to instrument the DGX-Pixels generation pipeline
with Prometheus metrics for observability.

Usage:
    1. Start metrics server in your main application:
       python examples/metrics_integration_example.py

    2. View metrics:
       curl http://localhost:8000/metrics

    3. Run generation workload and watch metrics update in Grafana

Features Demonstrated:
  - Automatic metric tracking with context managers
  - Manual metric recording for edge cases
  - ZeroMQ message tracking
  - ComfyUI API tracking
  - Queue depth monitoring
"""

import asyncio
import time
import random
from typing import Optional

# Import metrics from the metrics module
import sys
sys.path.insert(0, '/home/beengud/raibid-labs/dgx-pixels')

from python.metrics import (
    start_metrics_server,
    track_generation,
    track_zmq_latency,
    track_comfyui_api,
    images_generated,
    generation_failures,
    active_jobs,
    queue_depth,
    set_queue_depth,
    record_zmq_message_sent,
    record_zmq_message_received,
)


################################################################################
# Example 1: Basic Generation Tracking
################################################################################

async def generate_sprite_basic(prompt: str) -> dict:
    """
    Example: Basic sprite generation with automatic metrics tracking.

    The track_generation context manager automatically:
      - Increments active_jobs on entry
      - Decrements active_jobs on exit
      - Records generation duration
      - Increments success/failure counters
    """
    # Use context manager for automatic tracking
    with track_generation(workflow="sprite_basic", model="sdxl_base"):
        # Simulate generation work
        await asyncio.sleep(random.uniform(2.0, 5.0))

        # Simulate occasional failures (10% failure rate)
        if random.random() < 0.1:
            raise Exception("Generation failed: GPU OOM")

        return {"status": "success", "prompt": prompt}


################################################################################
# Example 2: Advanced Generation with Manual Metrics
################################################################################

async def generate_sprite_advanced(prompt: str, workflow: str = "sprite_optimized") -> dict:
    """
    Example: Advanced generation with manual metric recording.

    Use this approach when you need more control over metric labels
    or want to record additional metadata.
    """
    start_time = time.perf_counter()
    active_jobs.inc()

    try:
        # Step 1: Pre-process prompt
        processed_prompt = await preprocess_prompt(prompt)

        # Step 2: Call ComfyUI with API tracking
        with track_comfyui_api("/prompt"):
            result = await call_comfyui(processed_prompt)

        # Step 3: Post-process result
        final_result = await postprocess_result(result)

        # Success - record metrics
        images_generated.labels(workflow=workflow, model="sdxl_turbo").inc()

        return final_result

    except Exception as e:
        # Failure - record error type
        error_type = type(e).__name__
        generation_failures.labels(workflow=workflow, error_type=error_type).inc()
        raise

    finally:
        # Always record duration and decrement active jobs
        duration = time.perf_counter() - start_time
        active_jobs.dec()


################################################################################
# Example 3: ZeroMQ Worker with Metrics
################################################################################

class MetricsInstrumentedWorker:
    """
    Example: ZeroMQ worker with comprehensive metrics tracking.

    This worker tracks:
      - Messages sent/received
      - Message latency
      - Queue depth
      - Generation performance
    """

    def __init__(self, endpoint: str = "tcp://127.0.0.1:5555"):
        self.endpoint = endpoint
        self.job_queue = []

    async def process_message(self, message: dict) -> dict:
        """Process incoming ZeroMQ message with latency tracking."""

        # Track ZeroMQ latency
        with track_zmq_latency(self.endpoint):
            # Record message received
            record_zmq_message_received(
                endpoint=self.endpoint,
                message_type=message.get("type", "unknown")
            )

            # Update queue depth
            self.job_queue.append(message)
            set_queue_depth(len(self.job_queue))

            # Process job
            if message["type"] == "generate":
                result = await self.handle_generation(message)
            else:
                result = {"error": "unknown message type"}

            # Update queue depth after processing
            self.job_queue.pop(0)
            set_queue_depth(len(self.job_queue))

            # Record message sent
            record_zmq_message_sent(
                endpoint=self.endpoint,
                message_type="response"
            )

            return result

    async def handle_generation(self, message: dict) -> dict:
        """Handle generation request with metrics."""
        prompt = message.get("prompt", "")
        workflow = message.get("workflow", "default")

        with track_generation(workflow=workflow, model="sdxl_base"):
            # Simulate generation
            await asyncio.sleep(random.uniform(3.0, 7.0))

            if random.random() < 0.05:  # 5% failure rate
                raise Exception("ComfyUI timeout")

            return {
                "status": "success",
                "job_id": message.get("job_id"),
                "output_path": f"/outputs/{message['job_id']}.png"
            }


################################################################################
# Example 4: Batch Processing with Metrics
################################################################################

async def batch_generate_sprites(prompts: list[str], batch_size: int = 4):
    """
    Example: Batch generation with metrics tracking.

    Tracks:
      - Individual generation metrics
      - Batch-level throughput
      - Queue depth during batch processing
    """
    total_prompts = len(prompts)
    completed = 0
    failed = 0

    # Process in batches
    for i in range(0, total_prompts, batch_size):
        batch = prompts[i:i+batch_size]

        # Update queue depth (remaining jobs)
        set_queue_depth(total_prompts - completed - failed)

        # Process batch concurrently
        tasks = [generate_sprite_basic(prompt) for prompt in batch]
        results = await asyncio.gather(*tasks, return_exceptions=True)

        # Count successes and failures
        for result in results:
            if isinstance(result, Exception):
                failed += 1
            else:
                completed += 1

    # Clear queue depth
    set_queue_depth(0)

    return {
        "total": total_prompts,
        "completed": completed,
        "failed": failed,
        "success_rate": (completed / total_prompts) * 100
    }


################################################################################
# Example 5: Monitoring Queue Depth
################################################################################

class QueueManager:
    """
    Example: Queue manager that maintains accurate queue depth metrics.
    """

    def __init__(self):
        self.pending_jobs = []
        self.processing_jobs = []

    def add_job(self, job: dict):
        """Add job to queue and update metrics."""
        self.pending_jobs.append(job)
        self._update_queue_depth()

    def get_next_job(self) -> Optional[dict]:
        """Get next job and update metrics."""
        if not self.pending_jobs:
            return None

        job = self.pending_jobs.pop(0)
        self.processing_jobs.append(job)
        self._update_queue_depth()
        return job

    def complete_job(self, job_id: str):
        """Mark job as complete and update metrics."""
        self.processing_jobs = [j for j in self.processing_jobs if j["id"] != job_id]
        self._update_queue_depth()

    def _update_queue_depth(self):
        """Update queue depth metric."""
        total_queue = len(self.pending_jobs)
        set_queue_depth(total_queue)


################################################################################
# Helper Functions (Simulated)
################################################################################

async def preprocess_prompt(prompt: str) -> str:
    """Simulate prompt preprocessing."""
    await asyncio.sleep(0.1)
    return f"pixel art, {prompt}, 16-bit style"

async def call_comfyui(prompt: str) -> dict:
    """Simulate ComfyUI API call."""
    await asyncio.sleep(random.uniform(3.0, 5.0))
    return {"image_data": "base64_encoded_image"}

async def postprocess_result(result: dict) -> dict:
    """Simulate result post-processing."""
    await asyncio.sleep(0.2)
    return {
        "status": "success",
        "output": result["image_data"],
        "metadata": {"timestamp": time.time()}
    }


################################################################################
# Main Example
################################################################################

async def main():
    """
    Main example demonstrating metrics integration.

    This runs a simulation workload and exposes metrics on port 8000.
    """
    print("Starting DGX-Pixels Metrics Integration Example")
    print("=" * 60)

    # Start metrics server
    print("\n1. Starting Prometheus metrics server on port 8000...")
    start_metrics_server(port=8000)
    print("   Metrics endpoint: http://localhost:8000/metrics")

    # Wait for server to start
    await asyncio.sleep(2)

    print("\n2. Running simulation workload...")
    print("   Watch metrics at: http://localhost:8000/metrics")
    print("   Or view in Grafana: http://localhost:3000")
    print()

    # Simulate continuous generation workload
    try:
        iteration = 0
        while True:
            iteration += 1
            print(f"\nIteration {iteration}:")

            # Generate some sprites
            prompts = [
                "medieval knight character",
                "fantasy wizard sprite",
                "pixel art tree",
                "game UI button",
            ]

            print(f"  Generating {len(prompts)} sprites...")
            result = await batch_generate_sprites(prompts, batch_size=2)

            print(f"  Results: {result['completed']} succeeded, {result['failed']} failed")
            print(f"  Success rate: {result['success_rate']:.1f}%")

            # Wait before next iteration
            await asyncio.sleep(10)

    except KeyboardInterrupt:
        print("\n\nShutting down...")
        print("Final metrics available at: http://localhost:8000/metrics")


if __name__ == "__main__":
    print(__doc__)
    print("\nPress Ctrl+C to stop\n")
    asyncio.run(main())
