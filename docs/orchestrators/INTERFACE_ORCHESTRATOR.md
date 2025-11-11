# Interface Orchestrator

**Domain**: UI/UX & Client-Server Communication
**Milestone**: M2
**Timeline**: Weeks 3-6
**Workstreams**: WS-08, WS-09, WS-10, WS-11, WS-12
**Status**: Blocked by Foundation (WS-01), Model (WS-04)

---

## Responsibility

Build Rust TUI application, implement ZeroMQ IPC layer, create Python backend worker, enable Sixel image preview, and deliver side-by-side model comparison feature. This orchestrator creates the user-facing interface and communication infrastructure.

---

## Workstreams Managed

### Phase 2B: Interface Development (Weeks 3-6)

**Mixed Execution** (sequential start, then parallel):

1. **WS-08**: Rust TUI Core (6-8 days) - Must complete first
2. **WS-09**: ZeroMQ IPC Layer (4-5 days) - Depends on WS-08
3. **WS-10**: Python Backend Worker (5-6 days) - Depends on WS-04, WS-09
4. **WS-11**: Sixel Image Preview (3-4 days) - Depends on WS-08, WS-10, parallel with WS-12
5. **WS-12**: Side-by-Side Comparison (4-5 days) - Depends on WS-10, WS-11, needs WS-06

**Total Duration**: 22-28 days (4 weeks with overlapping execution)

**Critical Path**: WS-08 → WS-09 → WS-10 → WS-12

---

## Agent Spawn Commands

### Week 3: Rust TUI Foundation (WS-08)

```bash
# Day 1-8: Rust TUI Core (can start after WS-01)
npx claude-flow@alpha spawn agent rust-pro \
  --workstream WS-08 \
  --spec docs/workstreams/WS-08-rust-tui/README.md \
  --priority P0 \
  --depends WS-01 \
  --context "ARM64 target, ratatui, 60+ FPS, tokio async, TDD" \
  --output docs/workstreams/WS-08-rust-tui/COMPLETION_SUMMARY.md
```

### Week 4: IPC Layer (WS-09)

```bash
# Day 9-13: ZeroMQ IPC Layer
npx claude-flow@alpha spawn agent rust-pro \
  --workstream WS-09 \
  --spec docs/workstreams/WS-09-zeromq-ipc/README.md \
  --priority P0 \
  --depends WS-08 \
  --context "<1ms latency REQ-REP, <100μs PUB-SUB, MsgPack, ARM zmq" \
  --output docs/workstreams/WS-09-zeromq-ipc/COMPLETION_SUMMARY.md
```

### Week 4: Backend Worker (WS-10)

```bash
# Day 14-19: Python Backend Worker (needs WS-04 ComfyUI + WS-09 IPC)
npx claude-flow@alpha spawn agent python-pro \
  --workstream WS-10 \
  --spec docs/workstreams/WS-10-python-backend/README.md \
  --priority P0 \
  --depends "WS-04,WS-09" \
  --context "ComfyUI API client, ZeroMQ server, asyncio, job queue" \
  --output docs/workstreams/WS-10-python-backend/COMPLETION_SUMMARY.md
```

### Week 5-6: Preview + Comparison (Parallel)

```bash
# Day 20-23: Sixel Image Preview (parallel with WS-12)
npx claude-flow@alpha spawn agent rust-pro \
  --workstream WS-11 \
  --spec docs/workstreams/WS-11-sixel-preview/README.md \
  --priority P1 \
  --depends "WS-08,WS-10" \
  --context "Sixel protocol, <100ms render, terminal detection, zero-copy" \
  --output docs/workstreams/WS-11-sixel-preview/COMPLETION_SUMMARY.md

# Day 20-24: Side-by-Side Model Comparison (needs WS-06 LoRA for testing)
npx claude-flow@alpha spawn agent rust-pro \
  --workstream WS-12 \
  --spec docs/workstreams/WS-12-model-comparison/README.md \
  --priority P1 \
  --depends "WS-10,WS-11" \
  --context "Multi-model generation, 2-4 models parallel, preference tracking" \
  --output docs/workstreams/WS-12-model-comparison/COMPLETION_SUMMARY.md
```

