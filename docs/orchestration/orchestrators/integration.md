# Integration Orchestrator

**Domain**: Integration, Deployment & Observability
**Milestone**: M4, M5
**Timeline**: Weeks 7-12
**Workstreams**: WS-13, WS-14, WS-15, WS-16, WS-17, WS-18
**Status**: Blocked by Interface (WS-10), Model (WS-05)

---

## Responsibility

Integrate DGX-Pixels with Bevy game engine via MCP, implement production deployment infrastructure, establish observability with DCGM metrics, and automate CI/CD pipeline. This orchestrator takes the system from development to production-ready state.

---

## Workstreams Managed

### Phase 3: Integration & Production (Weeks 7-12)

**Mixed Execution** (sequential integration, parallel infrastructure):

1. **WS-13**: FastMCP Server (5-6 days) - Depends on WS-10, blocks WS-14/15
2. **WS-14**: Bevy Plugin Integration (6-7 days) - Depends on WS-13
3. **WS-15**: Asset Deployment Pipeline (4-5 days) - Depends on WS-13, WS-14
4. **WS-16**: DCGM Metrics & Observability (5-6 days) - Depends on WS-05, parallel with WS-13/14/15
5. **WS-17**: Docker Compose Deployment (4-5 days) - Depends on WS-10, WS-16, parallel with WS-14/15
6. **WS-18**: CI/CD Pipeline (6-8 days) - Depends on WS-17

**Total Duration**: 30-37 days (6 weeks with overlapping execution)

**Critical Path**: WS-13 → WS-14 → WS-15 (Bevy integration)

---

## Agent Spawn Commands

### Week 7: MCP Server (WS-13)

```bash
# Day 1-6: FastMCP Server (needs WS-10 backend)
npx claude-flow@alpha spawn agent backend-architect \
  --workstream WS-13 \
  --spec docs/orchestration/workstreams/ws13-fastmcp-server-server/README.md \
  --priority P0 \
  --depends WS-10 \
  --context "FastMCP library, stdio/SSE, integrate with backend worker, <200ms" \
  --output docs/orchestration/workstreams/ws13-fastmcp-server-server/COMPLETION_SUMMARY.md
```

### Week 7-8: Bevy Integration (WS-14)

```bash
# Day 7-13: Bevy Plugin Integration
npx claude-flow@alpha spawn agent rust-pro \
  --workstream WS-14 \
  --spec docs/orchestration/workstreams/ws14-bevy-integration/README.md \
  --priority P0 \
  --depends WS-13 \
  --context "bevy_brp_mcp plugin, asset hot-reload, example game" \
  --output docs/orchestration/workstreams/ws14-bevy-integration/COMPLETION_SUMMARY.md
```

### Week 8: Asset Pipeline (WS-15)

```bash
# Day 14-18: Asset Deployment Pipeline
npx claude-flow@alpha spawn agent devops-automator \
  --workstream WS-15 \
  --spec docs/orchestration/workstreams/ws15-asset-pipeline/README.md \
  --priority P1 \
  --depends "WS-13,WS-14" \
  --context "PNG format, assets/ structure, manifest generation, validation" \
  --output docs/orchestration/workstreams/ws15-asset-pipeline/COMPLETION_SUMMARY.md
```

### Week 7-8: Observability (WS-16, Parallel)

```bash
# Day 1-6: DCGM Metrics & Observability (parallel with WS-13/14)
npx claude-flow@alpha spawn agent infrastructure-maintainer \
  --workstream WS-16 \
  --spec docs/orchestration/workstreams/ws16-observability/README.md \
  --priority P1 \
  --depends WS-05 \
  --context "DCGM exporter, Prometheus, Grafana, alerting, 30-day retention" \
  --output docs/orchestration/workstreams/ws16-observability/COMPLETION_SUMMARY.md
```

### Week 8-9: Docker Deployment (WS-17, Parallel)

```bash
# Day 7-11: Docker Compose Deployment (parallel with WS-14/15)
npx claude-flow@alpha spawn agent devops-automator \
  --workstream WS-17 \
  --spec docs/orchestration/workstreams/ws17-docker-deployment/README.md \
  --priority P1 \
  --depends "WS-10,WS-16" \
  --context "NVIDIA runtime, volume mounts, health checks, <60s startup" \
  --output docs/orchestration/workstreams/ws17-docker-deployment/COMPLETION_SUMMARY.md
```

### Week 10-12: CI/CD (WS-18)

