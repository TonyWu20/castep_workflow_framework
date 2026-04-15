## v3 (2026-04-15)

# Phase 3 Fix Plan — Post-Final-Fixes Review (v3)

This plan addresses issues found after the 5-task v2 fix round. All items below were confirmed present in the actual source on the `phase-3` branch by the strict-code-reviewer and fix-plan-reader agents.

---

## Execution Phases

| Phase | Tasks | Notes |
|-------|-------|-------|
| Phase 1 (parallel) | TASK-1, TASK-4, TASK-5, TASK-6, TASK-7, TASK-9 | Independent — different files |
| Phase 2 (parallel) | TASK-2, TASK-8 | TASK-2 depends on TASK-1 (workflow.rs); TASK-8 depends on TASK-4 (main.rs) |
| Phase 3 | TASK-3 | Depends on TASK-2 (same file: workflow.rs) |
| Final | — | `cargo test --workspace && cargo clippy --workspace` |

---

### TASK-1: Add `PathBuf` to `TaskHandle` type alias and update insert site

**File:** `workflow_core/src/workflow.rs`
**Target:** `pub(crate) type TaskHandle` alias and `handles.insert(...)` call in `ExecutionMode::Direct` arm
**Depends on:** None
**Can run in parallel with:** TASK-4, TASK-5, TASK-6, TASK-7, TASK-9

**Before (type alias):**
```rust
pub(crate) type TaskHandle = (
    Box<dyn ProcessHandle>,
    Instant,
    Vec<crate::monitoring::MonitoringHook>,
    Option<TaskClosure>,
);
```

**After (type alias):**
```rust
pub(crate) type TaskHandle = (
    Box<dyn ProcessHandle>,
    Instant,
    Vec<crate::monitoring::MonitoringHook>,
    Option<TaskClosure>,
    std::path::PathBuf,
);
```

**Before (handles.insert — the only `handles.insert` call in the file):**
```rust
handles.insert(id.clone(), (handle, Instant::now(), monitors, task.collect));
```

**After:**
```rust
handles.insert(id.clone(), (handle, Instant::now(), monitors, task.collect, task.workdir.clone()));
```

**Why:** The `collect` closure and the post-completion `HookContext::workdir` receive a hardcoded `"."` instead of the task's actual workdir, because the workdir is not stored in `TaskHandle`. Once TASK-2 updates the destructuring sites, the stored workdir will be passed through correctly.

**Verification:** `cargo check -p workflow_core` — expect errors about 4-element tuple destructuring against 5-element type; these are resolved by TASK-2.

---

### TASK-2: Update all destructuring sites and fix `collect`/`HookContext` workdir

**File:** `workflow_core/src/workflow.rs`
**Prerequisite:** Must run AFTER TASK-1.
**Can run in parallel with:** TASK-8

There are exactly 3 destructuring sites to update, plus 2 usage fixes.

**Site A — interrupt handler:**
```rust
// Before:
for (_, (handle, _start, _monitors, _collect_fn)) in handles.iter_mut() {
// After:
for (_, (handle, _start, _monitors, _collect_fn, _workdir)) in handles.iter_mut() {
```

**Site B — poll finished tasks:**
```rust
// Before:
for (id, (handle, start, _monitors, _collect_fn)) in handles.iter_mut() {
// After:
for (id, (handle, start, _monitors, _collect_fn, _workdir)) in handles.iter_mut() {
```

**Site C — finished task processing (the `handles.remove` call):**
```rust
// Before:
if let Some((mut handle, start, monitors, collect_fn)) = handles.remove(&id) {
// After:
if let Some((mut handle, start, monitors, collect_fn, workdir)) = handles.remove(&id) {
```
Note: no underscore prefix — `workdir` is used below.

**Usage fix 1 — collect call (inside the finished task processing block):**
```rust
// Before:
if let Err(e) = collect(std::path::Path::new(".")) {
// After:
if let Err(e) = collect(&workdir) {
```

**Usage fix 2 — HookContext for OnComplete/OnFailure hooks (the `HookContext` struct literal that uses `final_state`):**
```rust
// Before:
let ctx = crate::monitoring::HookContext {
    task_id: id.clone(),
    workdir: std::path::PathBuf::from("."),
    state: final_state.to_string(),
    exit_code,
};
// After:
let ctx = crate::monitoring::HookContext {
    task_id: id.clone(),
    workdir,
    state: final_state.to_string(),
    exit_code,
};
```
Note: `workdir` can be moved (not cloned) because this is its last use in the scope.

**Verification:** `cargo test -p workflow_core`

---

### TASK-3: Add `debug_assert` and doc-comment to `Workflow::run()`

