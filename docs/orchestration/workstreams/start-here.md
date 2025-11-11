# DGX-Pixels Workstreams - START HERE

**Welcome!** This is the entry point for understanding and executing DGX-Pixels workstreams.

---

## Quick Navigation

### For Project Overview
👉 **Start with**: `../PROJECT_ORCHESTRATION_SUMMARY.md`
- Complete project breakdown
- Timeline and phases
- Orchestration hierarchy
- Next steps

### For Orchestrators
👉 **Meta Orchestrator**: `../meta-orchestrator.md`
- Top-level coordination
- Phase gates
- Cross-orchestrator dependencies

👉 **Domain Orchestrators**: `../orchestrators/`
- Foundation Orchestrator (M0, Weeks 1-2)
- Model Orchestrator (M1, M3, Weeks 3-6)
- Interface Orchestrator (M2, Weeks 3-6)
- Integration Orchestrator (M4, M5, Weeks 7-12)

### For Workstreams
👉 **Master Plan**: `../workstream-plan.md`
- All 18 workstreams listed
- Dependencies and timelines
- Success metrics

👉 **Individual Workstreams**: `WS-XX-name/README.md`
- Detailed specifications for each workstream
- Complete with acceptance criteria, tests, verification steps

---

## Workstream List

### Phase 1: Foundation (Weeks 1-2)

| ID | Name | Spec | Status |
|----|------|------|--------|
| WS-01 | Hardware Baselines | [README](WS-01-hardware-baselines/README.md) | ✅ Spec Ready |
| WS-02 | Reproducibility Framework | _Use template_ | ⏳ Needs Spec |
| WS-03 | Benchmark Suite | _Use template_ | ⏳ Needs Spec |

### Phase 2A: Model (Weeks 3-6)

| ID | Name | Spec | Status |
|----|------|------|--------|
| WS-04 | ComfyUI Setup | _Use template_ | ⏳ Needs Spec |
| WS-05 | SDXL Inference Optimization | _Use template_ | ⏳ Needs Spec |
| WS-06 | LoRA Training Pipeline | _Use template_ | ⏳ Needs Spec |
| WS-07 | Dataset Tools & Validation | _Use template_ | ⏳ Needs Spec |

### Phase 2B: Interface (Weeks 3-6)

| ID | Name | Spec | Status |
|----|------|------|--------|
| WS-08 | Rust TUI Core | [README](WS-08-rust-tui-core/README.md) | ✅ Spec Ready |
| WS-09 | ZeroMQ IPC Layer | _Use template_ | ⏳ Needs Spec |
| WS-10 | Python Backend Worker | _Use template_ | ⏳ Needs Spec |
| WS-11 | Sixel Image Preview | _Use template_ | ⏳ Needs Spec |
| WS-12 | Side-by-Side Model Comparison | _Use template_ | ⏳ Needs Spec |

### Phase 3: Integration (Weeks 7-12)

| ID | Name | Spec | Status |
|----|------|------|--------|
| WS-13 | FastMCP Server | [README](WS-13-fastmcp-server/README.md) | ✅ Spec Ready |
| WS-14 | Bevy Plugin Integration | _Use template_ | ⏳ Needs Spec |
| WS-15 | Asset Deployment Pipeline | _Use template_ | ⏳ Needs Spec |
| WS-16 | DCGM Metrics & Observability | _Use template_ | ⏳ Needs Spec |
| WS-17 | Docker Compose Deployment | _Use template_ | ⏳ Needs Spec |
| WS-18 | CI/CD Pipeline | _Use template_ | ⏳ Needs Spec |

---

## How to Use This Directory

### For Orchestrators

1. **Spawn an agent for a workstream**:
   ```bash
   npx claude-flow@alpha spawn agent [agent-type] \
     --workstream WS-XX \
     --spec docs/orchestration/workstreams/wsXX-name/README.md
   ```

2. **Monitor progress**:
   - Agent reads `WS-XX-name/README.md` for complete specification
   - Agent implements according to acceptance criteria
   - Agent creates `WS-XX-name/COMPLETION_SUMMARY.md` when done

3. **Verify completion**:
   ```bash
   ./scripts/verify_ws_xx.sh
   ```

