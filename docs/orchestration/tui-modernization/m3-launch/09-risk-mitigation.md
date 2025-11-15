# M3 Screen Migration - Risk Mitigation Strategies

**Version**: 1.0
**Purpose**: Identify, assess, and mitigate risks to M3 parallel screen migration success.

---

## Risk Assessment Framework

**Risk Levels**:
- **Critical**: Would prevent M3 completion or cause >2 day delay
- **High**: Could cause 1-2 day delay or require significant rework
- **Medium**: Minor delay (4-8 hours) or localized issue
- **Low**: Minimal impact (<4 hours) or easily resolved

**Mitigation Strategies**:
- **Prevent**: Eliminate risk before it occurs
- **Detect**: Identify risk early when mitigation is easier
- **Respond**: Have action plan ready if risk materializes
- **Accept**: Acknowledge risk, prepare fallback if needed

---

## Critical Risks

### CR-1: WS-06 (Image Assets) Has Critical Regression

**Scenario**: After M3 launches, WS-06 is discovered to have a critical bug that affects all screen migrations.

**Likelihood**: Low (5%)
**Impact**: Critical (blocks all M3 work)

**Detection**:
- Monitor for bug reports after WS-06 merge
- Early agent testing (Day 1) will reveal integration issues
- CI tests should catch most regressions

**Prevention**:
- Thorough review of WS-06 before merge
- Integration tests in WS-06 PR
- Post-merge validation before M3 launch

**Response** (if it occurs):
1. **Immediate**: Pause all M3 work (within 1 hour of detection)
2. **Assess**: Producer determines severity (blocker vs workaround)
3. **Hotfix**: Create emergency PR to fix WS-06 regression
4. **Fast-track**: Expedited review and merge of hotfix
5. **Resume**: All agents rebase onto hotfix, resume M3
6. **Impact**: +0.5 to 1 day delay to M3 timeline

**Fallback**:
- If hotfix takes >1 day: Revert WS-06 merge, postpone M3 until stable
- Risk acceptance: M3 delay acceptable if ensures stability

---

### CR-2: Multiple Agents Unavailable Simultaneously

**Scenario**: 2+ agents become unavailable (system failure, network issue, etc.) mid-workstream.

**Likelihood**: Very Low (2%)
**Impact**: Critical (could block multiple tracks)

**Detection**:
- No progress updates for >2 hours
- No git commits in >4 hours
- Agent unresponsive to messages

**Prevention**:
- Use reliable agent infrastructure
- Redundancy in agent provisioning
- Regular health checks (hourly)

**Response**:
1. **Immediate**: Attempt to restore unavailable agents (within 1 hour)
2. **Assess**: Determine if agents can be restored or need replacement
3. **Reassign**: Move incomplete work to available agents:
   - Track D agents (simplest) finish first → absorb Track B/C work
   - Alternative: Delay M3 by 1-2 days, provision new agents
4. **Preserve Work**: Original agent branches retained for continuity
5. **Impact**: +1 to 2 days delay

**Fallback**:
- If >50% of agents unavailable: Abort M3, reschedule launch
- Sequential execution fallback: Complete screens one-by-one (much slower)

---

## High Risks

### HR-1: Merge Conflicts Cascade Across Multiple PRs

**Scenario**: `plugins.rs` conflicts become non-trivial due to unforeseen issue (e.g., incorrect alphabetical ordering, accidental deletions).

**Likelihood**: Low (10%)
**Impact**: High (delays multiple merges, requires manual resolution)

**Detection**:
- First merge conflict reported by agent during rebase
- Studio producer reviews conflict in PR diff

**Prevention**:
- Alphabetical ordering strictly enforced
- Commented sections for each workstream
- Rebase-before-PR policy mandatory
- Studio producer reviews all `plugins.rs` changes before merge

**Response**:
1. **Identify**: Producer reviews conflict in first affected PR
2. **Pattern**: Determine if trivial (ordering) or non-trivial (logic change)
3. **Coordinate**: If non-trivial, producer assists agent with resolution
4. **Document**: Create guide for other agents if pattern repeats
5. **Stagger**: Increase merge spacing to 60 minutes (instead of 30)
6. **Impact**: +4 to 8 hours total across all merges

**Mitigation**:
- First merge (WS-15, WS-16) establishes clean pattern
- Subsequent merges follow established pattern
- Producer provides example conflict resolution

---

### HR-2: Performance Degradation After Multiple Merges

**Scenario**: Frame time degrades to >16ms after 4+ screens merged, violating performance budget.

