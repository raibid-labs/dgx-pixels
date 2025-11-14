"""Message protocol definitions for ZeroMQ IPC

Version: 1.0
Serialization: MessagePack (MsgPack)
Transport: ZeroMQ (REQ-REP + PUB-SUB)
"""

from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Dict, List, Optional, Union
import msgpack


# ============================================================================
# Protocol Constants
# ============================================================================

PROTOCOL_VERSION = "1.0.0"
DEFAULT_REQ_REP_PORT = 5555
DEFAULT_PUB_SUB_PORT = 5556
DEFAULT_REQ_REP_ADDR = "tcp://127.0.0.1:5555"
DEFAULT_PUB_SUB_ADDR = "tcp://127.0.0.1:5556"


# ============================================================================
# Enumerations
# ============================================================================


class ModelType(str, Enum):
    """Model type enumeration"""

    CHECKPOINT = "checkpoint"
    LORA = "lora"
    VAE = "vae"


class GenerationStage(str, Enum):
    """Generation stage enumeration"""

    INITIALIZING = "initializing"
    LOADING_MODELS = "loading_models"
    ENCODING = "encoding"
    SAMPLING = "sampling"
    DECODING = "decoding"
    POST_PROCESSING = "post_processing"


# ============================================================================
# Request Messages (TUI → Backend)
# ============================================================================


@dataclass
class GenerateRequest:
    """Request to generate a single sprite"""

    id: str
    prompt: str
    model: str
    size: List[int]  # [width, height]
    steps: int
    cfg_scale: float
    lora: Optional[str] = None
    batch_size: Optional[int] = None
    animation_frames: Optional[int] = None
    tileset_grid: Optional[List[int]] = None

    def to_dict(self) -> Dict[str, Any]:
        result = {
            "type": "generate",
            "id": self.id,
            "prompt": self.prompt,
            "model": self.model,
            "size": self.size,
            "steps": self.steps,
            "cfg_scale": self.cfg_scale,
        }
        if self.lora is not None:
            result["lora"] = self.lora
        if self.batch_size is not None:
            result["batch_size"] = self.batch_size
        if self.animation_frames is not None:
            result["animation_frames"] = self.animation_frames
        if self.tileset_grid is not None:
            result["tileset_grid"] = self.tileset_grid
        return result


@dataclass
class CancelRequest:
    """Request to cancel a running job"""

    job_id: str

    def to_dict(self) -> Dict[str, Any]:
        return {"type": "cancel", "job_id": self.job_id}


@dataclass
class ListModelsRequest:
    """Request to list available models"""

    def to_dict(self) -> Dict[str, Any]:
        return {"type": "list_models"}


@dataclass
class StatusRequest:
    """Request backend status"""

    def to_dict(self) -> Dict[str, Any]:
        return {"type": "status"}


@dataclass
class PingRequest:
    """Ping for health check"""

    def to_dict(self) -> Dict[str, Any]:
        return {"type": "ping"}


Request = Union[
    GenerateRequest, CancelRequest, ListModelsRequest, StatusRequest, PingRequest
]


# ============================================================================
# Response Messages (Backend → TUI)
# ============================================================================


@dataclass
class JobAcceptedResponse:
    """Job accepted and queued"""

    job_id: str
    estimated_time_s: float

    def to_dict(self) -> Dict[str, Any]:
        return {
            "type": "job_accepted",
            "job_id": self.job_id,
            "estimated_time_s": self.estimated_time_s,
        }


@dataclass
class JobCompleteResponse:
    """Job completed successfully"""

    job_id: str
    image_path: str
    duration_s: float

    def to_dict(self) -> Dict[str, Any]:
        return {
            "type": "job_complete",
            "job_id": self.job_id,
            "image_path": self.image_path,
            "duration_s": self.duration_s,
        }


@dataclass
class JobErrorResponse:
    """Job failed with error"""

    job_id: str
    error: str

    def to_dict(self) -> Dict[str, Any]:
        return {"type": "job_error", "job_id": self.job_id, "error": self.error}


@dataclass
class JobCancelledResponse:
    """Job cancelled"""

    job_id: str

    def to_dict(self) -> Dict[str, Any]:
        return {"type": "job_cancelled", "job_id": self.job_id}


@dataclass
class ModelInfo:
    """Model information"""

    name: str
    path: str
    model_type: ModelType
    size_mb: int

    def to_dict(self) -> Dict[str, Any]:
        return {
            "name": self.name,
            "path": self.path,
            "model_type": self.model_type.value,
            "size_mb": self.size_mb,
        }


@dataclass
class ModelListResponse:
    """List of available models"""

    models: List[ModelInfo]

    def to_dict(self) -> Dict[str, Any]:
        return {"type": "model_list", "models": [m.to_dict() for m in self.models]}


@dataclass
class StatusInfoResponse:
    """Backend status information"""

    version: str
    queue_size: int
    active_jobs: int
    uptime_s: int

    def to_dict(self) -> Dict[str, Any]:
        return {
            "type": "status_info",
            "version": self.version,
            "queue_size": self.queue_size,
            "active_jobs": self.active_jobs,
            "uptime_s": self.uptime_s,
        }


@dataclass
class PongResponse:
    """Pong response"""

    def to_dict(self) -> Dict[str, Any]:
        return {"type": "pong"}


@dataclass
class ErrorResponse:
    """Generic error response"""

    message: str

    def to_dict(self) -> Dict[str, Any]:
        return {"type": "error", "message": self.message}


