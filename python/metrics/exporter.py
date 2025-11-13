"""
Prometheus Metrics Exporter for DGX-Pixels

This module defines and exports custom application metrics for the DGX-Pixels
generation pipeline. Metrics are exposed in Prometheus format on /metrics endpoint.

Architecture:
  - Uses prometheus_client library
  - Metrics exposed on HTTP server (default port 8000)
  - Thread-safe counters, histograms, and gauges
  - Context managers for automatic instrumentation

Performance:
  - Minimal overhead (<1% CPU)
  - Lock-free atomic operations where possible
  - Efficient histogram bucketing for latency

Metrics Categories:
  1. Generation Pipeline: image counts, latency, failures
  2. Queue Management: active jobs, queue depth
  3. IPC Communication: ZeroMQ message queues, errors
  4. External APIs: ComfyUI API errors

Example:
    # Start metrics server
    start_metrics_server(port=8000)

    # Instrument generation
    @generation_duration.time()
    async def generate_sprite(prompt: str):
        active_jobs.inc()
        try:
            result = await comfyui_generate(prompt)
            images_generated.inc()
            return result
        except Exception as e:
            generation_failures.inc()
            raise
        finally:
            active_jobs.dec()
"""

import logging
import time
from contextlib import contextmanager
from typing import Optional

from prometheus_client import (
    Counter,
    Gauge,
    Histogram,
    Info,
    start_http_server,
    REGISTRY,
)

logger = logging.getLogger(__name__)

################################################################################
# Metric Definitions
################################################################################

# Generation Pipeline Metrics
# ----------------------------------------------------------------------------

images_generated = Counter(
    "dgx_pixels_images_generated_total",
    "Total number of images successfully generated",
    labelnames=["workflow", "model"],
)

generation_failures = Counter(
    "dgx_pixels_generation_failures_total",
    "Total number of generation failures",
    labelnames=["workflow", "error_type"],
)

generation_duration = Histogram(
    "dgx_pixels_generation_duration_seconds",
    "Time spent generating images (end-to-end)",
    labelnames=["workflow"],
    # Buckets optimized for SDXL generation times (3-30 seconds typical)
    buckets=[0.5, 1.0, 2.0, 3.0, 5.0, 7.0, 10.0, 15.0, 20.0, 30.0, 45.0, 60.0],
)

# Queue Management Metrics
# ----------------------------------------------------------------------------

active_jobs = Gauge(
    "dgx_pixels_active_jobs",
    "Number of currently active generation jobs",
)

queue_depth = Gauge(
    "dgx_pixels_queue_depth",
    "Number of jobs waiting in queue",
)

# ZeroMQ IPC Metrics
# ----------------------------------------------------------------------------

zmq_message_queue_depth = Gauge(
    "zmq_message_queue_depth",
    "Depth of ZeroMQ message queue",
    labelnames=["endpoint"],
)

zmq_connection_errors = Counter(
    "zmq_connection_errors_total",
    "Total ZeroMQ connection errors",
    labelnames=["endpoint", "error_type"],
)

zmq_message_sent = Counter(
    "zmq_messages_sent_total",
    "Total ZeroMQ messages sent",
    labelnames=["endpoint", "message_type"],
)

zmq_message_received = Counter(
    "zmq_messages_received_total",
    "Total ZeroMQ messages received",
    labelnames=["endpoint", "message_type"],
)

zmq_message_latency = Histogram(
    "zmq_message_latency_seconds",
    "ZeroMQ message round-trip latency",
    labelnames=["endpoint"],
    # Buckets for sub-millisecond to multi-second latency
    buckets=[0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0],
)

# ComfyUI Integration Metrics
# ----------------------------------------------------------------------------

comfyui_api_errors = Counter(
    "comfyui_api_errors_total",
    "Total ComfyUI API errors",
    labelnames=["endpoint", "error_type"],
)

comfyui_api_latency = Histogram(
    "comfyui_api_latency_seconds",
    "ComfyUI API request latency",
    labelnames=["endpoint"],
    buckets=[0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 5.0, 10.0],
)

comfyui_workflow_queue = Gauge(
    "comfyui_workflow_queue_depth",
    "ComfyUI workflow queue depth",
)

# System Information Metrics
# ----------------------------------------------------------------------------

system_info = Info(
    "dgx_pixels_system",
    "DGX-Pixels system information",
)

# Set system info (called once at startup)
system_info.info({
    "version": "0.1.0",
    "hardware": "dgx-spark-gb10",
    "gpu_model": "grace-blackwell",
    "backend": "python",
    "framework": "comfyui",
})


################################################################################
# Metric Helpers and Context Managers
################################################################################