---

## Phase Gate: Interface Complete

### M2 Gate: Core Interface Ready (After WS-10)

**Criteria**:
- ✅ Rust TUI renders at 60+ FPS
- ✅ ZeroMQ IPC achieves <1ms latency
- ✅ Python backend connects to ComfyUI API
- ✅ End-to-end generation workflow functional
- ✅ Job queue handles concurrent requests
- ✅ Progress updates display in TUI
- ✅ Integration Orchestrator can proceed with WS-13 (MCP Server)

**Gate Check**:
```bash
./scripts/check_interface_m2_gate.sh

# Expected output:
# ✅ WS-08: Rust TUI Core - COMPLETE
# ✅ WS-09: ZeroMQ IPC - COMPLETE
# ✅ WS-10: Python Backend - COMPLETE
# ✅ TUI Performance: 65 FPS average
# ✅ IPC Latency: 0.8ms REQ-REP, 85μs PUB-SUB
# ✅ M2 Gate: PASSED - MCP integration can proceed
```

### M2+ Gate: Advanced Features (After WS-11, WS-12)

**Criteria**:
- ✅ Sixel image preview working in compatible terminals
- ✅ Side-by-side comparison displays 2-4 models
- ✅ User preference tracking saves to JSON
- ✅ Multi-model generation completes in ≤ 1.5× single model time
- ✅ Complete UI/UX polish

**Gate Check**:
```bash
./scripts/check_interface_advanced_gate.sh

# Expected output:
# ✅ WS-11: Sixel Preview - COMPLETE
# ✅ WS-12: Model Comparison - COMPLETE
# ✅ Sixel Render: 92ms average
# ✅ Multi-model: 1.3× single model time
# ✅ M2+ Gate: PASSED - Full interface ready
```

---

## Coordination Points

### With Meta Orchestrator

**Status Reports** (every 6 hours during WS-10):
```json
{
  "orchestrator": "Interface",
  "phase": "M2",
  "workstreams": {
    "WS-08": {"status": "complete", "completion_date": "2025-11-20"},
    "WS-09": {"status": "complete", "completion_date": "2025-11-24"},
    "WS-10": {"status": "in_progress", "progress": 0.70, "eta": "2025-11-28"},
    "WS-11": {"status": "pending", "blocked_by": "WS-10"},
    "WS-12": {"status": "pending", "blocked_by": "WS-10,WS-06"}
  },
  "performance_metrics": {
    "tui_fps": "62",
    "ipc_latency_ms": "0.9",
    "backend_response_ms": "120"
  },
  "blockers": [
    {"ws": "WS-12", "blocker": "Waiting on WS-06 LoRA checkpoint"}
  ],
  "eta": "2025-12-01T17:00:00Z"
}
```

**Escalations**:
- ZeroMQ ARM build issues (WS-09)
- TUI performance below 60 FPS (WS-08)
- ComfyUI API integration failures (WS-10)
- Sixel terminal compatibility problems (WS-11)

### With Foundation Orchestrator

**Handoff Received** (After WS-01):
- ARM CPU specifications for Rust compilation
- Terminal capabilities (for Sixel detection)
- Baseline system performance

### With Model Orchestrator

**Handoff Received** (After WS-04):
- ComfyUI API endpoint: `http://localhost:8188`
- API authentication (if required)
- Workflow JSON template locations
- Model loading specifications

**Handoff Received** (After WS-05):
- Optimized workflow templates (enables WS-10 testing)
- Expected inference times: 3s per image
- Batch processing capabilities
- Memory usage profiles

**Handoff Received** (After WS-06):
- LoRA checkpoint for WS-12 testing
- Custom model loading instructions
- Comparison baseline (pre-trained vs custom)

**Coordination**:
- WS-10 agent must coordinate with Model Orchestrator if WS-05 changes API
- WS-12 agent waits for WS-06 LoRA checkpoint before final testing

### With Integration Orchestrator

**Handoff Provided** (After WS-10):
- Python backend API for MCP integration
- Job submission interface
- Progress notification patterns
- Enable WS-13 (FastMCP Server) to proceed

---

## Dependencies

### Blocking Dependencies