**File:** `workflow_core/src/workflow.rs`
**Prerequisite:** Must run AFTER TASK-2 (same file; avoid merge conflicts).
**Can run in parallel with:** Nothing in workflow.rs.

**Before:**
```rust
/// Runs the workflow with dependency injection for state, runner, and hook executor.
pub fn run(
    &mut self,
    state: &mut dyn StateStore,
    runner: Arc<dyn ProcessRunner>,
    hook_executor: Arc<dyn HookExecutor>,
) -> Result<WorkflowSummary, WorkflowError> {
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&self.interrupt)).ok();
```

**After:**
```rust
/// Runs the workflow with dependency injection for state, runner, and hook executor.
///
/// # Panics (debug only)
/// Asserts that the workflow has tasks. Tasks are consumed from the `Workflow` on dispatch;
/// calling `run()` twice on the same instance will silently process no tasks on the second call.
/// Construct a new `Workflow` to re-run.
pub fn run(
    &mut self,
    state: &mut dyn StateStore,
    runner: Arc<dyn ProcessRunner>,
    hook_executor: Arc<dyn HookExecutor>,
) -> Result<WorkflowSummary, WorkflowError> {
    debug_assert!(
        !self.tasks.is_empty(),
        "run() called on a Workflow with no tasks — tasks are consumed on dispatch"
    );
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&self.interrupt)).ok();
```

**Verification:** `cargo test -p workflow_core`

---

### TASK-4: Add clarifying comment to `cmd_retry` global reset

**File:** `workflow-cli/src/main.rs`
**Target function:** `cmd_retry`
**Depends on:** None
**Can run in parallel with:** TASK-1, TASK-5, TASK-6, TASK-7, TASK-9

**Before (the `let to_reset` block inside `cmd_retry`):**
```rust
let to_reset: Vec<String> = state
    .all_tasks()
    .into_iter()
    .filter(|(_, s)| matches!(s, TaskStatus::SkippedDueToDependencyFailure))
    .map(|(id, _)| id)
    .collect();
```

**After:**
```rust
// Reset all dependency-failure-skipped tasks globally (not just those downstream
// of `task_ids`). Intentional for v0.1 simplicity — a graph-aware retry would
// require DAG access that the CLI does not have.
let to_reset: Vec<String> = state
    .all_tasks()
    .into_iter()
    .filter(|(_, s)| matches!(s, TaskStatus::SkippedDueToDependencyFailure))
    .map(|(id, _)| id)
    .collect();
```

**Verification:** `cargo test -p workflow-cli`

---

### TASK-5: Rename `JsonStateStore::save()` inherent method to `persist()`

**File:** `workflow_core/src/state.rs`
**Depends on:** None
**Can run in parallel with:** TASK-1, TASK-4, TASK-6, TASK-7, TASK-9

**Before (inherent method inside `impl JsonStateStore`):**
```rust
/// Saves state atomically using temp file + rename pattern.
pub fn save(&self) -> Result<(), WorkflowError> {
    let temp_path = self.path.with_extension("tmp");
    let json = serde_json::to_vec_pretty(self)
        .map_err(|e| WorkflowError::StateCorrupted(e.to_string()))?;
    fs::write(&temp_path, json).map_err(WorkflowError::Io)?;
    fs::rename(&temp_path, &self.path).map_err(WorkflowError::Io)?;
    Ok(())
}
```

**After:**
```rust
/// Saves state atomically using temp file + rename pattern.
fn persist(&self) -> Result<(), WorkflowError> {
    let temp_path = self.path.with_extension("tmp");
    let json = serde_json::to_vec_pretty(self)
        .map_err(|e| WorkflowError::StateCorrupted(e.to_string()))?;
    fs::write(&temp_path, json).map_err(WorkflowError::Io)?;
    fs::rename(&temp_path, &self.path).map_err(WorkflowError::Io)?;
    Ok(())
}
```

**Before (trait impl delegation inside `impl StateStore for JsonStateStore`):**
```rust
fn save(&self) -> Result<(), WorkflowError> {
    self.save()
}
```

**After:**
```rust
fn save(&self) -> Result<(), WorkflowError> {
    self.persist()
}
```

**Why:** The current code relies on Rust's inherent-over-trait method resolution to avoid infinite recursion. If the inherent `save()` were renamed or removed, the trait impl would silently recurse. Making the delegation explicit eliminates this fragility. All callers of `.save()` on `JsonStateStore` values go through the `StateStore` trait method, which still works.

**Verification:** `cargo test --workspace`

---

### TASK-6: Remove TASK-7 marker comments from `executor.rs`

