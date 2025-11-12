"""Tests for message protocol serialization/deserialization"""

import sys
import os

# Add python directory to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "../../python"))

from workers.message_protocol import (
    # Requests
    GenerateRequest,
    CancelRequest,
    ListModelsRequest,
    StatusRequest,
    PingRequest,
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
    # Updates
    JobStartedUpdate,
    ProgressUpdate,
    PreviewUpdate,
    JobFinishedUpdate,
    # Enums
    ModelType,
    GenerationStage,
    # Functions
    serialize,
    deserialize_request,
    deserialize_response,
    deserialize_update,
)


def test_serialize_generate_request():
    """Test serialization of generate request"""
    req = GenerateRequest(
        id="job-001",
        prompt="16-bit knight sprite",
        model="sdxl-base",
        size=[1024, 1024],
        steps=30,
        cfg_scale=7.5,
        lora=None,
    )

    serialized = serialize(req)
    deserialized = deserialize_request(serialized)

    assert isinstance(deserialized, GenerateRequest)
    assert deserialized.id == req.id
    assert deserialized.prompt == req.prompt
    assert deserialized.model == req.model
    assert deserialized.size == req.size
    assert deserialized.steps == req.steps
    assert deserialized.cfg_scale == req.cfg_scale
    assert deserialized.lora == req.lora


def test_serialize_generate_request_with_lora():
    """Test serialization of generate request with LoRA"""
    req = GenerateRequest(
        id="job-002",
        prompt="pixel art warrior",
        model="sdxl-base",
        size=[512, 512],
        steps=20,
        cfg_scale=8.0,
        lora="pixelart",
    )

    serialized = serialize(req)
    deserialized = deserialize_request(serialized)

    assert isinstance(deserialized, GenerateRequest)
    assert deserialized.lora == "pixelart"


def test_serialize_cancel_request():
    """Test serialization of cancel request"""
    req = CancelRequest(job_id="job-001")

    serialized = serialize(req)
    deserialized = deserialize_request(serialized)

    assert isinstance(deserialized, CancelRequest)
    assert deserialized.job_id == req.job_id


def test_serialize_list_models_request():
    """Test serialization of list models request"""
    req = ListModelsRequest()

    serialized = serialize(req)
    deserialized = deserialize_request(serialized)

    assert isinstance(deserialized, ListModelsRequest)


def test_serialize_status_request():
    """Test serialization of status request"""
    req = StatusRequest()

    serialized = serialize(req)
    deserialized = deserialize_request(serialized)

    assert isinstance(deserialized, StatusRequest)


def test_serialize_ping_request():
    """Test serialization of ping request"""
    req = PingRequest()

    serialized = serialize(req)
    deserialized = deserialize_request(serialized)

    assert isinstance(deserialized, PingRequest)


def test_serialize_job_accepted_response():
    """Test serialization of job accepted response"""
    resp = JobAcceptedResponse(job_id="job-001", estimated_time_s=3.5)

    serialized = serialize(resp)
    deserialized = deserialize_response(serialized)

    assert isinstance(deserialized, JobAcceptedResponse)
    assert deserialized.job_id == resp.job_id
    assert deserialized.estimated_time_s == resp.estimated_time_s


def test_serialize_job_complete_response():
    """Test serialization of job complete response"""
    resp = JobCompleteResponse(
        job_id="job-001", image_path="/output/sprite-001.png", duration_s=3.2
    )

    serialized = serialize(resp)
    deserialized = deserialize_response(serialized)

    assert isinstance(deserialized, JobCompleteResponse)
    assert deserialized.job_id == resp.job_id
    assert deserialized.image_path == resp.image_path
    assert deserialized.duration_s == resp.duration_s


def test_serialize_job_error_response():
    """Test serialization of job error response"""
    resp = JobErrorResponse(job_id="job-001", error="Model not found: sdxl-custom")

    serialized = serialize(resp)
    deserialized = deserialize_response(serialized)

    assert isinstance(deserialized, JobErrorResponse)
    assert deserialized.job_id == resp.job_id
    assert deserialized.error == resp.error


def test_serialize_job_cancelled_response():
    """Test serialization of job cancelled response"""
    resp = JobCancelledResponse(job_id="job-001")

    serialized = serialize(resp)
    deserialized = deserialize_response(serialized)

    assert isinstance(deserialized, JobCancelledResponse)
    assert deserialized.job_id == resp.job_id