**From Foundation Orchestrator**:
- ✅ WS-01: Hardware Baselines - REQUIRED
  - ARM64 architecture for Rust compilation
  - Terminal capabilities for Sixel support

**From Model Orchestrator**:
- ✅ WS-04: ComfyUI Setup - REQUIRED for WS-10
  - ComfyUI API must be operational
  - Basic workflow templates available
- ⏳ WS-06: LoRA Training - SOFT for WS-12
  - Custom model checkpoint for comparison testing
  - Can develop WS-12 with pre-trained models only

**External Dependencies**:
- Rust 1.70+ toolchain with ARM64 target
- ZeroMQ library (ARM-compatible)
- Python 3.10+ with asyncio
- Sixel-capable terminal (iTerm2, WezTerm, Alacritty)

### Software Dependencies

**WS-08 (Rust TUI)**:
```toml
[dependencies]
ratatui = "0.24"
crossterm = "0.27"
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

**WS-09 (ZeroMQ IPC)**:
```toml
[dependencies]
zmq = "0.10"
rmp-serde = "1.1"  # MsgPack serialization
```

**WS-10 (Python Backend)**:
```bash
pip install pyzmq>=25.0
pip install aiohttp>=3.9
pip install msgpack>=1.0
pip install asyncio
```

**WS-11 (Sixel Preview)**:
```toml
[dependencies]
image = "0.24"
sixel-rs = "0.1"  # Or custom Sixel implementation
```

---

## Known Issues & Mitigations

### Issue 1: ZeroMQ ARM Availability

**Problem**: ZeroMQ Rust crate may not have ARM wheels
**Impact**: Blocks WS-09, delays entire Interface domain
**Mitigation**:
- Check `zmq` crate ARM support: https://crates.io/crates/zmq
- Build from source if needed (link against system libzmq)
- Fallback: Unix domain sockets or gRPC
- Test early in WS-09 (first day)

**Priority**: P0 (must resolve in WS-09)

### Issue 2: TUI Performance on DGX-Spark

**Problem**: Remote terminal may limit TUI performance
**Impact**: May not achieve 60 FPS target in WS-08
**Mitigation**:
- Optimize rendering (only redraw changed components)
- Use local terminal with Sixel support for demos
- Profile with `perf` to identify bottlenecks
- Acceptable fallback: 30 FPS minimum

**Priority**: P1 (optimize, adjust target if needed)

### Issue 3: Sixel Terminal Compatibility

**Problem**: Not all terminals support Sixel protocol
**Impact**: Image preview fails on some terminals
**Mitigation**:
- Detect terminal capabilities on startup
- Fallback: ASCII art or external image viewer
- Document compatible terminals in README
- Provide instructions for Sixel-capable terminal setup

**Priority**: P2 (graceful degradation)

### Issue 4: ComfyUI API Stability

**Problem**: ComfyUI API may change or have undocumented behavior
**Impact**: WS-10 backend integration fails or breaks
**Mitigation**:
- Pin ComfyUI version in WS-04
- Document API version in completion summary
- Implement retry logic with exponential backoff
- Monitor ComfyUI GitHub for API changes

**Priority**: P1 (version pinning essential)

### Issue 5: Multi-Model Memory Pressure

**Problem**: Loading 2-4 models for WS-12 may exceed 128GB
**Impact**: Side-by-side comparison fails or degrades performance
**Mitigation**:
- Test with 2 models first, scale to 3-4 if memory allows
- Implement model unload/reload between comparisons
- Use model offloading to CPU if needed
- Document memory requirements for multi-model

**Priority**: P1 (test early, adjust feature scope)

---

## Success Criteria

### Orchestrator Success

✅ All 5 workstreams complete within 4 weeks (6-week buffer acceptable)
✅ M2 gate (core interface) passed by end of week 5
✅ M2+ gate (advanced features) passed by end of week 6
✅ Integration Orchestrator unblocked for WS-13 (MCP)
✅ No unresolved performance or usability issues

### Quality Standards

**Code**:
- Rust code follows rustfmt standard
- All public functions documented with rustdoc
- Unit tests for all modules (≥80% coverage)
- Python code follows PEP 8
- Type hints for all Python functions

**Performance**:
- WS-08: TUI renders at ≥60 FPS (P0)
- WS-09: REQ-REP latency <1ms, PUB-SUB <100μs (P0)
- WS-10: End-to-end generation in ≤5s (P0)
- WS-11: Sixel render <100ms (P1)
- WS-12: Multi-model generation ≤1.5× single (P1)

**Documentation**:
- Each workstream has detailed README
- TUI user guide with keybindings
- IPC protocol specification
- Backend API documentation
- Sixel terminal compatibility matrix

---

## Timeline

```
Week 3 (Days 15-22):
  Mon-Mon: WS-08 (Rust TUI Core)
         → ratatui framework setup
         → Screen layouts implemented
         → Event handling complete
         → 60+ FPS verified

