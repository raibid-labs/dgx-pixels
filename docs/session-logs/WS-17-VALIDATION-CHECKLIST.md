# WS-17: Docker Compose Deployment - Validation Checklist

Quick checklist to validate WS-17 implementation before proceeding to WS-18.

## Prerequisites Validation

- [ ] Docker v20.10+ installed
  ```bash
  docker --version
  ```

- [ ] Docker Compose v2+ installed
  ```bash
  docker compose version
  ```

- [ ] NVIDIA Container Toolkit installed
  ```bash
  docker run --rm --gpus all nvidia/cuda:12.0.0-base-ubuntu22.04 nvidia-smi
  ```

- [ ] GPU accessible on host
  ```bash
  nvidia-smi
  ```

## File Structure Validation

- [ ] All Dockerfiles created
  ```bash
  ls -l docker/Dockerfile.*
  # Expected: Dockerfile.backend, Dockerfile.mcp, Dockerfile.comfyui
  ```

- [ ] docker-compose.yml exists and updated
  ```bash
  ls -l docker/docker-compose.yml
  ```

- [ ] Environment template exists
  ```bash
  ls -l docker/.env.production
  ```

- [ ] Setup script exists and executable
  ```bash
  ls -l scripts/setup_docker.sh
  test -x scripts/setup_docker.sh && echo "Executable" || echo "Not executable"
  ```

- [ ] Health check script exists and executable
  ```bash
  ls -l scripts/docker_health_check.sh
  test -x scripts/docker_health_check.sh && echo "Executable" || echo "Not executable"
  ```

- [ ] Cleanup script exists and executable
  ```bash
  ls -l scripts/docker_cleanup.sh
  test -x scripts/docker_cleanup.sh && echo "Executable" || echo "Not executable"
  ```

- [ ] Documentation exists
  ```bash
  ls -l docs/docker-deployment.md
  ls -l DOCKER-QUICKREF.md
  ```

- [ ] Justfile updated with Docker commands
  ```bash
  grep -q "docker-setup" justfile && echo "Updated" || echo "Not updated"
  ```

## Configuration Validation

- [ ] Docker Compose configuration is valid
  ```bash
  cd docker && docker compose config > /dev/null && echo "Valid" || echo "Invalid"
  ```

- [ ] All required services defined
  ```bash
  cd docker && docker compose config --services
  # Expected: comfyui, backend-worker, mcp-server, dcgm-exporter, prometheus, grafana, node-exporter
  ```

- [ ] Environment file can be created
  ```bash
  test -f docker/.env.production && echo "Template exists" || echo "Missing"
  ```

## Docker Image Build Validation

- [ ] Backend image builds successfully
  ```bash
  cd docker && docker compose build backend-worker
  ```

- [ ] MCP image builds successfully
  ```bash
  cd docker && docker compose build mcp-server
  ```

- [ ] ComfyUI image builds successfully
  ```bash
  cd docker && docker compose build comfyui
  ```

- [ ] Dev image builds successfully (optional)
  ```bash
  cd docker && docker compose build dgx-pixels-dev
  ```

- [ ] All images built
  ```bash
  docker images | grep dgx-pixels
  # Expected: dgx-pixels-comfyui, dgx-pixels-backend, dgx-pixels-mcp
  ```

## Setup Script Validation

- [ ] Setup script runs without errors
  ```bash
  ./scripts/setup_docker.sh
  ```

- [ ] Directories created
  ```bash
  test -d models && test -d outputs && test -d workflows && test -d config && echo "All created"
  ```

- [ ] Environment file created
  ```bash
  test -f docker/.env && echo "Created" || echo "Not created"
  ```

- [ ] ComfyUI cloned (optional)
  ```bash
  test -d ComfyUI && echo "Exists" || echo "Not cloned"
  ```

## Service Startup Validation

- [ ] Stack starts successfully
  ```bash
  cd docker && docker compose up -d
  ```

- [ ] All services running
  ```bash
  cd docker && docker compose ps
  # All services should be in "running" state
  ```

- [ ] All services healthy (wait 60s)
  ```bash
  sleep 60
  cd docker && docker compose ps
  # All services should show "(healthy)" status
  ```

- [ ] No errors in logs
  ```bash
  cd docker && docker compose logs --tail=50 | grep -i error
  # Should be minimal errors
  ```

## Health Check Validation

- [ ] Health check script runs
  ```bash
  ./scripts/docker_health_check.sh
  ```

- [ ] ComfyUI endpoint accessible
  ```bash
  curl -f http://localhost:8188/system_stats
  ```

- [ ] Backend metrics accessible
  ```bash
  curl -f http://localhost:8000/metrics
  ```

- [ ] Prometheus accessible
  ```bash
  curl -f http://localhost:9090/-/healthy
  ```

- [ ] Grafana accessible
  ```bash
  curl -f http://localhost:3000/api/health
  ```

- [ ] DCGM metrics accessible
  ```bash
  curl -f http://localhost:9400/metrics
  ```

- [ ] Node Exporter metrics accessible
  ```bash
  curl -f http://localhost:9100/metrics
  ```

## GPU Access Validation

- [ ] ComfyUI can access GPU
  ```bash
  cd docker && docker compose exec comfyui nvidia-smi
  ```

- [ ] Backend can access GPU
  ```bash
  cd docker && docker compose exec backend-worker nvidia-smi
  ```

- [ ] DCGM can access GPU
  ```bash
  cd docker && docker compose exec dcgm-exporter nvidia-smi
  ```

## Network Validation

- [ ] Network created
  ```bash
  docker network ls | grep dgx-pixels
  ```

