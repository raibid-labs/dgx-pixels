"""Progress tracking and ETA calculation for generation jobs

This module provides real-time progress tracking with:
- Stage detection (load, denoise, save)
- ETA calculation based on historical data
- Progress smoothing
- Performance metrics
"""

import time
from dataclasses import dataclass, field
from typing import Dict, List, Optional

try:
    from .message_protocol import GenerationStage
    from .comfyui_client import ExecutionStatus, WorkflowProgress
except ImportError:
    from message_protocol import GenerationStage
    from comfyui_client import ExecutionStatus, WorkflowProgress


# ============================================================================
# Data Structures
# ============================================================================


@dataclass
class StageTimings:
    """Historical timing data for a generation stage"""

    stage: GenerationStage
    samples: List[float] = field(default_factory=list)
    max_samples: int = 100

    def add_sample(self, duration_s: float) -> None:
        """Add a timing sample"""
        self.samples.append(duration_s)
        if len(self.samples) > self.max_samples:
            self.samples.pop(0)

    def average(self) -> float:
        """Get average duration"""
        if not self.samples:
            return 0.0
        return sum(self.samples) / len(self.samples)

    def estimate(self) -> float:
        """Estimate duration for this stage"""
        if not self.samples:
            # Default estimates (in seconds)
            defaults = {
                GenerationStage.INITIALIZING: 0.5,
                GenerationStage.LOADING_MODELS: 2.0,
                GenerationStage.ENCODING: 0.5,
                GenerationStage.SAMPLING: 10.0,  # Will be adjusted by steps
                GenerationStage.DECODING: 1.0,
                GenerationStage.POST_PROCESSING: 0.5,
            }
            return defaults.get(self.stage, 1.0)

        return self.average()


@dataclass
class JobProgress:
    """Detailed progress tracking for a single job"""

    job_id: str
    prompt_id: str
    total_steps: int
    start_time: float = field(default_factory=time.time)

    # Current state
    current_stage: GenerationStage = GenerationStage.INITIALIZING
    current_step: int = 0
    stage_start_time: float = field(default_factory=time.time)

    # Stage completion tracking
    completed_stages: List[GenerationStage] = field(default_factory=list)

    def update_stage(self, new_stage: GenerationStage) -> None:
        """Update to a new stage"""
        if new_stage != self.current_stage:
            self.completed_stages.append(self.current_stage)
            self.current_stage = new_stage
            self.stage_start_time = time.time()

    def elapsed_s(self) -> float:
        """Get total elapsed time"""
        return time.time() - self.start_time

    def stage_elapsed_s(self) -> float:
        """Get time spent in current stage"""
        return time.time() - self.stage_start_time


# ============================================================================
# Progress Tracker
# ============================================================================


