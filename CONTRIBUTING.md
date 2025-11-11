# Contributing to DGX-Pixels

Thank you for your interest in contributing to DGX-Pixels! This document provides guidelines for contributing to the project.

---

## Getting Started

### Prerequisites

- **Hardware**: NVIDIA DGX-Spark (GB10 Grace Blackwell Superchip) or compatible NVIDIA GPU
- **OS**: Ubuntu 22.04 LTS (ARM64 or x86_64)
- **Software**:
  - Rust 1.70+ (for TUI development)
  - Python 3.10+
  - CUDA 13.0+
  - Docker with NVIDIA Container Toolkit
  - Nushell 0.96+ (for automation scripts)
  - Just (command runner)
  - Git and GitHub CLI (`gh`)

### Initial Setup

```bash
# Clone the repository
git clone https://github.com/raibid-labs/dgx-pixels.git
cd dgx-pixels

# Initialize project (creates directories, virtual env, etc.)
just init

# Validate your hardware
just validate-gpu

# Run tests to verify setup
just test
```

---

## Development Workflow

### 1. Agent-Based Development (Automated)

DGX-Pixels uses an **agent-based workflow** where AI agents implement workstreams. Each agent:

1. **Creates a branch** for the issue/workstream
2. **Implements the changes** following the workstream specification
3. **Runs tests** (TDD approach: write tests first!)
4. **Creates a Pull Request** with detailed summary
5. **Rebases onto latest main** before merging
6. **Auto-merges after CI passes** (squash merge)

#### Agent Workflow Commands

```nushell
# Create branch for workstream WS-01
just branch WS-01

# Or using nushell directly:
use scripts/nu/modules/github.nu *
gh-create-branch "ws01-hardware-baselines"

# After implementing changes:
gh-create-pr "Implement WS-01: Hardware Baselines" --labels [workstream, M0]

# Enable auto-merge (squash)
gh-auto-merge --merge-method squash

# Before starting next workstream, rebase onto main
gh-rebase-main
```

### 2. Manual Development (Human Contributors)

If you're contributing manually (not via agent workflow):

#### Step 1: Pick a Workstream/Issue