@contextmanager
def track_generation(workflow: str = "default", model: str = "sdxl"):
    """
    Context manager to automatically track generation metrics.

    Tracks:
      - Generation duration (histogram)
      - Active jobs (gauge)
      - Success/failure counters

    Usage:
        with track_generation(workflow="sprite_optimized", model="sdxl_base"):
            result = generate_sprite(prompt)

    Args:
        workflow: Workflow name for labeling
        model: Model name for labeling

    Yields:
        None
    """
    active_jobs.inc()
    start_time = time.perf_counter()

    try:
        yield
        # Success - increment generated counter
        images_generated.labels(workflow=workflow, model=model).inc()
    except Exception as e:
        # Failure - increment failure counter
        error_type = type(e).__name__
        generation_failures.labels(workflow=workflow, error_type=error_type).inc()
        raise
    finally:
        # Always record duration and decrement active jobs
        duration = time.perf_counter() - start_time
        generation_duration.labels(workflow=workflow).observe(duration)
        active_jobs.dec()


@contextmanager
def track_zmq_latency(endpoint: str):
    """
    Context manager to track ZeroMQ message latency.

    Usage:
        with track_zmq_latency("tcp://127.0.0.1:5555"):
            response = socket.send_and_receive(message)

    Args:
        endpoint: ZeroMQ endpoint identifier
    """
    start_time = time.perf_counter()
    try:
        yield
    finally:
        latency = time.perf_counter() - start_time
        zmq_message_latency.labels(endpoint=endpoint).observe(latency)


@contextmanager
def track_comfyui_api(endpoint: str):
    """
    Context manager to track ComfyUI API latency and errors.

    Usage:
        with track_comfyui_api("/prompt"):
            response = requests.post(comfyui_url, json=payload)

    Args:
        endpoint: API endpoint path
    """
    start_time = time.perf_counter()
    try:
        yield
    except Exception as e:
        error_type = type(e).__name__
        comfyui_api_errors.labels(endpoint=endpoint, error_type=error_type).inc()
        raise
    finally:
        latency = time.perf_counter() - start_time
        comfyui_api_latency.labels(endpoint=endpoint).observe(latency)


################################################################################
# Metrics Server
################################################################################


def start_metrics_server(port: int = 8000, addr: str = "0.0.0.0"):
    """
    Start Prometheus metrics HTTP server.

    The server exposes metrics on the /metrics endpoint in Prometheus format.
    This should be called once at application startup.

    Args:
        port: HTTP port to listen on (default: 8000)
        addr: Address to bind to (default: 0.0.0.0 for all interfaces)

    Example:
        # Start metrics server on default port
        start_metrics_server()

        # Start on custom port
        start_metrics_server(port=9090)

        # Bind to localhost only
        start_metrics_server(addr="127.0.0.1")

    Note:
        This is a blocking call that starts a background thread.
        The server runs until the process exits.
    """
    try:
        start_http_server(port=port, addr=addr)
        logger.info(f"Prometheus metrics server started on {addr}:{port}")
        logger.info(f"Metrics endpoint: http://{addr}:{port}/metrics")
    except OSError as e:
        logger.error(f"Failed to start metrics server on {addr}:{port}: {e}")
        raise


################################################################################
# Manual Metric Recording Functions
################################################################################


def record_image_generated(workflow: str = "default", model: str = "sdxl"):
    """Manually record a successful image generation."""
    images_generated.labels(workflow=workflow, model=model).inc()


def record_generation_failure(
    workflow: str = "default", error_type: str = "UnknownError"
):
    """Manually record a generation failure."""
    generation_failures.labels(workflow=workflow, error_type=error_type).inc()


def record_generation_duration(duration_seconds: float, workflow: str = "default"):
    """Manually record generation duration."""
    generation_duration.labels(workflow=workflow).observe(duration_seconds)


def set_queue_depth(depth: int):
    """Set the current queue depth gauge."""
    queue_depth.set(depth)


def set_zmq_queue_depth(depth: int, endpoint: str):
    """Set the ZeroMQ message queue depth."""
    zmq_message_queue_depth.labels(endpoint=endpoint).set(depth)


def record_zmq_message_sent(endpoint: str, message_type: str):
    """Record a ZeroMQ message sent."""
    zmq_message_sent.labels(endpoint=endpoint, message_type=message_type).inc()


def record_zmq_message_received(endpoint: str, message_type: str):
    """Record a ZeroMQ message received."""
    zmq_message_received.labels(endpoint=endpoint, message_type=message_type).inc()


def record_zmq_error(endpoint: str, error_type: str):
    """Record a ZeroMQ connection error."""
    zmq_connection_errors.labels(endpoint=endpoint, error_type=error_type).inc()


def set_comfyui_queue_depth(depth: int):
    """Set the ComfyUI workflow queue depth."""
    comfyui_workflow_queue.set(depth)


################################################################################
# Health Check Endpoint Data
################################################################################


def get_current_metrics() -> dict:
    """
    Get current metric values for health checks or debugging.

    Returns:
        Dictionary with current metric values
    """
    return {
        "active_jobs": active_jobs._value.get(),
        "queue_depth": queue_depth._value.get(),
        "images_generated_total": sum(
            [
                images_generated.labels(workflow=w, model=m)._value.get()
                for w, m in [("default", "sdxl")]  # Add all known labels
            ]
        ),
        "generation_failures_total": sum(
            [
                generation_failures.labels(workflow=w, error_type=e)._value.get()
                for w, e in [("default", "UnknownError")]  # Add all known labels
            ]
        ),
    }