**Likelihood**: Low (15%)
**Impact**: High (requires optimization or rollback)

**Detection**:
- Post-merge validation measures frame time
- Automated performance tests in CI
- Manual profiling if degradation suspected

**Prevention**:
- Per-screen performance benchmarks required in PRs
- Post-merge validation after each merge
- Frame time budget strictly enforced (<16ms)

**Response**:
1. **Detect**: Frame time >16ms in post-merge validation
2. **Profile**: Producer runs profiler to identify bottleneck
3. **Isolate**: Determine if specific screen or systemic issue
4. **Optimize**: If specific screen, create optimization PR
5. **Rollback**: If optimization fails, revert screen merge
6. **Fix**: Optimize screen offline, re-merge when passing
7. **Impact**: +0.5 to 1 day per affected screen

**Mitigation**:
- Render systems should early-return if inactive (`if current_screen.0 != Screen::{Name}`)
- Use efficient ratatui widgets (avoid heavy layouts)
- Limit real-time updates (Monitor, Queue screens) to reasonable refresh rates

---

### HR-3: Visual Regression Across Merged Screens

**Scenario**: A merged screen introduces subtle visual regression that affects multiple screens (e.g., theme change, layout shift).

**Likelihood**: Low (10%)
**Impact**: High (requires identifying and fixing regression)

**Detection**:
- Visual comparison in quality gate (should catch before merge)
- Post-merge manual testing
- Screenshot comparison tools

**Prevention**:
- Quality gate requires visual parity verification
- Theme changes must be isolated to `AppTheme` resource
- Code review checks for global layout modifications

**Response**:
1. **Detect**: Visual regression reported (within 24 hours of merge)
2. **Bisect**: Identify which merge introduced regression
3. **Assess**: Severity (critical vs cosmetic)
4. **Fix**: Create hotfix PR for specific issue
5. **Merge**: Fast-track hotfix (within 4 hours)
6. **Impact**: +4 hours for hotfix cycle

**Mitigation**:
- Use `AppTheme` resource for all styling (prevents hardcoded colors)
- Avoid global layout changes (keep screens isolated)
- Screenshot comparison before/after each merge

---

## Medium Risks

### MR-1: Agent Misunderstands Pattern, Implements Incorrectly

**Scenario**: Agent implements screen using incorrect pattern (e.g., missing screen guard, wrong event handling).

**Likelihood**: Medium (20%)
**Impact**: Medium (requires rework, delays PR by 4-8 hours)

**Detection**:
- Code review by studio producer
- CI tests fail (missing screen guard causes all screens to render)
- Visual comparison shows incorrect behavior

