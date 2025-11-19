# Parallel Orchestration Work Plan

**Date**: 2025-11-19
**Status**: Planning Phase
**Goal**: Fix critical sixel/gallery issues using parallel orchestration

---

## Strategy: 3-Track Parallel Execution

### Track 1: Critical Bug Fixes (HIGHEST PRIORITY)
**Agent**: General-purpose
**Duration**: 2-3 hours
**Deliverables**:
1. img2sixel validation in ImageRenderer
2. Gallery error handling (timeout + error display)
3. Error state management in App
4. Retry functionality on 'r' key

**Files to Modify**:
- `rust/src/sixel/image_renderer.rs` - Add validation
- `rust/src/sixel/preview_manager.rs` - Update constructor
- `rust/src/app.rs` - Add error state tracking
- `rust/src/ui/screens/gallery.rs` - Add error handling + timeout
- `rust/src/ui/theme.rs` - Add error styling (if needed)

### Track 2: Preview Manager Tests
**Agent**: General-purpose (test-writer-fixer)
**Duration**: 1-2 hours
**Deliverables**:
1. Unit tests for PreviewManager (15-20 tests)
2. Tests for cache operations
3. Tests for async preview loading
4. Tests for error handling
5. Coverage > 85%

**Files to Create**:
- `rust/tests/preview_manager_test.rs`

### Track 3: Snapshot Testing
**Agent**: General-purpose
**Duration**: 1 hour
**Deliverables**:
1. Snapshot tests for gallery screen (empty, with images, errors)
2. Snapshot tests for generation screen
3. Snapshot tests for comparison screen
4. Insta review workflow setup

**Files to Create**:
- `rust/tests/snapshot_gallery_test.rs`
- `rust/tests/snapshot_screens_test.rs`

---

## Execution Plan

### Phase 1: Launch Parallel Agents (5 minutes)

Launch 3 agents simultaneously:

**Agent 1 - Critical Fixes**:
```
Task: Implement critical bug fixes for sixel preview and gallery
1. Add img2sixel validation to ImageRenderer::new()
2. Update PreviewManager to handle validation errors
3. Add preview_requests and preview_errors tracking to App
4. Implement gallery timeout (5s) and error display
5. Add retry on 'r' key in gallery input handler
6. Test all changes compile and work
Follow: docs/testing/NEXT_STEPS.md Day 1 tasks
```

**Agent 2 - Preview Manager Tests**:
```
Task: Write comprehensive tests for PreviewManager
1. Create rust/tests/preview_manager_test.rs
2. Write 15-20 tests covering:
   - Creation and initialization
   - Cache operations (hit/miss)
   - LRU eviction
   - Async preview loading
   - Error handling
   - Cache statistics
3. Ensure > 85% coverage
4. All tests must pass
Reference: docs/testing/NEXT_STEPS.md Task 3.1
```

**Agent 3 - Snapshot Tests**:
```
Task: Create snapshot tests for UI screens
1. Create rust/tests/snapshot_gallery_test.rs
2. Write tests for:
   - Gallery empty state
   - Gallery with images
   - Gallery with selection
   - Error states
3. Use insta for snapshots
4. Include test mode guard
Reference: docs/testing/NEXT_STEPS.md Task 3.2
```

### Phase 2: Monitor & Coordinate (ongoing)

As agents complete work:
- Verify no conflicts between changes
- Run full test suite
- Check code quality (clippy, fmt)
- Integrate changes

### Phase 3: Verification & Integration (30 minutes)

1. Run all tests: `cargo test`
2. Check clippy: `cargo clippy`
3. Format code: `cargo fmt`
4. Generate coverage: `cargo llvm-cov --html`
5. Review snapshots: `cargo insta review`
6. Commit and push all changes

---

## Success Criteria

### Critical Fixes (Track 1)
- [ ] ImageRenderer::new() validates img2sixel
- [ ] Clear error message when img2sixel missing
- [ ] Gallery shows timeout after 5s
- [ ] Gallery displays error messages
- [ ] Retry works on 'r' key
- [ ] All existing tests still pass
- [ ] Manual testing shows errors instead of infinite loading

### Preview Manager Tests (Track 2)
- [ ] 15+ tests created
- [ ] All tests pass
- [ ] Coverage > 85%
- [ ] Tests cover async behavior
- [ ] Tests cover error cases

### Snapshot Tests (Track 3)
- [ ] Gallery snapshots created
- [ ] Screen snapshots created
- [ ] Snapshots reviewed and accepted
- [ ] All snapshot tests pass

### Overall
- [ ] `cargo test` - all pass
- [ ] `cargo clippy` - no warnings
- [ ] `cargo fmt --check` - formatted
- [ ] Coverage increased from ~30% to > 60%
- [ ] All changes committed and pushed

---

## Risk Mitigation

### Potential Conflicts
- Multiple agents modifying App struct
- **Solution**: Agent 1 modifies App first, others wait

### Test Failures
- New code might break existing tests
- **Solution**: Run tests incrementally, fix as we go

### Integration Issues
- Changes might not work together
- **Solution**: Verify after each agent completes

---

## Rollback Plan

If critical issues arise:
1. Revert to commit: `59e78f1` (current HEAD)
2. Apply fixes incrementally
3. Test each change individually

---

## Timeline

| Phase | Duration | Status |
|-------|----------|--------|
| Planning | 5 min | ✅ Complete |
| Agent Launch | 2 min | 🔲 Ready |
| Agent Execution | 2-3 hours | 🔲 Pending |
| Integration | 30 min | 🔲 Pending |
| **Total** | **3-4 hours** | **🔲 Pending** |

---

## Commands Reference

```bash
# Launch agents (use Task tool with parallel calls)
# (See execution section below)

# Monitor progress
watch -n 5 'git status --short'

# Verify compilation
cargo check

# Run tests
cargo test

# Check quality
cargo clippy
cargo fmt --check

# Coverage
cargo llvm-cov --html --open

# Review snapshots
cargo insta review

# Commit and push
git add -A
git commit -m "feat: Fix critical sixel/gallery issues + comprehensive tests"
git push origin main
```

---

**Ready to Execute**: Yes
**Estimated Completion**: 3-4 hours
**Risk Level**: Low (tests provide safety net)
