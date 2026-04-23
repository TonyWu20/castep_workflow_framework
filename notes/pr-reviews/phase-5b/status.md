# Branch Status: `phase-5b` — 2026-04-23 (v2 review)

## Baseline

- **Baseline commit**: `68da603` review(phase-5b): update fix plan
- **Current HEAD**: `040bc83` review(phase-5b): update fix plan (v2)
- **Commits ahead of baseline**: 7

## Commits Since Baseline (v1 fix round)

| # | Commit | Description |
|---|--------|-------------|
| 1 | `501970b` | TASK-1: Fix 6 test call sites in state.rs using `.into()` with `downstream_of` |
| 2 | `b75fde9` | TASK-2: Remove unused `use std::collections::HashMap` from task.rs test module |
| 3 | `27cda16` | TASK-3: Declare `pub mod prelude` in workflow_core/src/lib.rs |
| 4 | `d17394b` | TASK-4: Create workflow_utils/src/prelude.rs and register it in lib.rs |
| 5 | `42268d2` | TASK-5: Fix remaining uninlined_format_args clippy warnings |
| 6 | `7499bb0` | fix: correct empty slice type annotation for downstream_of test |
| 7 | `040bc83` | review(phase-5b): update fix plan (v2) |

## Build / Test / Clippy (post v1 fix round)

- **cargo check**: Passed (workspace, 0 errors)
- **cargo test**: Passed (108 passed, 1 ignored)
- **cargo clippy (pedantic)**: FAILS — 1 error (`approx_constant` in config.rs:96), many pedantic warnings

## Outstanding Issues (v2 fix plan)

| TASK | Severity | Description |
|------|----------|-------------|
| TASK-1 | Blocking | `2.71828` triggers `approx_constant` for E — change to `42.0` |
| TASK-2 | Blocking | ARCHITECTURE.md / ARCHITECTURE_STATUS.md not updated |
| TASK-3 | Major | `uninlined_format_args` in config.rs test module lines 102, 108 |
| TASK-4 | Major | `doc_markdown` on prelude.rs:1, missing trailing newline |
| TASK-5 | Minor | Examples not using prelude imports |
| TASK-6 | Minor | `run_default()` not used in `hubbard_u_sweep_slurm` local mode |
| TASK-7 | Minor | `uninlined_format_args` in `workflow_core/src/lib.rs:31` |

## Files Changed Since Last Review

### Source code (v1 fixes applied)
- `workflow_core/src/lib.rs` — added `pub mod prelude`
- `workflow_core/src/state.rs` — fixed 6 `.into()` call sites for `downstream_of`
- `workflow_core/src/task.rs` — removed unused `HashMap` import from test module
- `workflow_utils/src/prelude.rs` — new file (re-exports)
- `workflow_utils/src/lib.rs` — registered prelude
- `examples/hubbard_u_sweep/src/main.rs` — clippy fix (uninlined_format_args)
- `examples/hubbard_u_sweep_slurm/src/config.rs` — 3.14→2.71828 (still broken), job_script.rs clippy fix
- `examples/hubbard_u_sweep_slurm/src/job_script.rs` — clippy fix

### Notes / review
- `notes/pr-reviews/phase-5b/fix-plan.toml` — v2 fix plan (this review round)
- `notes/pr-reviews/phase-5b/status.md` — this file
