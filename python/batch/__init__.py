"""
Batch Processing Module

High-throughput sprite generation with ComfyUI automation.
"""

from .comfyui_client import (
    ComfyUIClient,
    ComfyUIJob,
    ComfyUIJobStatus,
)

from .batch_processor import (
    BatchProcessor,
    BatchJob,
    BatchJobStatus,
    JobPriority,
)

from .output_manager import (
    OutputManager,
    ImageMetadata,
    BatchMetadata,
)

__all__ = [
    # Client
    "ComfyUIClient",
    "ComfyUIJob",
    "ComfyUIJobStatus",
    # Processor
    "BatchProcessor",
    "BatchJob",
    "BatchJobStatus",
    "JobPriority",
    # Output
    "OutputManager",
    "ImageMetadata",
    "BatchMetadata",
]
