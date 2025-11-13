# Docker Quick Reference

Fast command reference for DGX-Pixels Docker deployment.

## Setup

```bash
# Initial setup (run once)
./scripts/setup_docker.sh

# Or manually
cd docker
cp .env.production .env
nano .env  # Configure
docker compose build
```

## Basic Operations

```bash
# Start all services
cd docker && docker compose up -d

# Stop all services
docker compose down

# Restart all services
docker compose restart

# View status
docker compose ps

# View logs
docker compose logs -f

# Stop and remove volumes (CAUTION: deletes data)
docker compose down -v
```

## Service Control

```bash
# Start specific service
docker compose up -d comfyui

# Stop specific service
docker compose stop backend-worker

# Restart specific service
docker compose restart mcp-server

# View service logs
docker compose logs -f comfyui

# Execute command in service
docker compose exec backend-worker bash
```

## Health Checks

```bash
# Quick health check
./scripts/docker_health_check.sh

# Manual checks
curl http://localhost:8188/system_stats  # ComfyUI
curl http://localhost:8000/metrics       # Backend
curl http://localhost:9090/-/healthy     # Prometheus
curl http://localhost:3000/api/health    # Grafana

# GPU access
docker compose exec comfyui nvidia-smi
```

## Logs

```bash
# All logs (follow)
docker compose logs -f

# Specific service
docker compose logs -f backend-worker

# Last 100 lines
docker compose logs --tail=100

# Since timestamp
docker compose logs --since 2024-11-13T10:00:00

# Show errors only
docker compose logs | grep -i error
```

## Development

```bash
# Start with dev container
docker compose --profile dev up -d

# Enter dev container
docker compose exec dgx-pixels-dev bash

# Rebuild service
docker compose build backend-worker

# Rebuild without cache
docker compose build --no-cache comfyui
```

## Data Management

```bash
# List volumes
docker volume ls | grep dgx-pixels

# Inspect volume
docker volume inspect dgx-pixels-comfyui-models

# Backup volume
docker run --rm \
  -v dgx-pixels-comfyui-models:/data \
  -v $(pwd):/backup \
  ubuntu tar czf /backup/models-backup.tar.gz /data

# Restore volume
docker run --rm \
  -v dgx-pixels-comfyui-models:/data \
  -v $(pwd):/backup \
  ubuntu tar xzf /backup/models-backup.tar.gz -C /
```

## Monitoring

```bash
# Resource usage
docker stats

# Continuous monitoring
docker stats --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}"

# Disk usage
docker system df

# Volume sizes
docker system df -v | grep dgx-pixels
```

## Cleanup

```bash
# Interactive cleanup menu
./scripts/docker_cleanup.sh

# Stop containers
docker compose down

# Remove images
./scripts/docker_cleanup.sh --images

# Remove volumes (CAUTION)
./scripts/docker_cleanup.sh --volumes

# Full cleanup
./scripts/docker_cleanup.sh --all
```

## Troubleshooting

```bash
# Service won't start
docker compose logs <service-name>
docker compose ps
docker inspect <container-name>

# GPU not working
docker compose exec comfyui nvidia-smi
docker run --rm --gpus all nvidia/cuda:12.0.0-base-ubuntu22.04 nvidia-smi

# Port conflict
sudo netstat -tulpn | grep :<port>
# Edit docker/.env to change port

# Permission issues
sudo chown -R $USER:$USER models/ outputs/ config/

# Network issues
docker network inspect dgx-pixels-net
docker compose down && docker compose up -d

# Out of disk space
docker system prune -a
docker volume prune
```

## Service URLs

| Service | URL | Credentials |
|---------|-----|-------------|
| ComfyUI | http://localhost:8188 | None |
| Grafana | http://localhost:3000 | admin/admin |
| Prometheus | http://localhost:9090 | None |
| Backend | tcp://localhost:5555 | None |
| Metrics | http://localhost:8000/metrics | None |

## Common Tasks

### Add New Model

```bash
# Download model
cd models/checkpoints
wget <model-url>

# Restart ComfyUI
cd ../../docker
docker compose restart comfyui
```

### Update Services

```bash
# Pull latest images
docker compose pull

# Update one service
docker compose up -d --no-deps backend-worker

# Update all services
docker compose up -d
```

### View Metrics

```bash
# Open Grafana
open http://localhost:3000

# View Prometheus
open http://localhost:9090

# Query metrics
curl http://localhost:8000/metrics | grep dgx_pixels
```

### Backup Everything

```bash
# Backup script
./scripts/docker_backup.sh

# Or manual
docker compose down
tar czf dgx-pixels-backup-$(date +%Y%m%d).tar.gz \
  docker/ config/ models/ workflows/
```

## Configuration Files

```bash
# Main config
docker/.env

# Compose file
docker/docker-compose.yml

# MCP config
config/mcp_config.yaml

# Prometheus config
deploy/prometheus/prometheus.yml

# Grafana datasources
deploy/grafana/datasources/prometheus.yml
```

## Environment Variables

```bash
# GPU configuration
CUDA_VISIBLE_DEVICES=0
NVIDIA_VISIBLE_DEVICES=all

# Ports
COMFYUI_PORT=8188
ZMQ_PORT=5555
GRAFANA_PORT=3000

# Paths
PROJECT_ROOT=/path/to/dgx-pixels
MODELS_DIR=./models
OUTPUTS_DIR=./outputs
```

## Tips

- Use `docker compose` (v2), not `docker-compose` (v1)
- Check logs first when debugging: `docker compose logs -f`
- GPU must be accessible: `nvidia-smi` should work in containers
- Volumes persist data across restarts
- Use `--no-deps` to update single service without dependencies
- Health checks take 30-60s to become healthy
- Dev container requires `--profile dev` flag

## Next Steps

1. Verify setup: `./scripts/docker_health_check.sh`
2. Check Grafana dashboards: http://localhost:3000
3. Test generation: Use MCP server tools
4. Review logs: `docker compose logs -f`
5. Monitor metrics: http://localhost:9090

---

For detailed documentation, see `docs/docker-deployment.md`.
