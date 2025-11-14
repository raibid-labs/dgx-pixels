"""ZeroMQ server for handling backend requests and publishing updates"""

import time
import zmq
import signal
import sys
from pathlib import Path
from typing import Optional, List

try:
    from .message_protocol import (
        # Protocol constants
        DEFAULT_REQ_REP_ADDR,
        DEFAULT_PUB_SUB_ADDR,
        PROTOCOL_VERSION,
        # Requests
        GenerateRequest,
        CancelRequest,
        ListModelsRequest,
        StatusRequest,
        PingRequest,
        deserialize_request,
        # Responses
        JobAcceptedResponse,
        JobCompleteResponse,
        JobErrorResponse,
        JobCancelledResponse,
        ModelListResponse,
        ModelInfo,
        StatusInfoResponse,
        PongResponse,
        ErrorResponse,
        serialize,
        # Model types
        ModelType,
        # Updates
        JobStartedUpdate,
        ProgressUpdate,
        GenerationStage,
    )
    from .job_queue import JobQueue
except ImportError:
    # Allow running as standalone script
    from message_protocol import (
        DEFAULT_REQ_REP_ADDR,
        DEFAULT_PUB_SUB_ADDR,
        PROTOCOL_VERSION,
        GenerateRequest,
        CancelRequest,
        ListModelsRequest,
        StatusRequest,
        PingRequest,
        deserialize_request,
        JobAcceptedResponse,
        JobCompleteResponse,
        JobErrorResponse,
        JobCancelledResponse,
        ModelListResponse,
        ModelInfo,
        StatusInfoResponse,
        PongResponse,
        ErrorResponse,
        serialize,
        ModelType,
        JobStartedUpdate,
        ProgressUpdate,
        GenerationStage,
    )
    from job_queue import JobQueue


