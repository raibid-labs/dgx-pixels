"""Job queue management for generation tasks"""

import time
import uuid
from dataclasses import dataclass, field
from enum import Enum
from typing import Dict, List, Optional


class JobStatus(str, Enum):
    """Job status enumeration"""

    QUEUED = "queued"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"


@dataclass
class Job:
    """Represents a generation job"""

    job_id: str
    prompt: str
    model: str
    size: List[int]
    steps: int
    cfg_scale: float
    lora: Optional[str] = None
    status: JobStatus = JobStatus.QUEUED
    created_at: float = field(default_factory=time.time)
    started_at: Optional[float] = None
    completed_at: Optional[float] = None
    error: Optional[str] = None
    output_path: Optional[str] = None


class JobQueue:
    """FIFO job queue with status tracking"""

    def __init__(self) -> None:
        self._jobs: Dict[str, Job] = {}
        self._queue: List[str] = []

    def add_job(
        self,
        prompt: str,
        model: str,
        size: List[int],
        steps: int,
        cfg_scale: float,
        lora: Optional[str] = None,
        job_id: Optional[str] = None,
    ) -> Job:
        """Add a new job to the queue"""
        if job_id is None:
            job_id = str(uuid.uuid4())

        job = Job(
            job_id=job_id,
            prompt=prompt,
            model=model,
            size=size,
            steps=steps,
            cfg_scale=cfg_scale,
            lora=lora,
        )

        self._jobs[job_id] = job
        self._queue.append(job_id)

        return job

    def get_next_job(self) -> Optional[Job]:
        """Get the next job from the queue"""
        while self._queue:
            job_id = self._queue.pop(0)
            job = self._jobs.get(job_id)

            if job and job.status == JobStatus.QUEUED:
                job.status = JobStatus.RUNNING
                job.started_at = time.time()
                return job

        return None

    def get_job(self, job_id: str) -> Optional[Job]:
        """Get a job by ID"""
        return self._jobs.get(job_id)

    def complete_job(self, job_id: str, output_path: str) -> None:
        """Mark a job as completed"""
        job = self._jobs.get(job_id)
        if job:
            job.status = JobStatus.COMPLETED
            job.completed_at = time.time()
            job.output_path = output_path

    def fail_job(self, job_id: str, error: str) -> None:
        """Mark a job as failed"""
        job = self._jobs.get(job_id)
        if job:
            job.status = JobStatus.FAILED
            job.completed_at = time.time()
            job.error = error

    def cancel_job(self, job_id: str) -> bool:
        """Cancel a job"""
        job = self._jobs.get(job_id)
        if job and job.status in (JobStatus.QUEUED, JobStatus.RUNNING):
            job.status = JobStatus.CANCELLED
            job.completed_at = time.time()

            # Remove from queue if still queued
            if job_id in self._queue:
                self._queue.remove(job_id)

            return True

        return False

    def queue_size(self) -> int:
        """Get the number of queued jobs"""
        return sum(1 for job in self._jobs.values() if job.status == JobStatus.QUEUED)

    def active_jobs(self) -> int:
        """Get the number of running jobs"""
        return sum(1 for job in self._jobs.values() if job.status == JobStatus.RUNNING)

    def estimate_time(self, steps: int) -> float:
        """Estimate generation time based on steps"""
        # Simple estimation: 0.1 seconds per step
        # This should be updated based on actual performance
        return steps * 0.1

    def get_all_jobs(self) -> List[Job]:
        """Get all jobs"""
        return list(self._jobs.values())

    def clear_completed(self, max_age_seconds: float = 3600) -> int:
        """Clear old completed jobs"""
        now = time.time()
        to_remove = []

        for job_id, job in self._jobs.items():
            if job.status in (JobStatus.COMPLETED, JobStatus.FAILED, JobStatus.CANCELLED):
                if job.completed_at and (now - job.completed_at) > max_age_seconds:
                    to_remove.append(job_id)

        for job_id in to_remove:
            del self._jobs[job_id]

        return len(to_remove)
