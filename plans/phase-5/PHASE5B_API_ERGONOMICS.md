# Phase 5B: API Ergonomics & Deferred Items

**Date:** 2026-04-23
**Status:** Draft (revised after Phase 5 PR review)
**Precondition:** Phase 5A deployed and merged to main

## Context

Phase 5A (production sweep) validated the engine and surfaced friction. This plan absorbs:
- Four deferred items from the Phase 4 PR review (B.4.x)
- Eleven deferred items from the Phase 5 PR review (D.1–D.11)

No new capabilities. Only ergonomic improvement, cleanup, and debt reduction.

---

## Goals

1. **`downstream_of` signature ergonomics** — accept `&[impl AsRef<str>]` instead of `&[String]` (B.4.2)
2. **`ExecutionMode::direct()` constructor** — reduce Direct-mode boilerplate in binaries (B.1)
3. **`parse_u_values` cleanup** — extract `let trimmed`, add unit tests (D.8, D.3)
4. **`generate_job_script` formatting** — remove literal `\t`, fix inconsistent quoting, add unit tests (D.2, D.3)
5. **`main.rs` restructuring + `--local` flag + iterator-based sweep** — extract `build_sweep_tasks`, convert for-loop to iterator chain, add `--local`/`--mode` CLI flag (D.10, D.11, B.6, D.9)
6. **`run_default()` helper** — eliminate repeated Arc wiring in binaries (B.3)
7. **Prelude modules** — collapse import sprawl for binary authors (B.2)
8. **Pedantic clippy cleanup** — fix 8 `uninlined_format_args` + 1 `doc_markdown` (D.5)
9. **Update stale documentation** — ARCHITECTURE.md + ARCHITECTURE_STATUS.md + implementation guidelines (B.5, B.4.3)

---

## Sequencing

```
Step 1:  B.4.2  downstream_of signature              (workflow_core — isolated)
Step 2:  B.1    ExecutionMode::direct()               (workflow_core — isolated)
Step 3:  D.8    Extract let trimmed in parse_u_values (example/config.rs)
Step 4:  D.3a   Unit tests for parse_u_values         (example/config.rs)
Step 5:  D.2    generate_job_script formatting        (example/job_script.rs)
Step 6:  D.3b   Unit tests for generate_job_script    (example/job_script.rs)
Step 7:  D.10+  Extract build_sweep_tasks, iterator   (example/main.rs — centerpiece)
           D.11   sweep, --local flag, anyhow fix
           B.6
           D.9
Step 8:  B.3    run_default()                         (workflow_utils/src/lib.rs)
Step 9:  B.2    Prelude modules                       (workflow_core + workflow_utils)
Step 10: D.5    Pedantic clippy cleanup               (workspace-wide)
Step 11: B.5    ARCHITECTURE.md + STATUS.md           (docs last)
           B.4.3  Implementation guidelines
```

---

## B.4.2 `downstream_of` Signature Ergonomics

**File:** `workflow_core/src/state.rs`

Change signature from:
```rust
pub fn downstream_of(&self, start: &[String]) -> Vec<String>
```
to:
```rust
pub fn downstream_of<S: AsRef<str>>(&self, start: &[S]) -> Vec<String>
```

Update internal BFS to use `s.as_ref().to_owned()` where it currently calls `.to_owned()` on `String`. Allows callers to pass `&[&str]` or `&[String]` without pre-allocating owned strings.

---

## B.1 `ExecutionMode::direct()` Convenience Constructor

**File:** `workflow_core/src/task.rs`

Add to `impl ExecutionMode`:

```rust
impl ExecutionMode {
    /// Convenience constructor for Direct mode with no environment overrides and no timeout.
    pub fn direct(command: impl Into<String>, args: &[&str]) -> Self {
        ExecutionMode::Direct {
            command: command.into(),
            args: args.iter().map(|s| s.to_string()).collect(),
            env: std::collections::HashMap::new(),
            timeout: None,
        }
    }
}
```

Before:
```rust
ExecutionMode::Direct {
    command: "castep".into(),
    args: vec!["ZnO".into()],
    env: HashMap::new(),
    timeout: None,
}
```
After:
```rust
ExecutionMode::direct("castep", &["ZnO"])
```