Response = Union[
    JobAcceptedResponse,
    JobCompleteResponse,
    JobErrorResponse,
    JobCancelledResponse,
    ModelListResponse,
    StatusInfoResponse,
    PongResponse,
    ErrorResponse,
]


# ============================================================================
# Progress Updates (Backend → TUI via PUB-SUB)
# ============================================================================


@dataclass
class JobStartedUpdate:
    """Job started"""

    job_id: str
    timestamp: int

    def to_dict(self) -> Dict[str, Any]:
        return {"type": "job_started", "job_id": self.job_id, "timestamp": self.timestamp}


@dataclass
class ProgressUpdate:
    """Generation progress"""

    job_id: str
    stage: GenerationStage
    step: int
    total_steps: int
    percent: float
    eta_s: float

    def to_dict(self) -> Dict[str, Any]:
        return {
            "type": "progress",
            "job_id": self.job_id,
            "stage": self.stage.value,
            "step": self.step,
            "total_steps": self.total_steps,
            "percent": self.percent,
            "eta_s": self.eta_s,
        }


@dataclass
class PreviewUpdate:
    """Preview image available"""

    job_id: str
    image_path: str
    step: int

    def to_dict(self) -> Dict[str, Any]:
        return {
            "type": "preview",
            "job_id": self.job_id,
            "image_path": self.image_path,
            "step": self.step,
        }


@dataclass
class JobFinishedUpdate:
    """Job finished"""

    job_id: str
    success: bool
    duration_s: float

    def to_dict(self) -> Dict[str, Any]:
        return {
            "type": "job_finished",
            "job_id": self.job_id,
            "success": self.success,
            "duration_s": self.duration_s,
        }


Update = Union[JobStartedUpdate, ProgressUpdate, PreviewUpdate, JobFinishedUpdate]


# ============================================================================
# Serialization Functions
# ============================================================================


def serialize(message: Union[Request, Response, Update]) -> bytes:
    """Serialize a message to MessagePack format"""
    data = message.to_dict()
    return msgpack.packb(data, use_bin_type=True)


def deserialize_request(data: bytes) -> Request:
    """Deserialize a request message from MessagePack format"""
    obj = msgpack.unpackb(data, raw=False)
    msg_type = obj.get("type")

    if msg_type == "generate":
        return GenerateRequest(
            id=obj["id"],
            prompt=obj["prompt"],
            model=obj["model"],
            size=obj["size"],
            steps=obj["steps"],
            cfg_scale=obj["cfg_scale"],
            lora=obj.get("lora"),
            batch_size=obj.get("batch_size"),
            animation_frames=obj.get("animation_frames"),
            tileset_grid=obj.get("tileset_grid"),
        )
    elif msg_type == "cancel":
        return CancelRequest(job_id=obj["job_id"])
    elif msg_type == "list_models":
        return ListModelsRequest()
    elif msg_type == "status":
        return StatusRequest()
    elif msg_type == "ping":
        return PingRequest()
    else:
        raise ValueError(f"Unknown request type: {msg_type}")


def deserialize_response(data: bytes) -> Response:
    """Deserialize a response message from MessagePack format"""
    obj = msgpack.unpackb(data, raw=False)
    msg_type = obj.get("type")

    if msg_type == "job_accepted":
        return JobAcceptedResponse(
            job_id=obj["job_id"], estimated_time_s=obj["estimated_time_s"]
        )
    elif msg_type == "job_complete":
        return JobCompleteResponse(
            job_id=obj["job_id"],
            image_path=obj["image_path"],
            duration_s=obj["duration_s"],
        )
    elif msg_type == "job_error":
        return JobErrorResponse(job_id=obj["job_id"], error=obj["error"])
    elif msg_type == "job_cancelled":
        return JobCancelledResponse(job_id=obj["job_id"])
    elif msg_type == "model_list":
        models = [
            ModelInfo(
                name=m["name"],
                path=m["path"],
                model_type=ModelType(m["model_type"]),
                size_mb=m["size_mb"],
            )
            for m in obj["models"]
        ]
        return ModelListResponse(models=models)
    elif msg_type == "status_info":
        return StatusInfoResponse(
            version=obj["version"],
            queue_size=obj["queue_size"],
            active_jobs=obj["active_jobs"],
            uptime_s=obj["uptime_s"],
        )
    elif msg_type == "pong":
        return PongResponse()
    elif msg_type == "error":
        return ErrorResponse(message=obj["message"])
    else:
        raise ValueError(f"Unknown response type: {msg_type}")


def deserialize_update(data: bytes) -> Update:
    """Deserialize a progress update from MessagePack format"""
    obj = msgpack.unpackb(data, raw=False)
    msg_type = obj.get("type")

    if msg_type == "job_started":
        return JobStartedUpdate(job_id=obj["job_id"], timestamp=obj["timestamp"])
    elif msg_type == "progress":
        return ProgressUpdate(
            job_id=obj["job_id"],
            stage=GenerationStage(obj["stage"]),
            step=obj["step"],
            total_steps=obj["total_steps"],
            percent=obj["percent"],
            eta_s=obj["eta_s"],
        )
    elif msg_type == "preview":
        return PreviewUpdate(
            job_id=obj["job_id"], image_path=obj["image_path"], step=obj["step"]
        )
    elif msg_type == "job_finished":
        return JobFinishedUpdate(
            job_id=obj["job_id"], success=obj["success"], duration_s=obj["duration_s"]
        )
    else:
        raise ValueError(f"Unknown update type: {msg_type}")
