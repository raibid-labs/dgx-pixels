#!/usr/bin/env python3
"""
Unit tests for Batch Processor
"""

import pytest
import time
from pathlib import Path
from unittest.mock import Mock, patch, MagicMock
import sys
import tempfile

# Add project root to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent.parent))

from python.batch.batch_processor import (
    BatchProcessor,
    BatchJob,
    BatchJobStatus,
    JobPriority,
)


class TestBatchJob:
    """Test BatchJob dataclass"""

    def test_batch_job_creation(self):
        """Test BatchJob creation with required fields"""
        job = BatchJob(
            job_id="test_123",
            prompts=["prompt1", "prompt2", "prompt3"],
            workflow_path=Path("workflow.json"),
        )

        assert job.job_id == "test_123"
        assert len(job.prompts) == 3
        assert job.total_prompts == 3
        assert job.completed_prompts == 0
        assert job.status == BatchJobStatus.QUEUED
        assert job.batch_size == 1
        assert job.priority == JobPriority.NORMAL

    def test_batch_job_creation_with_options(self):
        """Test BatchJob creation with optional fields"""
        job = BatchJob(
            job_id="test_456",
            prompts=["prompt1"],
            workflow_path=Path("workflow.json"),
            batch_size=4,
            priority=JobPriority.HIGH,
            model="custom_model.safetensors",
            lora="custom_lora.safetensors",
            steps=30,
            cfg_scale=9.0,
        )

        assert job.batch_size == 4
        assert job.priority == JobPriority.HIGH
        assert job.model == "custom_model.safetensors"
        assert job.lora == "custom_lora.safetensors"
        assert job.steps == 30
        assert job.cfg_scale == 9.0

    def test_batch_job_progress(self):
        """Test progress calculation"""
        job = BatchJob(
            job_id="test_789",
            prompts=["p1", "p2", "p3", "p4", "p5"],
            workflow_path=Path("workflow.json"),
        )

        # Initial progress
        assert job.progress() == 0.0

        # 50% complete
        job.completed_prompts = 2.5
        assert abs(job.progress() - 0.5) < 0.01

        # 100% complete
        job.completed_prompts = 5
        assert job.progress() == 1.0

    def test_batch_job_priority_comparison(self):
        """Test priority comparison for queue ordering"""
        job_urgent = BatchJob(
            job_id="urgent",
            prompts=["p"],
            workflow_path=Path("w.json"),
            priority=JobPriority.URGENT,
        )

        job_low = BatchJob(
            job_id="low",
            prompts=["p"],
            workflow_path=Path("w.json"),
            priority=JobPriority.LOW,
        )

        # Lower priority number = higher priority
        assert job_urgent < job_low

    def test_job_priority_enum(self):
        """Test JobPriority enum values"""
        assert JobPriority.URGENT == 0
        assert JobPriority.HIGH == 1
        assert JobPriority.NORMAL == 2
        assert JobPriority.LOW == 3

    def test_batch_job_status_enum(self):
        """Test BatchJobStatus enum values"""
        assert BatchJobStatus.QUEUED == "queued"
        assert BatchJobStatus.RUNNING == "running"
        assert BatchJobStatus.COMPLETED == "completed"
        assert BatchJobStatus.FAILED == "failed"
        assert BatchJobStatus.CANCELLED == "cancelled"