---

## D.8 + D.3a: `parse_u_values` Cleanup and Tests

**File:** `examples/hubbard_u_sweep_slurm/src/config.rs`

**D.8:** Extract the double `s.trim()` call:

Before:
```rust
|s| s.trim().parse::<f64>().map_err(|_| format!("cannot parse '{}' as f64", s.trim()))
```
After:
```rust
|s| {
    let trimmed = s.trim();
    trimmed.parse::<f64>().map_err(|_| format!("cannot parse '{trimmed}' as f64"))
}
```

**D.3a:** Add `#[cfg(test)]` module covering:
- Normal input: `"1.0,2.0,3.0"` → `[1.0, 2.0, 3.0]`
- Empty string: `""` → `Err(...)`
- Trailing comma: `"1.0,2.0,"` → `Err(...)` (trailing empty token fails parse)
- Whitespace-padded: `" 1.0 , 2.0 "` → `[1.0, 2.0]`
- Non-numeric: `"1.0,abc,3.0"` → `Err(...)`
- Negative values: `"-1.0,2.0"` → `[-1.0, 2.0]`

---

## D.2 + D.3b: `generate_job_script` Formatting and Tests

**File:** `examples/hubbard_u_sweep_slurm/src/job_script.rs`

**D.2:** Fix formatting issues:
- Replace the literal `\t` character in the `--map-by` flag line with consistent spaces
- Standardise SBATCH directive quoting (either always quote values, or never)
- Consider using the `indoc!` macro (from the `indoc` crate) or a raw string literal `r#"..."#` for the template body to make the structure visually clear

**D.3b:** Add `#[cfg(test)]` module for `generate_job_script` covering:
- All expected SBATCH directives appear in output (`--job-name`, `--partition`, `--nodes`, etc.)
- The `mpirun` invocation contains the correct binary and seed name
- The job name and seed name are correctly substituted

---

## D.10 + D.11 + B.6 + D.9: `main.rs` Restructuring

**File:** `examples/hubbard_u_sweep_slurm/src/main.rs`

### D.10: Extract `build_sweep_tasks`

Extract a function that separates task construction from workflow orchestration:

```rust
fn build_sweep_tasks(config: &SweepConfig) -> anyhow::Result<Vec<Task>> {
    let u_values = parse_u_values(&config.u_values)?;
    u_values
        .into_iter()
        .map(|u| build_one_task(config, u))
        .collect()
}

fn build_one_task(config: &SweepConfig, u: f64) -> anyhow::Result<Task> {
    // construct cell document, job script, ExecutionMode, Task
    // ...
}
```

`fn main()` becomes:
```rust
fn main() -> anyhow::Result<()> {
    let config = SweepConfig::parse();
    let tasks = build_sweep_tasks(&config)?;
    // wire workflow, run
}
```

### D.11: Iterator-based sweep

Replace the `for u in &u_values` loop with an iterator chain in `build_sweep_tasks`. This makes future multi-parameter sweeps straightforward with `flat_map`.

### B.6: `--local` flag

Add to `SweepConfig`:
```rust
/// Run locally using Direct execution instead of submitting to SLURM
#[arg(long, default_value_t = false)]
pub local: bool,
```

Inside `build_one_task`, choose mode:
```rust
let mode = if config.local {
    ExecutionMode::direct(&config.castep_command, &[&config.seed_name])
} else {
    ExecutionMode::Queued
};
```

Conditionally attach the submitter in `main`:
```rust
if !config.local {
    workflow = workflow.with_queued_submitter(Arc::new(QueuedRunner::new(SchedulerKind::Slurm)));
}
```

In local mode, skip writing `job.sh` in the setup closure.

### D.9: `anyhow::Error::msg` at call site

**File:** `examples/hubbard_u_sweep_slurm/src/main.rs`

Replace:
```rust
.map_err(|e| anyhow::anyhow!(e))
```
with:
```rust
.map_err(anyhow::Error::msg)
```

