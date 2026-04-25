## Memory

No project memory available.

## Phase Plan

Two plan files found for phase-6:

### plans/phase-6/PHASE_PLAN.md (high-level plan)

Phase 6: Reliability, Multi-Parameter Patterns, and Ergonomics
- **Goal 1: CollectFailurePolicy** - Fix correctness bug where collect-closure failures are silently ignored. Add `CollectFailurePolicy` enum (FailTask/WarnOnly), reorder `process_finished()` to run collect before marking completed.
- **Goal 2: Multi-Parameter Sweep** - Build/test multi-parameter sweeps (product + pairwise modes), run on HPC cluster, document findings. Uses `itertools::iproduct!` and `.iter().zip()`. No new framework types needed.
- **Goal 3: --workdir / root_dir Support** - Allow workflow binary from any directory. Add `root_dir` field to `Workflow`, resolve relative workdirs at dispatch time.
- **Goal 4: workflow-cli retry stdin support** - Accept task IDs from stdin for Unix pipeline composition. Detect pipe vs TTY, support `-` for explicit stdin.
- **Goal 5: Documentation Accuracy Sweep** - Fix 6 known doc-vs-code mismatches: closure signatures, JsonStateStore constructors, ARCHITECTURE_STATUS stale entries, weak test assertion, trailing newline, clippy warnings.

**Sequencing:** Goal 1 -> Goal 3 -> Goal 4 -> Goal 2 -> Goal 5
**Out of scope:** Typed result collection, portable SLURM template, TaskChain abstraction, framework-level sweep builder, --match glob pattern, Tier 2 interactive CLI.

### plans/phase-6/phase6_implementation.toml (task breakdown)

6 tasks with dependency chain:
- TASK-1: CollectFailurePolicy enum + field wiring (no deps)
- TASK-2: root_dir / --workdir support (depends on TASK-1)
- TASK-3: Wire collect_failure_policy into process_finished + integration tests (depends on TASK-1, TASK-2)
- TASK-4: retry stdin support (no deps)
- TASK-5: Multi-Parameter Sweep - itertools + extended example (depends on TASK-4)
- TASK-6: Documentation accuracy sweep + clippy (depends on TASK-5)

## Snapshot

No snapshot — using raw diff from raw-diff.md.