```bash
# Day 19-26: CI/CD Pipeline
npx claude-flow@alpha spawn agent devops-automator \
  --workstream WS-18 \
  --spec docs/orchestration/workstreams/ws18-cicd-pipeline/README.md \
  --priority P2 \
  --depends WS-17 \
  --context "GitHub Actions, ARM runners, test automation, Docker builds" \
  --output docs/orchestration/workstreams/ws18-cicd-pipeline/COMPLETION_SUMMARY.md
```

---

## Phase Gate: Integration Complete

### M4 Gate: Bevy Integration Ready (After WS-13, WS-14, WS-15)

**Criteria**:
- ✅ FastMCP server operational and tested
- ✅ MCP tools: `generate_sprite`, `generate_batch`, `deploy_to_bevy` functional
- ✅ Bevy example project connects to MCP server
- ✅ Generated assets load automatically in Bevy
- ✅ Hot-reload updates sprites in running game
- ✅ Asset deployment pipeline automates PNG export
- ✅ End-to-end workflow: prompt → generate → deploy → play

**Gate Check**:
```bash
./scripts/check_integration_m4_gate.sh

# Expected output:
# ✅ WS-13: FastMCP Server - COMPLETE
# ✅ WS-14: Bevy Integration - COMPLETE
# ✅ WS-15: Asset Pipeline - COMPLETE
# ✅ MCP Response Time: 180ms average
# ✅ Bevy Example: Running with AI-generated sprites
# ✅ M4 Gate: PASSED - Bevy integration complete
```

### M5 Gate: Production Ready (After WS-16, WS-17, WS-18)

**Criteria**:
- ✅ DCGM metrics exported to Prometheus
- ✅ Grafana dashboards visualize performance
- ✅ Alerting rules configured and tested
- ✅ Docker Compose starts entire stack
- ✅ All services healthy and persistent
- ✅ CI/CD pipeline runs tests on every PR
- ✅ Docker images built and published automatically
- ✅ Documentation deployed to GitHub Pages

**Gate Check**:
```bash
./scripts/check_integration_m5_gate.sh

# Expected output:
# ✅ WS-16: Observability - COMPLETE
# ✅ WS-17: Docker Deployment - COMPLETE
# ✅ WS-18: CI/CD - COMPLETE
# ✅ Prometheus: Scraping metrics every 15s
# ✅ Grafana: 3 dashboards configured
# ✅ Docker: All services healthy
# ✅ CI/CD: Tests passing, images published
# ✅ M5 Gate: PASSED - Production ready
```

---

## Coordination Points

### With Meta Orchestrator

**Status Reports** (every 12 hours during WS-14):
```json
{
  "orchestrator": "Integration",
  "phase": "M4",
  "workstreams": {
    "WS-13": {"status": "complete", "completion_date": "2025-12-05"},
    "WS-14": {"status": "in_progress", "progress": 0.60, "eta": "2025-12-12"},
    "WS-15": {"status": "pending", "blocked_by": "WS-14"},
    "WS-16": {"status": "complete", "completion_date": "2025-12-06"},
    "WS-17": {"status": "in_progress", "progress": 0.80, "eta": "2025-12-10"},
    "WS-18": {"status": "pending", "blocked_by": "WS-17"}
  },
  "integration_status": {
    "mcp_functional": true,
    "bevy_connected": true,
    "docker_stack_healthy": true,
    "metrics_flowing": true
  },
  "blockers": [],
  "eta": "2025-12-20T17:00:00Z"
}
```

**Escalations**:
- MCP protocol compatibility issues (WS-13)
- Bevy asset loading failures (WS-14)
- Docker GPU passthrough problems (WS-17)
- CI/CD ARM runner unavailability (WS-18)

### With Interface Orchestrator

**Handoff Received** (After WS-10):
- Python backend API endpoint
- Job submission interface specification
- Progress notification patterns (PUB-SUB)
- Backend worker integration guide

**Coordination**:
- WS-13 must coordinate with Interface team if backend API changes
- Ensure MCP server can access backend without network issues

### With Model Orchestrator

**Handoff Received** (After WS-05):
- SDXL optimization results (for WS-16 baseline)
- Expected inference times and VRAM usage
- Workflow templates for benchmarking

**Coordination**:
- WS-16 uses WS-05 performance data for alert thresholds
- Observability dashboard includes model-specific metrics

---

## Dependencies

### Blocking Dependencies

**From Interface Orchestrator**:
- ✅ WS-10: Python Backend Worker - REQUIRED for WS-13
  - Backend must be operational for MCP integration
  - Job submission API must be stable

**From Model Orchestrator**:
- ✅ WS-05: SDXL Optimization - REQUIRED for WS-16
  - Performance baselines for observability
  - Expected VRAM and GPU utilization patterns