(The `anyhow!` macro is for format strings; `Error::msg` is the idiomatic wrapper for an existing `Display` value.)

---

## B.3 `run_default()` Convenience Function

**File:** `workflow_utils/src/lib.rs` (or `workflow_utils/src/runner.rs` re-exported from lib.rs)

```rust
/// Runs a workflow with the default SystemProcessRunner and ShellHookExecutor.
///
/// Equivalent to calling `workflow.run(state, Arc::new(SystemProcessRunner::new()), Arc::new(ShellHookExecutor))`.
pub fn run_default(
    workflow: &mut Workflow,
    state: &mut dyn StateStore,
) -> Result<WorkflowSummary, WorkflowError> {
    let runner: Arc<dyn ProcessRunner> = Arc::new(SystemProcessRunner::new());
    let executor: Arc<dyn HookExecutor> = Arc::new(ShellHookExecutor);
    workflow.run(state, runner, executor)
}
```

Before (every binary):
```rust
let runner: Arc<dyn ProcessRunner> = Arc::new(SystemProcessRunner::new());
let executor: Arc<dyn HookExecutor> = Arc::new(ShellHookExecutor);
workflow.run(&mut state, runner, executor)?;
```
After:
```rust
workflow_utils::run_default(&mut workflow, &mut state)?;
```

---

## B.2 Prelude Modules

Build after all public API additions are finalised.

**New file:** `workflow_core/src/prelude.rs`

```rust
//! Convenience re-exports for the most commonly needed workflow_core types.
pub use crate::error::WorkflowError;
pub use crate::process::{ProcessHandle, ProcessRunner, QueuedSubmitter};
pub use crate::state::{JsonStateStore, StateStore, StateStoreExt, TaskStatus, TaskSuccessors};
pub use crate::task::{ExecutionMode, Task, TaskClosure};
pub use crate::workflow::{FailedTask, Workflow, WorkflowSummary};
pub use crate::{HookExecutor, HookTrigger, MonitoringHook};
```

Add to `workflow_core/src/lib.rs`:
```rust
pub mod prelude;
```

**New file:** `workflow_utils/src/prelude.rs`

```rust
//! Convenience re-exports for all commonly needed types across workflow_core and workflow_utils.
pub use crate::{
    copy_file, create_dir, exists, read_file, remove_dir, run_default, write_file,
    QueuedRunner, SchedulerKind, ShellHookExecutor, SystemProcessRunner,
};
pub use workflow_core::prelude::*;
```

Add to `workflow_utils/src/lib.rs`:
```rust
pub mod prelude;
```

Usage collapses from 6 import lines to 1:
```rust
use workflow_utils::prelude::*;
```

---

## D.5 Pedantic Clippy Cleanup

**Files:** `examples/hubbard_u_sweep_slurm/src/config.rs`, `examples/hubbard_u_sweep_slurm/src/main.rs`

Fix 8 `uninlined_format_args` warnings and 1 `doc_markdown` warning from `clippy::pedantic`. These are style-only and do not affect correctness.

Run after all other code changes:
```bash
cargo clippy --workspace -- -W clippy::pedantic 2>&1 | grep "warning\["
```

---

## B.5 + B.4.3: Update Stale Documentation

Do this last so docs reflect all changes from Steps 1–10.

**File: `ARCHITECTURE.md`**
- Update "Implementation Status" to reflect Phases 3, 4, and 5 as complete
- Fix all code examples to match actual API (`Workflow::new()`, `Task::new(id, mode)`, `ExecutionMode::Queued`, `ExecutionMode::direct()`)
- Add Phase 3 components: `StateStore`, `JsonStateStore`, `load_raw()`, `WorkflowError`, `WorkflowSummary`, signal handling, `workflow-cli`
- Add Phase 4 components: log persistence, `HookTrigger::Periodic`, `ExecutionMode::Queued`, `QueuedRunner`, graph-aware retry, `TaskSuccessors`
- Add Phase 5 components: `hubbard_u_sweep_slurm` example, `run_default()`, prelude modules, `ExecutionMode::direct()`
- Add "Implementation Guidelines" section (from B.4.3):
  - **Guideline 1 — Newtype encapsulation:** Design newtypes with full encapsulation on introduction. Never expose the raw inner type via a public accessor.
  - **Guideline 2 — Domain logic placement:** Place domain logic operating on `workflow_core` types in `workflow_core` from the initial implementation.

