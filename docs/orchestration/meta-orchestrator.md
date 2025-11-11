# DGX-Pixels Meta Orchestrator

**Role**: Orchestrator of Orchestrators - Coordinates multiple domain orchestrators for large-scale parallel development

**Status**: Active
**Created**: 2025-11-10
**Pattern**: Based on raibid-labs multi-agent orchestration patterns

---

## Overview

DGX-Pixels is a large project (M0-M5 milestones, 12+ week timeline) that requires coordinated parallel development across multiple technical domains. The Meta Orchestrator manages four domain-specific orchestrators, each responsible for 3-5 workstreams.

### Why Multiple Orchestrators?

**Complexity Management**:
- 18 workstreams total across 5 milestones
- 4 distinct technical domains (Infrastructure, Models, UI, Integration)
- Different skill requirements (Python, Rust, DevOps, ML)
- Parallel execution maximizes velocity

**Domain Orchestrators**:
1. **Foundation Orchestrator** - Hardware, baselines, reproducibility (M0)
2. **Model Orchestrator** - AI inference and training pipeline (M1, M3)
3. **Interface Orchestrator** - Rust TUI and Python backend (M2)
4. **Integration Orchestrator** - Bevy, MCP, deployment (M4, M5)

---

## Meta Orchestrator Responsibilities

### 1. Domain Orchestrator Coordination

**Spawn Domain Orchestrators**:
```bash
# Phase 1: Foundation (Week 1-2)
claude-flow spawn orchestrator "Foundation Orchestrator" \
  --spec docs/orchestration/orchestrators/foundation.md \
  --milestone M0

# Phase 2A: Models (Week 3-5)
claude-flow spawn orchestrator "Model Orchestrator" \
  --spec docs/orchestration/orchestrators/model.md \
  --milestone M1,M3

# Phase 2B: Interface (Week 3-6)
claude-flow spawn orchestrator "Interface Orchestrator" \
  --spec docs/orchestration/orchestrators/interface.md \
  --milestone M2

# Phase 3: Integration & Production (Week 7-12)
claude-flow spawn orchestrator "Integration Orchestrator" \
  --spec docs/orchestration/orchestrators/integration.md \
  --milestone M4,M5
```

### 2. Inter-Orchestrator Dependency Management

**Key Dependencies**:
- Model Orchestrator BLOCKS Interface Orchestrator (need ComfyUI working before TUI integration)
- Foundation Orchestrator BLOCKS ALL (baselines required first)
- Interface Orchestrator BLOCKS Integration Orchestrator (need TUI before Bevy integration)

**Dependency Resolution**:
```
M0 (Foundation) → M1 (Model Inference) → M2 (Interface) → M3 (Training) → M4 (Integration) → M5 (Production)
                                    ↓
                                 M3 can start after M1 completes
                                 M5 can start after M4 completes
```

### 3. Cross-Domain Blocker Resolution

**Monitor for Cross-Cutting Issues**:
- Hardware compatibility issues affecting multiple domains
- ARM architecture dependencies blocking multiple workstreams
- Unified memory optimizations needed across Model + Interface domains
- MCP protocol changes affecting Integration domain

**Escalation Triggers**:
- Orchestrator reports blocker lasting >24 hours
- Cross-domain dependency discovered during implementation
- Architecture decision required (create ADR)
- Performance target not met (requires re-scoping)

### 4. Phase Transitions

**Phase Gates**:

**Gate 1: Foundation → Model/Interface** (End of Week 2)
- ✅ Hardware verification complete
- ✅ Baseline measurements recorded
- ✅ Reproducibility framework working
- ✅ Benchmark suite running

**Gate 2: Model/Interface → Integration** (End of Week 6)
- ✅ ComfyUI generating images (M1)
- ✅ Rust TUI functional with preview (M2)
- ✅ Python backend operational (M2)
- ✅ LoRA training pipeline working (M3)

**Gate 3: Integration → Production** (End of Week 11)
- ✅ Bevy MCP integration complete (M4)
- ✅ Asset deployment pipeline working (M4)
- ✅ Example game using generated sprites (M4)

### 5. Progress Reporting

