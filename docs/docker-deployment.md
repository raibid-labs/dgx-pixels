# Docker Deployment Guide

Complete guide for deploying DGX-Pixels AI sprite generation stack using Docker Compose.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Quick Start](#quick-start)
3. [Service Architecture](#service-architecture)
4. [Configuration](#configuration)
5. [Starting the Stack](#starting-the-stack)
6. [Service URLs](#service-urls)
7. [Health Checks](#health-checks)
8. [Monitoring](#monitoring)
9. [Data Management](#data-management)
10. [Troubleshooting](#troubleshooting)
11. [Production Considerations](#production-considerations)
12. [Advanced Usage](#advanced-usage)

---

## Prerequisites

### Required Software

- **Docker**: v20.10 or higher
  - Install: https://docs.docker.com/engine/install/
- **Docker Compose**: v2.0 or higher (included with Docker Desktop)
  - Check version: `docker compose version`
- **NVIDIA Container Toolkit**: Required for GPU access
  - Install: https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/install-guide.html

### Required Hardware

- NVIDIA DGX-Spark GB10 (or compatible NVIDIA GPU)
- Minimum 128GB RAM (for full stack)
- 100GB+ free disk space (for models and outputs)

### Verification

```bash
# Check Docker
docker --version

# Check Docker Compose
docker compose version

# Check NVIDIA Container Toolkit
docker run --rm --gpus all nvidia/cuda:12.0.0-base-ubuntu22.04 nvidia-smi

# Check GPU access
nvidia-smi
```

---

## Quick Start

### 1. Setup

Run the automated setup script:

```bash
./scripts/setup_docker.sh
```

This script will:
- Check all prerequisites
- Create necessary directories
- Clone ComfyUI repository
- Create environment configuration
- Build Docker images
- Validate configuration

### 2. Configure

Review and update environment settings:

```bash
nano docker/.env
```

Key settings to review:
- `CUDA_VISIBLE_DEVICES`: Which GPU(s) to use
- `GRAFANA_ADMIN_PASSWORD`: Change from default
- `PROJECT_ROOT`: Path to project directory

### 3. Download Models

Download SDXL base model (6.5GB):

```bash
# Using justfile (if available)
just download-model

# Or manually
mkdir -p models/checkpoints
cd models/checkpoints
wget https://huggingface.co/stabilityai/stable-diffusion-xl-base-1.0/resolve/main/sd_xl_base_1.0.safetensors
```

### 4. Start Stack

```bash
cd docker
docker compose up -d
```

### 5. Verify

Check all services are running:

```bash
docker compose ps
```

Expected output:
```
NAME                        STATUS              PORTS
dgx-pixels-backend          running (healthy)   5555-5556, 8000
dgx-pixels-comfyui          running (healthy)   8188
dgx-pixels-dcgm             running (healthy)   9400
dgx-pixels-grafana          running (healthy)   3000
dgx-pixels-mcp              running (healthy)   3001
dgx-pixels-node-exporter    running (healthy)   9100
dgx-pixels-prometheus       running (healthy)   9090
```

---

## Service Architecture

The stack consists of three layers:

### AI Generation Layer

- **ComfyUI**: SDXL inference engine
  - Port: 8188
  - GPU: Required
  - Purpose: AI sprite generation

- **Backend Worker**: ZeroMQ server and job queue
  - Port: 5555 (REQ-REP), 5556 (PUB-SUB)
  - GPU: Optional
  - Purpose: Job orchestration and ComfyUI client

- **MCP Server**: FastMCP server for Bevy integration
  - Port: 3001
  - GPU: Not required
  - Purpose: Model Context Protocol tools

### Observability Layer

- **DCGM Exporter**: GPU metrics collection
  - Port: 9400
  - GPU: Required (monitoring only)

- **Prometheus**: Metrics storage and querying
  - Port: 9090

- **Grafana**: Metrics visualization
  - Port: 3000

- **Node Exporter**: System metrics
  - Port: 9100

### Development Layer (Optional)

- **Dev Container**: Interactive development environment
  - Enabled with: `docker compose --profile dev up -d`

---

## Configuration

### Environment Variables

All configuration is in `docker/.env`:

```bash
# GPU Configuration
CUDA_VISIBLE_DEVICES=0              # Which GPU to use
NVIDIA_VISIBLE_DEVICES=all          # All GPUs visible
NVIDIA_DRIVER_CAPABILITIES=compute,utility

# Service Ports
COMFYUI_PORT=8188
ZMQ_PORT=5555
METRICS_PORT=8000
PROMETHEUS_PORT=9090
GRAFANA_PORT=3000

# Security
GRAFANA_ADMIN_PASSWORD=changeme     # CHANGE THIS!

# Storage
PROJECT_ROOT=/path/to/dgx-pixels
MODELS_DIR=./models
OUTPUTS_DIR=./outputs
```

### MCP Configuration

MCP server uses `config/mcp_config.yaml`:

```yaml
mcp_server:
  name: "dgx-pixels-mcp"
  version: "0.1.0"
  transports:
    - stdio

backend:
  zmq_endpoint: "tcp://backend-worker:5555"
  comfyui_url: "http://comfyui:8188"

generation:
  output_dir: "/app/outputs"
  default_style: "pixel_art"
  default_resolution: "1024x1024"
  default_steps: 30
  default_cfg_scale: 7.5
```

---

## Starting the Stack

### Production Mode (Default)

Start all production services:

```bash
cd docker
docker compose up -d
```

### Development Mode

Include development container:

```bash
docker compose --profile dev up -d
```

### Selective Services

Start only specific services:

```bash
# Only AI generation stack
docker compose up -d comfyui backend-worker mcp-server

# Only observability stack
docker compose up -d dcgm-exporter prometheus grafana
```

### Build and Start

Force rebuild images before starting:

```bash
docker compose up -d --build
```

---

## Service URLs

After starting, access services at:

| Service | URL | Credentials |
|---------|-----|-------------|
| ComfyUI | http://localhost:8188 | None |
| Grafana | http://localhost:3000 | admin/admin |
| Prometheus | http://localhost:9090 | None |
| Backend Metrics | http://localhost:8000/metrics | None |
| DCGM Metrics | http://localhost:9400/metrics | None |
| Node Metrics | http://localhost:9100/metrics | None |

### ZeroMQ Endpoints

Backend worker exposes:
- REQ-REP: `tcp://localhost:5555`
- PUB-SUB: `tcp://localhost:5556`

---

## Health Checks

### Check All Services

```bash
docker compose ps
```

Look for `(healthy)` status on all services.

### Individual Service Logs

```bash
# ComfyUI logs
docker compose logs -f comfyui

# Backend worker logs
docker compose logs -f backend-worker

# MCP server logs
docker compose logs -f mcp-server

# All services
docker compose logs -f
```

### Manual Health Checks

```bash
# ComfyUI API
curl http://localhost:8188/system_stats

# Backend metrics
curl http://localhost:8000/metrics

# Prometheus health
curl http://localhost:9090/-/healthy

# Grafana health
curl http://localhost:3000/api/health

# DCGM metrics
curl http://localhost:9400/metrics
```

### GPU Access

Verify GPU is accessible in containers:

```bash
# Check ComfyUI GPU access
docker compose exec comfyui nvidia-smi

# Check backend GPU access
docker compose exec backend-worker nvidia-smi
```

---

## Monitoring

### Grafana Dashboards

Access Grafana at http://localhost:3000 (admin/admin):

1. **DGX-Pixels Overview**: High-level metrics
2. **GPU Metrics**: DCGM GPU monitoring
3. **System Metrics**: CPU, memory, disk, network

### Prometheus Queries

Access Prometheus at http://localhost:9090:

```promql
# GPU utilization
DCGM_FI_DEV_GPU_UTIL

# GPU memory used
DCGM_FI_DEV_FB_USED

# Backend job queue size
dgx_pixels_queue_size

# Backend active jobs
dgx_pixels_active_jobs
```

### Logs

View logs for all services:

```bash
# Follow all logs
docker compose logs -f

# Last 100 lines
docker compose logs --tail=100

# Specific service
docker compose logs -f backend-worker

# Since timestamp
docker compose logs --since 2024-11-13T10:00:00
```

---

## Data Management

### Persistent Volumes

Data is stored in Docker volumes:

```bash
# List all volumes
docker volume ls | grep dgx-pixels

# Inspect volume
docker volume inspect dgx-pixels-comfyui-models

# Backup volume
docker run --rm -v dgx-pixels-comfyui-models:/data -v $(pwd):/backup \
  ubuntu tar czf /backup/comfyui-models-backup.tar.gz /data

# Restore volume
docker run --rm -v dgx-pixels-comfyui-models:/data -v $(pwd):/backup \
  ubuntu tar xzf /backup/comfyui-models-backup.tar.gz -C /
```

### Volume Locations

| Volume | Purpose | Persistence |
|--------|---------|-------------|
| `comfyui-models` | SDXL models, LoRAs | Persistent |
| `comfyui-outputs` | Generated sprites | Persistent |
| `backend-outputs` | Backend outputs | Persistent |
| `backend-logs` | Backend logs | Persistent |
| `prometheus-data` | Metrics data | Persistent |
| `grafana-data` | Dashboards, users | Persistent |

### Model Management

Models are stored in `models/checkpoints`:

```bash
# List models
ls -lh models/checkpoints/

# Add new model
cp /path/to/model.safetensors models/checkpoints/

# Remove old model
rm models/checkpoints/old_model.safetensors
```

### Output Management

Generated sprites are in `outputs/`:

```bash
# List recent outputs
ls -lt outputs/ | head -20

# Clean old outputs (older than 30 days)
find outputs/ -type f -mtime +30 -delete
```

---

## Troubleshooting

### Services Not Starting

**Problem**: Container exits immediately

```bash
# Check logs for errors
docker compose logs backend-worker

# Check container status
docker compose ps

# Restart specific service
docker compose restart backend-worker
```

**Common causes**:
- GPU not accessible: Check `nvidia-smi` in container
- Port conflict: Check `netstat -tulpn | grep <port>`
- Configuration error: Check `docker compose config`

### GPU Not Visible

**Problem**: `nvidia-smi` fails in container

```bash
# Verify NVIDIA Container Toolkit installed
docker run --rm --gpus all nvidia/cuda:12.0.0-base-ubuntu22.04 nvidia-smi

# Check Docker daemon config
cat /etc/docker/daemon.json

# Restart Docker daemon
sudo systemctl restart docker

# Recreate containers
docker compose down
docker compose up -d
```

**Required daemon.json**:
```json
{
  "runtimes": {
    "nvidia": {
      "path": "nvidia-container-runtime",
      "runtimeArgs": []
    }
  }
}
```

### Service Unhealthy

**Problem**: Service stuck in "starting" or "unhealthy"

```bash
# Check health check logs
docker compose ps
docker inspect dgx-pixels-backend | grep -A 20 Health

# Increase health check timeout
# Edit docker-compose.yml:
healthcheck:
  start_period: 120s  # Increase from 60s
  interval: 60s       # Increase from 30s
```

### Port Conflicts

**Problem**: "Address already in use"

```bash
# Find process using port
sudo netstat -tulpn | grep :8188

# Kill process
sudo kill -9 <PID>

# Or change port in .env
COMFYUI_PORT=8189
```

### Out of Memory

**Problem**: Container killed (OOM)

```bash
# Check container stats
docker stats

# Increase memory limit in docker-compose.yml
mem_limit: 128g  # Increase from 100g

# Or disable limit (use all available)
# mem_limit: 0
```

### Volume Permissions

**Problem**: Permission denied errors

```bash
# Fix ownership (run as root)
sudo chown -R 1000:1000 models/ outputs/ config/

# Or use current user
sudo chown -R $USER:$USER models/ outputs/ config/
```

### Network Issues

**Problem**: Services can't communicate

```bash
# Check network exists
docker network ls | grep dgx-pixels

# Inspect network
docker network inspect dgx-pixels-net

# Recreate network
docker compose down
docker compose up -d
```

### Image Build Failures

**Problem**: Build fails with dependency errors

```bash
# Clear build cache
docker builder prune -a

# Rebuild with no cache
docker compose build --no-cache

# Build specific service
docker compose build --no-cache backend-worker
```

---

## Production Considerations

### Security

1. **Change Default Passwords**:
   ```bash
   # In docker/.env
   GRAFANA_ADMIN_PASSWORD=<strong-password>
   ```

2. **Use Secrets** (for Docker Swarm):
   ```yaml
   secrets:
     grafana_password:
       external: true
   ```

3. **Restrict Network Access**:
   ```yaml
   # Only expose necessary ports
   ports:
     - "127.0.0.1:3000:3000"  # Only localhost
   ```

4. **Non-Root Containers**: All services run as non-root users

5. **Security Options**:
   ```yaml
   security_opt:
     - no-new-privileges:true
   ```

### Reverse Proxy

Use nginx or Traefik for SSL/TLS:

```nginx
# nginx config
server {
    listen 443 ssl http2;
    server_name comfyui.example.com;

    ssl_certificate /etc/ssl/certs/cert.pem;
    ssl_certificate_key /etc/ssl/private/key.pem;

    location / {
        proxy_pass http://localhost:8188;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### Firewall

Allow only necessary ports:

```bash
# UFW example
sudo ufw allow 22/tcp    # SSH
sudo ufw allow 443/tcp   # HTTPS
sudo ufw allow 8188/tcp  # ComfyUI (if public)
sudo ufw enable
```

### Backup Strategy

Automated backups:

```bash
#!/bin/bash
# backup.sh
DATE=$(date +%Y%m%d-%H%M%S)
BACKUP_DIR=/backups/dgx-pixels

# Backup volumes
docker run --rm \
  -v dgx-pixels-comfyui-models:/data \
  -v $BACKUP_DIR:/backup \
  ubuntu tar czf /backup/models-$DATE.tar.gz /data

docker run --rm \
  -v prometheus-data:/data \
  -v $BACKUP_DIR:/backup \
  ubuntu tar czf /backup/prometheus-$DATE.tar.gz /data

# Backup config
cp -r config/ $BACKUP_DIR/config-$DATE/
```

Schedule with cron:
```cron
0 2 * * * /path/to/backup.sh
```

### Update Strategy

Rolling updates:

```bash
# Pull latest images
docker compose pull

# Update one service at a time
docker compose up -d --no-deps backend-worker

# Wait for health check
docker compose ps backend-worker

# Update next service
docker compose up -d --no-deps comfyui
```

### Resource Limits

Set appropriate limits:

```yaml
services:
  comfyui:
    deploy:
      resources:
        limits:
          memory: 64G
          cpus: '16'
        reservations:
          memory: 32G
          cpus: '8'
```

### Logging

Configure log rotation:

```yaml
services:
  comfyui:
    logging:
      driver: "json-file"
      options:
        max-size: "100m"
        max-file: "10"
```

Or use external logging:

```yaml
logging:
  driver: "syslog"
  options:
    syslog-address: "tcp://logserver:514"
```

---

## Advanced Usage

### Custom Workflows

Add ComfyUI workflows:

```bash
# Copy workflow JSON to workflows/
cp my_workflow.json workflows/

# Restart ComfyUI
docker compose restart comfyui
```

### Multiple GPU Setup

Use multiple GPUs:

```bash
# In docker/.env
CUDA_VISIBLE_DEVICES=0,1,2,3

# In docker-compose.yml
deploy:
  resources:
    reservations:
      devices:
        - driver: nvidia
          count: 4
          capabilities: [gpu]
```

### Development Container

Access development environment:

```bash
# Start with dev profile
docker compose --profile dev up -d

# Enter container
docker compose exec dgx-pixels-dev bash

# Run Python scripts
python scripts/test_generation.py

# Run Rust build
cd rust && cargo build
```

### Custom Models

Add custom SDXL checkpoints or LoRAs:

```bash
# Add checkpoint
cp custom_sdxl.safetensors models/checkpoints/

# Add LoRA
cp pixel_art_lora.safetensors models/loras/

# Restart ComfyUI to detect new models
docker compose restart comfyui
```

### Scaling

Run multiple backend workers:

```bash
docker compose up -d --scale backend-worker=3
```

Note: Requires load balancer for ZeroMQ endpoints.

### CI/CD Integration

GitHub Actions example:

```yaml
# .github/workflows/deploy.yml
name: Deploy
on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: self-hosted
    steps:
      - uses: actions/checkout@v3

      - name: Deploy Stack
        run: |
          cd docker
          docker compose pull
          docker compose up -d
          docker compose ps
```

---

## Support

For issues or questions:

- Documentation: `docs/`
- GitHub Issues: https://github.com/your-org/dgx-pixels/issues
- Architecture: `docs/07-rust-python-architecture.md`

---

## Next Steps

After deployment:

1. **Test Generation**: Use MCP server to generate test sprite
2. **Configure Alerts**: Set up Prometheus alerting rules
3. **Train LoRA**: Follow training roadmap in `docs/05-training-roadmap.md`
4. **Integrate Bevy**: Connect Bevy game engine using MCP
5. **Monitor Performance**: Track metrics in Grafana dashboards

---

**Last Updated**: 2024-11-13
**Version**: 1.0.0
