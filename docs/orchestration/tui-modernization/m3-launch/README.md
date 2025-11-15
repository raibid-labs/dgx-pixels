# M3 Screen Migration - Launch Materials

**Version**: 1.0
**Created**: 2025-11-14
**Status**: Ready for Launch
**Launch Trigger**: WS-06 (Image Assets) merged to main

---

## Executive Summary

This directory contains all materials required to launch M3 Screen Migration with **zero delays** when WS-06 completes.

**M3 Overview**:
- **Workstreams**: 8 parallel screen migrations (WS-09 through WS-16)
- **Timeline**: 5 days (with 1-day buffer acceptable)
- **Execution**: 4 tracks, 2 screens each, massive parallelization
- **Risk**: Low (zero file conflicts by design)

**Expected Outcome**: All 8 screens migrated to Bevy ECS cleanly within 5 days.

---

## Quick Start Guide

### Studio Producer: Pre-Launch Checklist

When WS-06 merges to main, follow these steps:

1. **Verify WS-06 Complete**:
   ```bash
   git pull origin main
   git log --oneline -n 5 | grep "WS-06"
   just ci  # Verify everything green
   ```

2. **Create All Branches**:
   ```bash
   bash docs/orchestration/tui-modernization/m3-launch/01-branch-creation.sh
   ```

3. **Review Launch Materials**:
   - [ ] Agent assignments: `02-agent-assignments.md`
   - [ ] Agent prompts: `03-agent-prompts/` (all 8 files)
   - [ ] Merge strategy: `05-merge-strategy.md`
   - [ ] Quality gates: `06-quality-gate-checklist.md`

4. **Initialize Dashboard**:
   - [ ] Open `08-progress-dashboard.md`
   - [ ] Update "Day 0" section with actual date
   - [ ] Mark "WS-06 merged" as complete

5. **Launch Agents** (staggered 15-minute intervals):
   ```bash
   # 09:00 AM - Track A
   # Provide agent-1 with: 03-agent-prompts/ws09-generation-screen.md

   # 09:15 AM - Track B
   # Provide agent-2 with: 03-agent-prompts/ws10-gallery-screen.md

   # 09:30 AM - Track C
   # Provide agent-3 with: 03-agent-prompts/ws13-queue-screen.md

   # 09:45 AM - Track D
   # Provide agent-4 with: 03-agent-prompts/ws15-settings-screen.md
   ```

6. **Monitor Progress**:
   - Use `08-progress-dashboard.md` for daily tracking
   - Coordinate merges using `05-merge-strategy.md`
   - Respond to blockers using `09-risk-mitigation.md`

---

## Directory Structure

```
m3-launch/
├── README.md                          # This file - overview and quick start
├── 01-branch-creation.sh              # Executable script to create all 8 branches
├── 02-agent-assignments.md            # Track assignments and agent coordination
├── 03-agent-prompts/                  # Standardized prompts for each screen
│   ├── ws09-generation-screen.md      #   - Track A: High complexity
│   ├── ws10-gallery-screen.md         #   - Track B: Medium complexity
│   ├── ws11-comparison-screen.md      #   - Track A: High complexity
│   ├── ws12-models-screen.md          #   - Track B: Medium complexity
│   ├── ws13-queue-screen.md           #   - Track C: Medium complexity
│   ├── ws14-monitor-screen.md         #   - Track C: Medium complexity
│   ├── ws15-settings-screen.md        #   - Track D: Low complexity
│   └── ws16-help-screen.md            #   - Track D: Low complexity
├── 04-conflict-prevention.md          # File ownership matrix and conflict strategies
├── 05-merge-strategy.md               # Merge ordering, workflow, and coordination
├── 06-quality-gate-checklist.md       # Per-screen quality gate template
├── 07-launch-plan.md                  # Day-by-day launch coordination plan
├── 08-progress-dashboard.md           # Live progress tracking dashboard
└── 09-risk-mitigation.md              # Risk assessment and mitigation strategies
```

---

## Document Purposes

### 1. Branch Creation Script (`01-branch-creation.sh`)

**Purpose**: Automate creation of all 8 branches in one command.

**Usage**:
```bash
bash docs/orchestration/tui-modernization/m3-launch/01-branch-creation.sh
```

