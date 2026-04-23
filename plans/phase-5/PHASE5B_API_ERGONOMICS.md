# Phase 5B: API Ergonomics & Deferred Items

**Date:** 2026-04-22
**Status:** Draft
**Precondition:** Phase 5A deployed and validated on cluster

## Context

Phase 5A (production sweep) validates the engine and surfaces friction. Phase 5B fixes that friction and absorbs the four deferred items from the Phase 4 PR review. No new capabilities — only ergonomic improvement and debt cleanup.

---

## Goals

1. **Convenience constructors** — reduce per-binary boilerplate
2. **Prelude modules** — collapse import sprawl
3. **`run_default()` helper** — eliminate repeated Arc wiring
4. **Absorb Phase 4 deferred items** — 4 small fixes
5. **Update stale documentation** — ARCHITECTURE.md describes Phase 2.2

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

## B.2 Prelude Modules

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
    copy_file, create_dir, exists, read_file, remove_dir, write_file,
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

## B.3 `run_default()` Convenience Function

`workflow_core` cannot depend on `workflow_utils`, so this lives in `workflow_utils`.

**File:** `workflow_utils/src/lib.rs` (or `workflow_utils/src/runner.rs` re-exported from lib.rs)

```rust
use std::sync::Arc;
use workflow_core::{HookExecutor, ProcessRunner};
use workflow_core::state::{StateStore};
use workflow_core::workflow::{Workflow, WorkflowSummary};
use workflow_core::error::WorkflowError;
use crate::{ShellHookExecutor, SystemProcessRunner};

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

Before (every binary has these 3 lines):
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

## B.4 Absorb Phase 4 Deferred Items

Source: `notes/pr-reviews/phase-4/deferred.md`

### B.4.1 Whitespace artifact (cosmetic)

**File:** `workflow-cli/src/main.rs` ~line 71

Two blank lines remain where `downstream_tasks` was removed. Delete one blank line.
This is a one-line edit — precondition for any future edit to main.rs per the deferred doc.

### B.4.2 `downstream_of` signature ergonomics

**File:** `workflow_core/src/state.rs`

Change signature from:
```rust
pub fn downstream_of(&self, start: &[String]) -> Vec<String>
```
to:
```rust
pub fn downstream_of<S: AsRef<str>>(&self, start: &[S]) -> Vec<String>
```

Update internal BFS to use `s.as_ref().to_owned()` where it currently calls `.to_owned()` on `String`.

This allows callers to pass `&[&str]` or `&[String]` without allocating owned strings at the call site.

### B.4.3 Implementation guidelines (documentation)

Add a new **"Implementation Guidelines"** section to `ARCHITECTURE.md` with two rules:

**Guideline 1 — Newtype encapsulation:**
> Design newtypes with full encapsulation on introduction. Expose methods that delegate to the inner collection, never expose the raw inner type via a public accessor. Introducing `inner()` and then removing it one phase later causes churn across fix plans.

**Guideline 2 — Domain logic placement:**
> Place domain logic operating on `workflow_core` types in `workflow_core` from the initial implementation. Logic written in the CLI binary and later migrated to `workflow_core` causes churn (BFS `downstream_tasks` pattern, v2/v4/v5).

---

## B.5 Update Stale Documentation

Both `ARCHITECTURE.md` and `ARCHITECTURE_STATUS.md` describe Phase 2.2 as the latest. The code examples in `ARCHITECTURE.md` reference `Workflow::builder()` which does not exist.

**File: `ARCHITECTURE.md`**
- Update "Implementation Status" to reflect Phases 3 and 4 as complete
- Fix all code examples to match actual API (`Workflow::new()`, `Task::new(id, mode)`, `ExecutionMode::Queued`)
- Add Phase 3 components: `StateStore`, `JsonStateStore`, `load_raw()`, `WorkflowError`, `WorkflowSummary`, signal handling, `workflow-cli`
- Add Phase 4 components: log persistence, `HookTrigger::Periodic`, `ExecutionMode::Queued`, `QueuedRunner`, graph-aware retry, `TaskSuccessors`
- Add "Implementation Guidelines" section (from B.4.3)

**File: `ARCHITECTURE_STATUS.md`**
- Mark Phases 3 and 4 as complete
- Add Phase 5 entry (in progress)

Do this **last** so docs reflect all B.1-B.4 changes.

---

## B.6 Runtime Mode Switching

**Problem:** Binaries like `hubbard_u_sweep_slurm` hard-code `ExecutionMode::Queued`. There is no way to run the same binary locally for testing without modifying source.

**Pattern (application-level, no core change needed):**

Add a `--local` flag (or `--mode direct|queued`) to the binary's `SweepConfig`:

```rust
/// Run locally using Direct execution instead of submitting to SLURM
#[arg(long, default_value_t = false)]
pub local: bool,
```

Construct the mode in the task loop:

```rust
let mode = if config.local {
    ExecutionMode::direct(&config.castep_command, &[&config.seed_name])
} else {
    ExecutionMode::Queued
};
let task = Task::new(&task_id, mode)
    .setup(move |workdir| {
        // ... always write .cell and .param ...
        if !local {
            write_file(workdir.join("job.sh"), &job_script)?;
        }
        Ok(())
    });