class ProgressTracker:
    """Tracks progress and calculates ETAs for generation jobs

    Features:
    - Historical timing data for accurate ETAs
    - Stage-based progress tracking
    - Smooth progress updates
    - Performance metrics
    """

    def __init__(self):
        # Historical timing data
        self.stage_timings: Dict[GenerationStage, StageTimings] = {
            stage: StageTimings(stage=stage) for stage in GenerationStage
        }

        # Active job tracking
        self.active_jobs: Dict[str, JobProgress] = {}

        # Per-step timing (for sampling stage)
        self.step_durations: List[float] = []
        self.max_step_samples = 1000

    def start_job(self, job_id: str, prompt_id: str, total_steps: int) -> JobProgress:
        """Start tracking a new job

        Args:
            job_id: Unique job identifier
            prompt_id: ComfyUI prompt ID
            total_steps: Total number of sampling steps

        Returns:
            JobProgress instance
        """
        progress = JobProgress(
            job_id=job_id,
            prompt_id=prompt_id,
            total_steps=total_steps,
        )

        self.active_jobs[job_id] = progress
        return progress

    def update_progress(
        self,
        job_id: str,
        workflow_progress: WorkflowProgress,
    ) -> tuple[GenerationStage, int, int, float, float]:
        """Update job progress from ComfyUI workflow progress

        Args:
            job_id: Job identifier
            workflow_progress: Progress from ComfyUI

        Returns:
            Tuple of (stage, step, total_steps, percent, eta_s)
        """
        job = self.active_jobs.get(job_id)
        if not job:
            # Job not tracked - return neutral progress
            return (
                GenerationStage.INITIALIZING,
                0,
                100,
                0.0,
                0.0,
            )

        # Map ComfyUI status to our stages
        stage = self._map_status_to_stage(workflow_progress, job)

        # Update stage if changed
        if stage != job.current_stage:
            # Record previous stage timing
            stage_duration = job.stage_elapsed_s()
            self.stage_timings[job.current_stage].add_sample(stage_duration)

            # Update to new stage
            job.update_stage(stage)

        # Calculate progress metrics
        step = workflow_progress.step
        total_steps = job.total_steps
        percent = self._calculate_percent(job, stage, step, total_steps)
        eta_s = self._calculate_eta(job, stage, step, total_steps)

        # Update step tracking
        job.current_step = step

        return (stage, step, total_steps, percent, eta_s)

    def complete_job(self, job_id: str) -> None:
        """Mark a job as complete and record timings

        Args:
            job_id: Job identifier
        """
        job = self.active_jobs.get(job_id)
        if not job:
            return

        # Record final stage timing
        stage_duration = job.stage_elapsed_s()
        self.stage_timings[job.current_stage].add_sample(stage_duration)

        # Remove from active jobs
        del self.active_jobs[job_id]

    def _map_status_to_stage(
        self,
        workflow_progress: WorkflowProgress,
        job: JobProgress,
    ) -> GenerationStage:
        """Map ComfyUI execution status to generation stage

        Args:
            workflow_progress: ComfyUI progress
            job: Job progress tracking

        Returns:
            Current generation stage
        """
        status = workflow_progress.status

        if status == ExecutionStatus.PENDING:
            return GenerationStage.INITIALIZING

        if status == ExecutionStatus.RUNNING:
            # Estimate stage based on progress and node info
            if workflow_progress.current_node:
                node_name = workflow_progress.node_name or ""

                if "checkpoint" in node_name.lower() or "load" in node_name.lower():
                    return GenerationStage.LOADING_MODELS
                elif "encode" in node_name.lower() or "clip" in node_name.lower():
                    return GenerationStage.ENCODING
                elif "sampler" in node_name.lower() or "ksampler" in node_name.lower():
                    return GenerationStage.SAMPLING
                elif "decode" in node_name.lower() or "vae" in node_name.lower():
                    return GenerationStage.DECODING
                elif "save" in node_name.lower():
                    return GenerationStage.POST_PROCESSING

            # Fallback: estimate based on elapsed time
            elapsed = job.elapsed_s()

            if elapsed < 1.0:
                return GenerationStage.INITIALIZING
            elif elapsed < 3.0:
                return GenerationStage.LOADING_MODELS
            elif elapsed < 4.0:
                return GenerationStage.ENCODING
            else:
                return GenerationStage.SAMPLING

        if status == ExecutionStatus.SUCCESS:
            return GenerationStage.POST_PROCESSING

        # Default to current stage
        return job.current_stage

    def _calculate_percent(
        self,
        job: JobProgress,
        stage: GenerationStage,
        step: int,
        total_steps: int,
    ) -> float:
        """Calculate overall completion percentage

        Args:
            job: Job progress
            stage: Current stage
            step: Current step within stage
            total_steps: Total steps in job

        Returns:
            Completion percentage (0-100)
        """
        # Stage weights (total = 100%)
        stage_weights = {
            GenerationStage.INITIALIZING: 2.0,
            GenerationStage.LOADING_MODELS: 10.0,
            GenerationStage.ENCODING: 3.0,
            GenerationStage.SAMPLING: 80.0,  # Majority of time
            GenerationStage.DECODING: 4.0,
            GenerationStage.POST_PROCESSING: 1.0,
        }

        # Calculate completed percentage
        completed_percent = 0.0
        for completed_stage in job.completed_stages:
            completed_percent += stage_weights.get(completed_stage, 0.0)

        # Add current stage progress
        current_weight = stage_weights.get(stage, 0.0)

        if stage == GenerationStage.SAMPLING and total_steps > 0:
            # Use actual step progress for sampling
            stage_progress = (step / total_steps) * current_weight
        else:
            # Estimate progress within stage
            stage_duration = job.stage_elapsed_s()
            estimated_duration = self.stage_timings[stage].estimate()

            if estimated_duration > 0:
                stage_progress = min(
                    1.0, stage_duration / estimated_duration
                ) * current_weight
            else:
                stage_progress = 0.0

        total_percent = completed_percent + stage_progress

        return min(100.0, max(0.0, total_percent))

    def _calculate_eta(
        self,
        job: JobProgress,
        stage: GenerationStage,
        step: int,
        total_steps: int,
    ) -> float:
        """Calculate estimated time remaining

        Args:
            job: Job progress
            stage: Current stage
            step: Current step
            total_steps: Total steps

        Returns:
            Estimated seconds remaining
        """
        # Estimate remaining time for current stage
        if stage == GenerationStage.SAMPLING:
            # Use step-based estimation for sampling
            if step > 0 and total_steps > 0:
                steps_remaining = total_steps - step
                step_duration = self._estimate_step_duration(job, step)
                stage_eta = steps_remaining * step_duration
            else:
                stage_eta = self.stage_timings[stage].estimate()
        else:
            elapsed = job.stage_elapsed_s()
            estimated = self.stage_timings[stage].estimate()
            stage_eta = max(0.0, estimated - elapsed)

        # Add time for remaining stages
        remaining_stages = self._get_remaining_stages(stage)
        remaining_eta = sum(
            self.stage_timings[s].estimate() for s in remaining_stages
        )

        return stage_eta + remaining_eta

    def _estimate_step_duration(self, job: JobProgress, current_step: int) -> float:
        """Estimate duration per step

        Args:
            job: Job progress
            current_step: Current sampling step

        Returns:
            Estimated seconds per step
        """
        if current_step > 0:
            # Use actual timing from this job
            sampling_elapsed = job.stage_elapsed_s()
            return sampling_elapsed / current_step

        # Use historical average
        if self.step_durations:
            return sum(self.step_durations) / len(self.step_durations)

        # Default estimate
        return 0.5  # 0.5 seconds per step

    def _get_remaining_stages(self, current_stage: GenerationStage) -> List[GenerationStage]:
        """Get list of stages remaining after current stage

        Args:
            current_stage: Current generation stage

        Returns:
            List of remaining stages
        """
        stage_order = [
            GenerationStage.INITIALIZING,
            GenerationStage.LOADING_MODELS,
            GenerationStage.ENCODING,
            GenerationStage.SAMPLING,
            GenerationStage.DECODING,
            GenerationStage.POST_PROCESSING,
        ]

        try:
            current_idx = stage_order.index(current_stage)
            return stage_order[current_idx + 1:]
        except ValueError:
            return []

    def get_stats(self) -> Dict[str, any]:
        """Get performance statistics

        Returns:
            Dictionary of performance metrics
        """
        return {
            "active_jobs": len(self.active_jobs),
            "stage_timings": {
                stage.value: {
                    "samples": len(timing.samples),
                    "average_s": timing.average(),
                    "estimate_s": timing.estimate(),
                }
                for stage, timing in self.stage_timings.items()
            },
        }