**Output**: 8 branches created from latest main:
- `tui-modernization/ws09-generation-screen`
- `tui-modernization/ws10-gallery-screen`
- ... (all 8 screens)

**When to Run**: Day 0 (after WS-06 merges, before agent launch)

---

### 2. Agent Assignments (`02-agent-assignments.md`)

**Purpose**: Define which agent works on which screens, organized by complexity tracks.

**Key Sections**:
- Track A (High): WS-09, WS-11 - agent-1
- Track B (Medium): WS-10, WS-12 - agent-2
- Track C (Medium): WS-13, WS-14 - agent-3
- Track D (Low): WS-15, WS-16 - agent-4

**When to Use**: Agent provisioning, track coordination

---

### 3. Agent Prompts (`03-agent-prompts/`)

**Purpose**: Standardized, ready-to-use prompts for each screen workstream.

**Structure** (per prompt):
- Mission statement
- Context and dependencies
- Implementation requirements (render system, input handler)
- Success criteria
- Testing requirements
- Return deliverables

**Usage**: Copy-paste prompt to agent when launching workstream.

**When to Use**: Day 1 (agent launch), each workstream start

---

### 4. Conflict Prevention (`04-conflict-prevention.md`)

**Purpose**: Document file ownership to ensure zero conflicts during parallel execution.

**Key Sections**:
- File ownership matrix (exclusive files per screen)
- Shared file coordination (`plugins.rs`)
- Conflict resolution workflow
- Verification checklist

**When to Use**: Reference during development, PR reviews

---

### 5. Merge Strategy (`05-merge-strategy.md`)

**Purpose**: Define merge ordering, workflow, and quality gates.

**Key Sections**:
- Recommended merge order (simple → complex)
- Per-screen merge workflow (checklist → PR → review → merge)
- Conflict resolution protocol
- Rollback strategy

**When to Use**: PR creation, merge coordination, conflict resolution

---

### 6. Quality Gate Checklist (`06-quality-gate-checklist.md`)

**Purpose**: Per-screen checklist that MUST be completed before merge.

**Sections**:
- Implementation complete (files created, registered)
- Functional requirements (visual parity, interactions)
- Testing (unit tests, integration tests, coverage >75%)
- Performance (<16ms frame time)
- Code quality (fmt, lint, CI passing)
- Git hygiene (rebased, no conflicts)

**When to Use**: Before creating PR (agent self-review), during PR review (producer validation)

---

### 7. Launch Plan (`07-launch-plan.md`)

**Purpose**: Day-by-day coordination plan from pre-launch (Day 0) through completion (Day 5).

**Key Sections**:
- Pre-launch checklist (Day 0)
- Launch day timeline (Day 1, staggered starts)
- Daily rhythm (standups, syncs, EOD updates)
- Expected completion timeline (Day 2-5)
- Communication protocols

**When to Use**: Studio producer's primary coordination document

---

### 8. Progress Dashboard (`08-progress-dashboard.md`)

**Purpose**: Live tracking of all 8 workstreams, blockers, merges, and metrics.

**Key Sections**:
- Screen migration progress table (updated EOD)
- Track progress summary
- Daily progress log (producer maintains)
- Blockers and issues
- Merge queue
- Performance metrics
- Milestone tracking

**When to Use**: Daily updates (agents + producer), real-time status checks

---

### 9. Risk Mitigation (`09-risk-mitigation.md`)

**Purpose**: Identify, assess, and prepare responses for all potential risks.

**Risk Categories**:
- **Critical**: WS-06 regression, multiple agents unavailable
- **High**: Cascade merge conflicts, performance degradation, visual regressions
- **Medium**: Pattern misunderstanding, low test coverage, CI failures
- **Low**: Agent questions, trivial rebase conflicts, cosmetic visual diffs

**When to Use**: Reference when risks materialize, proactive mitigation planning

---

## Launch Workflow

### Phase 1: Pre-Launch (Day 0)

**Trigger**: WS-06 merged to main

**Tasks**:
1. Studio producer verifies WS-06 stable
2. Run branch creation script
3. Review all launch materials
4. Initialize progress dashboard
5. Provision agent instances

**Duration**: 2-4 hours

**Outcome**: Ready to launch M3 on Day 1

---

### Phase 2: Launch Day (Day 1)

