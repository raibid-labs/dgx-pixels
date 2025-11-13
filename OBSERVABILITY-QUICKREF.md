# DGX-Pixels Observability Quick Reference

**One-page reference for monitoring, metrics, and troubleshooting**

---

## Access URLs

| Service | URL | Credentials |
|---------|-----|-------------|
| Grafana | http://localhost:3000 | admin / admin |
| Prometheus | http://localhost:9090 | - |
| DCGM Metrics | http://localhost:9400/metrics | - |
| App Metrics | http://localhost:8000/metrics | - |
| Node Metrics | http://localhost:9100/metrics | - |

---

## Quick Start

```bash
# Start observability stack
cd /home/beengud/raibid-labs/dgx-pixels
./scripts/setup-observability.sh

# Test everything is working
./scripts/test-metrics.sh

# View dashboards
open http://localhost:3000
```

---

## Dashboards

### 1. GPU Performance (dgx-pixels-gpu)
- GPU utilization, VRAM, temperature, power
- Clock speeds, PCIe throughput
- NVLink bandwidth (Grace Blackwell C2C)

### 2. Generation Quality (dgx-pixels-quality)
- Images/second, active jobs, queue depth
- P50/P95/P99 latency
- Success rate, failures

### 3. System Health (dgx-pixels-health)
- Service status (Backend, ComfyUI, DCGM)
- GPU errors (XID, ECC)
- Active alerts, system resources

---

## Key Metrics

### GPU Metrics (DCGM)
```promql
# GPU utilization
DCGM_FI_DEV_GPU_UTIL

# VRAM usage percentage
(DCGM_FI_DEV_FB_USED / DCGM_FI_DEV_FB_TOTAL) * 100

# GPU temperature
DCGM_FI_DEV_GPU_TEMP

# Power consumption
DCGM_FI_DEV_POWER_USAGE

# Tensor Core utilization
DCGM_FI_PROF_PIPE_TENSOR_ACTIVE
```

### Application Metrics
```promql
# Images per second
rate(dgx_pixels_images_generated_total[1m])

# P95 generation latency
histogram_quantile(0.95, rate(dgx_pixels_generation_duration_seconds_bucket[5m]))

# Success rate
(rate(dgx_pixels_images_generated_total[5m]) /
 (rate(dgx_pixels_images_generated_total[5m]) +
  rate(dgx_pixels_generation_failures_total[5m]))) * 100

# Queue depth
dgx_pixels_queue_depth
```

---

## Critical Alerts

| Alert | Threshold | Action |
|-------|-----------|--------|
| VRAM >98% | 1 minute | Reduce batch size, check for leaks |
| Temp >90°C | 2 minutes | Check cooling, reduce load |
| XID Errors | Any | Run `nvidia-smi`, check `dmesg` |
| ECC Errors | Any DBE | Memory corruption - check GPU health |
| No Images | 5 minutes | Check worker and ComfyUI status |
| Failure Rate >10% | 5 minutes | Check logs for error patterns |

---

## Troubleshooting

### DCGM Not Working
```bash
# Check DCGM container
docker-compose ps dcgm-exporter
docker-compose logs dcgm-exporter

# Test GPU access
nvidia-smi
curl http://localhost:9400/metrics | grep DCGM_FI_DEV_GPU_UTIL
```

### Prometheus Not Scraping
```bash
# Check targets
curl http://localhost:9090/api/v1/targets | jq

# Test network connectivity
docker-compose exec prometheus wget -O- http://dcgm-exporter:9400/metrics
```

### Grafana Shows "No Data"
```bash
# Test Prometheus datasource
curl http://localhost:9090/api/v1/query?query=up

# Check time range (default: last 15 min)
# Verify metrics exist in Prometheus
```

### Application Metrics Missing
```bash
# Check metrics endpoint
curl http://localhost:8000/metrics

# Verify metrics server started
docker-compose logs dgx-pixels | grep "metrics server"
```

---

## Code Integration

### Start Metrics Server
```python
from python.metrics import start_metrics_server

# In worker main()
start_metrics_server(port=8000)
```

### Track Generation
```python
from python.metrics import track_generation

# Automatic tracking
with track_generation(workflow="sprite_optimized", model="sdxl_base"):
    result = generate_sprite(prompt)
```

### Track ZeroMQ Latency
```python
from python.metrics import track_zmq_latency

with track_zmq_latency("tcp://127.0.0.1:5555"):
    response = socket.send_and_receive(message)
```