```

Conditionally attach the submitter:

```rust
let mut workflow = Workflow::new("hubbard_u_sweep_slurm")
    .with_max_parallel(config.max_parallel)?
    .with_log_dir("logs");
if !config.local {
    workflow = workflow.with_queued_submitter(Arc::new(QueuedRunner::new(SchedulerKind::Slurm)));
}
```

**Scope:** Apply this pattern to `examples/hubbard_u_sweep_slurm` (added in Phase 5A). No changes to `workflow_core` or `workflow_utils` required — `ExecutionMode::direct()` from B.1 makes the Direct arm concise enough.

---

## Sequencing

```
B.4.1  Fix whitespace in workflow-cli/src/main.rs        (trivial)
B.4.2  Fix downstream_of signature                       (1 function)
B.1    Add ExecutionMode::direct()                       (1 method)
B.2    Add prelude modules                               (2 new files + 2 lib.rs edits)
B.3    Add run_default()                                 (1 function)
B.6    Add --local flag to hubbard_u_sweep_slurm         (pattern, example only)
B.5    Update ARCHITECTURE.md + ARCHITECTURE_STATUS.md   (docs last)
```

B.4.3 (guidelines) is absorbed into B.5 since they go into ARCHITECTURE.md.

---

## Scope Boundaries

**In scope:**
- Convenience constructors and prelude (additive, no breaking changes)
- 4 deferred items from Phase 4 review
- Documentation update

**Out of scope:**
- `CollectFailurePolicy` (not needed — collect failure is a warn today; good enough for the sweep)
- SLURM `script_template()` helper on `QueuedRunner` (deferred — let Part A show whether it's needed)
- Typed result collection (design in Phase 6)
- Convergence/iteration patterns (design in Phase 6)

---

## Critical Files

| File | Change |
|---|---|
| `workflow_core/src/task.rs` | Add `ExecutionMode::direct()` |
| `workflow_core/src/prelude.rs` | New file |
| `workflow_core/src/lib.rs` | Add `pub mod prelude` |
| `workflow_core/src/state.rs` | Fix `downstream_of` signature |
| `workflow_utils/src/prelude.rs` | New file |
| `workflow_utils/src/lib.rs` | Add `pub mod prelude` + `run_default()` |
| `workflow-cli/src/main.rs` | Remove extra blank line ~line 71 |
| `examples/hubbard_u_sweep_slurm/src/config.rs` | Add `--local` flag |
| `examples/hubbard_u_sweep_slurm/src/main.rs` | Conditional mode, submitter, job.sh |
| `ARCHITECTURE.md` | Full update for Phases 3-5 |
| `ARCHITECTURE_STATUS.md` | Update phase completion status |

---

## Verification

After each task:
```
cargo check --workspace
cargo clippy --workspace
cargo test --workspace
```

No new tests required for convenience methods (they delegate to tested code).
Documentation changes: build with `cargo doc --workspace` and verify links.