### For Developers (Manual Execution)

1. **Pick a workstream** (check dependencies first!)
2. **Read the specification**: `WS-XX-name/README.md`
3. **Follow the implementation plan** (3 phases)
4. **Run tests as you go** (TDD approach)
5. **Create completion summary** when done

### For Creating New Workstream Specs

1. **Copy the template**:
   ```bash
   cp template.md wsXX-new-workstream/README.md
   ```

2. **Fill in all sections**:
   - Objective
   - Deliverables (specific file paths)
   - Acceptance criteria (testable)
   - Technical requirements
   - Implementation plan (3 phases)
   - Tests (TDD)
   - Known issues

3. **Reference examples**:
   - WS-01 (bash/DevOps heavy)
   - WS-08 (Rust heavy)
   - WS-13 (Python API heavy)

---

## Template

📄 **template.md** - Use this to create new workstream specifications

The template includes:
- Complete structure for all sections
- Examples and placeholders
- Verification commands
- Completion checklist

---

## Critical Path

The fastest route through the project (must complete in order):

```
WS-01 → WS-04 → WS-05 → WS-10 → WS-13 → WS-14
(4d)    (5d)    (7d)    (6d)    (6d)    (7d)
= 35 days minimum
```

**Note**: Other workstreams can happen in parallel with these!

---

## Dependencies Visualization

```
WS-01 (Hardware) [MUST START FIRST]
  ├─> WS-02 (Reproducibility)
  ├─> WS-03 (Benchmarks)
  ├─> WS-04 (ComfyUI)
  │     ├─> WS-05 (SDXL)
  │     │     ├─> WS-06 (LoRA Training)
  │     │     ├─> WS-07 (Dataset Tools)
  │     │     └─> WS-16 (Metrics)
  │     └─> WS-10 (Backend)
  │           ├─> WS-11 (Sixel)
  │           ├─> WS-12 (Comparison)
  │           ├─> WS-13 (MCP Server)
  │           │     ├─> WS-14 (Bevy)
  │           │     └─> WS-15 (Asset Pipeline)
  │           └─> WS-17 (Docker)
  │                 └─> WS-18 (CI/CD)
  └─> WS-08 (Rust TUI)
        └─> WS-09 (ZeroMQ)
              └─> WS-10 (Backend)
```

---

## Status Tracking

Track workstream completion here (or use GitHub Issues):

- [x] **WS-01**: Hardware Baselines - _Status_
- [ ] **WS-02**: Reproducibility Framework - _Not Started_
- [ ] **WS-03**: Benchmark Suite - _Not Started_
- [ ] **WS-04**: ComfyUI Setup - _Not Started_
- [ ] **WS-05**: SDXL Inference Optimization - _Not Started_
- [ ] **WS-06**: LoRA Training Pipeline - _Not Started_
- [ ] **WS-07**: Dataset Tools & Validation - _Not Started_
- [ ] **WS-08**: Rust TUI Core - _Not Started_
- [ ] **WS-09**: ZeroMQ IPC Layer - _Not Started_
- [ ] **WS-10**: Python Backend Worker - _Not Started_
- [ ] **WS-11**: Sixel Image Preview - _Not Started_
- [ ] **WS-12**: Side-by-Side Model Comparison - _Not Started_
- [ ] **WS-13**: FastMCP Server - _Not Started_
- [ ] **WS-14**: Bevy Plugin Integration - _Not Started_
- [ ] **WS-15**: Asset Deployment Pipeline - _Not Started_
- [ ] **WS-16**: DCGM Metrics & Observability - _Not Started_
- [ ] **WS-17**: Docker Compose Deployment - _Not Started_
- [ ] **WS-18**: CI/CD Pipeline - _Not Started_

---

## Help & Support

- **Questions about orchestration?** See `../meta-orchestrator.md`
- **Questions about a specific workstream?** Read the workstream's `README.md`
- **Questions about the overall project?** See `../PROJECT_ORCHESTRATION_SUMMARY.md`
- **Questions about hardware?** See `../hardware.md`
- **Questions about metrics?** See `../metrics.md`

---

**Ready to start?** Begin with reviewing the `PROJECT_ORCHESTRATION_SUMMARY.md`!
