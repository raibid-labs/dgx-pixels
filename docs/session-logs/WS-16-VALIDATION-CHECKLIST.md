# WS-16 Validation Checklist

Use this checklist to validate the observability stack implementation.

## Pre-Deployment Validation

### File Structure
- [ ] `/home/beengud/raibid-labs/dgx-pixels/deploy/dcgm/dcgm-exporter.yaml` exists
- [ ] `/home/beengud/raibid-labs/dgx-pixels/deploy/prometheus/prometheus.yml` exists
- [ ] `/home/beengud/raibid-labs/dgx-pixels/deploy/prometheus/alerts/dgx-pixels.yml` exists
- [ ] `/home/beengud/raibid-labs/dgx-pixels/deploy/grafana/datasources/prometheus.yaml` exists
- [ ] `/home/beengud/raibid-labs/dgx-pixels/deploy/grafana/dashboards/dashboards.yaml` exists
- [ ] Three dashboard JSON files exist in `deploy/grafana/dashboards/`
- [ ] Python metrics module exists in `python/metrics/`
- [ ] Setup script is executable: `scripts/setup-observability.sh`
- [ ] Test script is executable: `scripts/test-metrics.sh`

### Configuration Validation
- [ ] DCGM config includes Grace Blackwell specific metrics (NVLink, Tensor Core)
- [ ] Prometheus scrape targets include: dcgm, dgx-pixels-backend, node, comfyui
- [ ] Alert rules file contains 14+ alert definitions
- [ ] Docker compose includes all observability services

## Deployment Validation

### Service Startup
- [ ] Run `./scripts/setup-observability.sh` completes successfully
- [ ] DCGM Exporter container is running: `docker-compose ps dcgm-exporter`
- [ ] Prometheus container is running: `docker-compose ps prometheus`
- [ ] Grafana container is running: `docker-compose ps grafana`
- [ ] Node Exporter container is running: `docker-compose ps node-exporter`

### Health Checks
- [ ] DCGM metrics accessible: `curl http://localhost:9400/metrics`
- [ ] Prometheus accessible: `curl http://localhost:9090/-/healthy`
- [ ] Grafana accessible: `curl http://localhost:3000/api/health`
- [ ] Node Exporter accessible: `curl http://localhost:9100/metrics`

### Metrics Collection
- [ ] DCGM exports GPU utilization: `curl http://localhost:9400/metrics | grep DCGM_FI_DEV_GPU_UTIL`
- [ ] DCGM exports VRAM metrics: `curl http://localhost:9400/metrics | grep DCGM_FI_DEV_FB_USED`
- [ ] DCGM exports temperature: `curl http://localhost:9400/metrics | grep DCGM_FI_DEV_GPU_TEMP`
- [ ] DCGM exports NVLink bandwidth: `curl http://localhost:9400/metrics | grep DCGM_FI_DEV_NVLINK`
- [ ] DCGM exports Tensor Core metrics: `curl http://localhost:9400/metrics | grep DCGM_FI_PROF_PIPE_TENSOR`

### Prometheus Scraping
- [ ] DCGM target is UP: Check http://localhost:9090/targets
- [ ] Node Exporter target is UP: Check http://localhost:9090/targets
- [ ] Prometheus self-monitoring is UP: Check http://localhost:9090/targets
- [ ] GPU metrics queryable: `curl 'http://localhost:9090/api/v1/query?query=DCGM_FI_DEV_GPU_UTIL'`

### Grafana Configuration
- [ ] Grafana UI loads: http://localhost:3000
- [ ] Login works with admin/admin
- [ ] Prometheus datasource configured: Grafana → Configuration → Data sources
- [ ] DGX-Pixels folder exists in Dashboards
- [ ] GPU Performance dashboard exists and loads
- [ ] Generation Quality dashboard exists and loads
- [ ] System Health dashboard exists and loads

### Dashboard Functionality
- [ ] GPU Performance dashboard shows GPU utilization gauge
- [ ] GPU Performance dashboard shows VRAM usage gauge
- [ ] GPU Performance dashboard shows temperature graph
- [ ] Generation Quality dashboard shows images/second gauge
- [ ] System Health dashboard shows service status

