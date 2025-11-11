# WS-XX: [Workstream Name]

**ID**: WS-XX
**Orchestrator**: [Foundation | Model | Interface | Integration]
**Milestone**: MX
**Duration**: X-X days
**Priority**: P0/P1/P2
**Dependencies**: WS-XX, WS-YY (or "None")
**Agent Type**: `agent-type`
**Status**: Not Started / In Progress / Completed

---

## Objective

One paragraph describing the goal of this workstream and its role in the overall project.

---

## Deliverables

1. **File/Component 1** - Description
2. **File/Component 2** - Description
3. **File/Component 3** - Description
4. **Documentation** - What docs are expected
5. **Tests** - What testing is required

---

## Acceptance Criteria

**Functional**:
- ✅ Criterion 1: Specific, testable requirement
- ✅ Criterion 2: Specific, testable requirement
- ✅ Criterion 3: Specific, testable requirement

**Performance**:
- ✅ Performance target 1 (with metrics)
- ✅ Performance target 2 (with metrics)

**Quality**:
- ✅ Test coverage ≥ 80%
- ✅ Code review completed
- ✅ Documentation complete

---

## Technical Requirements

### Environment
- Hardware: DGX-Spark GB10
- OS: Ubuntu 22.04 (ARM64)
- CUDA: 13.0
- Python: 3.10+ (if applicable)
- Rust: 1.70+ (if applicable)

### Dependencies
**System Packages**:
```bash
sudo apt install package1 package2
```

**Python Packages** (if applicable):
```
package1>=1.0.0
package2>=2.0.0
```

**Rust Crates** (if applicable):
```toml
[dependencies]
crate1 = "1.0"
crate2 = "2.0"
```

### Technical Constraints
- Constraint 1
- Constraint 2
- Constraint 3

---

## Implementation Plan

### Phase 1: Foundation (Days 1-X)
**Goal**: Set up basic structure

**Tasks**:
1. Task 1
2. Task 2
3. Task 3

**Output**: What exists after this phase

### Phase 2: Core Implementation (Days X-Y)
**Goal**: Build main functionality

**Tasks**:
1. Task 1
2. Task 2
3. Task 3

**Output**: What exists after this phase

### Phase 3: Testing & Documentation (Days Y-Z)
**Goal**: Validate and document

**Tasks**:
1. Write tests (unit + integration)
2. Run benchmarks
3. Write documentation
4. Code review

**Output**: Completed workstream

---

## Test-Driven Development (TDD)

### Test Requirements

**Unit Tests**:
- Test 1: Description
- Test 2: Description
- Test 3: Description

**Integration Tests**:
- Test 1: Description
- Test 2: Description

**Performance Tests**:
- Benchmark 1: Target metric
- Benchmark 2: Target metric

### Test Commands

```bash
# Run unit tests
pytest tests/unit/ws_xx/ -v

# Run integration tests
pytest tests/integration/ws_xx/ -v

# Run benchmarks
python bench/ws_xx_benchmark.py

# Generate coverage report
pytest --cov=src/ws_xx --cov-report=html
```

---

## Dependencies

### Blocked By
- **WS-XX**: Why blocked
- **WS-YY**: Why blocked

### Blocks
- **WS-XX**: Why blocking
- **WS-YY**: Why blocking

### Soft Dependencies
- **WS-XX**: Helpful but not required

---

## Known Issues & Risks

### Issue 1: [Issue Name]
**Problem**: Description
**Impact**: High / Medium / Low
**Mitigation**: How to handle
**Fallback**: Alternative approach

### Issue 2: [Issue Name]
**Problem**: Description
**Impact**: High / Medium / Low
**Mitigation**: How to handle
**Fallback**: Alternative approach

---

## Integration Points

### With Other Workstreams
- **WS-XX**: How they integrate
- **WS-YY**: How they integrate

### With External Systems
- **System 1**: Integration details
- **System 2**: Integration details

---

## Verification & Validation

### Verification Steps (Agent Self-Check)

```bash
# Step 1: Verify deliverable 1
test -f path/to/deliverable1 && echo "✅ Deliverable 1 exists"

# Step 2: Run tests
pytest tests/ws_xx/ --exitfirst && echo "✅ Tests passing"

# Step 3: Verify performance
python bench/ws_xx_benchmark.py && echo "✅ Performance meets target"

# Step 4: Verify documentation
test -f docs/ws_xx/README.md && echo "✅ Documentation exists"
```

### Acceptance Verification (Orchestrator)

```bash
# Run complete verification
./scripts/verify_ws_xx.sh

# Expected output:
# ✅ All deliverables present
# ✅ All tests passing
# ✅ Performance targets met
# ✅ Documentation complete
# ✅ Code reviewed
# WS-XX: READY FOR COMPLETION
```

---

## Success Metrics

**Completion Criteria**:
- All acceptance criteria met
- All tests passing (≥80% coverage)
- Performance targets achieved
- Documentation complete
- Code reviewed and merged
- Completion summary created

**Quality Metrics**:
- Test coverage: ≥80%
- Code review: Approved
- Documentation: Complete
- Performance: Meets targets

---

## Completion Checklist

Before marking WS-XX complete:

- [ ] All deliverables created and committed
- [ ] All acceptance criteria verified
- [ ] Unit tests written and passing
- [ ] Integration tests written and passing
- [ ] Performance benchmarks run and passing
- [ ] Documentation written (README, API docs, guides)
- [ ] Code reviewed and approved
- [ ] No known blockers or critical issues
- [ ] Integration points tested
- [ ] Completion summary created (`COMPLETION_SUMMARY.md`)
- [ ] GitHub issue closed with summary link

---

## Completion Summary Template

After completing workstream, create `docs/orchestration/workstreams/wsXX/COMPLETION_SUMMARY.md`:

```markdown
# WS-XX: [Workstream Name] - Completion Summary

**Status**: ✅ COMPLETE
**Completion Date**: YYYY-MM-DD
**Duration**: X days (estimated: Y days)
**Agent**: [Agent type]

## Deliverables Created
1. File/Component 1 (XXX lines)
2. File/Component 2 (XXX lines)
...

## Acceptance Criteria Verification
✅ Criterion 1: [How verified]
✅ Criterion 2: [How verified]
...

## Test Results
- Unit tests: XX/XX passing (100%)
- Integration tests: XX/XX passing (100%)
- Coverage: XX%
- Performance: [Benchmark results]

## Code Quality Metrics
- Lines of code: XXX
- Test coverage: XX%
- Code review: Approved by [reviewer]
- Documentation: Complete

## Known Limitations
- Limitation 1
- Limitation 2

## Future Enhancements
- Enhancement 1
- Enhancement 2

## Blockers Resolved
- Blocker 1: [How resolved]
- Blocker 2: [How resolved]

## Integration Status
- Integrates with WS-XX: ✅ Tested
- Blocks WS-YY: ✅ Unblocked

## Next Steps
- WS-YY can now proceed
- [Other follow-up items]
```

---

## Related Issues

- GitHub Issue: #PIXELS-XXX
- Related Workstreams: WS-YY, WS-ZZ
- Related Docs: [List relevant docs]

---

## References

- Architecture: docs/02-architecture-proposals.md
- Roadmap: docs/ROADMAP.md
- Metrics: docs/metrics.md
- Hardware: docs/hardware.md
- [Other relevant docs]

---

**Status**: Ready for agent spawn
**Last Updated**: YYYY-MM-DD
