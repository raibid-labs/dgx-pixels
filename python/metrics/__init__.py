"""
DGX-Pixels Custom Metrics Module

This module provides Prometheus metrics instrumentation for the DGX-Pixels
generation pipeline, exposing application-level metrics for observability.

Metrics exposed:
  - dgx_pixels_images_generated_total: Total images successfully generated
  - dgx_pixels_generation_failures_total: Total generation failures
  - dgx_pixels_generation_duration_seconds: Histogram of generation times
  - dgx_pixels_active_jobs: Current number of active generation jobs
  - dgx_pixels_queue_depth: Current job queue depth
  - zmq_message_queue_depth: ZeroMQ message queue depth
  - zmq_connection_errors_total: ZeroMQ connection errors
  - comfyui_api_errors_total: ComfyUI API errors

Usage:
    from python.metrics import (
        start_metrics_server,
        track_generation,
        images_generated,
        generation_failures
    )

    # Start metrics server on port 8000
    start_metrics_server(port=8000)

    # Track generation with context manager
    with track_generation():
        generate_sprite(prompt)

    # Or manually
    images_generated.inc()
"""

from .exporter import (
    # Metrics
    images_generated,
    generation_failures,
    generation_duration,
    active_jobs,
    queue_depth,
    zmq_message_queue_depth,
    zmq_connection_errors,
    comfyui_api_errors,
    # Utilities
    start_metrics_server,
    track_generation,
)

__all__ = [
    "images_generated",
    "generation_failures",
    "generation_duration",
    "active_jobs",
    "queue_depth",
    "zmq_message_queue_depth",
    "zmq_connection_errors",
    "comfyui_api_errors",
    "start_metrics_server",
    "track_generation",
]