**External Dependencies**:
- Bevy 0.13+ with bevy_brp_mcp plugin
- Docker with NVIDIA Container Toolkit
- Prometheus + Grafana (for WS-16)
- DCGM installed on DGX-Spark
- GitHub Actions (for WS-18)

### Software Dependencies

**WS-13 (FastMCP Server)**:
```bash
pip install fastmcp>=0.1.0
pip install pydantic>=2.0
pip install uvicorn  # For SSE transport
```

**WS-14 (Bevy Integration)**:
```toml
[dependencies]
bevy = "0.13"
bevy_brp_mcp = "0.1"  # MCP plugin for Bevy
```

**WS-15 (Asset Pipeline)**:
```bash
pip install pillow  # Image processing
pip install jsonschema  # Manifest validation
```

**WS-16 (Observability)**:
```bash
# Docker images
docker pull prom/prometheus:latest
docker pull grafana/grafana:latest
docker pull nvidia/dcgm-exporter:latest
```

**WS-17 (Docker Deployment)**:
```bash
# Docker Compose v2
sudo apt install docker-compose-plugin

# NVIDIA Container Toolkit
distribution=$(. /etc/os-release;echo $ID$VERSION_ID)
curl -s -L https://nvidia.github.io/nvidia-docker/gpgkey | sudo apt-key add -
curl -s -L https://nvidia.github.io/nvidia-docker/$distribution/nvidia-docker.list | \
  sudo tee /etc/apt/sources.list.d/nvidia-docker.list
sudo apt update && sudo apt install -y nvidia-container-toolkit
```

**WS-18 (CI/CD)**:
- GitHub repository with Actions enabled
- ARM runners (self-hosted or GitHub-provided)
- Docker Hub or GitHub Container Registry credentials

---

## Known Issues & Mitigations

### Issue 1: MCP Protocol Maturity

**Problem**: MCP is relatively new, may have incomplete Python implementations
**Impact**: WS-13 may encounter undocumented edge cases
**Mitigation**:
- Use FastMCP library (official Anthropic implementation)
- Test with MCP Inspector tool during development
- Document any workarounds or protocol quirks
- Contribute fixes upstream if needed

**Priority**: P1 (iterate on implementation)

### Issue 2: Bevy Hot-Reload Race Conditions

**Problem**: Asset hot-reload may have timing issues with MCP deployment
**Impact**: WS-14 assets may not reload consistently
**Mitigation**:
- Implement retry logic for asset loading
- Add explicit sync point after deployment
- Test with various asset sizes and formats
- Document known race conditions and workarounds

**Priority**: P1 (acceptable with documented workarounds)

### Issue 3: DCGM ARM Compatibility

**Problem**: DCGM may have limited ARM support on GB10
**Impact**: WS-16 metrics may be incomplete or unavailable
**Mitigation**:
- Test DCGM early (first day of WS-16)
- Fallback to nvidia-smi for basic metrics
- Custom metrics exporter if DCGM unavailable
- Document any ARM-specific limitations

**Priority**: P1 (fallback acceptable)

### Issue 4: Docker ARM Image Availability

**Problem**: Some Docker images may not have ARM builds
**Impact**: WS-17 stack may not start on DGX-Spark
**Mitigation**:
- Verify all images have ARM variants before WS-17
- Build custom images if needed (add Dockerfiles)
- Use multi-arch images where available
- Test early on ARM hardware

**Priority**: P0 (must resolve in WS-17)

### Issue 5: CI/CD ARM Runner Cost

**Problem**: GitHub-hosted ARM runners may be expensive or unavailable
**Impact**: WS-18 pipeline may not run or cost too much
**Mitigation**:
- Use self-hosted runners on DGX-Spark
- Run critical tests only (not full suite on every PR)
- Use x86 runners for non-hardware tests
- Document runner setup for self-hosting

**Priority**: P2 (self-hosted acceptable)

---

## Success Criteria

### Orchestrator Success

✅ All 6 workstreams complete within 6 weeks (8-week buffer acceptable)
✅ M4 gate (Bevy integration) passed by end of week 9
✅ M5 gate (production) passed by end of week 12
✅ End-to-end workflow functional: prompt → generate → deploy → play
✅ Production infrastructure operational and documented

### Quality Standards

**Code**:
- MCP server follows MCP specification exactly
- Bevy integration has example project with tests
- All Docker images have health checks
- CI/CD pipeline has comprehensive test coverage

**Integration**:
- WS-13: MCP response time <200ms (P0)
- WS-14: Bevy asset load time <500ms (P0)
- WS-15: Asset validation catches format errors (P1)
- WS-16: Metrics scraped every 15s with no gaps (P1)
- WS-17: Docker stack starts in <60s (P1)
- WS-18: CI/CD pipeline completes in <30 min (P2)