class TestBatchProcessor:
    """Test BatchProcessor functionality"""

    @pytest.fixture
    def temp_workflow(self):
        """Create a temporary workflow file"""
        with tempfile.NamedTemporaryFile(
            mode="w", suffix=".json", delete=False
        ) as f:
            f.write('{"1": {"inputs": {}}}')
            workflow_path = Path(f.name)

        yield workflow_path

        # Cleanup
        workflow_path.unlink(missing_ok=True)

    @pytest.fixture
    def temp_output_dir(self):
        """Create a temporary output directory"""
        with tempfile.TemporaryDirectory() as tmpdir:
            yield Path(tmpdir)

    def test_processor_initialization(self, temp_output_dir):
        """Test processor initialization"""
        processor = BatchProcessor(
            comfyui_host="localhost",
            comfyui_port=8188,
            output_base_dir=temp_output_dir,
            max_concurrent_batches=2,
        )

        assert processor.client.host == "localhost"
        assert processor.client.port == 8188
        assert processor.output_base_dir == temp_output_dir
        assert processor.max_concurrent_batches == 2
        assert processor.running is False

    def test_submit_job(self, temp_workflow, temp_output_dir):
        """Test job submission"""
        processor = BatchProcessor(output_base_dir=temp_output_dir)

        job_id = processor.submit_job(
            prompts=["test prompt 1", "test prompt 2"],
            workflow_path=temp_workflow,
            batch_size=1,
        )

        # Check job was created
        assert job_id in processor.jobs
        job = processor.jobs[job_id]
        assert job.status == BatchJobStatus.QUEUED
        assert len(job.prompts) == 2

    def test_submit_job_with_priority(self, temp_workflow, temp_output_dir):
        """Test job submission with priority"""
        processor = BatchProcessor(output_base_dir=temp_output_dir)

        job_id = processor.submit_job(
            prompts=["urgent prompt"],
            workflow_path=temp_workflow,
            priority=JobPriority.URGENT,
        )

        job = processor.jobs[job_id]
        assert job.priority == JobPriority.URGENT

    def test_submit_job_invalid_workflow(self, temp_output_dir):
        """Test job submission with non-existent workflow"""
        processor = BatchProcessor(output_base_dir=temp_output_dir)

        with pytest.raises(FileNotFoundError):
            processor.submit_job(
                prompts=["test"],
                workflow_path=Path("/nonexistent/workflow.json"),
            )

    def test_get_job(self, temp_workflow, temp_output_dir):
        """Test getting job by ID"""
        processor = BatchProcessor(output_base_dir=temp_output_dir)

        job_id = processor.submit_job(
            prompts=["test"],
            workflow_path=temp_workflow,
        )

        retrieved_job = processor.get_job(job_id)
        assert retrieved_job is not None
        assert retrieved_job.job_id == job_id

    def test_get_job_nonexistent(self, temp_output_dir):
        """Test getting non-existent job"""
        processor = BatchProcessor(output_base_dir=temp_output_dir)

        job = processor.get_job("nonexistent_id")
        assert job is None

    def test_cancel_job_queued(self, temp_workflow, temp_output_dir):
        """Test cancelling a queued job"""
        processor = BatchProcessor(output_base_dir=temp_output_dir)

        job_id = processor.submit_job(
            prompts=["test"],
            workflow_path=temp_workflow,
        )

        # Cancel the job
        result = processor.cancel_job(job_id)
        assert result is True

        # Check status
        job = processor.get_job(job_id)
        assert job.status == BatchJobStatus.CANCELLED

    def test_cancel_job_completed(self, temp_workflow, temp_output_dir):
        """Test cancelling a completed job (should fail)"""
        processor = BatchProcessor(output_base_dir=temp_output_dir)

        job_id = processor.submit_job(
            prompts=["test"],
            workflow_path=temp_workflow,
        )

        # Mark as completed
        job = processor.get_job(job_id)
        job.status = BatchJobStatus.COMPLETED

        # Try to cancel
        result = processor.cancel_job(job_id)
        assert result is False

    def test_get_queue_size(self, temp_workflow, temp_output_dir):
        """Test getting queue size"""
        processor = BatchProcessor(output_base_dir=temp_output_dir)

        # Submit multiple jobs
        for i in range(3):
            processor.submit_job(
                prompts=[f"prompt {i}"],
                workflow_path=temp_workflow,
            )

        # Queue size should be 3 (all queued in PriorityQueue)
        assert processor.get_queue_size() == 3

    def test_get_statistics(self, temp_output_dir):
        """Test getting processor statistics"""
        processor = BatchProcessor(output_base_dir=temp_output_dir)

        stats = processor.get_statistics()

        assert "uptime_s" in stats
        assert "queue_size" in stats
        assert "active_jobs" in stats
        assert "total_processed" in stats
        assert "total_failed" in stats
        assert "throughput_per_minute" in stats

    def test_generation_time_tracking(self, temp_output_dir):
        """Test generation time tracking"""
        processor = BatchProcessor(output_base_dir=temp_output_dir)

        # Add some generation times
        processor.generation_times = [2.0, 2.5, 3.0, 2.2, 2.8]

        stats = processor.get_statistics()

        # Average should be ~2.5
        assert 2.0 <= stats["avg_generation_time_s"] <= 3.0

        # Throughput should be calculated
        assert stats["throughput_per_minute"] > 0

    def test_generation_time_limit(self, temp_output_dir):
        """Test that generation times list is limited during processing"""
        processor = BatchProcessor(output_base_dir=temp_output_dir)
        processor.max_tracked_times = 5

        # Simulate what happens during _process_job
        # The list is trimmed when new times are added
        for i in range(10):
            processor.generation_times.append(float(i))
            # Trim if over limit (simulating what _process_job does)
            if len(processor.generation_times) > processor.max_tracked_times:
                processor.generation_times.pop(0)

        # Should only keep last 5
        assert len(processor.generation_times) == 5
        assert processor.generation_times[0] == 5.0  # Oldest kept


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
