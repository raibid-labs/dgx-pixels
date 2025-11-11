# WS-01: Hardware Baselines

**ID**: WS-01
**Orchestrator**: Foundation
**Milestone**: M0
**Duration**: 3-4 days
**Priority**: P0 (CRITICAL PATH)
**Dependencies**: None
**Agent Type**: `devops-automator`
**Status**: Not Started

---

## Objective

Document verified DGX-Spark GB10 hardware specifications and establish baseline performance metrics. This is the foundational workstream that validates hardware capabilities and provides baseline data required by all downstream workstreams for optimization and regression testing.

**Importance**: This workstream blocks all other phases. Without accurate hardware baselines, we cannot validate optimizations, detect regressions, or ensure reproducibility across development cycles.

---

## Deliverables

1. **Hardware Verification Script** (`/home/beengud/raibid-labs/dgx-pixels/repro/hardware_verification.sh`)
   - Automated hardware detection and validation
   - JSON output for CI integration
   - Zero manual intervention required

2. **Baseline Metrics File** (`/home/beengud/raibid-labs/dgx-pixels/bench/baselines/hardware_baseline.json`)
   - GPU specifications (model, VRAM, compute capability)
   - CPU architecture (ARM Grace, core count, topology)
   - Memory characteristics (unified 128GB architecture)
   - Storage I/O throughput
   - Network interfaces (RoCE NICs)

3. **Updated Hardware Documentation** (`/home/beengud/raibid-labs/dgx-pixels/docs/hardware.md`)
   - Actual measured values replacing placeholders
   - Topology diagrams with nvidia-smi output
   - Unified memory architecture explanation
   - Known hardware quirks and ARM-specific notes

4. **Topology Documentation** (`/home/beengud/raibid-labs/dgx-pixels/docs/topology.txt`)
   - Complete nvidia-smi topo -m output
   - PCI bus configuration
   - NUMA node mapping

5. **Verification Test Suite**
   - Unit tests for hardware detection functions
   - Integration test verifying full hardware scan
   - CI/CD integration script

---

## Acceptance Criteria

**Functional**:
- ✅ Script captures: GPU model (GB10), VRAM (128GB), CUDA version (13.0), driver version (580.95.05)
- ✅ Script captures: CPU architecture (ARM), model (Grace), core count (20), NUMA topology
- ✅ Script captures: Total RAM (119 GiB available), unified memory architecture confirmed
- ✅ Script captures: Storage mount points, available space, I/O characteristics
- ✅ Script captures: Network interfaces (4× RoCE NICs with proper naming)
- ✅ Baseline JSON validates against schema (all required fields present, types correct)
- ✅ Documentation updated with actual measurements (no TBD placeholders)
- ✅ Verification script exits 0 on success, non-zero with descriptive error on failure

**Performance**:
- ✅ Hardware verification completes in ≤ 30 seconds
- ✅ Storage I/O test confirms ≥ 8 GB/s sustained read throughput
- ✅ Memory bandwidth baseline established for unified architecture
- ✅ GPU detection latency ≤ 5 seconds (nvidia-smi query time)

**Quality**:
- ✅ Test coverage ≥ 80% for all detection functions
- ✅ Code follows project style guide (shellcheck clean, proper error handling)
- ✅ Documentation includes code examples and usage instructions
- ✅ All outputs JSON-formatted for machine parsing

---

## Technical Requirements

### Environment
- **Hardware**: DGX-Spark GB10 (Grace Blackwell Superchip)
- **OS**: Ubuntu 22.04 (ARM64)
- **CUDA**: 13.0 (V13.0.88)
- **Driver**: nvidia-driver-580.95.05 or compatible
- **Kernel**: Linux 6.11.0-1016-nvidia or newer

### Dependencies

**System Packages**:
```bash
# Required for hardware detection
sudo apt install -y \
  nvidia-utils-580 \
  pciutils \
  util-linux \
  coreutils \
  jq \
  bc \
  sysstat \
  hdparm
```

**No Python/Rust dependencies** - Pure bash for maximum portability