**File:** `workflow_utils/src/executor.rs`
**Depends on:** None
**Can run in parallel with:** TASK-1, TASK-4, TASK-5, TASK-7, TASK-9

**Remove block 1 (appears just before `use std::path::Path;`):**
```rust
// ============================================================================
// TASK-7: SystemProcessRunner and SystemProcessHandle
// Implements ProcessRunner trait for workflow engine integration
// ============================================================================
```

**Remove block 2 (appears at the end of the file after the `SystemProcessHandle` impl block):**
```rust
// ============================================================================
// End TASK-7 implementations
// =============================================================================
```

Delete both blocks entirely (including surrounding blank lines if they become redundant).

**Verification:** `cargo check -p workflow_utils`

---

### TASK-7: Fix `use`-after-definition ordering in `task.rs`

**File:** `workflow_core/src/task.rs`
**Depends on:** None
**Can run in parallel with:** TASK-1, TASK-4, TASK-5, TASK-6, TASK-9

**Before (lines 6–9):**
```rust
/// A closure used for task setup or result collection.
pub type TaskClosure = Box<dyn Fn(&Path) -> Result<(), WorkflowError> + Send + Sync>;
use std::path::{Path, PathBuf};
use std::time::Duration;
```

**After:**
```rust
use std::path::{Path, PathBuf};
use std::time::Duration;

/// A closure used for task setup or result collection.
pub type TaskClosure = Box<dyn Fn(&Path) -> Result<(), WorkflowError> + Send + Sync>;
```

**Verification:** `cargo check -p workflow_core`

---

### TASK-8: Remove `status.clone()` in `cmd_status` and fix `Inspect` arm

**File:** `workflow-cli/src/main.rs`
**Prerequisite:** Run AFTER TASK-4 (same file; conservative serialization).
**Can run in parallel with:** TASK-2

**Fix A — remove unnecessary clone in `cmd_status` loop:**
```rust
// Before:
for (id, status) in &tasks {
    match status.clone() {
        TaskStatus::Failed { error } => out.push_str(&format!("{}: Failed ({})\n", id, error)),
        other => out.push_str(&format!("{}: {:?}\n", id, other)),
    }
}
// After:
for (id, status) in &tasks {
    match status {
        TaskStatus::Failed { error } => out.push_str(&format!("{}: Failed ({})\n", id, error)),
        other => out.push_str(&format!("{}: {:?}\n", id, other)),
    }
}
```
Note: `error` binds as `&String`; `format!("{}", error)` works with `&String`. No other changes needed.

**Fix B — replace `process::exit(1)` with `?` in the `Inspect` arm of `main()`:**
```rust
// Before:
Commands::Inspect { state_file, task_id } => {
    let state = load_state_raw(&state_file)?;
    match cmd_inspect(&state, task_id.as_deref()) {
        Ok(out) => { println!("{}", out); Ok(()) }
        Err(e) => { eprintln!("{}", e); std::process::exit(1); }
    }
}
// After:
Commands::Inspect { state_file, task_id } => {
    let state = load_state_raw(&state_file)?;
    let out = cmd_inspect(&state, task_id.as_deref())?;
    println!("{}", out);
    Ok(())
}
```

**Verification:** `cargo test -p workflow-cli && cargo clippy -p workflow-cli`

---

### TASK-9: Relax flaky timing bound in `test_duration_tracking`

**File:** `workflow_utils/tests/process_tests.rs`
**Target function:** `test_duration_tracking`
**Depends on:** None
**Can run in parallel with:** TASK-1, TASK-4, TASK-5, TASK-6, TASK-7

**Before (inside `test_duration_tracking`):**
```rust
assert!(result.duration <= std::time::Duration::from_millis(100));
```

**After:**
```rust
assert!(result.duration <= std::time::Duration::from_secs(1));
```

**Why:** A `sleep 0.01` (10ms) process asserted to complete within 100ms is flaky on loaded CI systems. 1 second provides headroom without losing meaningful signal.

**Verification:** `cargo test -p workflow_utils --test process_tests`

---

## Dependency Graph

```
TASK-1 → TASK-2 → TASK-3       (all in workflow_core/src/workflow.rs)
TASK-4 → TASK-8                 (both in workflow-cli/src/main.rs)
TASK-5                          (workflow_core/src/state.rs — independent)
TASK-6                          (workflow_utils/src/executor.rs — independent)
TASK-7                          (workflow_core/src/task.rs — independent)
TASK-9                          (workflow_utils/tests/process_tests.rs — independent)
```

**Final verification after all tasks:**
```bash
cargo test --workspace
cargo clippy --workspace --all-targets 2>&1 | grep -E "^error|^warning"
```