def test_serialize_model_list_response():
    """Test serialization of model list response"""
    resp = ModelListResponse(
        models=[
            ModelInfo(
                name="SDXL Base",
                path="/models/sdxl-base.safetensors",
                model_type=ModelType.CHECKPOINT,
                size_mb=6500,
            ),
            ModelInfo(
                name="Pixel Art LoRA",
                path="/models/loras/pixelart.safetensors",
                model_type=ModelType.LORA,
                size_mb=144,
            ),
        ]
    )

    serialized = serialize(resp)
    deserialized = deserialize_response(serialized)

    assert isinstance(deserialized, ModelListResponse)
    assert len(deserialized.models) == 2
    assert deserialized.models[0].name == "SDXL Base"
    assert deserialized.models[0].model_type == ModelType.CHECKPOINT
    assert deserialized.models[1].name == "Pixel Art LoRA"
    assert deserialized.models[1].model_type == ModelType.LORA


def test_serialize_status_info_response():
    """Test serialization of status info response"""
    resp = StatusInfoResponse(version="1.0.0", queue_size=3, active_jobs=1, uptime_s=3600)

    serialized = serialize(resp)
    deserialized = deserialize_response(serialized)

    assert isinstance(deserialized, StatusInfoResponse)
    assert deserialized.version == resp.version
    assert deserialized.queue_size == resp.queue_size
    assert deserialized.active_jobs == resp.active_jobs
    assert deserialized.uptime_s == resp.uptime_s


def test_serialize_pong_response():
    """Test serialization of pong response"""
    resp = PongResponse()

    serialized = serialize(resp)
    deserialized = deserialize_response(serialized)

    assert isinstance(deserialized, PongResponse)


def test_serialize_error_response():
    """Test serialization of error response"""
    resp = ErrorResponse(message="Internal server error")

    serialized = serialize(resp)
    deserialized = deserialize_response(serialized)

    assert isinstance(deserialized, ErrorResponse)
    assert deserialized.message == resp.message


def test_serialize_job_started_update():
    """Test serialization of job started update"""
    update = JobStartedUpdate(job_id="job-001", timestamp=1699564800)

    serialized = serialize(update)
    deserialized = deserialize_update(serialized)

    assert isinstance(deserialized, JobStartedUpdate)
    assert deserialized.job_id == update.job_id
    assert deserialized.timestamp == update.timestamp


def test_serialize_progress_update():
    """Test serialization of progress update"""
    update = ProgressUpdate(
        job_id="job-001",
        stage=GenerationStage.SAMPLING,
        step=15,
        total_steps=30,
        percent=50.0,
        eta_s=1.8,
    )

    serialized = serialize(update)
    deserialized = deserialize_update(serialized)

    assert isinstance(deserialized, ProgressUpdate)
    assert deserialized.job_id == update.job_id
    assert deserialized.stage == update.stage
    assert deserialized.step == update.step
    assert deserialized.total_steps == update.total_steps
    assert deserialized.percent == update.percent
    assert deserialized.eta_s == update.eta_s


def test_serialize_preview_update():
    """Test serialization of preview update"""
    update = PreviewUpdate(job_id="job-001", image_path="/tmp/preview-001.png", step=10)

    serialized = serialize(update)
    deserialized = deserialize_update(serialized)

    assert isinstance(deserialized, PreviewUpdate)
    assert deserialized.job_id == update.job_id
    assert deserialized.image_path == update.image_path
    assert deserialized.step == update.step


def test_serialize_job_finished_update():
    """Test serialization of job finished update"""
    update = JobFinishedUpdate(job_id="job-001", success=True, duration_s=3.2)

    serialized = serialize(update)
    deserialized = deserialize_update(serialized)

    assert isinstance(deserialized, JobFinishedUpdate)
    assert deserialized.job_id == update.job_id
    assert deserialized.success == update.success
    assert deserialized.duration_s == update.duration_s


def test_all_generation_stages():
    """Test all generation stages serialize correctly"""
    stages = [
        GenerationStage.INITIALIZING,
        GenerationStage.LOADING_MODELS,
        GenerationStage.ENCODING,
        GenerationStage.SAMPLING,
        GenerationStage.DECODING,
        GenerationStage.POST_PROCESSING,
    ]

    for stage in stages:
        update = ProgressUpdate(
            job_id="test",
            stage=stage,
            step=1,
            total_steps=10,
            percent=10.0,
            eta_s=5.0,
        )

        serialized = serialize(update)
        deserialized = deserialize_update(serialized)

        assert deserialized.stage == stage


def test_message_size_reasonable():
    """Ensure messages don't get too large"""
    req = GenerateRequest(
        id="job-001",
        prompt="A" * 500,  # 500 character prompt
        model="sdxl-base",
        size=[1024, 1024],
        steps=30,
        cfg_scale=7.5,
        lora="pixelart",
    )

    serialized = serialize(req)
    # MessagePack should compress well, expect < 1KB for reasonable prompts
    assert len(serialized) < 1024, f"Message too large: {len(serialized)} bytes"


if __name__ == "__main__":
    # Run all tests
    import pytest

    pytest.main([__file__, "-v"])
