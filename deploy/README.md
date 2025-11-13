# DGX-Pixels Observability Deployment

This directory contains the complete observability stack configuration for DGX-Pixels.

## Directory Structure

```
deploy/
├── dcgm/                      # NVIDIA DCGM GPU metrics exporter
│   └── dcgm-exporter.yaml    # Metric collection configuration
├── grafana/                   # Grafana dashboards and datasources
│   ├── dashboards/
│   │   ├── dashboards.yaml           # Dashboard provisioning config
│   │   ├── gpu-performance.json      # GPU Performance dashboard
│   │   ├── generation-quality.json   # Generation Quality dashboard
│   │   └── system-health.json        # System Health dashboard
│   └── datasources/
│       └── prometheus.yaml           # Prometheus datasource config
└── prometheus/                # Prometheus metrics storage
    ├── prometheus.yml         # Main configuration
    └── alerts/
        └── dgx-pixels.yml    # Alert rules (14 rules)
```

## Components

### DCGM Exporter

**Purpose**: Exports GPU metrics from NVIDIA Grace Blackwell GB10 in Prometheus format.

**Port**: 9400

**Metrics Exported**:
- GPU utilization, VRAM usage, temperature, power
- NVLink bandwidth (Grace Blackwell C2C interconnect)
- Tensor Core utilization
- XID errors, ECC errors

**Configuration**: `dcgm/dcgm-exporter.yaml`

### Prometheus

**Purpose**: Time-series database for metrics collection and storage.

**Port**: 9090

**Retention**: 30 days, 50 GB max

**Scrape Targets**:
- DCGM Exporter (GPU metrics) - 15s interval
- DGX-Pixels Backend (app metrics) - 15s interval
- Node Exporter (system metrics) - 15s interval
- ComfyUI (inference metrics) - 30s interval

**Configuration**: `prometheus/prometheus.yml`

**Alert Rules**: `prometheus/alerts/dgx-pixels.yml`

### Grafana

**Purpose**: Dashboard visualization for metrics.

**Port**: 3000

**Default Credentials**: admin / admin

**Dashboards**:
1. **GPU Performance** (dgx-pixels-gpu) - 8 panels
2. **Generation Quality** (dgx-pixels-quality) - 8 panels
3. **System Health** (dgx-pixels-health) - 9 panels

**Configuration**: `grafana/datasources/` and `grafana/dashboards/`

## Quick Start

### 1. Start Services

```bash
cd /home/beengud/raibid-labs/dgx-pixels/docker
docker-compose up -d dcgm-exporter prometheus grafana node-exporter
```

### 2. Verify Services

```bash
# Check service health
docker-compose ps

# Check DCGM metrics
curl http://localhost:9400/metrics | grep DCGM_FI_DEV_GPU_UTIL

# Check Prometheus targets
curl http://localhost:9090/api/v1/targets | jq

# Access Grafana
open http://localhost:3000
```

### 3. View Dashboards

1. Open Grafana: http://localhost:3000
2. Login with admin/admin
3. Navigate to Dashboards → DGX-Pixels folder
4. Open "GPU Performance" dashboard

## Configuration Files

### DCGM Exporter Configuration

**File**: `dcgm/dcgm-exporter.yaml`

**Metric Groups**:
- `gpu_utilization` (1s) - GPU activity, SM occupancy
- `memory_metrics` (1s) - VRAM usage, memory clock
- `power_thermal` (5s) - Power, temperature, violations
- `performance` (1s) - Clocks, PCIe/NVLink throughput
- `errors` (10s) - XID, ECC errors
- `compute` (1s) - Tensor Core, FP16/32/64 utilization

### Prometheus Configuration

**File**: `prometheus/prometheus.yml`

**Key Settings**:
- Scrape interval: 15s
- Evaluation interval: 15s
- Retention: 30 days
- Storage size limit: 50 GB

**Scrape Configs**:
```yaml
scrape_configs:
  - job_name: 'dcgm'
    static_configs:
      - targets: ['dcgm-exporter:9400']

  - job_name: 'dgx-pixels-backend'
    static_configs:
      - targets: ['dgx-pixels:8000']

  - job_name: 'node'
    static_configs:
      - targets: ['node-exporter:9100']
```

### Alert Rules

**File**: `prometheus/alerts/dgx-pixels.yml`

**Alert Groups**:
1. `gpu_critical` - GPU VRAM, temperature, XID errors
2. `generation_pipeline` - Latency, failure rate, queue depth
3. `system_resources` - Memory, disk space
4. `zeromq_communication` - ZMQ queue, connection errors
5. `comfyui_integration` - API errors, service down

**Critical Alerts**:
- GPU VRAM >98% for 1 minute
- GPU temperature >90°C for 2 minutes
- Any XID or ECC double-bit errors
- Generation failure rate >10% for 5 minutes
- No images generated for 5 minutes

### Grafana Datasource

