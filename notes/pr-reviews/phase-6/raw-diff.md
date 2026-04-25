# Phase 6 PR Raw Diff Data

**Branch:** phase-6
**Base:** main
**Date:** 2026-04-25

## Commits

```
46ed39a chore(phase-6): add execution report and remove compiled artifacts
b045ecd feat(phase6-implementation): TASK-6: Update ARCHITECTURE.md/ARCHITECTURE_STATUS.md, fix config assertion, trailing newline, and clippy
9bc4705 feat(phase6-implementation): TASK-5: Add itertools and extend hubbard_u_sweep_slurm with product/pairwise sweep modes
7b45cea feat(phase6-implementation): TASK-3: Wire collect_failure_policy into process_finished; add integration tests
4975f6c feat(phase6-implementation): TASK-2: Add root_dir field and builder to Workflow; resolve relative workdirs at dispatch
676889c feat(phase6-implementation): TASK-1: Add CollectFailurePolicy enum, field on Task/InFlightTask, and re-exports
2b738c7 Revised phase6 implementation toml
0936c09 chore(deferred): consolidate stale deferred items into phase-6/deferred.md
dbb981b plan-review(PHASE_PLAN): architectural review and deferred item decisions
dc6442d plan(phase-6): initial phase plan -- Reliability, Multi-Parameter Patterns, and Ergonomics
```

## Diff Stat

```
 ARCHITECTURE.md                                    |   22 +-
 ARCHITECTURE_STATUS.md                             |   20 +-
 Cargo.lock                                         |   16 +
 Cargo.toml                                         |    1 +
 examples/hubbard_u_sweep_slurm/Cargo.toml          |    1 +
 examples/hubbard_u_sweep_slurm/src/config.rs       |   14 +-
 examples/hubbard_u_sweep_slurm/src/main.rs         |   85 +-
 workflow_core/.checkpoint_phase6-implementation.json |   13 +
 workflow_core/execution_report/execution_phase6-implementation_20260425.md |   45 +
 workflow_core/execution_report/execution_phase6_implementation_20260425.md |  103 ++
 flake.nix                                          |    8 +-
 notes/plan-reviews/PHASE_PLAN/decisions.md         |  118 ++
 notes/pr-reviews/phase-4/deferred.md               |   25 -
 notes/pr-reviews/phase-5/deferred.md               |   96 --
 notes/pr-reviews/phase-5b/deferred.md              |   39 -
 notes/pr-reviews/phase-6/deferred.md               |   30 +
 plans/phase-6/PHASE_PLAN.md                        |  211 +++
 plans/phase-6/phase6_implementation.toml           | 1618 ++++++++++++++++++++
 workflow-cli/src/main.rs                           |   53 +-
 workflow_core/src/lib.rs                           |    2 +-
 workflow_core/src/prelude.rs                       |    2 +-
 workflow_core/src/task.rs                          |   22 +
 workflow_core/src/workflow.rs                      |   93 +-
 workflow_core/tests/collect_failure_policy.rs      |  153 ++
 workflow_core/tests/hook_recording.rs              |    3 +-
 25 files changed, 2575 insertions(+), 218 deletions(-)
```

## File: ARCHITECTURE.md

Changes to `impl Task` section: `setup` and `collect` builder signatures updated from single generic `F` returning `Result<(), WorkflowError>` to two generics `<F, E>` where `E: std::error::Error + Send + Sync + 'static`. Added trailing periods to doc comments.

Changes to `impl JsonStateStore` section: `load` changed from instance method `&mut self` to static factory `path: impl AsRef<Path>` returning `Result<Self, WorkflowError>`. `load_raw` similarly changed from `&self` instance method to static factory with same signature. `new` retains `impl Into<String>` signature.

## File: ARCHITECTURE_STATUS.md

Phase 5 section: `TaskClosure` type alias description updated from `WorkflowError` return to `Box<dyn std::error::Error + Send + Sync>` return. Added `CollectFailurePolicy` entry. Added `CollectFailurePolicy` re-export entry.

Phase 6 section: New section added describing `CollectFailurePolicy`, multi-parameter sweep, `--workdir`/root_dir, retry stdin support, and documentation accuracy sweep.

Next Steps: Updated to reflect Phase 6 completion status and restructured future work entries.

## File: Cargo.lock

Added `itertools` 0.14.0 and its dependency `either` 1.15.0. Added `itertools` to `hubbard_u_sweep_slurm` dependencies.

## File: Cargo.toml

Added `itertools = "0.14"` to workspace dependencies.

## File: examples/hubbard_u_sweep_slurm/Cargo.toml

Added `itertools = { workspace = true }` dependency. Removed trailing newline.

## File: examples/hubbard_u_sweep_slurm/src/config.rs

Added three new CLI fields to `SweepConfig`: `sweep_mode` (String, default "single"), `second_values` (Option<String>), `workdir` (String, default ".").

Test `parse_empty_string` assertion changed from `!err.is_empty()` to `err.contains("invalid")` with explanatory message.

## File: examples/hubbard_u_sweep_slurm/src/main.rs

`build_one_task` gained `second: &str` parameter; task ID and workdir now include second param in naming (`scf_U{u:.1}_{second}`, `runs/U{u:.1}/{second}`).

New `build_chain` function: builds SCF + DOS task pairs with dependency wiring. DOS task placeholder (no setup/collect closures).

New `parse_second_values` helper: parses comma-separated string labels.

`build_sweep_tasks` refactored from simple iterator to match on `sweep_mode`: "product" uses `iproduct!`, "pairwise" uses `.zip()`, "single" passes "default" as second param.