### Update Queue Depth
```python
from python.metrics import set_queue_depth

set_queue_depth(len(job_queue))
```

---

## Service Management

### Start/Stop Services
```bash
# Start all
docker-compose up -d

# Start observability only
docker-compose up -d dcgm-exporter prometheus grafana node-exporter

# Stop all
docker-compose down

# Restart specific service
docker-compose restart prometheus
```

### View Logs
```bash
# All observability services
docker-compose logs -f grafana prometheus dcgm-exporter

# Specific service
docker-compose logs -f prometheus

# Last 100 lines
docker-compose logs --tail=100 grafana
```

### Check Health
```bash
# All services
docker-compose ps

# Specific health checks
curl http://localhost:9090/-/healthy   # Prometheus
curl http://localhost:3000/api/health  # Grafana
curl http://localhost:9400/metrics     # DCGM
```

---

## Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| Throughput | >12 images/min | 5-min average |
| P95 Latency | <5 seconds | 5-min window |
| Success Rate | >98% | 24-hour average |
| Queue Depth | <20 jobs | Real-time |
| GPU Utilization | 70-90% | 5-min average |
| VRAM Usage | <90% | Real-time |
| GPU Temperature | <85°C | Real-time |

---

## Useful Commands

### Prometheus Queries
```bash
# Query via curl
curl -s 'http://localhost:9090/api/v1/query?query=DCGM_FI_DEV_GPU_UTIL' | jq

# Query time range
curl -s 'http://localhost:9090/api/v1/query_range?query=up&start=2025-01-12T00:00:00Z&end=2025-01-12T12:00:00Z&step=15s' | jq

# List all metrics
curl http://localhost:9090/api/v1/label/__name__/values | jq
```

### Grafana API
```bash
# List datasources
curl -s -u admin:admin http://localhost:3000/api/datasources | jq

# List dashboards
curl -s -u admin:admin http://localhost:3000/api/search?type=dash-db | jq

# Get dashboard by UID
curl -s -u admin:admin http://localhost:3000/api/dashboards/uid/dgx-pixels-gpu | jq
```

### DCGM Metrics
```bash
# View all GPU metrics
curl -s http://localhost:9400/metrics | grep DCGM_FI_DEV

# Check specific metric
curl -s http://localhost:9400/metrics | grep DCGM_FI_DEV_GPU_UTIL

# Count metric types
curl -s http://localhost:9400/metrics | grep DCGM | wc -l
```

---

## File Locations

### Configuration
```
/home/beengud/raibid-labs/dgx-pixels/deploy/dcgm/dcgm-exporter.yaml
/home/beengud/raibid-labs/dgx-pixels/deploy/prometheus/prometheus.yml
/home/beengud/raibid-labs/dgx-pixels/deploy/prometheus/alerts/dgx-pixels.yml
/home/beengud/raibid-labs/dgx-pixels/deploy/grafana/datasources/prometheus.yaml
```

### Dashboards
```
/home/beengud/raibid-labs/dgx-pixels/deploy/grafana/dashboards/gpu-performance.json
/home/beengud/raibid-labs/dgx-pixels/deploy/grafana/dashboards/generation-quality.json
/home/beengud/raibid-labs/dgx-pixels/deploy/grafana/dashboards/system-health.json
```

### Python Metrics
```
/home/beengud/raibid-labs/dgx-pixels/python/metrics/__init__.py
/home/beengud/raibid-labs/dgx-pixels/python/metrics/exporter.py
```

### Scripts
```
/home/beengud/raibid-labs/dgx-pixels/scripts/setup-observability.sh
/home/beengud/raibid-labs/dgx-pixels/scripts/test-metrics.sh
```

### Documentation
```
/home/beengud/raibid-labs/dgx-pixels/docs/observability-guide.md
/home/beengud/raibid-labs/dgx-pixels/WS-16-COMPLETION.md
```

---

## Resources

- **Full Documentation**: `/home/beengud/raibid-labs/dgx-pixels/docs/observability-guide.md`
- **Integration Example**: `/home/beengud/raibid-labs/dgx-pixels/examples/metrics_integration_example.py`
- **DCGM Docs**: https://docs.nvidia.com/datacenter/dcgm/
- **Prometheus Docs**: https://prometheus.io/docs/
- **Grafana Docs**: https://grafana.com/docs/

---

**Last Updated**: 2025-01-12
**Quick Reference Version**: 1.0.0