**File**: `grafana/datasources/prometheus.yaml`

Automatically configures Prometheus as the default datasource when Grafana starts.

### Grafana Dashboards

**File**: `grafana/dashboards/dashboards.yaml`

Configures auto-loading of JSON dashboard files from `grafana/dashboards/` directory.

**Dashboard Files**:
- `gpu-performance.json` - GPU metrics dashboard
- `generation-quality.json` - Generation pipeline dashboard
- `system-health.json` - System health dashboard

## Customization

### Adding New Metrics

To add custom application metrics:

1. Add metric definition in `/home/beengud/raibid-labs/dgx-pixels/python/metrics/exporter.py`
2. Expose metric in application code
3. Prometheus will auto-discover on next scrape

### Adding New Dashboards

To add a new Grafana dashboard:

1. Create dashboard in Grafana UI
2. Export as JSON
3. Save to `grafana/dashboards/new-dashboard.json`
4. Restart Grafana: `docker-compose restart grafana`

### Modifying Alert Rules

To modify alert rules:

1. Edit `prometheus/alerts/dgx-pixels.yml`
2. Reload Prometheus config: `curl -X POST http://localhost:9090/-/reload`
3. Verify rules loaded: http://localhost:9090/rules

## Monitoring Best Practices

### Metric Naming

Follow Prometheus naming conventions:
- Use suffixes: `_total` (counters), `_seconds` (durations), `_bytes` (sizes)
- Use base units: seconds (not milliseconds), bytes (not KB/MB)
- Namespace with prefix: `dgx_pixels_*`

### Label Design

Keep label cardinality low:
- Avoid high-cardinality labels (user IDs, timestamps, random strings)
- Use meaningful label names: `workflow`, `model`, `error_type`
- Limit unique label combinations to <1000 per metric

### Dashboard Design

- Use appropriate visualization types (gauge, graph, stat, table)
- Set meaningful thresholds (green/yellow/red)
- Include variables for filtering (GPU ID, time range)
- Use legends to explain metrics

### Alerting Strategy

- Set alerts on symptoms, not causes (user impact, not resource usage)
- Avoid alert fatigue (tune thresholds, use `for` duration)
- Include actionable information in annotations
- Test alerts fire correctly

## Troubleshooting

### DCGM Exporter Not Starting

**Issue**: Container exits immediately

**Solution**:
```bash
# Check NVIDIA drivers
nvidia-smi

# Check NVIDIA Container Toolkit
docker run --rm --gpus all nvidia/cuda:12.1.0-base-ubuntu22.04 nvidia-smi

# Check DCGM config
docker-compose logs dcgm-exporter
```

### Prometheus Not Scraping

**Issue**: Targets show as "DOWN" in Prometheus

**Solution**:
```bash
# Check network connectivity
docker-compose exec prometheus wget -O- http://dcgm-exporter:9400/metrics

# Verify service is running
docker-compose ps dcgm-exporter

# Check Prometheus logs
docker-compose logs prometheus
```

### Grafana Dashboards Not Loading

**Issue**: Dashboards missing in Grafana UI

**Solution**:
```bash
# Check dashboard files exist
ls -la grafana/dashboards/

# Check Grafana logs
docker-compose logs grafana

# Verify provisioning config
docker-compose exec grafana cat /etc/grafana/provisioning/dashboards/dashboards.yaml
```

## Performance Tuning

### Reducing Storage

If Prometheus storage grows too large:

1. Reduce retention time: `--storage.tsdb.retention.time=15d`
2. Reduce scrape frequency: Change `scrape_interval` to 30s
3. Drop unnecessary metrics with `metric_relabel_configs`

### Optimizing Queries

For slow dashboard queries:

1. Use recording rules for expensive queries
2. Limit time range to necessary window
3. Use `$__interval` variable for dynamic step size
4. Avoid regex matching in queries

## Security Considerations

### Production Deployment

For production use:

1. Change Grafana admin password
2. Enable HTTPS for Grafana (reverse proxy)
3. Restrict network access to metrics endpoints
4. Use Prometheus basic auth for sensitive metrics
5. Enable Grafana LDAP/OAuth for user management

## Resources

- **Full Documentation**: `/home/beengud/raibid-labs/dgx-pixels/docs/observability-guide.md`
- **Quick Reference**: `/home/beengud/raibid-labs/dgx-pixels/OBSERVABILITY-QUICKREF.md`
- **Setup Script**: `/home/beengud/raibid-labs/dgx-pixels/scripts/setup-observability.sh`
- **Test Script**: `/home/beengud/raibid-labs/dgx-pixels/scripts/test-metrics.sh`

## Support

For issues or questions:

1. Check logs: `docker-compose logs <service>`
2. Review documentation: `docs/observability-guide.md`
3. Run tests: `./scripts/test-metrics.sh`
4. Check service health: `docker-compose ps`

---

**Last Updated**: 2025-01-12
**Version**: 1.0.0