**Prevention**:
- Clear agent prompts with code examples
- WS-06 serves as reference implementation
- Early agent questions encouraged (don't wait >1 hour if stuck)

**Response**:
1. **Identify**: Producer spots pattern deviation in PR review
2. **Feedback**: Request changes with clear guidance
3. **Revise**: Agent fixes implementation
4. **Re-review**: Producer approves after fix
5. **Impact**: +4 to 8 hours for revision cycle

**Mitigation**:
- Agent prompts include pattern templates
- Common pitfalls documented in prompts
- Producer available for real-time questions

---

### MR-2: Test Coverage Below 75% Target

**Scenario**: Agent submits PR with <75% test coverage, failing quality gate.

**Likelihood**: Medium (25%)
**Impact**: Medium (delays PR by 2-4 hours)

**Detection**:
- Quality gate checklist requires coverage measurement
- CI runs coverage tools automatically
- Producer reviews coverage report in PR

**Prevention**:
- Agent prompts emphasize >75% coverage target
- Test templates provided in prompts
- TDD encouraged (write tests first)

**Response**:
1. **Identify**: Coverage <75% in PR
2. **Request**: Producer requests additional tests
3. **Add**: Agent adds tests to reach 75%
4. **Re-measure**: Verify coverage improved
5. **Impact**: +2 to 4 hours for test additions

**Mitigation**:
- Quality gate checklist catches before PR
- Coverage measured early in development (not just at PR time)

---

### MR-3: CI Failures Due to Linting or Formatting

**Scenario**: PR fails CI due to `just lint` or `just fmt` failures.

**Likelihood**: Medium (30%)
**Impact**: Low (easily fixed, <1 hour delay)

**Detection**:
- CI automatically runs `just ci`
- Failures show up in PR checks
- Producer reviews CI status before merge

**Prevention**:
- Agent prompts remind to run `just ci` before PR
- Quality gate checklist includes `just fmt` and `just lint`
- Agents run checks locally before pushing

**Response**:
1. **Identify**: CI failures in PR
2. **Fix**: Agent runs `just fmt` and `just lint` locally
3. **Push**: Update PR with fixes
4. **Re-run**: CI passes
5. **Impact**: +0.5 to 1 hour

**Mitigation**:
- Pre-commit hooks (optional but recommended)
- Automated formatting on save (IDE configuration)

---

## Low Risks

### LR-1: Agent Questions Cause Minor Delays

**Scenario**: Agent has questions about implementation, waits for producer response.

**Likelihood**: High (60%)
**Impact**: Low (2-4 hours per question if producer responsive)

**Detection**:
- Agent reports question in coordination channel
- Agent marks as blocked in progress dashboard

**Prevention**:
- Comprehensive agent prompts reduce questions
- Reference implementations available (WS-06)
- Encourage early questions (don't wait >1 hour)

**Response**:
1. **Identify**: Agent posts question
2. **Respond**: Producer answers within 1 hour
3. **Unblock**: Agent continues work
4. **Impact**: +1 to 2 hours per question

**Mitigation**:
- Producer availability during agent work hours
- FAQ document created after first question (reusable)
- Async communication for non-blocking questions

---

### LR-2: Rebase Conflicts (Trivial)

**Scenario**: Agent rebases onto main, encounters trivial conflict in `plugins.rs` (alphabetical ordering).

**Likelihood**: High (70%)
**Impact**: Low (<1 hour to resolve)

**Detection**:
- Git reports conflict during rebase
- Agent reviews conflict in `plugins.rs`

**Prevention**:
- Alphabetical ordering makes conflicts predictable
- Commented sections provide clear boundaries
- Conflict resolution guide in documentation

**Response**:
1. **Identify**: Conflict during rebase
2. **Resolve**: Keep both changes, maintain alphabetical order
3. **Continue**: `git add plugins.rs && git rebase --continue`
4. **Verify**: Run `just ci` to ensure no breakage
5. **Impact**: +0.25 to 0.5 hours

**Mitigation**:
- Conflict resolution template provided
- Producer available if agent unsure
- Most conflicts auto-resolve (alphabetical ordering)

---

### LR-3: Minor Visual Differences (Cosmetic)

**Scenario**: Bevy implementation has minor cosmetic differences from classic (e.g., spacing slightly off).

**Likelihood**: Medium (40%)
**Impact**: Low (cosmetic only, can be fixed later)

**Detection**:
- Visual comparison during quality gate
- Producer review during PR

**Prevention**:
- Quality gate requires pixel-perfect matching
- Screenshot comparison tools help identify differences
- Agent iterates until matching

**Response**:
1. **Identify**: Minor visual difference noted
2. **Assess**: Critical (blocks merge) vs cosmetic (nice-to-fix)
3. **Decision**:
   - Critical: Request changes before merge
   - Cosmetic: Create follow-up issue, merge PR
4. **Impact**: +0.5 to 2 hours if critical

**Mitigation**:
- Most cosmetic differences acceptable if functionality matches
- Follow-up issues tracked for polish later

---

## Risk Matrix

| Risk ID | Risk Name | Likelihood | Impact | Mitigation Strategy | Owner |
|---------|-----------|-----------|--------|---------------------|-------|
| **CR-1** | WS-06 Regression | Low | Critical | Prevent + Respond | Producer |
| **CR-2** | Multiple Agents Unavailable | Very Low | Critical | Prevent + Respond | Producer |
| **HR-1** | Cascade Merge Conflicts | Low | High | Prevent + Respond | Producer |
| **HR-2** | Performance Degradation | Low | High | Detect + Respond | Producer |
| **HR-3** | Visual Regression | Low | High | Prevent + Detect | Producer |
| **MR-1** | Pattern Misunderstanding | Medium | Medium | Prevent + Respond | Agents |
| **MR-2** | Low Test Coverage | Medium | Medium | Prevent + Detect | Agents |
| **MR-3** | CI Failures (Lint/Fmt) | Medium | Low | Prevent + Respond | Agents |
| **LR-1** | Agent Questions | High | Low | Prevent + Accept | Producer |
| **LR-2** | Trivial Rebase Conflicts | High | Low | Accept + Respond | Agents |
| **LR-3** | Cosmetic Visual Diffs | Medium | Low | Accept + Respond | Producer |

---

## Mitigation Action Plan

### Pre-Launch Actions (Day 0)

**Producer**:
- [ ] Verify WS-06 thoroughly (prevent CR-1)
- [ ] Provision reliable agent infrastructure (prevent CR-2)
- [ ] Review all launch materials (prevent MR-1)
- [ ] Prepare conflict resolution guide (prevent HR-1)

**Agents**:
- [ ] Review agent prompts thoroughly
- [ ] Understand pattern from WS-06
- [ ] Set up development environment (`just ci` works)

---

### During Execution (Days 1-5)

**Producer**:
- [ ] Monitor agent progress (hourly)
- [ ] Respond to questions within 1 hour
- [ ] Post-merge validation after each merge (detect HR-2, HR-3)
- [ ] Coordinate merge queue to avoid conflicts

**Agents**:
- [ ] Run `just ci` before each PR (prevent MR-3)
- [ ] Measure test coverage early (prevent MR-2)
- [ ] Ask questions early (<1 hour stuck)
- [ ] Rebase before PR (prevent conflicts)

---

### Emergency Response Protocols

#### Protocol 1: Critical Blocker (CR-1, CR-2)

**Trigger**: Work cannot proceed on multiple tracks

**Actions**:
1. Producer announces pause in coordination channel
2. All agents save work and await instructions
3. Producer investigates and resolves blocker
4. Timeline reassessed and communicated
5. Work resumes when blocker resolved

**Timeline**: <4 hours for investigation + resolution

---

#### Protocol 2: Merge Conflict Escalation (HR-1)

**Trigger**: Non-trivial conflict in `plugins.rs` or unexpected conflict

**Actions**:
1. Agent reports conflict in PR comments
2. Tags producer: `@producer Non-trivial conflict in plugins.rs`
3. Producer reviews within 1 hour
4. Producer coordinates resolution (pair with agent if needed)
5. Conflict resolved and documented for future agents

**Timeline**: <2 hours for coordination + resolution

---

#### Protocol 3: Performance Regression (HR-2)

**Trigger**: Frame time >16ms after merge

**Actions**:
1. Producer detects in post-merge validation
2. Producer profiles application (identify bottleneck)
3. If specific screen: Create optimization issue, optionally rollback
4. If systemic: Pause merges, optimize common code
5. Resume merges when performance restored

**Timeline**: <4 hours for profiling + optimization

---

## Risk Monitoring

### Key Indicators (Monitor Daily)

**Green (On Track)**:
- All agents active (commits within 4 hours)
- No blockers reported
- Merges proceeding on schedule
- CI passing for all PRs
- Frame time <16ms

**Yellow (At Risk)**:
- 1 agent delayed or blocked >4 hours
- Minor conflicts requiring manual resolution
- CI failures (lint/fmt) delaying PRs
- Frame time 14-16ms (approaching limit)

**Red (Critical)**:
- 2+ agents blocked or unavailable
- WS-06 regression detected
- Frame time >16ms after merge
- Non-trivial conflicts in multiple PRs
- >1 day delay to M3 timeline

---

## Contingency Buffers

### Timeline Buffer

**Target**: 5 days
**Buffer**: +1 day acceptable (6 days total)
**Critical**: >7 days requires reassessment

**Use Buffer For**:
- Agent unavailability (CR-2)
- Hotfixes (CR-1, HR-2, HR-3)
- Complex conflict resolution (HR-1)

---

### Resource Buffer

**Primary**: 4 agents (1 per track)
**Buffer**: Provision 1 additional agent on standby

**Use Buffer Agent For**:
- Replace unavailable agent (CR-2)
- Absorb delayed workstream (MR-1 causes delay)
- Parallel hotfix work (while primary agents continue)

---

## Lessons Learned (Post-M3)

**Document After M3 Completion**:
- Which risks materialized?
- How effective were mitigations?
- What unexpected risks occurred?
- What would we change for M4/M5?

**Format**: Retrospective document (Day 6)

---

## Summary

M3 risk mitigation focuses on:
1. **Prevention**: Clear prompts, proven patterns, thorough WS-06 review
2. **Detection**: Early monitoring, CI automation, post-merge validation
3. **Response**: Fast producer response (<1-2 hours), clear escalation protocols
4. **Acceptance**: Some risks (trivial conflicts, agent questions) are expected and manageable

**Risk Level**: **Low to Medium** - Architecture designed for safe parallelization, mitigation strategies in place.

**Expected Outcome**: M3 completes within 5-6 days with minimal disruption from risks.

---

**Risk mitigation strategies: READY FOR M3 LAUNCH**