- Browse open issues labeled `status:ready`
- Check `docs/orchestration/workstream-plan.md` for available workstreams
- Ensure you have the required skills (Rust, Python, ML, DevOps)
- Verify dependencies are met (blocked workstreams can't start yet)

#### Step 2: Create a Branch

```bash
# Standard naming: workstream ID or feature description
git checkout -b ws01-hardware-baselines

# Or for bug fixes/features
git checkout -b fix/gpu-detection
git checkout -b feature/sixel-preview
```

#### Step 3: Implement Using TDD

**Test-Driven Development (TDD) is required:**

```bash
# 1. Write tests FIRST (they will fail - that's expected!)
# For Rust:
touch rust/tests/ws01_hardware_baselines.rs
# Write tests...

# For Python:
touch python/tests/test_ws01_hardware_baselines.py
# Write tests...

# 2. Commit the failing tests
git add tests/
git commit -m "test: add tests for WS-01 hardware baselines"

# 3. Implement the functionality to make tests pass
# Write code...

# 4. Verify tests pass
just test

# 5. Commit implementation
git add src/
git commit -m "feat(ws01): implement hardware baseline detection"
```

#### Step 4: Code Quality Checks

```bash
# Run all quality checks
just ci

# Or individually:
just fmt          # Format code
just lint         # Run linters (clippy for Rust)
just test         # Run all tests
just test-coverage # Check coverage (aim for ≥80%)
```

#### Step 5: Create Pull Request

```bash
# Push your branch
git push -u origin ws01-hardware-baselines

# Create PR using GitHub CLI
gh pr create \
  --title "feat(WS-01): Implement hardware baselines" \
  --body "$(cat <<EOF
## Summary
Implements WS-01: Hardware Baselines

## Changes
- Added hardware detection script (nushell)
- Created baseline JSON export
- Updated docs/hardware.md with actual measurements
- Added tests for all detection functions

## Workstream
WS-01: Hardware Baselines (Foundation domain)

## Acceptance Criteria
- [x] Hardware verification script exists and runs successfully
- [x] Baseline JSON recorded in \`bench/baselines/\`
- [x] docs/hardware.md updated with actual measurements
- [x] All hardware specs verified: GB10, 128GB unified, ARM CPU

## Test Results
\`\`\`
cargo test --workspace
...all tests passing...

pytest python/tests/
...all tests passing...
\`\`\`

## Dependencies
- None (first workstream, blocks all others)

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)" \
  --label "workstream,M0,foundation"
```

#### Step 6: Address Review Feedback

```bash
# Make changes based on review
# Commit and push

git add .
git commit -m "fix: address review feedback"
git push
```

#### Step 7: Rebase Before Merge

**Always rebase onto latest main before merging:**

```bash
# Update main
git checkout main
git pull origin main

# Rebase your branch
git checkout ws01-hardware-baselines
git rebase main

# Force push (rebased commits)
git push --force-with-lease
```

#### Step 8: Merge (Squash)

Once CI passes and PR is approved:
- Use **Squash and Merge** (keeps history clean)
- Delete branch after merge
- Start next workstream

---

## Commit Message Guidelines

Follow [Conventional Commits](https://www.conventionalcommits.org/):

### Format

```
<type>(<scope>): <subject>

[optional body]

[optional footer]
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Add or update tests
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `chore`: Build process, dependencies, tooling
- `ci`: CI/CD changes

### Scopes

- `ws01`, `ws02`, etc.: Workstream-specific
- `tui`: Rust TUI components
- `backend`: Python backend worker
- `comfyui`: ComfyUI integration
- `training`: LoRA training pipeline
- `bevy`: Bevy integration
- `docs`: Documentation
- `ci`: CI/CD configuration

### Examples

```
feat(ws01): implement hardware baseline detection

- Add nushell script for GPU detection
- Export baseline metrics to JSON
- Update hardware.md with verified specs

Closes #PIXELS-001
```

```
test(backend): add tests for ZeroMQ communication

- Test REQ-REP pattern
- Test PUB-SUB pattern
- Test error handling and reconnection
```

```
fix(tui): resolve Sixel rendering on ARM architecture

The Sixel library had issues on ARM. Switched to a pure Rust
implementation that works across architectures.

Fixes #PIXELS-035
```

---

## Code Style

### Rust

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Run `cargo fmt` before committing
- Run `cargo clippy -- -D warnings` (no warnings allowed)
- Document public APIs with doc comments
- Write tests for all public functions

### Python

- Follow [PEP 8](https://peps.python.org/pep-0008/)
- Use type hints (Python 3.10+ syntax)
- Run `ruff format` or `black` before committing
- Docstrings for all functions/classes (Google style)
- Write tests using pytest

### Nushell

- Follow examples in `scripts/nu/config.nu`
- Use header template for all scripts
- Export functions that other modules might use
- Add doc comments for all exported functions
- Use consistent logging (log-success, log-error, etc.)

---

## Testing Requirements

### Minimum Coverage

- **Overall**: ≥80% code coverage
- **Critical paths**: 100% coverage (GPU detection, model loading, API endpoints)
- **New features**: Must include tests

### Test Categories

1. **Unit Tests**: Test individual functions/modules
2. **Integration Tests**: Test component interactions
3. **Performance Tests**: Verify speed/latency targets
4. **End-to-End Tests**: Full workflow validation

### Running Tests

```bash
# All tests
just test

# With coverage
just test-coverage

# Integration tests only
just test-integration

# Performance benchmarks
just bench
```

---

## Documentation Requirements

### Required Documentation

Every workstream must include:

1. **Code Documentation**:
   - Rust: Doc comments (`///`) for all public APIs
   - Python: Docstrings for all functions/classes
   - Nushell: Export comments for all functions

2. **Usage Examples**:
   - In-code examples
   - `examples/` directory for complete examples
   - README in each major component directory

3. **Completion Summary**:
   - Create `docs/orchestration/workstreams/wsXX-name/COMPLETION_SUMMARY.md`
   - Include: deliverables, test results, known limitations, next steps

### Updating Documentation

```bash
# Generate Rust docs
just docs

# Serve docs locally
just docs-serve

# Update relevant markdown files in docs/
```

---

## Workstream Implementation

### Workstream Structure

Each workstream has:
- Specification: `docs/orchestration/workstreams/wsXX-name/README.md`
- Deliverables: Files/components to create
- Acceptance Criteria: Testable requirements
- Dependencies: What must complete first

### Implementation Steps

1. **Read the Spec**: Thoroughly review workstream README
2. **Verify Dependencies**: Ensure blocking workstreams are complete
3. **Plan Implementation**: Break down into phases (Foundation → Core → Testing)
4. **Write Tests First**: TDD approach
5. **Implement**: Follow the spec's technical requirements
6. **Verify**: Run all verification commands from spec
7. **Document**: Create completion summary
8. **Submit PR**: Use template with acceptance criteria checklist

---

## Pull Request Guidelines

### PR Title Format

```
<type>(<scope>): <description>

Examples:
- feat(WS-01): Implement hardware baselines
- fix(tui): Resolve Sixel rendering on ARM
- docs(orchestration): Update workstream dependencies
```

### PR Description Template

```markdown
## Summary
One paragraph describing the changes.

## Workstream
WS-XX: Workstream Name (Domain)

## Changes
- Bullet point list of changes
- Be specific

## Acceptance Criteria
- [x] Criterion 1
- [x] Criterion 2
- [ ] Criterion 3 (if any remain)

## Test Results
\`\`\`
Paste test output here
\`\`\`

## Dependencies
- Depends on: #PIXELS-XXX
- Blocks: #PIXELS-YYY

## Screenshots/Demos
(If applicable)

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

### PR Checklist

Before submitting:
- [ ] Code follows style guidelines
- [ ] Tests added/updated and passing
- [ ] Coverage ≥80%
- [ ] Documentation updated
- [ ] Commit messages follow conventions
- [ ] No merge conflicts
- [ ] CI checks passing

---

## CI/CD Pipeline

### Automated Checks

Every PR runs:
1. **Rust Tests**: `cargo test --workspace`
2. **Rust Linting**: `cargo clippy -- -D warnings`
3. **Rust Formatting**: `cargo fmt --check`
4. **Python Tests**: `pytest python/tests/`
5. **Build Verification**: `cargo build --release`

### Local Pre-Commit

Run before pushing:
```bash
just pre-commit
```

---

## Issue Labels

### Status Labels
- `status:draft` - Issue needs more definition
- `status:ready` - Ready for implementation
- `status:in-progress` - Agent/developer working on it
- `status:review` - PR submitted, needs review
- `status:completed` - Done and merged

### Priority Labels
- `priority:P0` - Critical path, blocks other work
- `priority:P1` - High priority
- `priority:P2` - Nice to have

### Domain Labels
- `domain:foundation` - M0 workstreams
- `domain:model` - M1/M3 workstreams
- `domain:interface` - M2 workstreams
- `domain:integration` - M4/M5 workstreams

### Type Labels
- `workstream` - Part of orchestrated workstream plan
- `bug` - Bug fix
- `enhancement` - New feature
- `documentation` - Documentation changes

---

## Questions?

- **Documentation**: See `docs/` directory
- **Workstreams**: See `docs/orchestration/workstream-plan.md`
- **Orchestration**: See `docs/orchestration/meta-orchestrator.md`
- **Architecture**: See `docs/02-architecture-proposals.md`
- **Hardware**: See `docs/hardware.md`

For questions not covered in documentation:
- Open a GitHub Discussion
- Check existing issues for similar questions

---

**Thank you for contributing to DGX-Pixels!** 🎨🤖