- [ ] Services can communicate
  ```bash
  cd docker && docker compose exec backend-worker ping -c 3 comfyui
  ```

- [ ] Backend can reach ComfyUI
  ```bash
  cd docker && docker compose exec backend-worker curl -f http://comfyui:8188/system_stats
  ```

## Volume Validation

- [ ] All volumes created
  ```bash
  docker volume ls | grep dgx-pixels
  # Expected: comfyui-models, comfyui-outputs, backend-outputs, backend-logs,
  # mcp-logs, prometheus-data, grafana-data, models, outputs, config
  ```

- [ ] Volumes are persistent
  ```bash
  # Create test file
  cd docker && docker compose exec backend-worker touch /app/outputs/test.txt
  # Restart service
  docker compose restart backend-worker
  # Check file still exists
  docker compose exec backend-worker ls /app/outputs/test.txt
  ```

- [ ] Bind mounts working
  ```bash
  # Check models directory is mounted
  cd docker && docker compose exec backend-worker ls /app/models
  ```

## Justfile Commands Validation

- [ ] docker-setup command works
  ```bash
  just docker-setup --help 2>/dev/null || echo "Command exists"
  ```

- [ ] docker-up command works
  ```bash
  just docker-up
  ```

- [ ] docker-ps command works
  ```bash
  just docker-ps
  ```

- [ ] docker-logs command works
  ```bash
  just docker-logs | head -20
  ```

- [ ] docker-health command works
  ```bash
  just docker-health
  ```

- [ ] docker-down command works
  ```bash
  just docker-down
  ```

## Documentation Validation

- [ ] Deployment guide is complete
  ```bash
  wc -l docs/docker-deployment.md
  # Should be 400+ lines
  ```

- [ ] Quick reference is complete
  ```bash
  wc -l DOCKER-QUICKREF.md
  # Should be 200+ lines
  ```

- [ ] Completion summary exists
  ```bash
  test -f WS-17-COMPLETION.md && echo "Exists"
  ```

## Cleanup Validation

- [ ] Cleanup script has interactive menu
  ```bash
  ./scripts/docker_cleanup.sh
  # Should show menu with options
  ```

- [ ] Can stop containers
  ```bash
  cd docker && docker compose down
  ```

- [ ] Can remove volumes (CAUTION)
  ```bash
  # Only test if you want to delete data
  # cd docker && docker compose down -v
  ```

## Integration Validation

- [ ] Observability stack integrated (WS-16)
  ```bash
  # Check Prometheus targets
  curl -s http://localhost:9090/api/v1/targets | grep -q dgx-pixels && echo "Integrated"
  ```

- [ ] Grafana dashboards available
  ```bash
  # Login to Grafana and check dashboards
  open http://localhost:3000
  ```

- [ ] DCGM metrics being collected
  ```bash
  curl -s http://localhost:9090/api/v1/query?query=DCGM_FI_DEV_GPU_UTIL | grep -q result && echo "Collecting"
  ```

## Performance Validation

- [ ] Stack starts in <60 seconds
  ```bash
  time (cd docker && docker compose up -d && sleep 45 && docker compose ps)
  ```

- [ ] Memory usage acceptable (<10GB idle)
  ```bash
  docker stats --no-stream --format "table {{.Name}}\t{{.MemUsage}}"
  ```

- [ ] No excessive CPU usage (idle <5%)
  ```bash
  docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}"
  ```

## Security Validation

- [ ] Containers run as non-root
  ```bash
  cd docker && docker compose exec backend-worker id
  # Should show uid=1000 (ubuntu) or similar
  ```

- [ ] Security options applied
  ```bash
  docker inspect dgx-pixels-backend | grep -i "no-new-privileges"
  ```

- [ ] No hardcoded passwords in compose file
  ```bash
  grep -i "password.*:" docker/docker-compose.yml | grep -v "\${" && echo "Found hardcoded password!" || echo "OK"
  ```

## Final Validation

- [ ] Complete stack operational
  ```bash
  just docker-up && sleep 60 && just docker-health
  ```

- [ ] All acceptance criteria met
  - One command start: `docker compose up -d`
  - All services running and healthy
  - Persistent data configured
  - GPU passthrough working
  - Health checks operational
  - Stack starts quickly (<60s)
  - Documentation complete

- [ ] Ready for WS-18
  - Docker Compose stack fully functional
  - Monitoring integrated
  - Documentation complete
  - Scripts operational
  - CI/CD pipeline can build upon this

---

## Validation Summary

**Date**: __________
**Validated by**: __________
**Result**: [ ] PASS / [ ] FAIL

**Notes**:
_____________________________
_____________________________
_____________________________

**Issues Found**:
_____________________________
_____________________________
_____________________________

**Next Steps**:
- [ ] Proceed to WS-18: CI/CD Pipeline
- [ ] Address issues found
- [ ] Document edge cases

---

## Quick Validation (Minimal)

For a quick validation, run these essential checks:

```bash
# 1. Prerequisites
docker --version && docker compose version && nvidia-smi

# 2. Build images
cd docker && docker compose build

# 3. Start stack
docker compose up -d

# 4. Wait for health
sleep 60

# 5. Check status
docker compose ps

# 6. Run health checks
cd .. && ./scripts/docker_health_check.sh

# 7. Verify endpoints
curl http://localhost:8188/system_stats
curl http://localhost:8000/metrics
curl http://localhost:3000/api/health

# 8. Check GPU
cd docker && docker compose exec comfyui nvidia-smi

# 9. View logs
docker compose logs --tail=20

# 10. Cleanup
docker compose down
```

All checks passing = WS-17 validation successful!

---

**Last Updated**: 2024-11-13
**Version**: 1.0