**Timeline**:
- **08:00 AM**: Producer prep
- **09:00 AM**: Launch Track A (agent-1, WS-09)
- **09:15 AM**: Launch Track B (agent-2, WS-10)
- **09:30 AM**: Launch Track C (agent-3, WS-13)
- **09:45 AM**: Launch Track D (agent-4, WS-15)
- **10:00 AM**: All agents active
- **12:00 PM**: Midday check-in
- **05:00 PM**: EOD summary

**Outcome**: All 8 workstreams in progress, zero blockers

---

### Phase 3: Execution & Merging (Days 2-5)

**Day 2-3**: Track D (simple screens) complete, first merges
**Day 3-4**: Track B/C (medium complexity) complete, multiple merges
**Day 4-5**: Track A (complex screens) complete, final merges

**Daily Rhythm**:
- 09:00 AM: Daily standup (15 minutes)
- Throughout day: Work execution, PR creation
- Merges: Staggered 30-minute intervals
- 05:00 PM: EOD progress update

**Outcome**: All 8 screens merged, M3 complete

---

### Phase 4: Completion (Day 5 late / Day 6)

**Final Validation**:
- Producer runs full integration test
- Verify all 8 screens functional
- Performance maintained (<16ms)
- No regressions

**Announcement**: M3 completion in coordination channel

**Retrospective** (Day 6): Document lessons learned

---

## Success Criteria

### Per-Screen Success
- [ ] Visual parity with classic screen verified
- [ ] All interactions functional
- [ ] >75% test coverage
- [ ] Performance <16ms frame time
- [ ] Quality gate checklist complete
- [ ] Merged to main with zero rollbacks

### Overall M3 Success
- [ ] All 8 screens merged within 5 days (buffer: 6 days acceptable)
- [ ] <10% conflict resolution rate (target: 0-1 conflicts total)
- [ ] <5% rollback rate (target: 0 rollbacks)
- [ ] Zero regressions in M1/M2 systems
- [ ] Full user workflow functional (all screens navigable)

---

## Communication Channels

### Primary: Coordination Channel

**Platform**: Slack, Discord, or similar

**Participants**: 4 agents + studio producer

**Usage**: Real-time coordination, announcements, blockers, questions

---

### Secondary: Progress Dashboard

**Platform**: Markdown file (this repository)

**Update Frequency**: EOD (minimum), optionally after major milestones

**Usage**: Asynchronous status tracking, historical record

---

### Official: GitHub

**Platform**: GitHub PRs and issues

**Usage**: Code review, merge tracking, CI status

---

## Agent Responsibilities

### Each Agent Completes 2 Screens

**Assigned Screens** (see `02-agent-assignments.md`):
- Agent 1: WS-09 (Generation), WS-11 (Comparison)
- Agent 2: WS-10 (Gallery), WS-12 (Models)
- Agent 3: WS-13 (Queue), WS-14 (Monitor)
- Agent 4: WS-15 (Settings), WS-16 (Help)

### Per-Screen Workflow

1. **Checkout branch**: `git checkout tui-modernization/ws##-{screen}-screen`
2. **Read prompt**: `03-agent-prompts/ws##-{screen}-screen.md`
3. **Implement**:
   - Create render system: `rust/src/bevy_app/systems/render/screens/{screen}.rs`
   - Create input handler: `rust/src/bevy_app/systems/input/screens/{screen}.rs`
   - Register systems in `plugins.rs` (alphabetically)
4. **Test**:
   - Write unit tests (>75% coverage)
   - Write integration tests (screen transitions)
   - Run `just ci` (fmt, lint, test all pass)
5. **Quality gate**:
   - Complete checklist: `06-quality-gate-checklist.md`
   - Visual parity verified
   - Performance measured (<16ms)
6. **Create PR**:
   - Title: `WS-##: Migrate {Screen} Screen to Bevy ECS`
   - Description: Use template from `05-merge-strategy.md`
   - Rebase onto latest main before PR
7. **Merge**:
   - Studio producer reviews and merges
   - Agent rebases other workstream branch onto latest main

---

## Studio Producer Responsibilities

### Pre-Launch
- [ ] Verify WS-06 complete and stable
- [ ] Execute branch creation script
- [ ] Review all launch materials
- [ ] Provision agent instances
- [ ] Initialize progress dashboard