**File: `ARCHITECTURE_STATUS.md`**
- Mark Phases 3, 4, and 5 as complete
- Add Phase 5B entry (in progress / complete)

---

## Scope Boundaries

**In scope:**
- `downstream_of` signature ergonomics (B.4.2)
- `ExecutionMode::direct()` constructor (B.1)
- `parse_u_values` cleanup and unit tests (D.8, D.3)
- `generate_job_script` formatting and unit tests (D.2, D.3)
- `main.rs` restructuring: extract `build_sweep_tasks`, iterator-based sweep, `--local` flag (D.10, D.11, B.6)
- `anyhow::Error::msg` idiomatic fix (D.9)
- `run_default()` helper in `workflow_utils` (B.3)
- Prelude modules in `workflow_core` and `workflow_utils` (B.2)
- Pedantic clippy cleanup (D.5)
- Documentation update for Phases 3–5 (B.5, B.4.3)

**Out of scope (deferred):**
- D.1: Portable SLURM config fields — design decision on job script templating strategy; no second user yet
- D.4: `std::path::absolute` for log paths — edge case (paths with `..`); low value in this phase
- D.6: `--workdir` flag — requires touching `workflow_core` (Workflow root dir); properly a Phase 6 feature
- D.7: squeue false-positive as job success — requires `CollectFailurePolicy` design in `workflow_core`; Phase 6
- `CollectFailurePolicy` — not needed for the sweep; warn-only is acceptable for now
- SLURM `script_template()` helper on `QueuedRunner` — let production runs show whether needed
- Typed result collection, convergence/iteration patterns — Phase 6

---

## Critical Files

| File | Change |
|---|---|
| `workflow_core/src/task.rs` | Add `ExecutionMode::direct()` |
| `workflow_core/src/state.rs` | Fix `downstream_of` signature |
| `workflow_core/src/prelude.rs` | New file |
| `workflow_core/src/lib.rs` | Add `pub mod prelude` |
| `workflow_utils/src/lib.rs` | Add `pub mod prelude` + `run_default()` |
| `workflow_utils/src/prelude.rs` | New file |
| `examples/hubbard_u_sweep_slurm/src/config.rs` | D.8 trim cleanup, D.3 tests |
| `examples/hubbard_u_sweep_slurm/src/job_script.rs` | D.2 formatting, D.3 tests |
| `examples/hubbard_u_sweep_slurm/src/main.rs` | D.10 extraction, D.11 iterator, B.6 --local, D.9 anyhow fix |
| `ARCHITECTURE.md` | Full update for Phases 3–5 + implementation guidelines |
| `ARCHITECTURE_STATUS.md` | Mark Phases 3–5 complete |

---

## Verification

After each step:
```
cargo check --workspace
cargo clippy --workspace
cargo test --workspace
```

Documentation changes: build with `cargo doc --workspace` and verify links.

---

## Deferred Items Absorbed (from Phase 5 review)

| Item | Absorbed into |
|---|---|
| D.2 generate_job_script formatting | Step 5 |
| D.3 unit tests (parse_u_values + job_script) | Steps 4, 6 |
| D.5 pedantic clippy | Step 10 |
| D.8 double trim() | Step 3 |
| D.9 anyhow::Error::msg | Step 7 |
| D.10 main.rs abstraction | Step 7 |
| D.11 iterator-based sweep | Step 7 |

## Deferred Items from Phase 5 Review (not yet absorbed)

| Item | Reason deferred |
|---|---|
| D.1 portable SLURM config | Templating strategy decision; no second user yet |
| D.4 std::path::absolute | Edge case; not worth touching queued.rs in an ergonomics pass |
| D.6 --workdir flag | Requires Workflow root_dir in workflow_core; Phase 6 feature |
| D.7 squeue false-positive | Requires CollectFailurePolicy design; Phase 6 |