**Documentation**:
- MCP server API documentation with examples
- Bevy integration guide with step-by-step setup
- Docker deployment guide for production
- Observability runbook with alerting procedures
- CI/CD contribution guide

---

## Timeline

```
Week 7 (Days 43-49):
  Mon-Sat: WS-13 (FastMCP Server)
         → MCP tools implemented
         → Integrated with backend worker
         → Tested with MCP Inspector
         → HANDOFF to WS-14

  Mon-Sat: WS-16 (Observability) PARALLEL
         → DCGM exporter configured
         → Prometheus scraping
         → Grafana dashboards created

Week 8 (Days 50-56):
  Sun-Sat: WS-14 (Bevy Integration)
         → bevy_brp_mcp plugin added
         → Example game created
         → Hot-reload tested
         → HANDOFF to WS-15

  Mon-Thu: WS-17 (Docker Deployment) PARALLEL
         → Dockerfiles created
         → docker-compose.yml written
         → GPU passthrough tested

Week 9 (Days 57-61):
  Sun-Thu: WS-15 (Asset Pipeline)
         → Deployment script written
         → Validation pipeline tested
         → M4 GATE CHECK (end of week)

  Fri: WS-17 completion, handoff to WS-18

Week 10-12 (Days 62-84):
  Fri-Fri: WS-18 (CI/CD Pipeline)
         → GitHub Actions workflows
         → Test automation
         → Docker image publishing
         → Documentation deployment
         → M5 GATE CHECK (end of week 12)
```

**Buffer**: 2 weeks for integration issues or ARM compatibility

---

## Parallel Execution Strategy

### Week 7-8: Parallel Infrastructure + Integration

**Track A (Integration - Sequential)**:
WS-13 (MCP) → WS-14 (Bevy) → WS-15 (Assets)

**Track B (Infrastructure - Parallel)**:
WS-16 (Observability) + WS-17 (Docker)

**Resource Allocation**:
- **Agent 1 (backend-architect)**: WS-13 MCP Server
- **Agent 2 (rust-pro)**: WS-14 Bevy Integration (after WS-13)
- **Agent 3 (devops-automator)**: WS-15 Asset Pipeline (after WS-14)
- **Agent 4 (infrastructure-maintainer)**: WS-16 Observability (parallel)
- **Agent 5 (devops-automator)**: WS-17 Docker (parallel after WS-16)

**Coordination**:
- Daily sync between Track A and Track B
- WS-17 Docker includes WS-16 Prometheus/Grafana services
- WS-14 Bevy example uses WS-13 MCP server

**Expected Timeline Savings**: 1-2 weeks (vs fully sequential)

### Week 10-12: CI/CD (Final Polish)

WS-18 runs after all other workstreams complete.
Uses outputs from all previous workstreams to build comprehensive pipeline.

---

## Completion Checklist

Before marking Integration Orchestrator complete:

- [ ] WS-13 completion summary created
- [ ] WS-14 completion summary created
- [ ] WS-15 completion summary created
- [ ] WS-16 completion summary created
- [ ] WS-17 completion summary created
- [ ] WS-18 completion summary created
- [ ] M4 gate check passed and documented
- [ ] M5 gate check passed and documented
- [ ] All files committed to git
- [ ] End-to-end demo video recorded (prompt to Bevy game)
- [ ] MCP server tested with real Bevy project
- [ ] Docker Compose verified on DGX-Spark
- [ ] Grafana dashboards operational with live data
- [ ] CI/CD pipeline runs successfully on merge
- [ ] Production deployment guide published
- [ ] All issues closed or documented as known limitations
- [ ] Final status report posted to Meta Orchestrator
- [ ] Project marked as production-ready

---

## Start Command

```bash
# Wait for Interface Orchestrator M2 gate (WS-10) to pass
./scripts/check_interface_m2_gate.sh || exit 1

# Wait for Model Orchestrator M1 gate (WS-05) to pass
./scripts/check_model_m1_gate.sh || exit 1

# Initialize Integration Orchestrator
./scripts/spawn_integration_orchestrator.sh

# Or manually:
cd /home/beengud/raibid-labs/dgx-pixels
cat docs/orchestration/orchestrators/integration.md
./scripts/spawn_agent.sh backend-architect WS-13
./scripts/spawn_agent.sh infrastructure-maintainer WS-16  # Parallel
```

**Status**: Ready to spawn after Interface Orchestrator completes WS-10 and Model Orchestrator completes WS-05.

**Timeline Note**: This is the final orchestrator. Upon completion of M5 gate, DGX-Pixels is production-ready.