**Weekly Status Report** (to be posted as GitHub comment):
```markdown
## Week N Status Report

### Foundation Orchestrator (M0)
- WS-01: ✅ Complete
- WS-02: 🟡 In Progress (75%)
- WS-03: ⚪ Blocked (waiting for hardware access)

### Model Orchestrator (M1, M3)
- WS-04: ✅ Complete
- WS-05: 🟢 In Progress (40%)
- WS-06: ⚪ Not Started

### Interface Orchestrator (M2)
- WS-08: 🟢 In Progress (60%)
- WS-09: 🟢 In Progress (30%)
- WS-10: ⚪ Blocked (waiting for WS-05)

### Integration Orchestrator (M4, M5)
- Not yet started (blocked by Gate 2)

### Blockers
1. Hardware access for WS-03 (escalated to infra team)
2. WS-10 blocked by WS-05 completion

### Decisions Needed
- None this week

### Next Week Plan
- Complete WS-02, WS-05
- Unblock WS-03 (hardware access secured)
- Start WS-06 after WS-05 completes
```

---

## Orchestrator Spawning Protocol

### Sequential Spawning (Respects Dependencies)

**Week 1-2: Foundation Only**
```bash
# Spawn Foundation Orchestrator
claude-flow spawn orchestrator foundation \
  --workstreams WS-01,WS-02,WS-03 \
  --phase sequential
```

**Week 3-6: Models + Interface (Parallel)**
```bash
# After Foundation Gate passes
claude-flow spawn orchestrator models \
  --workstreams WS-04,WS-05,WS-06,WS-07 \
  --phase parallel \
  --depends-on foundation

claude-flow spawn orchestrator interface \
  --workstreams WS-08,WS-09,WS-10,WS-11,WS-12 \
  --phase parallel \
  --depends-on WS-04  # ComfyUI must be working
```

**Week 7-12: Integration + Production (Sequential then Parallel)**
```bash
# After Model/Interface Gate passes
claude-flow spawn orchestrator integration \
  --workstreams WS-13,WS-14,WS-15,WS-16,WS-17,WS-18 \
  --phase sequential-then-parallel \
  --depends-on interface,models
```

---

## Communication Protocols

### 1. Orchestrator → Meta Orchestrator

**Status Updates** (every 4 hours):
```json
{
  "orchestrator": "Model Orchestrator",
  "status": "active",
  "workstreams": {
    "WS-04": {"status": "complete", "completion_time": "2025-11-12T14:30:00Z"},
    "WS-05": {"status": "in_progress", "progress": 0.65, "eta": "2025-11-13T10:00:00Z"},
    "WS-06": {"status": "blocked", "blocker": "WS-05 incomplete"}
  },
  "blockers": [],
  "decisions_needed": []
}
```

**Escalation** (immediate):
```json
{
  "orchestrator": "Interface Orchestrator",
  "type": "blocker",
  "severity": "high",
  "issue": "ZeroMQ IPC not working on ARM architecture",
  "affected_workstreams": ["WS-09", "WS-10"],
  "attempted_resolutions": ["Tried zeromq 4.3.4", "Checked libzmq ARM build"],
  "help_needed": "Need ARM-compatible ZeroMQ build or alternative IPC"
}
```

### 2. Meta Orchestrator → User

**Weekly Summary** (Fridays):
- Progress against roadmap
- Blockers and resolutions
- Phase gate status
- Next week priorities

**Decision Required** (immediate):
- Architecture decisions (create ADR)
- Scope changes (update MVP_SCOPE.md)
- Resource allocation (switch agent priorities)

---

## Failure Handling

### Orchestrator Failure Scenarios

**1. Orchestrator Stalls** (no progress for 24 hours):
- Meta Orchestrator spawns diagnostic agent
- Review logs, identify blocker
- Escalate to user if unresolvable

**2. Workstream Exceeds Timeline** (>150% estimated time):
- Meta Orchestrator analyzes root cause
- Options: Re-scope, add resources, defer to next phase
- Update docs/ROADMAP.md and notify user

**3. Phase Gate Failure** (acceptance criteria not met):
- Meta Orchestrator halts dependent orchestrators
- Create recovery plan with user
- Adjust timeline and dependencies

**4. Cross-Domain Conflict** (incompatible decisions):
- Meta Orchestrator creates conflict resolution ADR
- Convene user review meeting
- Document decision and update all affected workstreams

---

## Meta Orchestrator Lifecycle

### Initialization (Week 0)

```bash
# 1. Verify foundation documents exist
ls docs/orchestration/workstream-plan.md
ls docs/orchestration/orchestrators/*.md
ls docs/orchestration/workstreams/ws*/README.md

# 2. Verify hardware and dependencies
./repro/verify_hardware.sh
./repro/check_dependencies.sh

# 3. Create tracking infrastructure
mkdir -p .orchestration/{logs,status,decisions}

# 4. Initialize status tracking
cat > .orchestration/status/meta_status.json <<EOF
{
  "phase": "M0",
  "active_orchestrators": [],
  "completed_workstreams": [],
  "blockers": []
}
EOF

# 5. Spawn Foundation Orchestrator
claude-flow spawn orchestrator foundation
```