### Alert Rules
- [ ] Alert rules loaded in Prometheus: http://localhost:9090/rules
- [ ] Alert groups visible: gpu_critical, generation_pipeline, system_resources
- [ ] Critical alerts present: GPUCriticalVRAMUsage, GPUXIDErrors
- [ ] Warning alerts present: GPUHighVRAMUsage, HighGenerationLatency

## Automated Testing

### Test Script Execution
- [ ] Run `./scripts/test-metrics.sh`
- [ ] All endpoint tests pass (4/4)
- [ ] All DCGM metric tests pass (6/6)
- [ ] All Prometheus target tests pass (3/3)
- [ ] All Prometheus query tests pass (3/3)
- [ ] All Grafana datasource tests pass (2/2)
- [ ] All Grafana dashboard tests pass (3/3)
- [ ] All alert rule tests pass (3/3)
- [ ] Overall: 25/25 tests pass

## Integration Testing

### Python Metrics Module
- [ ] Can import metrics module: `python3 -c "from python.metrics import start_metrics_server"`
- [ ] Metrics server starts without errors
- [ ] Metrics endpoint accessible: `curl http://localhost:8000/metrics`
- [ ] Custom metrics defined: dgx_pixels_images_generated_total
- [ ] Histogram buckets configured correctly (12 buckets)

### Example Integration
- [ ] Example code runs: `python3 examples/metrics_integration_example.py`
- [ ] Metrics appear at http://localhost:8000/metrics
- [ ] Context managers work correctly
- [ ] Metrics update in real-time

## Performance Validation

### Resource Usage
- [ ] DCGM Exporter CPU <5%
- [ ] Prometheus CPU <10%
- [ ] Grafana CPU <5%
- [ ] Total memory usage <2 GB

### Storage
- [ ] Prometheus data directory created: `/var/lib/docker/volumes/dgx-pixels-prometheus-data`
- [ ] Grafana data directory created: `/var/lib/docker/volumes/dgx-pixels-grafana-data`
- [ ] Initial storage <500 MB

### Query Performance
- [ ] Simple queries complete <100ms
- [ ] Histogram quantile queries complete <200ms
- [ ] Dashboard refresh <2 seconds

## Documentation Validation

### Documentation Exists
- [ ] `/home/beengud/raibid-labs/dgx-pixels/docs/observability-guide.md` exists (550+ lines)
- [ ] `/home/beengud/raibid-labs/dgx-pixels/WS-16-COMPLETION.md` exists
- [ ] `/home/beengud/raibid-labs/dgx-pixels/OBSERVABILITY-QUICKREF.md` exists
- [ ] `/home/beengud/raibid-labs/dgx-pixels/deploy/README.md` exists

### Documentation Quality
- [ ] Architecture diagrams present
- [ ] Quick start instructions clear
- [ ] All components documented
- [ ] Troubleshooting section complete
- [ ] Examples provided

## End-to-End Validation

### Complete Workflow
- [ ] Start observability stack
- [ ] Generate test image (simulated or real)
- [ ] Watch metrics update in real-time
- [ ] Verify generation metrics appear in Grafana
- [ ] Verify GPU metrics update during generation
- [ ] Check queue depth metric updates
- [ ] Verify success/failure counters work

### Alert Testing
- [ ] Can query alert rules: `curl http://localhost:9090/api/v1/rules`
- [ ] Alerts would fire on threshold breach (manual verification)
- [ ] Alert annotations include actionable information

## Production Readiness

### Security
- [ ] Change Grafana admin password (post-deployment)
- [ ] Review exposed ports
- [ ] Consider enabling HTTPS (future)

### Monitoring
- [ ] 15-second scrape interval acceptable for use case
- [ ] 30-day retention sufficient
- [ ] Metric cardinality <10,000 series

### Operations
- [ ] Backup plan for Prometheus data (future)
- [ ] Grafana dashboard export tested
- [ ] Service restart procedures documented

## Sign-Off

### Acceptance Criteria
- [ ] All 7 acceptance criteria from WS-16 met
- [ ] All automated tests pass
- [ ] Documentation complete
- [ ] Production-ready

### Final Validation
- [ ] Demo to stakeholders
- [ ] Handoff to operations team
- [ ] Training materials provided

---

**Validation Date**: _________________
**Validated By**: _________________
**Status**: PASS / FAIL
**Notes**: _________________