### Technical Constraints
- Must run on ARM64 architecture (no x86-specific tools)
- Must work in containerized environments (no privileged operations where possible)
- Must handle missing hardware gracefully (don't crash if optional hardware absent)
- Must be idempotent (can run multiple times safely)
- Must not modify system state (read-only operations)
- Output must be valid JSON (parse with jq to validate)

### Known ARM-Specific Issues
- Some x86 hardware monitoring tools may not be available
- Use `lscpu` instead of `dmidecode` for CPU info (dmidecode unreliable on ARM)
- Use `nvidia-smi` for all GPU queries (no NVML Python bindings needed)
- RoCE NICs may have different naming conventions than Ethernet

---

## Implementation Plan

### Phase 1: Foundation (Day 1)
**Goal**: Set up basic script structure and core detection functions

**Tasks**:
1. Create `repro/hardware_verification.sh` with header and usage function
2. Implement GPU detection function using nvidia-smi
3. Implement CPU detection function using lscpu
4. Implement memory detection function using free
5. Add JSON output framework with jq
6. Write unit tests for each detection function

**Output**: Script skeleton with core detection working

**Verification**:
```bash
# Test GPU detection
bash repro/hardware_verification.sh --test-gpu

# Test CPU detection
bash repro/hardware_verification.sh --test-cpu

# Test JSON output
bash repro/hardware_verification.sh --json | jq .
```

### Phase 2: Extended Detection (Day 2)
**Goal**: Add storage, network, and topology detection

**Tasks**:
1. Implement storage detection (df, hdparm, fio if available)
2. Implement network interface detection (ip link, ethtool)
3. Implement GPU topology detection (nvidia-smi topo -m)
4. Add NUMA topology detection
5. Create baseline metrics JSON schema
6. Write integration test for full hardware scan

**Output**: Complete hardware detection with all fields populated

**Verification**:
```bash
# Run full hardware scan
bash repro/hardware_verification.sh > bench/baselines/hardware_baseline.json

# Validate JSON schema
jq -e '.gpu.model == "GB10"' bench/baselines/hardware_baseline.json
jq -e '.cpu.architecture == "aarch64"' bench/baselines/hardware_baseline.json
jq -e '.memory.total_gb >= 120' bench/baselines/hardware_baseline.json
```

### Phase 3: Documentation & Testing (Day 3-4)
**Goal**: Complete documentation, testing, and validation

**Tasks**:
1. Update `docs/hardware.md` with actual measurements from baseline JSON
2. Create topology diagram from nvidia-smi output
3. Add unified memory architecture explanation
4. Document known ARM-specific quirks
5. Write comprehensive test suite
6. Run benchmarks and record baseline performance
7. Create CI integration script
8. Write completion summary

**Output**: Fully documented and tested workstream

**Verification**:
```bash
# Run complete test suite
bash tests/test_hardware_verification.sh

# Run CI integration test
bash scripts/ci_hardware_check.sh

# Verify documentation completeness
grep -c "TBD" docs/hardware.md  # Should be 0
```

---

## Test-Driven Development (TDD)

### Test Requirements

**Unit Tests** (`tests/unit/ws_01/test_hardware_detection.sh`):
- `test_gpu_detection`: Verify nvidia-smi parsing extracts GB10, 128GB, compute 12.1
- `test_cpu_detection`: Verify lscpu parsing extracts ARM, Grace, 20 cores
- `test_memory_detection`: Verify free -h parsing extracts unified memory size
- `test_storage_detection`: Verify df parsing and I/O measurement
- `test_network_detection`: Verify ip link parsing for RoCE NICs
- `test_json_output`: Verify jq can parse all JSON output
- `test_error_handling`: Verify graceful failure when hardware missing

**Integration Tests** (`tests/integration/ws_01/test_full_scan.sh`):
- `test_full_hardware_scan`: Run complete script, verify all fields present
- `test_baseline_generation`: Generate baseline JSON, validate schema
- `test_idempotency`: Run script twice, verify outputs identical
- `test_ci_integration`: Verify CI script can consume baseline JSON

**Performance Tests** (`bench/ws_01_benchmark.sh`):
- `bench_script_runtime`: Measure total execution time (target: ≤ 30s)
- `bench_storage_io`: Measure storage read/write throughput (target: ≥ 8 GB/s)
- `bench_memory_bandwidth`: Measure unified memory bandwidth baseline

### Test Commands

```bash
# Run unit tests
bash tests/unit/ws_01/test_hardware_detection.sh -v

# Run integration tests
bash tests/integration/ws_01/test_full_scan.sh -v

# Run benchmarks
bash bench/ws_01_benchmark.sh

# Generate coverage report (using kcov or similar)
kcov coverage/ repro/hardware_verification.sh --test-all

# Run all tests
make test-ws-01
```

### Expected Test Output
```
✅ test_gpu_detection: PASSED (GB10 detected, 128GB VRAM)
✅ test_cpu_detection: PASSED (ARM Grace 20-core)
✅ test_memory_detection: PASSED (128GB unified memory)
✅ test_storage_detection: PASSED (I/O ≥ 8 GB/s)
✅ test_network_detection: PASSED (4× RoCE NICs found)
✅ test_json_output: PASSED (valid JSON schema)
✅ test_error_handling: PASSED (graceful degradation)

Integration Tests:
✅ test_full_hardware_scan: PASSED (all fields populated)
✅ test_baseline_generation: PASSED (schema valid)
✅ test_idempotency: PASSED (outputs match)
✅ test_ci_integration: PASSED (CI script success)

Benchmarks:
⏱️  Script runtime: 18.3s (target: ≤ 30s) ✅
⏱️  Storage I/O: 12.4 GB/s read (target: ≥ 8 GB/s) ✅
⏱️  Memory bandwidth: 435 GB/s (unified architecture)

WS-01 Test Suite: 11/11 PASSED ✅
```

---

## Dependencies

### Blocked By
- None (this is the first workstream in the project)

### Blocks
- **WS-02** (Reproducibility Framework): Needs hardware baseline to capture environment
- **WS-03** (Benchmark Suite): Needs hardware specs for performance normalization
- **WS-04** (ComfyUI Setup): Needs GPU verification before installation
- **WS-05** (SDXL Optimization): Needs memory/GPU baselines for optimization targets
- **WS-08** (Rust TUI Core): Needs hardware validation for testing
- **All other workstreams**: Foundation data required for all development

### Soft Dependencies
- None (truly independent foundation workstream)

---

## Known Issues & Risks

### Issue 1: ARM Architecture Compatibility
**Problem**: Some hardware detection tools are x86-only or behave differently on ARM
**Impact**: High (core functionality)
**Mitigation**:
- Use ARM-native tools: `lscpu`, `free`, `nvidia-smi` (all confirmed working)
- Avoid x86-specific tools: `dmidecode` (unreliable on ARM), `lshw` (limited ARM support)
- Test extensively on actual DGX-Spark hardware
**Fallback**: Manual hardware documentation if automated detection fails
**Status**: Low risk - nvidia-smi confirmed working on ARM GB10

### Issue 2: Unified Memory Detection
**Problem**: Unified memory architecture may not report separately as "GPU VRAM" and "CPU RAM"
**Impact**: Medium (documentation accuracy)
**Mitigation**:
- Use `nvidia-smi --query-gpu=memory.total` to get unified pool size
- Use `free -h` to get available system memory
- Document that both refer to same physical memory pool
- Add unified memory architecture explanation to docs
**Fallback**: Manual calculation from kernel logs if detection unclear
**Status**: Need to verify on actual hardware

### Issue 3: RoCE NIC Naming
**Problem**: RoCE NICs may have non-standard interface names (rocep1s0f0 vs eth0)
**Impact**: Low (network detection completeness)
**Mitigation**:
- Use `ip link show` to enumerate all interfaces
- Detect RoCE NICs by driver type (mlx5_core)
- Don't assume standard ethernet naming
**Fallback**: Document all detected interfaces regardless of naming
**Status**: Known issue, handled by flexible detection

### Issue 4: Storage I/O Measurement
**Problem**: fio may not be installed by default, hdparm requires root
**Impact**: Low (optional benchmark)
**Mitigation**:
- Check for fio availability, use if present
- Fall back to dd for basic throughput test (no root needed)
- Document limitation in baseline JSON
**Fallback**: Skip I/O benchmark if tools unavailable, document as "NOT_MEASURED"
**Status**: Low priority - can add later if needed

---

## Integration Points

### With Other Workstreams
- **WS-02 (Reproducibility)**: Provides hardware baseline for environment capture
- **WS-03 (Benchmarks)**: Provides hardware specs for performance normalization
- **WS-05 (SDXL Optimization)**: Provides memory/GPU constraints for tuning
- **WS-16 (DCGM Metrics)**: Provides baseline for comparison with runtime metrics

### With External Systems
- **CI/CD Pipeline**: JSON output consumed by GitHub Actions for regression detection
- **DCGM**: Baseline metrics used as reference for runtime monitoring
- **Docker Builds**: Hardware detection run during container build for validation
- **Documentation**: Baseline JSON used to auto-generate hardware specs

---

## Verification & Validation

### Verification Steps (Agent Self-Check)

```bash
# Step 1: Verify script exists and is executable
test -x /home/beengud/raibid-labs/dgx-pixels/repro/hardware_verification.sh && echo "✅ Script exists and executable"

# Step 2: Run script and verify JSON output
/home/beengud/raibid-labs/dgx-pixels/repro/hardware_verification.sh > /tmp/hw_baseline.json
jq -e . /tmp/hw_baseline.json && echo "✅ Valid JSON output"

# Step 3: Verify required fields present
jq -e '.gpu.model, .gpu.memory_gb, .cpu.architecture, .cpu.cores, .memory.total_gb' /tmp/hw_baseline.json && echo "✅ Required fields present"

# Step 4: Verify hardware matches expected DGX-Spark specs
jq -e 'select(.gpu.model == "GB10" and .cpu.architecture == "aarch64")' /tmp/hw_baseline.json && echo "✅ Hardware matches DGX-Spark"

# Step 5: Verify baseline file saved
test -f /home/beengud/raibid-labs/dgx-pixels/bench/baselines/hardware_baseline.json && echo "✅ Baseline file saved"

# Step 6: Verify documentation updated
grep -q "GB10" /home/beengud/raibid-labs/dgx-pixels/docs/hardware.md && echo "✅ Documentation updated"

# Step 7: Run test suite
bash /home/beengud/raibid-labs/dgx-pixels/tests/unit/ws_01/test_hardware_detection.sh && echo "✅ Unit tests passing"

# Step 8: Verify no TBD placeholders in docs
! grep -q "TBD" /home/beengud/raibid-labs/dgx-pixels/docs/hardware.md && echo "✅ Documentation complete"
```

### Acceptance Verification (Orchestrator)

```bash
# Run complete verification script
/home/beengud/raibid-labs/dgx-pixels/scripts/verify_ws_01.sh

# Expected output:
# ✅ Hardware verification script exists and is executable
# ✅ Script produces valid JSON output
# ✅ All required fields present in baseline
# ✅ Hardware matches DGX-Spark GB10 specifications
# ✅ Storage I/O meets minimum threshold (≥ 8 GB/s)
# ✅ Baseline file saved and validated
# ✅ Documentation updated with actual measurements
# ✅ No TBD placeholders remaining
# ✅ Unit tests passing (7/7)
# ✅ Integration tests passing (4/4)
# ✅ Performance benchmarks complete
# ✅ Test coverage ≥ 80%
#
# WS-01: READY FOR COMPLETION ✅
```

---

## Success Metrics

**Completion Criteria**:
- All acceptance criteria met (functional, performance, quality)
- All tests passing (≥80% coverage)
- Documentation complete with actual measurements
- Baseline JSON file generated and validated
- No blocking issues or critical bugs
- Completion summary created

**Quality Metrics**:
- Test coverage: ≥80% (measured with kcov)
- Code quality: shellcheck clean (no warnings)
- Documentation: Complete (0 TBD placeholders)
- Performance: Script runtime ≤ 30s
- Reliability: 100% success rate over 10 runs

**Baseline Metrics Captured**:
- GPU: Model, VRAM, compute capability, CUDA version, driver
- CPU: Architecture, model, cores, frequency, topology
- Memory: Total unified memory, available, bandwidth
- Storage: Mount points, capacity, I/O throughput
- Network: Interfaces, speeds, RoCE configuration
- Topology: PCI layout, NUMA nodes, GPU-CPU affinity

---

## Completion Checklist

Before marking WS-01 complete:

- [ ] Hardware verification script created and tested (`repro/hardware_verification.sh`)
- [ ] Baseline JSON generated and validated (`bench/baselines/hardware_baseline.json`)
- [ ] Documentation updated with actual measurements (`docs/hardware.md`)
- [ ] Topology documented (`docs/topology.txt`)
- [ ] Unit tests written and passing (≥7 tests)
- [ ] Integration tests written and passing (≥4 tests)
- [ ] Performance benchmarks run and recorded
- [ ] Test coverage ≥ 80%
- [ ] Code passes shellcheck with no warnings
- [ ] All acceptance criteria verified
- [ ] No TBD placeholders in documentation
- [ ] CI integration script created
- [ ] Completion summary created (`docs/orchestration/workstreams/ws01-hardware-baselines/COMPLETION_SUMMARY.md`)
- [ ] GitHub issue PIXELS-001 closed with summary link

---

## Example Baseline JSON Schema

```json
{
  "version": "1.0",
  "timestamp": "2025-11-10T12:00:00Z",
  "hostname": "dgx-spark-01",
  "gpu": {
    "model": "GB10",
    "count": 1,
    "memory_gb": 128,
    "compute_capability": "12.1",
    "cuda_version": "13.0.88",
    "driver_version": "580.95.05",
    "architecture": "Grace Blackwell"
  },
  "cpu": {
    "model": "Grace",
    "architecture": "aarch64",
    "cores": 20,
    "threads": 20,
    "vendor": "NVIDIA",
    "frequency_mhz": {
      "min": 1000,
      "max": 3000
    }
  },
  "memory": {
    "type": "unified",
    "total_gb": 128,
    "available_gb": 119,
    "bandwidth_gbs": 435
  },
  "storage": {
    "root": {
      "mount": "/",
      "size_gb": 1000,
      "used_gb": 250,
      "available_gb": 750,
      "io_throughput_gbs": 12.4
    }
  },
  "network": {
    "interfaces": [
      {"name": "rocep1s0f0", "type": "RoCE", "speed_gbps": 100},
      {"name": "rocep1s0f1", "type": "RoCE", "speed_gbps": 100},
      {"name": "rocep2s0f0", "type": "RoCE", "speed_gbps": 100},
      {"name": "rocep2s0f1", "type": "RoCE", "speed_gbps": 100}
    ]
  },
  "topology": {
    "numa_nodes": 1,
    "gpu_numa_id": "N/A",
    "pci_topology": "Single GPU system"
  }
}
```

---

## Related Issues

- GitHub Issue: #PIXELS-001 (Hardware Baseline Capture)
- GitHub Issue: #PIXELS-002 (ARM Architecture Validation)
- Related Workstreams: WS-02, WS-03, WS-04, WS-05
- Related Docs: `docs/hardware.md`, `docs/ROADMAP.md`

---

## References

- Architecture: `docs/02-architecture-proposals.md` (Proposal 2B)
- Roadmap: `docs/ROADMAP.md` (M0 - Foundation & Reproducibility)
- Hardware: `docs/hardware.md` (DGX-Spark GB10 specifications)
- Metrics: `docs/metrics.md` (Performance targets)
- NVIDIA GB10 Specs: [DGX-Spark Documentation]
- nvidia-smi Reference: [NVIDIA System Management Interface Documentation]

---

**Status**: Ready for agent spawn
**Last Updated**: 2025-11-10
**Estimated LOC**: 200-300 (bash + documentation + tests)