`main`: added `.with_root_dir(&config.workdir)` to workflow builder. File ends with trailing newline.

## File: workflow_core/.checkpoint_phase6-implementation.json

New file: JSON checkpoint tracking TASK-1 through TASK-6 completion. All tasks completed, none failed or blocked.

## File: workflow_core/execution_report/execution_phase6-implementation_20260425.md

New file: In-progress execution report showing TASK-1 through TASK-6 all passed cargo check/clippy validation.

## File: workflow_core/execution_report/execution_phase6_implementation_20260425.md

New file: Completed execution report with full details for all 6 tasks, global clippy verification, and summary.

## File: flake.nix

Changed `ANTHROPIC_BASE_URL` from `localhost:8001` to `10.0.0.3:4000`. Updated model names: `opus`/`sonnet` → `qwen3.6-apex-think`, `haiku` → `qwen3.6-apex`.

## File: notes/plan-reviews/PHASE_PLAN/decisions.md

New file: 118-line plan review decisions document. Covers design assessment, 21 deferred item decisions (close/defer/absorb), and 6 plan amendments (InFlightTask changes, file path correction, resolution semantics, clap argument change, whitespace artifact absorption, pedantic clippy item).

## File: notes/pr-reviews/phase-4/deferred.md

Deleted file (25 lines removed). All deferred items from phase-4 were absorbed into phase-6 plan or closed.

## File: notes/pr-reviews/phase-5/deferred.md

Deleted file (96 lines removed). All deferred items from phase-5 were absorbed into phase-6 plan or closed.

## File: notes/pr-reviews/phase-5b/deferred.md

Deleted file (39 lines removed). All deferred items from phase-5b were absorbed into phase-6 plan or closed.

## File: notes/pr-reviews/phase-6/deferred.md

New file (30 lines): Consolidated deferred items for phase-6. Only D.1 (portable config fields), D.2 (job script formatting), and D.3 (generate_job_script tests) carried forward. Rationale and preconditions documented.

## File: plans/phase-6/PHASE_PLAN.md

New file (211 lines): Phase 6 plan document with 5 goals (CollectFailurePolicy, Multi-Parameter Sweep, root_dir, retry stdin, documentation sweep), scope boundaries, design notes, deferred items table, sequencing, and verification criteria.

## File: plans/phase-6/phase6_implementation.toml

New file (1618 lines): Detailed implementation plan with before/after code blocks for all tasks, dependency ordering, and acceptance criteria.

## File: workflow-cli/src/main.rs

Added `use std::io::{self, IsTerminal, Read}`.

`Retry` command: `task_ids` changed from `#[arg(required = true)]` to `#[arg(required = false, default_value = "-")]`.

New `read_task_ids` function: resolves task IDs from CLI args or stdin. Handles `"-"` prefix, empty vec with TTY (error), empty vec with pipe (read stdin), and regular args pass-through.

`main` Retry handler: calls `read_task_ids` before passing to `cmd_retry`.

Tests: added `read_task_ids_from_vec` and `read_task_ids_dash_empty_stdin_errors`.

## File: workflow_core/src/lib.rs

Added `CollectFailurePolicy` to the `pub use task::` re-export line.

## File: workflow_core/src/prelude.rs

Added `CollectFailurePolicy` to the `pub use crate::task::` re-export line. File retains no trailing newline.

## File: workflow_core/src/task.rs

New `CollectFailurePolicy` enum (Debug, Clone, Copy, Default, PartialEq, Eq) with `FailTask` (default) and `WarnOnly` variants. Full doc comment.

`Task` struct: added `pub(crate) collect_failure_policy: CollectFailurePolicy` field.

`Task::new`: initializes `collect_failure_policy` to default.

New `Task::collect_failure_policy` builder method.

## File: workflow_core/src/workflow.rs

`InFlightTask` struct: added `pub collect_failure_policy: crate::task::CollectFailurePolicy` field.

`Workflow` struct: added `root_dir: Option<std::path::PathBuf>` field.

`Workflow::new`: initializes `root_dir` to None.

New `Workflow::with_root_dir` builder method.

`run()` method: resolved log dir against root_dir early (lines 123-135). In dispatch loop, workdir resolved against root_dir (lines 234-240). All task execution paths (Direct, Queued, setup, hooks, InFlightTask construction) use resolved_workdir instead of task.workdir. `InFlightTask` construction populates `collect_failure_policy` from task.

`process_finished()` function fully rewritten: changed from early `mark_completed` + warn pattern to `(exit_ok, exit_code)` tuple. On exit 0, runs collect before deciding phase. On collect failure with `FailTask`, calls `mark_failed`. Re-reads state to determine final phase. Preserves `WarnOnly` backward compatibility.

## File: workflow_core/tests/collect_failure_policy.rs

New file (153 lines): Integration test file with `StubRunner`, `StubHandle`, `StubHookExecutor` test doubles. Two tests: `collect_failure_with_failtask_marks_failed` (verifies task is Failed when collect closure errors with FailTask policy), `collect_failure_with_warnonly_marks_completed` (verifies task is Completed when collect closure errors with WarnOnly policy).

## File: workflow_core/tests/hook_recording.rs

Added `CollectFailurePolicy` import. Updated `collect_failure_does_not_fail_task` test: added `.collect_failure_policy(CollectFailurePolicy::WarnOnly)` to task builder (previously relied on default, which is now `FailTask` -- the test behavior is preserved explicitly).