Week 4 (Days 23-27):
  Tue-Fri: WS-09 (ZeroMQ IPC Layer)
         → ZeroMQ client in Rust
         → MsgPack serialization
         → Connection management
         → Latency benchmarks <1ms

Week 4-5 (Days 28-33):
  Sat-Thu: WS-10 (Python Backend Worker)
         → ZeroMQ server
         → ComfyUI API integration
         → Job queue implementation
         → M2 GATE CHECK (end of day 33)
         → HANDOFF to Integration (WS-13 can proceed)

Week 5-6 (Days 34-42):
  Fri-Fri: WS-11 (Sixel Preview) + WS-12 (Comparison) PARALLEL
         → WS-11: Sixel rendering, terminal detection
         → WS-12: Multi-model generation, preference tracking
         → Both complete by end of week 6
         → M2+ GATE CHECK

Week 6 (Buffer):
  Mon-Fri: Polish, bug fixes, integration testing
         → End-to-end workflow validation
         → Performance tuning
         → Documentation completion
```

**Buffer**: 1 week for ZeroMQ ARM issues or performance tuning

---

## Parallel Execution Strategy

### Week 3-4: Sequential Foundation

WS-08 and WS-09 MUST be sequential (TUI before IPC).
WS-09 and WS-10 MUST be sequential (IPC protocol before backend).

**Reason**: Each builds on the previous component.

### Week 5-6: Parallel Advanced Features

WS-11 and WS-12 can run in parallel after WS-10 completes.

**Resource Allocation**:
- **Agent 1 (rust-pro)**: Focus on WS-11 (Sixel preview)
- **Agent 2 (rust-pro)**: Focus on WS-12 (Model comparison)

**Coordination**:
- Both agents share WS-10 backend API
- WS-12 waits for WS-06 LoRA checkpoint (soft dependency)
- Daily sync to ensure UI component compatibility

**Expected Timeline Savings**: 2-3 days (vs sequential execution)

---

## Completion Checklist

Before marking Interface Orchestrator complete:

- [ ] WS-08 completion summary created
- [ ] WS-09 completion summary created
- [ ] WS-10 completion summary created
- [ ] WS-11 completion summary created
- [ ] WS-12 completion summary created
- [ ] M2 gate check passed and documented
- [ ] M2+ gate check passed and documented
- [ ] All files committed to git
- [ ] TUI binary built and tested: `dgx-pixels-tui`
- [ ] Backend worker tested end-to-end
- [ ] Example session recorded (TUI demo video or GIF)
- [ ] Handoff documentation sent to Integration Orchestrator
- [ ] All issues closed or transferred
- [ ] Final status report posted to Meta Orchestrator
- [ ] User guide published: `docs/interface/USER_GUIDE.md`

---

## Start Command

```bash
# Wait for Foundation Orchestrator M0 gate to pass
./scripts/check_foundation_gate.sh || exit 1

# Initialize Interface Orchestrator
./scripts/spawn_interface_orchestrator.sh

# Or manually:
cd /home/beengud/raibid-labs/dgx-pixels
cat docs/orchestrators/INTERFACE_ORCHESTRATOR.md
./scripts/spawn_agent.sh rust-pro WS-08
```

**Status**: Ready to spawn after Foundation Orchestrator completes WS-01.

**Note**: WS-10 will be blocked until Model Orchestrator completes WS-04 (ComfyUI Setup). Plan WS-08 and WS-09 to complete while waiting for WS-04.