### Launch Day
- [ ] Launch agents (staggered 15-minute intervals)
- [ ] Monitor initial progress (first 4 hours critical)
- [ ] Respond to blockers (<1 hour response time)
- [ ] Update progress dashboard

### Execution (Days 2-5)
- [ ] Daily standups (facilitate, capture blockers)
- [ ] Review PRs (target: <2 hours for simple, <4 hours for complex)
- [ ] Coordinate merge queue (stagger 30-minute intervals)
- [ ] Post-merge validation (verify no regressions)
- [ ] Update progress dashboard daily

### Completion
- [ ] Final integration test (all 8 screens)
- [ ] Announce M3 completion
- [ ] Facilitate retrospective (Day 6)
- [ ] Document lessons learned

---

## Key Metrics (Track in Dashboard)

### Timeline Metrics
- Launch date (Day 1)
- First screen merged (expected: Day 2-3)
- Track D complete (expected: Day 3)
- Track B/C complete (expected: Day 4)
- Track A complete (expected: Day 5)
- M3 completion date (target: ≤Day 5, acceptable: Day 6)

### Quality Metrics
- Screens merged: X/8 (100% target)
- Average test coverage: X% (>75% target)
- Average frame time: Xms (<16ms target)
- Conflicts encountered: X (<1 target)
- Rollbacks performed: X (0 target)

### Coordination Metrics
- Blockers reported: X
- Average blocker resolution time: Xh (<2h target)
- Agent questions: X (expected: 5-10 total)
- Merge conflicts: X (0-1 expected)

---

## Emergency Contacts

### Critical Issues
**Who**: Studio producer
**When**: Critical blockers (WS-06 regression, multiple agents unavailable)
**Response Time**: <1 hour

### Technical Questions
**Who**: Studio producer
**When**: Pattern unclear, implementation questions
**Response Time**: <1 hour during work hours

### Merge Conflicts
**Who**: Studio producer
**When**: Non-trivial conflicts, agent uncertain
**Response Time**: <2 hours

---

## Appendix: Quick Command Reference

```bash
# Pre-launch
git pull origin main
bash docs/orchestration/tui-modernization/m3-launch/01-branch-creation.sh

# Development (per agent)
git checkout tui-modernization/ws##-{screen}-screen
cargo build
cargo test
just fmt
just lint
just ci

# Pre-PR
git rebase main
git push --force-with-lease
# Create PR on GitHub

# Post-merge (all agents)
git checkout main
git pull origin main
git checkout tui-modernization/ws##-{screen}-screen
git rebase main
git push --force-with-lease
```

---

## FAQ

**Q: What if WS-06 hasn't merged yet?**
A: DO NOT launch M3. Wait for WS-06 to complete and merge first.

**Q: What if an agent becomes unavailable mid-workstream?**
A: See `09-risk-mitigation.md` → CR-2. Producer reassigns work to available agent from Track D (fastest track).

**Q: What if multiple PRs are ready simultaneously?**
A: Producer coordinates merge queue (see `05-merge-strategy.md`). Merge in recommended order with 30-minute spacing.

**Q: What if a screen fails quality gate?**
A: Agent addresses failing items before creating PR. If PR already created, producer requests changes.

**Q: What if frame time exceeds 16ms after merge?**
A: See `09-risk-mitigation.md` → HR-2. Producer profiles, identifies bottleneck, creates optimization PR or rollback.

---

## Status: Ready for Launch

**Checklist**:
- [x] All 9 launch documents created
- [x] Branch creation script executable
- [x] Agent prompts standardized (8 total)
- [x] Quality gate checklist defined
- [x] Merge strategy documented
- [x] Risk mitigation strategies prepared
- [x] Progress dashboard template ready
- [x] Launch plan detailed (Day 0 through Day 5)

**Blocking Dependencies**:
- [ ] WS-06 (Image Assets) merged to main

**When WS-06 Completes**: M3 can launch with **zero delays**.

---

**M3 Screen Migration Launch Materials: COMPLETE AND READY** ✅

---

**Next Steps**: When `frontend-developer` completes WS-06, immediately execute `01-branch-creation.sh` and begin M3 launch following `07-launch-plan.md`.
