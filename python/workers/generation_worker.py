"""Main generation worker that coordinates job execution

This worker:
- Integrates ZeroMQ server for request handling
- Manages job queue and execution
- Publishes progress updates
- Handles concurrent job processing
"""

import signal
import sys
import threading
import time
from typing import Optional

try:
    from .zmq_server import ZmqServer
    from .job_queue import JobQueue, JobStatus
    from .job_executor import JobExecutor, ExecutorConfig
    from .message_protocol import (
        serialize,
        JobCompleteResponse,
        JobErrorResponse,
    )
except ImportError:
    from zmq_server import ZmqServer
    from job_queue import JobQueue, JobStatus
    from job_executor import JobExecutor, ExecutorConfig
    from message_protocol import (
        serialize,
        JobCompleteResponse,
        JobErrorResponse,
    )


# ============================================================================
# Generation Worker
# ============================================================================


class GenerationWorker:
    """Main worker that processes generation jobs

    Architecture:
    - ZeroMQ server handles incoming requests
    - Job queue manages job lifecycle
    - Job executor runs ComfyUI workflows
    - Progress updates published via PUB-SUB
    """

    def __init__(
        self,
        req_addr: str = "tcp://127.0.0.1:5555",
        pub_addr: str = "tcp://127.0.0.1:5556",
        executor_config: Optional[ExecutorConfig] = None,
    ):
        # ZeroMQ server
        self.zmq_server = ZmqServer(req_addr=req_addr, pub_addr=pub_addr)

        # Job executor
        if executor_config is None:
            executor_config = ExecutorConfig()
        self.executor = JobExecutor(
            config=executor_config,
            update_callback=self._publish_update,
        )

        # Worker state
        self.running = False
        self.worker_thread: Optional[threading.Thread] = None

    def start(self) -> None:
        """Start the generation worker"""
        print("=" * 60)
        print("DGX-Pixels Generation Worker")
        print("=" * 60)

        # Initialize executor
        try:
            self.executor.initialize()
        except Exception as e:
            print(f"Failed to initialize executor: {e}")
            sys.exit(1)

        # Start worker thread
        self.running = True
        self.worker_thread = threading.Thread(target=self._worker_loop, daemon=True)
        self.worker_thread.start()
        print("Worker thread started")

        # Start ZeroMQ server (blocking)
        try:
            self.zmq_server.start()
        except KeyboardInterrupt:
            print("\nShutdown requested...")
            self.shutdown()

    def shutdown(self) -> None:
        """Shutdown the worker"""
        print("Shutting down worker...")

        self.running = False

        if self.worker_thread:
            self.worker_thread.join(timeout=5.0)

        self.executor.shutdown()

        print("Worker stopped")

    def _worker_loop(self) -> None:
        """Worker thread main loop

        Continuously processes jobs from the queue
        """
        print("Worker loop started")

        while self.running:
            try:
                # Get next job from queue
                job = self.zmq_server.job_queue.get_next_job()

                if job:
                    print(f"[{job.job_id}] Processing job...")

                    # Execute job
                    success, output_path, error = self.executor.execute_job(job)

                    if success:
                        # Mark job as complete
                        self.zmq_server.job_queue.complete_job(job.job_id, output_path)

                        # Publish completion response (non-blocking)
                        self._publish_response(
                            JobCompleteResponse(
                                job_id=job.job_id,
                                image_path=output_path,
                                duration_s=job.completed_at - job.started_at
                                if job.completed_at and job.started_at
                                else 0.0,
                            )
                        )
                    else:
                        # Mark job as failed
                        self.zmq_server.job_queue.fail_job(job.job_id, error or "Unknown error")

                        # Publish error response
                        self._publish_response(
                            JobErrorResponse(
                                job_id=job.job_id,
                                error=error or "Unknown error",
                            )
                        )

                else:
                    # No jobs available, sleep briefly
                    time.sleep(0.1)

            except Exception as e:
                print(f"Worker error: {e}")
                time.sleep(1.0)

        print("Worker loop stopped")

    def _publish_update(self, update: object) -> None:
        """Publish a progress update via ZeroMQ PUB-SUB

        Args:
            update: Update message to publish
        """
        if self.zmq_server.pub_socket:
            data = serialize(update)
            self.zmq_server.pub_socket.send(data)

    def _publish_response(self, response: object) -> None:
        """Publish an async response (e.g., job complete)

        These responses are published via PUB-SUB since the original
        REQ-REP exchange has already completed.

        Args:
            response: Response message to publish
        """
        self._publish_update(response)

    def get_stats(self) -> dict:
        """Get worker statistics

        Returns:
            Statistics dictionary
        """
        return {
            "running": self.running,
            "executor": self.executor.get_stats(),
            "queue": {
                "size": self.zmq_server.job_queue.queue_size(),
                "active": self.zmq_server.job_queue.active_jobs(),
            },
        }


# ============================================================================
# Main Entry Point
# ============================================================================


def main() -> None:
    """Main entry point for generation worker"""
    import argparse

    parser = argparse.ArgumentParser(description="DGX-Pixels Generation Worker")

    # ZeroMQ configuration
    parser.add_argument(
        "--req-addr",
        default="tcp://127.0.0.1:5555",
        help="REQ-REP bind address",
    )
    parser.add_argument(
        "--pub-addr",
        default="tcp://127.0.0.1:5556",
        help="PUB-SUB bind address",
    )

    # ComfyUI configuration
    parser.add_argument(
        "--comfyui-url",
        default="http://localhost:8188",
        help="ComfyUI API URL",
    )
    parser.add_argument(
        "--comfyui-timeout",
        type=float,
        default=300.0,
        help="ComfyUI request timeout (seconds)",
    )

    # Workflow configuration
    parser.add_argument(
        "--workflow-dir",
        default="/home/beengud/raibid-labs/dgx-pixels/workflows",
        help="Workflow directory",
    )
    parser.add_argument(
        "--output-dir",
        default="/home/beengud/raibid-labs/dgx-pixels/outputs",
        help="Output directory for generated images",
    )

    args = parser.parse_args()

    # Create executor config
    executor_config = ExecutorConfig(
        comfyui_url=args.comfyui_url,
        comfyui_timeout_s=args.comfyui_timeout,
        workflow_dir=args.workflow_dir,
        output_dir=args.output_dir,
    )

    # Create and start worker
    worker = GenerationWorker(
        req_addr=args.req_addr,
        pub_addr=args.pub_addr,
        executor_config=executor_config,
    )

    # Setup signal handlers
    def signal_handler(signum, frame):
        print(f"\nReceived signal {signum}")
        worker.shutdown()
        sys.exit(0)

    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)

    # Start worker
    worker.start()


if __name__ == "__main__":
    main()
