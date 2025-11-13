# DGX-Pixels Observability Guide

**Comprehensive guide to monitoring, metrics, and observability for DGX-Pixels**

This guide covers the complete observability stack for DGX-Pixels, including GPU metrics, application performance, system health, and alerting.

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Quick Start](#quick-start)
3. [Component Details](#component-details)
4. [Dashboard Guide](#dashboard-guide)
5. [Metrics Reference](#metrics-reference)
6. [Alerting](#alerting)
7. [Troubleshooting](#troubleshooting)
8. [Performance Tuning](#performance-tuning)
9. [Capacity Planning](#capacity-planning)

---

## Architecture Overview

The DGX-Pixels observability stack consists of five main components:

```
┌─────────────────────────────────────────────────────────────┐
│                      DGX-Pixels Stack                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│  │ Python       │  │ ComfyUI      │  │ Grace Blackwell │  │
│  │ Backend      │  │ (Inference)  │  │ GB10 GPU        │  │
│  └──────┬───────┘  └──────┬───────┘  └────────┬────────┘  │
│         │ metrics          │ metrics            │ metrics   │
│         ▼                  ▼                    ▼           │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│  │ App Metrics  │  │ Node         │  │ DCGM Exporter   │  │
│  │ :8000/metrics│  │ Exporter     │  │ :9400/metrics   │  │
│  └──────┬───────┘  └──────┬───────┘  └────────┬────────┘  │
│         │                  │                    │           │
│         └──────────────────┴────────────────────┘           │
│                            │                                │
│                            ▼                                │
│                   ┌─────────────────┐                       │
│                   │  Prometheus     │                       │
│                   │  :9090          │                       │
│                   │  (30d retention)│                       │
│                   └────────┬────────┘                       │
│                            │                                │
│                            ▼                                │
│                   ┌─────────────────┐                       │
│                   │  Grafana        │                       │
│                   │  :3000          │                       │
│                   │  (Visualization)│                       │
│                   └─────────────────┘                       │
└─────────────────────────────────────────────────────────────┘
```

### Component Ports

| Component        | Port | Purpose                          |
|------------------|------|----------------------------------|
| App Metrics      | 8000 | Python backend metrics           |
| Worker Metrics   | 8001 | ZeroMQ worker metrics            |
| Grafana          | 3000 | Dashboard UI                     |
| Prometheus       | 9090 | Metrics storage & query          |
| Node Exporter    | 9100 | System metrics                   |
| DCGM Exporter    | 9400 | GPU metrics                      |

---

## Quick Start

### 1. Start the Observability Stack

```bash
# Navigate to docker directory
cd /home/beengud/raibid-labs/dgx-pixels/docker

# Start all services (includes observability stack)
docker-compose up -d

# Verify services are running
docker-compose ps

# Check logs
docker-compose logs -f grafana prometheus dcgm-exporter
```

### 2. Access Dashboards

Open your browser and navigate to:

- **Grafana**: http://localhost:3000
  - Username: `admin`
  - Password: `admin` (change on first login)

- **Prometheus**: http://localhost:9090
  - Direct metric queries and exploration

- **DCGM Metrics**: http://localhost:9400/metrics
  - Raw GPU metrics in Prometheus format

### 3. View Pre-Built Dashboards

In Grafana, navigate to **Dashboards** → **DGX-Pixels** folder:

1. **GPU Performance** - Real-time GPU utilization, VRAM, temperature, power
2. **Generation Quality** - Image generation throughput, latency, success rate
3. **System Health** - Service status, errors, alerts, resource usage

---

## Component Details

### DCGM Exporter

**Purpose**: Exposes GPU metrics from NVIDIA Grace Blackwell GB10 in Prometheus format.

**Key Metrics**:
- GPU utilization (%)
- VRAM usage (GB)
- Temperature (°C)
- Power consumption (W)
- NVLink bandwidth (GB/s)
- ECC errors, XID errors

**Configuration**: `/home/beengud/raibid-labs/dgx-pixels/deploy/dcgm/dcgm-exporter.yaml`

**Metric Groups**:
- `gpu_utilization`: 1s interval - GPU activity, SM occupancy
- `memory_metrics`: 1s interval - VRAM usage, memory clock
- `power_thermal`: 5s interval - Power, temperature, thermal violations
- `performance`: 1s interval - Clock speeds, PCIe/NVLink throughput
- `errors`: 10s interval - XID errors, ECC errors, retired pages
- `compute`: 1s interval - Tensor Core, FP16/32/64 pipe utilization

**Health Check**:
```bash
curl http://localhost:9400/metrics | grep DCGM_FI_DEV_GPU_UTIL
```

### Prometheus

**Purpose**: Time-series database for metrics storage and querying.

**Configuration**: `/home/beengud/raibid-labs/dgx-pixels/deploy/prometheus/prometheus.yml`

**Scrape Targets**:
- DCGM (GPU metrics) - 15s interval
- DGX-Pixels Backend - 15s interval
- Node Exporter (system) - 15s interval
- ComfyUI - 30s interval

**Retention**:
- Time: 30 days
- Size: 50 GB max
- Estimated usage: ~10 GB for 30 days

**Query Examples**:

```promql
# GPU utilization
DCGM_FI_DEV_GPU_UTIL

# VRAM usage percentage
(DCGM_FI_DEV_FB_USED / DCGM_FI_DEV_FB_TOTAL) * 100

# P95 generation latency
histogram_quantile(0.95, rate(dgx_pixels_generation_duration_seconds_bucket[5m]))

# Images per minute
rate(dgx_pixels_images_generated_total[1m]) * 60

# Success rate
(rate(dgx_pixels_images_generated_total[5m]) /
 (rate(dgx_pixels_images_generated_total[5m]) +
  rate(dgx_pixels_generation_failures_total[5m]))) * 100
```

**Web UI**: http://localhost:9090

### Grafana

**Purpose**: Visualization and dashboarding for metrics.

**Configuration**:
- Datasources: `/home/beengud/raibid-labs/dgx-pixels/deploy/grafana/datasources/`
- Dashboards: `/home/beengud/raibid-labs/dgx-pixels/deploy/grafana/dashboards/`

**Pre-Built Dashboards**:
1. **GPU Performance** (UID: `dgx-pixels-gpu`)
2. **Generation Quality** (UID: `dgx-pixels-quality`)
3. **System Health** (UID: `dgx-pixels-health`)

**Web UI**: http://localhost:3000

### Python Application Metrics

**Purpose**: Custom application-level metrics for generation pipeline.

**Instrumentation**: `/home/beengud/raibid-labs/dgx-pixels/python/metrics/exporter.py`

**Usage Example**:

```python
from python.metrics import start_metrics_server, track_generation

# Start metrics server (call once at startup)
start_metrics_server(port=8000)

# Instrument generation with context manager
with track_generation(workflow="sprite_optimized", model="sdxl_base"):
    result = generate_sprite(prompt)
```

**Exposed Metrics**:
- `dgx_pixels_images_generated_total{workflow, model}`
- `dgx_pixels_generation_failures_total{workflow, error_type}`
- `dgx_pixels_generation_duration_seconds{workflow}`
- `dgx_pixels_active_jobs`
- `dgx_pixels_queue_depth`

**Endpoint**: http://localhost:8000/metrics

---

## Dashboard Guide

### Dashboard 1: GPU Performance

**Purpose**: Real-time monitoring of Grace Blackwell GB10 GPU performance.

**Panels**:

1. **GPU Utilization** (Gauge)
   - Current GPU usage percentage
   - Thresholds: Green <60%, Yellow 60-85%, Red >85%

2. **VRAM Usage** (Gauge)
   - Used vs. Total VRAM (out of 128GB)
   - Thresholds: Green <100GB, Yellow 100-120GB, Red >120GB

3. **GPU Temperature** (Time Series)
   - GPU core and memory temperature
   - Threshold line at 85°C

4. **Power Consumption** (Time Series)
   - Current power draw vs. power limit
   - Tracks power efficiency

5. **Clock Speeds** (Time Series)
   - SM clock and memory clock frequencies
   - Useful for detecting throttling

6. **PCIe Throughput** (Time Series)
   - Stacked area chart of TX + RX bandwidth
   - Identifies I/O bottlenecks

7. **NVLink Bandwidth** (Time Series)
   - Grace Blackwell C2C interconnect bandwidth
   - Critical for unified memory architecture

**Use Cases**:
- Monitor GPU saturation during generation
- Detect thermal throttling
- Identify memory bottlenecks
- Track power efficiency

### Dashboard 2: Generation Quality

**Purpose**: Monitor image generation pipeline performance and quality.

**Panels**:

1. **Images Generated per Second** (Gauge)
   - Real-time throughput
   - Target: >0.2 images/s (>12 images/min)

2. **Active Jobs** (Stat)
   - Currently running generation jobs
   - Should be 0-1 (single worker)

3. **Queue Depth** (Stat)
   - Jobs waiting in queue
   - Thresholds: Green <20, Yellow 20-50, Red >50

4. **Generation Time** (Time Series)
   - P50, P95, P99 latency percentiles
   - Target: P95 <5s, P99 <10s

5. **Success Rate** (Time Series)
   - Percentage of successful generations
   - Target: >98%

6. **Queue Depth Over Time** (Time Series)
   - Track queue growth/shrinkage
   - Identifies capacity issues

7. **Failed Generations** (Time Series - Bars)
   - Count of failures in 5-minute windows
   - Helps identify error spikes

8. **24 Hour Summary** (Table)
   - Total images generated
   - Total failures
   - Overall success rate
   - P95 latency

**Use Cases**:
- Verify generation performance meets SLOs
- Detect quality regressions
- Monitor queue backlog
- Track daily throughput

### Dashboard 3: System Health

**Purpose**: Overall system health, service status, and alerts.

**Panels**:

1. **Service Status** (Stats)
   - Backend, ComfyUI, DCGM status
   - Green = UP, Red = DOWN

2. **GPU XID Errors** (Time Series)
   - Hardware error counter
   - Any non-zero value is critical

3. **GPU ECC Errors** (Time Series)
   - Single-bit (correctable) and double-bit (uncorrectable) errors
   - DBE errors indicate memory corruption

4. **Active Alerts** (Table)
   - Currently firing alerts
   - Color-coded by severity

5. **System Memory Usage** (Time Series)
   - Host system RAM utilization
   - Target: <90%

6. **Disk Space Usage** (Time Series)
   - `/workspace` partition usage
   - Alert at 85%, critical at 95%

7. **System Uptime** (Stat)
   - Time since last system boot

8. **Active Alert Count** (Stat)
   - Number of currently firing alerts
   - Target: 0

**Use Cases**:
- Quick health check of entire stack
- Identify service outages
- Monitor hardware errors
- Track alert status

---

## Metrics Reference

### GPU Metrics (DCGM)

| Metric | Description | Unit | Normal Range |
|--------|-------------|------|--------------|
| `DCGM_FI_DEV_GPU_UTIL` | GPU utilization | % | 70-95% during generation |
| `DCGM_FI_DEV_FB_USED` | VRAM used | MB | <100,000 (100GB) |
| `DCGM_FI_DEV_FB_TOTAL` | Total VRAM | MB | 131,072 (128GB) |
| `DCGM_FI_DEV_GPU_TEMP` | GPU temperature | °C | 60-80 |
| `DCGM_FI_DEV_POWER_USAGE` | Power draw | W | Varies by model |
| `DCGM_FI_DEV_SM_CLOCK` | SM clock speed | MHz | Varies |
| `DCGM_FI_DEV_NVLINK_BANDWIDTH_TOTAL` | NVLink bandwidth | GB/s | High during inference |
| `DCGM_FI_DEV_XID_ERRORS` | Hardware errors | Count | 0 (any value is bad) |
| `DCGM_FI_PROF_PIPE_TENSOR_ACTIVE` | Tensor Core utilization | % | High during SDXL |

### Application Metrics

| Metric | Description | Type | Labels |
|--------|-------------|------|--------|
| `dgx_pixels_images_generated_total` | Images generated | Counter | workflow, model |
| `dgx_pixels_generation_failures_total` | Generation failures | Counter | workflow, error_type |
| `dgx_pixels_generation_duration_seconds` | Generation time | Histogram | workflow |
| `dgx_pixels_active_jobs` | Active jobs | Gauge | - |
| `dgx_pixels_queue_depth` | Queue depth | Gauge | - |
| `zmq_message_latency_seconds` | ZMQ round-trip time | Histogram | endpoint |
| `comfyui_api_errors_total` | ComfyUI errors | Counter | endpoint, error_type |

### System Metrics (Node Exporter)

| Metric | Description | Unit |
|--------|-------------|------|
| `node_memory_MemAvailable_bytes` | Available RAM | Bytes |
| `node_filesystem_avail_bytes` | Free disk space | Bytes |
| `node_cpu_seconds_total` | CPU time | Seconds |
| `node_boot_time_seconds` | System boot time | Unix timestamp |

---

## Alerting

### Alert Rules

Alerts are defined in `/home/beengud/raibid-labs/dgx-pixels/deploy/prometheus/alerts/dgx-pixels.yml`.

### Critical Alerts

| Alert | Trigger | Severity | Action |
|-------|---------|----------|--------|
| `GPUCriticalVRAMUsage` | VRAM >98% for 1m | Critical | Reduce batch size, check for leaks |
| `GPUCriticalTemperature` | Temp >90°C for 2m | Critical | Check cooling, reduce load |
| `GPUXIDErrors` | Any XID error | Critical | Check `nvidia-smi`, `dmesg`, contact support |
| `GPUECCDoubleBitErrors` | Any DBE error | Critical | Memory corruption - check GPU health |
| `HighFailureRate` | >10% failures for 5m | Critical | Check logs, ComfyUI status |
| `NoImagesGenerated` | 0 images for 5m | Critical | Check worker, ComfyUI connectivity |

### Warning Alerts

| Alert | Trigger | Severity | Action |
|-------|---------|----------|--------|
| `GPUHighVRAMUsage` | VRAM >95% for 2m | Warning | Monitor, consider scaling down |
| `GPUHighTemperature` | Temp >85°C for 5m | Warning | Monitor cooling |
| `HighGenerationLatency` | P95 >10s for 10m | Warning | Check GPU utilization |
| `QueueDepthGrowing` | Queue growing for 10m | Warning | Workers may be slow |

### Alert Response Procedures

**GPU XID Errors**:
1. Check GPU status: `nvidia-smi`
2. Check kernel logs: `dmesg | grep -i nvidia`
3. Review XID error code: [NVIDIA XID Error Reference](https://docs.nvidia.com/deploy/xid-errors/)
4. If persistent, contact NVIDIA support

**High VRAM Usage**:
1. Check current model loaded: `nvidia-smi`
2. Verify only one model in VRAM
3. Reduce batch size in workflow
4. Check for VRAM leaks in logs

**High Failure Rate**:
1. Check backend logs: `docker-compose logs dgx-pixels`
2. Check ComfyUI logs: `docker-compose logs comfyui`
3. Verify ComfyUI API reachable: `curl http://localhost:8188`
4. Review recent prompt patterns for issues

---

## Troubleshooting

### DCGM Exporter Not Running

**Symptoms**: No GPU metrics in Prometheus, DCGM exporter container exited.

**Checks**:
```bash
# Check DCGM container status
docker-compose ps dcgm-exporter

# Check logs
docker-compose logs dcgm-exporter

# Verify GPU visibility
docker run --rm --gpus all nvidia/cuda:12.1.0-base-ubuntu22.04 nvidia-smi
```

**Solutions**:
- Ensure NVIDIA Container Toolkit is installed
- Verify GPU access: `nvidia-smi` on host
- Check DCGM config file exists: `deploy/dcgm/dcgm-exporter.yaml`

### Prometheus Not Scraping Targets

**Symptoms**: Targets show as "DOWN" in Prometheus UI.

**Checks**:
```bash
# Check Prometheus targets
curl http://localhost:9090/api/v1/targets | jq

# Check network connectivity
docker-compose exec prometheus wget -O- http://dcgm-exporter:9400/metrics
```

**Solutions**:
- Verify all containers on same network: `dgx-pixels-net`
- Check firewall rules
- Verify service hostnames in `prometheus.yml` match container names

### Grafana Shows "No Data"

**Symptoms**: Dashboards load but panels show "No data".

**Checks**:
```bash
# Test Prometheus datasource
curl http://localhost:9090/api/v1/query?query=up

# Check Grafana datasource config
docker-compose exec grafana cat /etc/grafana/provisioning/datasources/prometheus.yaml
```

**Solutions**:
- Verify Prometheus datasource configured in Grafana
- Check time range (default: last 15 minutes)
- Verify metrics exist in Prometheus: http://localhost:9090/graph

### Metrics Not Being Exported

**Symptoms**: Application metrics missing in Prometheus.

**Checks**:
```bash
# Check metrics endpoint directly
curl http://localhost:8000/metrics

# Check if metrics server started
docker-compose logs dgx-pixels | grep "metrics server"
```

**Solutions**:
- Ensure `start_metrics_server()` called in application code
- Verify port 8000 not blocked
- Check Python worker is running

---

## Performance Tuning

### Prometheus Performance

**Reduce Cardinality**:
```yaml
# In prometheus.yml, limit labels
metric_relabel_configs:
  - source_labels: [__name__]
    regex: 'unnecessary_metric_.*'
    action: drop
```

**Adjust Scrape Intervals**:
- GPU metrics: 15s (high frequency needed)
- System metrics: 30s (less critical)
- Application metrics: 15s (important for SLO tracking)

**Retention Tuning**:
```bash
# Increase retention (in docker-compose.yml)
--storage.tsdb.retention.time=90d
--storage.tsdb.retention.size=100GB
```

### Grafana Performance

**Dashboard Optimization**:
- Use `$__interval` variable for dynamic step size
- Limit time range to relevant period (1h, 6h, 24h)
- Use recording rules for expensive queries

**Query Optimization**:
```promql
# Bad (scans all series)
sum(dgx_pixels_images_generated_total)

# Good (pre-aggregated)
sum(rate(dgx_pixels_images_generated_total[5m]))
```

---

## Capacity Planning

### Storage Requirements

**Prometheus Data**:
- Scrape interval: 15s
- Metric samples: ~100,000 per scrape
- Sample size: ~2 bytes
- Daily data: `(100,000 * 2) * (86400 / 15) = ~1.15 GB/day`
- 30-day retention: ~35 GB

**Actual usage may be lower due to compression (~10 GB for 30 days).**

### Scaling Recommendations

**Current Capacity** (Single GPU):
- Throughput: ~12-20 images/minute
- Queue capacity: 100 jobs
- Expected latency: P95 <5s

**When to Scale**:
- Queue depth consistently >50
- P95 latency >10s for >30 minutes
- Success rate <95%

**Scaling Options**:
1. **Horizontal**: Add more GPU workers
2. **Vertical**: Use larger batch sizes (if memory allows)
3. **Optimization**: Use faster SDXL variants (e.g., Turbo, Lightning)

### SLO Targets

| Metric | Target | Measurement Window |
|--------|--------|-------------------|
| Availability | 99.9% | 30 days |
| P95 Latency | <5s | 5 minutes |
| Success Rate | >98% | 24 hours |
| Queue Depth | <20 | Real-time |
| GPU Utilization | 70-90% | 5 minutes |

---

## Advanced Topics

### Recording Rules

For expensive queries, create recording rules in Prometheus:

```yaml
# deploy/prometheus/recording_rules.yml
groups:
  - name: dgx_pixels_recording
    interval: 30s
    rules:
      - record: dgx_pixels:generation_success_rate:5m
        expr: |
          (
            rate(dgx_pixels_images_generated_total[5m]) /
            (rate(dgx_pixels_images_generated_total[5m]) +
             rate(dgx_pixels_generation_failures_total[5m]))
          ) * 100
```

### Exemplars and Tracing

To link metrics to traces (future enhancement):

```python
# Add trace IDs to metrics
from opentelemetry import trace

tracer = trace.get_tracer(__name__)
with tracer.start_as_current_span("generate_sprite") as span:
    trace_id = span.get_span_context().trace_id
    # Add as label to metric
```

### External Alerting

Configure Alertmanager for Slack/PagerDuty/Email notifications:

```yaml
# deploy/alertmanager/alertmanager.yml
route:
  receiver: 'slack'
  group_by: ['alertname', 'severity']

receivers:
  - name: 'slack'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/YOUR/WEBHOOK/URL'
        channel: '#dgx-pixels-alerts'
```

---

## Resources

**Official Documentation**:
- [DCGM User Guide](https://docs.nvidia.com/datacenter/dcgm/latest/user-guide/index.html)
- [Prometheus Docs](https://prometheus.io/docs/)
- [Grafana Docs](https://grafana.com/docs/)

**DGX-Pixels Specific**:
- DCGM Config: `/home/beengud/raibid-labs/dgx-pixels/deploy/dcgm/dcgm-exporter.yaml`
- Prometheus Config: `/home/beengud/raibid-labs/dgx-pixels/deploy/prometheus/prometheus.yml`
- Alert Rules: `/home/beengud/raibid-labs/dgx-pixels/deploy/prometheus/alerts/dgx-pixels.yml`
- Metrics Exporter: `/home/beengud/raibid-labs/dgx-pixels/python/metrics/exporter.py`

**Support**:
- For GPU metrics issues: Review DCGM logs
- For application metrics: Check Python backend logs
- For infrastructure issues: Review Docker Compose logs

---

**Last Updated**: 2025-01-12
**Version**: 1.0.0
**Maintainer**: DGX-Pixels Team
