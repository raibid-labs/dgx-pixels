"""Tests for progress tracking"""

import pytest
import time

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '../../python/workers'))

from progress_tracker import ProgressTracker, JobProgress, StageTimings
from message_protocol import GenerationStage
from comfyui_client import ExecutionStatus, WorkflowProgress


@pytest.fixture
def tracker():
    """Create a test progress tracker"""
    return ProgressTracker()


@pytest.fixture
def mock_workflow_progress():
    """Create mock workflow progress"""
    def create_progress(status=ExecutionStatus.RUNNING, step=0, total=20):
        return WorkflowProgress(
            prompt_id="test-prompt",
            status=status,
            step=step,
            total_steps=total,
        )
    return create_progress


class TestStageTimings:
    """Test stage timing statistics"""

    def test_add_sample(self):
        """Test adding timing samples"""
        timings = StageTimings(stage=GenerationStage.SAMPLING, max_samples=5)

        timings.add_sample(1.0)
        timings.add_sample(2.0)
        timings.add_sample(3.0)

        assert len(timings.samples) == 3
        assert timings.average() == 2.0

    def test_max_samples_limit(self):
        """Test sample limit enforcement"""
        timings = StageTimings(stage=GenerationStage.SAMPLING, max_samples=3)

        for i in range(10):
            timings.add_sample(float(i))

        assert len(timings.samples) == 3
        assert timings.samples == [7.0, 8.0, 9.0]  # Last 3 samples

    def test_estimate_no_samples(self):
        """Test estimation with no historical data"""
        timings = StageTimings(stage=GenerationStage.SAMPLING)

        estimate = timings.estimate()
        assert estimate > 0.0  # Should use default

    def test_estimate_with_samples(self):
        """Test estimation with historical data"""
        timings = StageTimings(stage=GenerationStage.SAMPLING)

        timings.add_sample(5.0)
        timings.add_sample(10.0)
        timings.add_sample(15.0)

        assert timings.estimate() == 10.0  # Average


class TestJobProgress:
    """Test job progress tracking"""

    def test_initialization(self):
        """Test job progress initialization"""
        progress = JobProgress(
            job_id="test-job",
            prompt_id="test-prompt",
            total_steps=20,
        )

        assert progress.job_id == "test-job"
        assert progress.current_stage == GenerationStage.INITIALIZING
        assert progress.current_step == 0
        assert len(progress.completed_stages) == 0

    def test_update_stage(self):
        """Test stage updates"""
        progress = JobProgress(
            job_id="test-job",
            prompt_id="test-prompt",
            total_steps=20,
        )

        initial_time = progress.stage_start_time

        time.sleep(0.01)  # Small delay

        progress.update_stage(GenerationStage.LOADING_MODELS)

        assert progress.current_stage == GenerationStage.LOADING_MODELS
        assert GenerationStage.INITIALIZING in progress.completed_stages
        assert progress.stage_start_time > initial_time

    def test_elapsed_time(self):
        """Test elapsed time tracking"""
        progress = JobProgress(
            job_id="test-job",
            prompt_id="test-prompt",
            total_steps=20,
        )

        time.sleep(0.1)

        elapsed = progress.elapsed_s()
        assert elapsed >= 0.1

    def test_stage_elapsed_time(self):
        """Test stage-specific elapsed time"""
        progress = JobProgress(
            job_id="test-job",
            prompt_id="test-prompt",
            total_steps=20,
        )

        time.sleep(0.05)
        progress.update_stage(GenerationStage.SAMPLING)
        time.sleep(0.05)

        stage_elapsed = progress.stage_elapsed_s()
        assert stage_elapsed >= 0.05
        assert stage_elapsed < progress.elapsed_s()