### Runtime (Weeks 1-12)

**Every 4 hours**:
1. Poll all active orchestrators for status
2. Update meta status JSON
3. Check for blockers and escalations
4. Verify phase gate readiness

**Every Friday**:
1. Generate weekly status report
2. Review phase gate progress
3. Plan next week orchestrator spawns
4. Post summary to GitHub

**On Completion**:
1. Verify all acceptance criteria met
2. Generate project completion report
3. Archive orchestration logs
4. Update docs/ROADMAP.md with actuals

### Shutdown (End of Project)

```bash
# 1. Verify all workstreams complete
./scripts/verify_completion.sh

# 2. Generate final report
./scripts/generate_completion_report.sh

# 3. Archive artifacts
tar -czf orchestration_archive.tar.gz .orchestration/

# 4. Update documentation
git add docs/COMPLETION_REPORT.md
git commit -m "Project complete: DGX-Pixels MVP"
```

---

## Directory Structure

```
dgx-pixels/
├── docs/
│   ├── orchestration/
│   │   ├── meta-orchestrator.md              # This file
│   │   ├── workstream-plan.md                 # All workstreams master plan
│   │   ├── project-summary.md                 # Summary document
│   │   ├── orchestrators/                     # Domain orchestrators
│   │   │   ├── foundation.md
│   │   │   ├── model.md
│   │   │   ├── interface.md
│   │   │   └── integration.md
│   │   └── workstreams/                       # Individual workstream specs
│   │       ├── start-here.md
│   │       ├── ws01-hardware-baselines/
│   │       ├── ws02-reproducibility/
│   │       └── ...
├── .orchestration/                        # Runtime tracking (gitignored)
│   ├── logs/
│   ├── status/
│   └── decisions/
└── scripts/                               # Orchestration utilities
    ├── spawn_orchestrator.sh
    ├── check_gate.sh
    └── generate_status.sh
```

---

## Agent Types for Domain Orchestrators

Based on raibid-labs patterns and workstream requirements:

| Orchestrator | Primary Agent Type | Rationale |
|--------------|-------------------|-----------|
| Foundation | `devops-automator` | Hardware setup, Docker, benchmarking |
| Model | `ai-engineer` + `python-pro` | ComfyUI, SDXL, LoRA training |
| Interface | `rust-pro` + `python-pro` | Rust TUI + Python backend |
| Integration | `backend-architect` + `frontend-developer` | MCP, Bevy, API design |

---

## Success Criteria

**Meta Orchestrator Success** means:

✅ All 18 workstreams completed within 12-week timeline (+/- 1 week acceptable)
✅ All phase gates passed with acceptance criteria met
✅ No unresolved cross-domain blockers
✅ All orchestrators coordinated effectively (minimal idle time)
✅ Project completion report generated with metrics

**Failure Modes to Avoid**:
- ❌ Orchestrators spawned out of dependency order (causes rework)
- ❌ Blockers unresolved for >48 hours (stalls progress)
- ❌ Phase gates skipped (quality issues downstream)
- ❌ Cross-domain conflicts unresolved (architectural debt)

---

## Quick Start for User

```bash
# 1. Review this document and workstream-plan.md
cat docs/orchestration/meta-orchestrator.md
cat docs/orchestration/workstream-plan.md

# 2. Verify hardware setup
./repro/verify_hardware.sh

# 3. Initialize meta orchestrator
./scripts/init_meta_orchestrator.sh

# 4. Spawn Foundation Orchestrator (Week 1)
claude-flow spawn orchestrator foundation --phase M0

# 5. Monitor progress
watch -n 300 ./scripts/show_status.sh  # Updates every 5 minutes

# 6. Review weekly reports every Friday
cat .orchestration/reports/week_01_status.md
```

---

**Next Steps**:
1. Review `docs/orchestration/workstream-plan.md` for detailed workstream breakdown
2. Review `docs/orchestration/orchestrators/` for domain orchestrator specifications
3. Review `docs/orchestration/workstreams/start-here.md` for workstream entry point
4. Approve meta-orchestrator approach
5. Begin Foundation Orchestrator spawn

**Questions for User**:
- Should orchestrators post status to GitHub Issues or use local tracking?
- Preferred notification method for blockers (GitHub comment, Slack, email)?
- Should Meta Orchestrator create ADRs automatically or request user approval first?
