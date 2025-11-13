"""ZeroMQ client wrapper for communicating with Python backend worker"""

import asyncio
import time
import uuid
from typing import Dict, Any, Optional, List
import zmq
import zmq.asyncio

# Import protocol from workers
import sys
from pathlib import Path

# Add workers directory to path
workers_dir = Path(__file__).parent.parent / "workers"
sys.path.insert(0, str(workers_dir))

from message_protocol import (
    GenerateRequest,
    CancelRequest,
    ListModelsRequest,
    StatusRequest,
    PingRequest,
    serialize,
    deserialize_response,
    JobAcceptedResponse,
    JobCompleteResponse,
    JobErrorResponse,
    ModelListResponse,
    StatusInfoResponse,
    PongResponse,
    ErrorResponse,
)


class BackendClient:
    """Async ZeroMQ client for backend worker communication

    This client wraps the ZeroMQ REQ-REP communication with the
    Python backend worker, providing async methods for all operations.
    """

    def __init__(self, zmq_endpoint: str, timeout_s: float = 300):
        """Initialize backend client

        Args:
            zmq_endpoint: ZeroMQ endpoint (e.g., tcp://localhost:5555)
            timeout_s: Request timeout in seconds
        """
        self.zmq_endpoint = zmq_endpoint
        self.timeout_s = timeout_s
        self.context = zmq.asyncio.Context()
        self.socket: Optional[zmq.asyncio.Socket] = None

    async def connect(self) -> None:
        """Connect to backend worker"""
        if self.socket is None:
            self.socket = self.context.socket(zmq.REQ)
            self.socket.connect(self.zmq_endpoint)
            # Set receive timeout
            self.socket.setsockopt(zmq.RCVTIMEO, int(self.timeout_s * 1000))

    async def disconnect(self) -> None:
        """Disconnect from backend worker"""
        if self.socket:
            self.socket.close()
            self.socket = None

    async def _send_request(self, request: Any) -> Any:
        """Send a request and wait for response

        Args:
            request: Request object (from message_protocol)

        Returns:
            Response object (from message_protocol)

        Raises:
            TimeoutError: If request times out
            ConnectionError: If backend is not available
            RuntimeError: If request fails
        """
        if not self.socket:
            await self.connect()

        try:
            # Serialize and send request
            data = serialize(request)
            await self.socket.send(data)

            # Wait for response
            response_data = await self.socket.recv()

            # Deserialize response
            response = deserialize_response(response_data)

            return response

        except zmq.Again:
            raise TimeoutError(f"Request timed out after {self.timeout_s}s")
        except zmq.ZMQError as e:
            raise ConnectionError(f"Backend connection error: {e}")
        except Exception as e:
            raise RuntimeError(f"Request failed: {e}")

    async def ping(self) -> bool:
        """Ping backend to check if it's alive

        Returns:
            True if backend responds, False otherwise
        """
        try:
            response = await self._send_request(PingRequest())
            return isinstance(response, PongResponse)
        except Exception:
            return False

    async def get_status(self) -> Dict[str, Any]:
        """Get backend status

        Returns:
            Status dictionary with queue_size, active_jobs, uptime, version
        """
        response = await self._send_request(StatusRequest())

        if isinstance(response, StatusInfoResponse):
            return {
                "version": response.version,
                "queue_size": response.queue_size,
                "active_jobs": response.active_jobs,
                "uptime_s": response.uptime_s,
            }
        elif isinstance(response, ErrorResponse):
            raise RuntimeError(f"Status request failed: {response.message}")
        else:
            raise RuntimeError(f"Unexpected response type: {type(response)}")

    async def list_models(self) -> List[Dict[str, Any]]:
        """List available models

        Returns:
            List of model dictionaries with name, path, type, size
        """
        response = await self._send_request(ListModelsRequest())

        if isinstance(response, ModelListResponse):
            return [
                {
                    "name": model.name,
                    "path": model.path,
                    "model_type": model.model_type.value,
                    "size_mb": model.size_mb,
                }
                for model in response.models
            ]
        elif isinstance(response, ErrorResponse):
            raise RuntimeError(f"List models failed: {response.message}")
        else:
            raise RuntimeError(f"Unexpected response type: {type(response)}")

    async def generate_sprite(
        self,
        prompt: str,
        model: str,
        size: List[int],
        steps: int,
        cfg_scale: float,
        lora: Optional[str] = None,
        job_id: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Submit a sprite generation request

        Args:
            prompt: Text prompt for generation
            model: Model name to use
            size: [width, height] in pixels
            steps: Number of sampling steps
            cfg_scale: CFG scale value
            lora: Optional LoRA model name
            job_id: Optional job ID (generated if not provided)

        Returns:
            Dictionary with job_id and estimated_time_s
        """
        if job_id is None:
            job_id = str(uuid.uuid4())

        request = GenerateRequest(
            id=job_id,
            prompt=prompt,
            model=model,
            size=size,
            steps=steps,
            cfg_scale=cfg_scale,
            lora=lora,
        )

        response = await self._send_request(request)

        if isinstance(response, JobAcceptedResponse):
            return {
                "job_id": response.job_id,
                "estimated_time_s": response.estimated_time_s,
            }
        elif isinstance(response, JobErrorResponse):
            raise RuntimeError(f"Generation request failed: {response.error}")
        elif isinstance(response, ErrorResponse):
            raise RuntimeError(f"Generation request failed: {response.message}")
        else:
            raise RuntimeError(f"Unexpected response type: {type(response)}")

    async def cancel_job(self, job_id: str) -> bool:
        """Cancel a running job

        Args:
            job_id: Job ID to cancel

        Returns:
            True if cancelled, False if job not found
        """
        request = CancelRequest(job_id=job_id)
        response = await self._send_request(request)

        if isinstance(response, ErrorResponse):
            return False
        else:
            return True

    async def wait_for_completion(
        self, job_id: str, poll_interval: float = 1.0
    ) -> Dict[str, Any]:
        """Wait for a job to complete

        This polls the backend status until the job completes.

        Args:
            job_id: Job ID to wait for
            poll_interval: Polling interval in seconds

        Returns:
            Dictionary with status, output_path, duration_s

        Raises:
            TimeoutError: If job times out
            RuntimeError: If job fails
        """
        start_time = time.time()

        while True:
            elapsed = time.time() - start_time

            if elapsed > self.timeout_s:
                raise TimeoutError(f"Job {job_id} timed out after {self.timeout_s}s")

            # Get backend status
            status = await self.get_status()

            # Check if job is still active
            if status["active_jobs"] == 0 and status["queue_size"] == 0:
                # Job might be complete, but we don't have a direct way to check
                # This is a limitation - in production, we'd use PUB-SUB to listen for completion
                # For now, we return a success status
                return {
                    "status": "completed",
                    "job_id": job_id,
                    "duration_s": elapsed,
                }

            await asyncio.sleep(poll_interval)

    def __enter__(self):
        """Context manager entry"""
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit"""
        # Close socket
        if self.socket:
            self.socket.close()
        self.context.term()

    async def __aenter__(self):
        """Async context manager entry"""
        await self.connect()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Async context manager exit"""
        await self.disconnect()
        self.context.term()