class TestProgressTracker:
    """Test progress tracker functionality"""

    def test_start_job(self, tracker):
        """Test starting job tracking"""
        progress = tracker.start_job(
            job_id="test-job",
            prompt_id="test-prompt",
            total_steps=20,
        )

        assert progress.job_id == "test-job"
        assert "test-job" in tracker.active_jobs

    def test_update_progress_pending(self, tracker, mock_workflow_progress):
        """Test progress update for pending job"""
        tracker.start_job("test-job", "test-prompt", 20)

        workflow_progress = mock_workflow_progress(status=ExecutionStatus.PENDING)

        stage, step, total, percent, eta = tracker.update_progress(
            "test-job", workflow_progress
        )

        assert stage == GenerationStage.INITIALIZING
        assert step == 0
        assert percent == 0.0

    def test_update_progress_running(self, tracker, mock_workflow_progress):
        """Test progress update for running job"""
        tracker.start_job("test-job", "test-prompt", 20)

        workflow_progress = mock_workflow_progress(
            status=ExecutionStatus.RUNNING,
            step=10,
            total=20,
        )

        stage, step, total, percent, eta = tracker.update_progress(
            "test-job", workflow_progress
        )

        assert step == 10
        assert total == 20
        assert percent > 0.0

    def test_update_progress_stage_transition(self, tracker, mock_workflow_progress):
        """Test stage transition tracking"""
        tracker.start_job("test-job", "test-prompt", 20)

        # Start in INITIALIZING
        wp1 = mock_workflow_progress(status=ExecutionStatus.PENDING)
        tracker.update_progress("test-job", wp1)

        time.sleep(0.01)

        # Transition to SAMPLING
        wp2 = mock_workflow_progress(status=ExecutionStatus.RUNNING)
        wp2.current_node = "sampler_node"
        wp2.node_name = "KSampler"

        stage, _, _, _, _ = tracker.update_progress("test-job", wp2)

        assert stage == GenerationStage.SAMPLING

        # Check timing was recorded
        init_timings = tracker.stage_timings[GenerationStage.INITIALIZING]
        assert len(init_timings.samples) > 0

    def test_complete_job(self, tracker):
        """Test job completion"""
        tracker.start_job("test-job", "test-prompt", 20)

        assert "test-job" in tracker.active_jobs

        tracker.complete_job("test-job")

        assert "test-job" not in tracker.active_jobs

    def test_complete_job_records_timing(self, tracker, mock_workflow_progress):
        """Test that completion records final stage timing"""
        tracker.start_job("test-job", "test-prompt", 20)

        # Do some work in SAMPLING stage
        wp = mock_workflow_progress(status=ExecutionStatus.RUNNING)
        wp.current_node = "sampler"
        wp.node_name = "KSampler"
        tracker.update_progress("test-job", wp)

        time.sleep(0.01)

        # Complete job
        tracker.complete_job("test-job")

        # Check timing was recorded
        sampling_timings = tracker.stage_timings[GenerationStage.SAMPLING]
        assert len(sampling_timings.samples) > 0

    def test_calculate_percent_initializing(self, tracker, mock_workflow_progress):
        """Test percentage calculation in initializing stage"""
        tracker.start_job("test-job", "test-prompt", 20)

        wp = mock_workflow_progress(status=ExecutionStatus.PENDING)
        _, _, _, percent, _ = tracker.update_progress("test-job", wp)

        assert 0.0 <= percent <= 10.0  # Should be low percentage

    def test_calculate_percent_sampling(self, tracker, mock_workflow_progress):
        """Test percentage calculation in sampling stage"""
        tracker.start_job("test-job", "test-prompt", 20)

        # Move to sampling stage
        wp = mock_workflow_progress(status=ExecutionStatus.RUNNING)
        wp.current_node = "sampler"
        wp.node_name = "KSampler"
        wp.step = 10
        wp.total_steps = 20

        _, _, _, percent, _ = tracker.update_progress("test-job", wp)

        # Sampling is 80% of total, halfway through = ~40-60%
        assert 30.0 <= percent <= 70.0

    def test_calculate_eta(self, tracker, mock_workflow_progress):
        """Test ETA calculation"""
        tracker.start_job("test-job", "test-prompt", 20)

        wp = mock_workflow_progress(
            status=ExecutionStatus.RUNNING,
            step=5,
            total=20,
        )

        _, _, _, _, eta = tracker.update_progress("test-job", wp)

        assert eta > 0.0  # Should have time remaining

    def test_get_stats(self, tracker):
        """Test statistics retrieval"""
        tracker.start_job("test-job-1", "prompt-1", 20)
        tracker.start_job("test-job-2", "prompt-2", 30)

        stats = tracker.get_stats()

        assert stats["active_jobs"] == 2
        assert "stage_timings" in stats

    def test_multiple_jobs(self, tracker, mock_workflow_progress):
        """Test tracking multiple concurrent jobs"""
        tracker.start_job("job-1", "prompt-1", 20)
        tracker.start_job("job-2", "prompt-2", 30)

        # Update job 1
        wp1 = mock_workflow_progress(step=10, total=20)
        _, step1, _, _, _ = tracker.update_progress("job-1", wp1)
        assert step1 == 10

        # Update job 2
        wp2 = mock_workflow_progress(step=15, total=30)
        _, step2, _, _, _ = tracker.update_progress("job-2", wp2)
        assert step2 == 15

        # Jobs should be independent
        assert len(tracker.active_jobs) == 2

    def test_stage_mapping_node_names(self, tracker, mock_workflow_progress):
        """Test stage detection from node names"""
        tracker.start_job("test-job", "test-prompt", 20)

        test_cases = [
            ("CheckpointLoader", GenerationStage.LOADING_MODELS),
            ("CLIPTextEncode", GenerationStage.ENCODING),
            ("KSampler", GenerationStage.SAMPLING),
            ("VAEDecode", GenerationStage.DECODING),
            ("SaveImage", GenerationStage.POST_PROCESSING),
        ]

        for node_name, expected_stage in test_cases:
            wp = mock_workflow_progress(status=ExecutionStatus.RUNNING)
            wp.node_name = node_name

            stage, _, _, _, _ = tracker.update_progress("test-job", wp)
            assert stage == expected_stage


class TestHistoricalTimings:
    """Test historical timing data usage"""

    def test_timings_improve_estimates(self, tracker):
        """Test that historical data improves estimates"""
        # Get initial estimate
        initial_estimate = tracker.stage_timings[GenerationStage.SAMPLING].estimate()

        # Add several samples
        for _ in range(5):
            tracker.start_job(f"job-{_}", f"prompt-{_}", 20)

            # Simulate fast execution
            time.sleep(0.01)
            tracker.complete_job(f"job-{_}")

        # New estimate should be different (and likely lower)
        new_estimate = tracker.stage_timings[GenerationStage.SAMPLING].estimate()

        # Estimates should differ (historical data is being used)
        assert new_estimate != initial_estimate


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