class ZmqServer:
    """ZeroMQ server for backend communication

    Implements:
    - REP socket for request/response (REQ-REP pattern)
    - PUB socket for progress updates (PUB-SUB pattern)
    """

    # Model directory paths
    COMFYUI_BASE = Path.home() / "ComfyUI"
    CHECKPOINT_DIR = COMFYUI_BASE / "models" / "checkpoints"
    LORA_DIR = COMFYUI_BASE / "models" / "loras"
    VAE_DIR = COMFYUI_BASE / "models" / "vae"

    # Supported file extensions
    MODEL_EXTENSIONS = {".safetensors", ".ckpt", ".pt", ".pth"}

    def __init__(
        self, req_addr: str = DEFAULT_REQ_REP_ADDR, pub_addr: str = DEFAULT_PUB_SUB_ADDR
    ) -> None:
        self.req_addr = req_addr
        self.pub_addr = pub_addr
        self.job_queue = JobQueue()
        self.start_time = time.time()
        self.running = False

        # ZeroMQ context and sockets
        self.context: Optional[zmq.Context] = None
        self.rep_socket: Optional[zmq.Socket] = None
        self.pub_socket: Optional[zmq.Socket] = None

    def start(self) -> None:
        """Start the ZeroMQ server"""
        print(f"Starting ZeroMQ server v{PROTOCOL_VERSION}")
        print(f"REQ-REP endpoint: {self.req_addr}")
        print(f"PUB-SUB endpoint: {self.pub_addr}")

        # Create ZeroMQ context
        self.context = zmq.Context()

        # Create REP socket
        self.rep_socket = self.context.socket(zmq.REP)
        self.rep_socket.bind(self.req_addr)

        # Create PUB socket
        self.pub_socket = self.context.socket(zmq.PUB)
        self.pub_socket.bind(self.pub_addr)

        # Set receive timeout to allow checking running flag
        self.rep_socket.setsockopt(zmq.RCVTIMEO, 1000)  # 1 second timeout

        print("Server started successfully")
        self.running = True

        # Setup signal handlers
        signal.signal(signal.SIGINT, self._signal_handler)
        signal.signal(signal.SIGTERM, self._signal_handler)

        # Main loop
        self._run_loop()

    def _signal_handler(self, signum: int, frame: Optional[object]) -> None:
        """Handle shutdown signals"""
        print(f"\nReceived signal {signum}, shutting down...")
        self.running = False

    def _run_loop(self) -> None:
        """Main server loop"""
        request_count = 0

        while self.running:
            try:
                # Wait for request
                data = self.rep_socket.recv()
                request_count += 1

                # Deserialize request
                try:
                    request = deserialize_request(data)
                    print(f"[{request_count}] Received: {type(request).__name__}")

                    # Handle request
                    response = self._handle_request(request)

                    # Serialize and send response
                    response_data = serialize(response)
                    self.rep_socket.send(response_data)

                except Exception as e:
                    print(f"Error processing request: {e}")
                    error_response = ErrorResponse(message=str(e))
                    self.rep_socket.send(serialize(error_response))

            except zmq.Again:
                # Timeout, continue
                continue
            except Exception as e:
                print(f"Server error: {e}")
                if self.running:
                    # Send error response if still running
                    try:
                        error_response = ErrorResponse(message=f"Server error: {e}")
                        self.rep_socket.send(serialize(error_response))
                    except:
                        pass

        # Cleanup
        self._shutdown()

    def _handle_request(
        self, request: object
    ) -> object:  # Returns Response type
        """Handle incoming requests"""

        if isinstance(request, GenerateRequest):
            return self._handle_generate(request)
        elif isinstance(request, CancelRequest):
            return self._handle_cancel(request)
        elif isinstance(request, ListModelsRequest):
            return self._handle_list_models()
        elif isinstance(request, StatusRequest):
            return self._handle_status()
        elif isinstance(request, PingRequest):
            return PongResponse()
        else:
            return ErrorResponse(message=f"Unknown request type: {type(request).__name__}")

    def _handle_generate(self, request: GenerateRequest) -> object:
        """Handle generation request"""
        try:
            # Add job to queue
            job = self.job_queue.add_job(
                prompt=request.prompt,
                model=request.model,
                size=request.size,
                steps=request.steps,
                cfg_scale=request.cfg_scale,
                lora=request.lora,
                job_id=request.id,
            )

            # Estimate time
            estimated_time = self.job_queue.estimate_time(request.steps, request.batch_size, request.animation_frames)

            # Publish job started update
            self._publish_update(
                JobStartedUpdate(job_id=job.job_id, timestamp=int(time.time()))
            )

            return JobAcceptedResponse(job_id=job.job_id, estimated_time_s=estimated_time)

        except Exception as e:
            return JobErrorResponse(job_id=request.id, error=str(e))

    def _handle_cancel(self, request: CancelRequest) -> object:
        """Handle cancel request"""
        if self.job_queue.cancel_job(request.job_id):
            return JobCancelledResponse(job_id=request.job_id)
        else:
            return JobErrorResponse(
                job_id=request.job_id, error="Job not found or already completed"
            )

    def _scan_model_directory(
        self, directory: Path, model_type: ModelType
    ) -> List[ModelInfo]:
        """Scan a directory for model files

        Args:
            directory: Path to scan
            model_type: Type of models in this directory

        Returns:
            List of ModelInfo objects for found models
        """
        models = []

        # Check if directory exists
        if not directory.exists():
            print(f"Warning: Model directory does not exist: {directory}")
            return models

        if not directory.is_dir():
            print(f"Warning: Path is not a directory: {directory}")
            return models

        # Scan for model files
        try:
            for file_path in directory.iterdir():
                # Skip directories
                if not file_path.is_file():
                    continue

                # Check extension
                if file_path.suffix.lower() not in self.MODEL_EXTENSIONS:
                    continue

                # Get file size in MB
                try:
                    size_bytes = file_path.stat().st_size
                    size_mb = int(size_bytes / (1024 * 1024))
                except OSError as e:
                    print(f"Warning: Could not get size for {file_path}: {e}")
                    size_mb = 0

                # Create ModelInfo
                models.append(
                    ModelInfo(
                        name=file_path.name,
                        path=str(file_path),
                        model_type=model_type,
                        size_mb=size_mb,
                    )
                )
        except OSError as e:
            print(f"Error scanning directory {directory}: {e}")

        return models

    def _handle_list_models(self) -> ModelListResponse:
        """Handle list models request

        Scans the ComfyUI model directories for checkpoint, LoRA, and VAE files.
        Returns a sorted list of all found models.
        """
        models = []

        # Scan checkpoint directory
        models.extend(
            self._scan_model_directory(self.CHECKPOINT_DIR, ModelType.CHECKPOINT)
        )

        # Scan LoRA directory
        models.extend(
            self._scan_model_directory(self.LORA_DIR, ModelType.LORA)
        )

        # Scan VAE directory
        models.extend(
            self._scan_model_directory(self.VAE_DIR, ModelType.VAE)
        )

        # Sort models alphabetically by name
        models.sort(key=lambda m: m.name.lower())

        print(f"Found {len(models)} models")
        return ModelListResponse(models=models)

    def _handle_status(self) -> StatusInfoResponse:
        """Handle status request"""
        uptime = int(time.time() - self.start_time)

        return StatusInfoResponse(
            version=PROTOCOL_VERSION,
            queue_size=self.job_queue.queue_size(),
            active_jobs=self.job_queue.active_jobs(),
            uptime_s=uptime,
        )

    def _publish_update(self, update: object) -> None:
        """Publish a progress update"""
        if self.pub_socket:
            data = serialize(update)
            self.pub_socket.send(data)

    def _shutdown(self) -> None:
        """Shutdown the server"""
        print("Shutting down server...")

        if self.rep_socket:
            self.rep_socket.close()
        if self.pub_socket:
            self.pub_socket.close()
        if self.context:
            self.context.term()

        print("Server stopped")


def main() -> None:
    """Main entry point"""
    import argparse

    parser = argparse.ArgumentParser(description="DGX-Pixels ZeroMQ Backend Server")
    parser.add_argument(
        "--req-addr",
        default=DEFAULT_REQ_REP_ADDR,
        help=f"REQ-REP bind address (default: {DEFAULT_REQ_REP_ADDR})",
    )
    parser.add_argument(
        "--pub-addr",
        default=DEFAULT_PUB_SUB_ADDR,
        help=f"PUB-SUB bind address (default: {DEFAULT_PUB_SUB_ADDR})",
    )

    args = parser.parse_args()

    server = ZmqServer(req_addr=args.req_addr, pub_addr=args.pub_addr)
    server.start()


if __name__ == "__main__":
    main()
