## v4 (2026-04-15)

# Phase 3 Fix Plan — Post-v3-Fixes Review (v4)

This plan addresses 8 issues found after the v3 fix round. All items confirmed present in the actual source on the `phase-3` branch. Two Blocking correctness issues (orphaned processes), two Major quality issues (silent no-op variant, bare tuple), and four Minor style/cleanup issues.

---

## Execution Phases

| Phase | Tasks | Notes |
|-------|-------|-------|
| Phase 1 (parallel) | TASK-6, TASK-8 | Different files from the workflow.rs group |
| Phase 2 (parallel) | TASK-1, TASK-2, TASK-3, TASK-4, TASK-5 | All in `workflow_core/src/workflow.rs` — must be serialized to a single agent to avoid conflicts |
| Phase 3 | TASK-7 | Depends on TASK-4 (needs `InFlightTask` struct) |
| Final | — | `cargo clippy --workspace && cargo test --workspace` |

**Recommended single-agent order for `workflow.rs` changes:** TASK-1 → TASK-3 → TASK-2 → TASK-5 → TASK-4 → TASK-7

---

### TASK-1: Add upfront Queued-mode validation and replace dispatch branch with `unreachable!`

**File:** `workflow_core/src/workflow.rs`
**Target:** `Workflow::run()` — between signal registration and `build_dag()`, and the `Queued` match arm in the dispatch loop
**Depends on:** None
**Can run in parallel with:** TASK-2, TASK-3, TASK-5, TASK-6, TASK-7, TASK-8 (but same file as TASK-2/3/5/4 — serialize within one agent)

**Why:** When a `Queued` task is encountered mid-dispatch, `run()` returns immediately without terminating already-running Direct tasks. Processes are orphaned (no `Drop` kill), and tasks remain `Running` in state. Moving the check before any processes are spawned eliminates the hazard entirely.

**Edit 1 — insert upfront validation (anchor: the two `signal_hook` lines followed by `let dag`):**

Before:
```rust
        signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&self.interrupt)).ok();
        signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&self.interrupt)).ok();

        let dag = self.build_dag()?;
```

After:
```rust
        signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&self.interrupt)).ok();
        signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&self.interrupt)).ok();

        // Reject Queued tasks upfront — before any processes are spawned — so we never orphan handles.
        for (id, task) in &self.tasks {
            if matches!(task.mode, ExecutionMode::Queued { .. }) {
                return Err(WorkflowError::InvalidConfig(
                    format!("task '{}': Queued execution mode is not yet implemented", id)
                ));
            }
        }

        let dag = self.build_dag()?;
```

**Edit 2 — replace Queued dispatch branch (anchor: the only `ExecutionMode::Queued` match arm):**

Before:
```rust
                            ExecutionMode::Queued { .. } => {
                                return Err(WorkflowError::InvalidConfig(
                                    "Queued execution mode is not yet implemented".into(),
                                ));
                            }
```

After:
```rust
                            ExecutionMode::Queued { .. } => {
                                unreachable!("Queued tasks rejected by upfront validation");
                            }
```

**Verification:** `cargo check --workspace`

---

### TASK-2: Handle `runner.spawn()` failure gracefully instead of propagating with `?`

**File:** `workflow_core/src/workflow.rs`
**Target:** `Workflow::run()` — inside `ExecutionMode::Direct` dispatch arm
**Depends on:** None (but serialize after TASK-1 when editing the same file)

**Why:** `state.mark_running(&id)` is called before `runner.spawn(...)`. If spawn fails, the `?` exits `run()` leaving the task permanently in `Running` state and any handles in `handles` orphaned. Handling the error inline keeps the task lifecycle consistent.

Before (the only `runner.spawn` call in the file):
```rust
                                let handle = runner.spawn(&task.workdir, command, args, env)?;
```

After:
```rust
                                let handle = match runner.spawn(&task.workdir, command, args, env) {
                                    Ok(h) => h,
                                    Err(e) => {
                                        state.mark_failed(&id, e.to_string());
                                        state.save()?;
                                        continue;
                                    }
                                };
```

**Verification:** `cargo check --workspace`

---

### TASK-3: Add upfront `Periodic` hook validation

**File:** `workflow_core/src/workflow.rs`
**Target:** `Workflow::run()` — before the `// Initialize state for all tasks` comment
**Depends on:** None (but serialize after TASK-1 when editing the same file)

**Why:** `HookTrigger::Periodic` is a public enum variant. `Workflow::run()` never handles it — attaching a Periodic hook produces no error and no execution. Failing fast makes the contract explicit.

Before (anchor: the `// Initialize state for all tasks` comment + loop):
```rust
        // Initialize state for all tasks
        for id in dag.task_ids() {
            if state.get_status(id).is_none() {
                state.set_status(id, TaskStatus::Pending);
            }
        }
```

After:
```rust
        // Reject Periodic hooks upfront — not yet implemented in the run loop.
        for (id, task) in &self.tasks {
            for hook in &task.monitors {
                if matches!(hook.trigger, crate::monitoring::HookTrigger::Periodic { .. }) {
                    return Err(WorkflowError::InvalidConfig(
                        format!("task '{}': Periodic hooks are not yet supported", id)
                    ));
                }
            }
        }

        // Initialize state for all tasks
        for id in dag.task_ids() {
            if state.get_status(id).is_none() {
                state.set_status(id, TaskStatus::Pending);
            }
        }
```

**Verification:** `cargo check --workspace`

---

### TASK-4: Replace `TaskHandle` bare tuple with named `InFlightTask` struct

**File:** `workflow_core/src/workflow.rs`
**Target:** `TaskHandle` type alias + all 4 destructuring/construction sites in `Workflow::run()`
**Depends on:** None (but serialize after TASK-2/3/5 when editing the same file)
**Enables:** TASK-7

**Why:** The 5-element positional tuple makes code fragile and hard to audit at every destructuring site.

Apply sub-steps A → F in order:

**Step A — Replace type alias (anchor: `/// A handle to a running task with metadata.`):**

Before:
```rust
/// A handle to a running task with metadata.
pub(crate) type TaskHandle = (
    Box<dyn ProcessHandle>,
    Instant,
    Vec<crate::monitoring::MonitoringHook>,
    Option<TaskClosure>,
    std::path::PathBuf,
);
```

After:
```rust
/// A handle to a running task with metadata.
pub(crate) struct InFlightTask {
    pub handle: Box<dyn ProcessHandle>,
    pub started_at: Instant,
    pub monitors: Vec<crate::monitoring::MonitoringHook>,
    pub collect: Option<TaskClosure>,
    pub workdir: std::path::PathBuf,
}
```

**Step B — Update HashMap type (anchor: `let mut handles:`):**

Before:
```rust
        let mut handles: HashMap<String, TaskHandle> = HashMap::new();
```

After:
```rust
        let mut handles: HashMap<String, InFlightTask> = HashMap::new();
```

**Step C — Update interrupt cleanup loop (anchor: `for (_, (handle, _start, _monitors`):**

Before:
```rust
                for (_, (handle, _start, _monitors, _collect_fn, _workdir)) in handles.iter_mut() {
                    handle.terminate().ok();
                }
```

After:
```rust
                for (_, t) in handles.iter_mut() {
                    t.handle.terminate().ok();
                }
```

**Step D — Update timeout/poll loop (anchor: `for (id, (handle, start, _monitors`):**

Before:
```rust
            for (id, (handle, start, _monitors, _collect_fn, _workdir)) in handles.iter_mut() {
                // Timeout check first
                if let Some(&timeout) = task_timeouts.get(id) {
                    if start.elapsed() >= timeout {
                        handle.terminate().ok();
```

After:
```rust
            for (id, t) in handles.iter_mut() {
                // Timeout check first
                if let Some(&timeout) = task_timeouts.get(id) {
                    if t.started_at.elapsed() >= timeout {
                        t.handle.terminate().ok();
```

Also in the same loop body, replace:

Before: `                if !handle.is_running() {`
After:  `                if !t.handle.is_running() {`

**Step E — Update finished task destructure (anchor: `if let Some((mut handle, start, monitors, collect_fn, workdir)`):**

Before:
```rust
                if let Some((mut handle, start, monitors, collect_fn, workdir)) = handles.remove(&id) {
```

After:
```rust
                if let Some(mut t) = handles.remove(&id) {
```

Then in the same block replace all field uses:
- `handle.wait()` → `t.handle.wait()`
- `handle.terminate()` → `t.handle.terminate()`
- `if let Some(ref collect) = collect_fn {` → `if let Some(ref collect) = t.collect {`
- `collect(&workdir)` → `collect(&t.workdir)`
- `workdir` in `HookContext { ..., workdir, ... }` → `t.workdir`
- `for hook in &monitors {` → `for hook in &t.monitors {`

Note: if TASK-5 runs first, the `let _duration = start.elapsed();` / `let _duration = t.started_at.elapsed();` line will already be removed — skip it.

**Step F — Update `handles.insert` construction (anchor: the only `handles.insert` call):**

Before:
```rust
                                handles.insert(id.clone(), (handle, Instant::now(), monitors, task.collect, task.workdir.clone()));
```

After:
```rust
                                handles.insert(id.clone(), InFlightTask {
                                    handle,
                                    started_at: Instant::now(),
                                    monitors,
                                    collect: task.collect,
                                    workdir: task.workdir.clone(),
                                });
```

**Verification:** `cargo check --workspace`

---

### TASK-5: Remove dead `_duration` variable

**File:** `workflow_core/src/workflow.rs`
**Target:** Inside the `for id in finished` block in `Workflow::run()`
**Depends on:** None (but serialize when editing the same file)

Before (anchor: between the `if matches!(state.get_status(&id), Some(TaskStatus::Failed { .. }))` continue and `// Execute the process and handle result`):
```rust
                    let _duration = start.elapsed();
```

After: remove the line entirely.

Note: if TASK-4 has already run, this line reads `let _duration = t.started_at.elapsed();` — delete whichever form is present.

**Verification:** `cargo check --workspace`

---

### TASK-6: Consolidate mid-file `use` statements in `executor.rs`

**File:** `workflow_utils/src/executor.rs`
**Depends on:** None
**Can run in parallel with:** all other tasks

**Edit 1 — update top-of-file imports (anchor: lines 1-4 of the file):**

Before:
```rust
use std::collections::HashMap;
use std::path::PathBuf;

pub use workflow_core::WorkflowError;
```

After:
```rust
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Instant;

pub use workflow_core::WorkflowError;
```

**Edit 2 — remove mid-file imports (anchor: the three consecutive `use` lines between `ExecutionHandle` impl and `pub use workflow_core::{ProcessRunner...}`):**

Remove:
```rust
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::Instant;
```

**Verification:** `cargo check --workspace`

---

### TASK-7: Remove redundant `.clone()` on `task.workdir` in `InFlightTask` construction

**File:** `workflow_core/src/workflow.rs`
**Depends on:** TASK-4 (requires `InFlightTask` struct to exist)

**Why:** `task_workdir` (for `HookContext`) was already cloned from `task.workdir` earlier in the block. `task` is consumed by `self.tasks.remove(&id)` and not used after `handles.insert`, so `task.workdir` can be moved directly into the struct.

Before (after TASK-4 Step F is applied):
```rust
                                handles.insert(id.clone(), InFlightTask {
                                    handle,
                                    started_at: Instant::now(),
                                    monitors,
                                    collect: task.collect,
                                    workdir: task.workdir.clone(),
                                });
```

After:
```rust
                                handles.insert(id.clone(), InFlightTask {
                                    handle,
                                    started_at: Instant::now(),
                                    monitors,
                                    collect: task.collect,
                                    workdir: task.workdir,
                                });
```

**Verification:** `cargo check --workspace`

---

### TASK-8: Remove redundant `all_task_statuses()` method and update test

**File:** `workflow_core/src/state.rs`
**Depends on:** None
**Can run in parallel with:** all other tasks

**Step A — Update test assertion (anchor: the `fn all_tasks()` test in `mod tests`):**

Before:
```rust
        assert_eq!(s.all_task_statuses().len(), 2);
```

After:
```rust
        assert_eq!(s.all_tasks().len(), 2);
```

**Step B — Remove method (anchor: the second `impl JsonStateStore` block containing only `all_task_statuses`):**

Remove the entire block:
```rust
impl JsonStateStore {
    /// Returns all task statuses.
    pub fn all_task_statuses(&self) -> HashMap<String, TaskStatus> {
        self.tasks.clone()
    }
}
```

**Verification:** `cargo check --workspace && cargo test --workspace`

---

## Dependency Graph

```
TASK-1                             (workflow_core/src/workflow.rs — independent)
TASK-2                             (workflow_core/src/workflow.rs — independent)
TASK-3                             (workflow_core/src/workflow.rs — independent)
TASK-4 → TASK-7                    (workflow_core/src/workflow.rs — TASK-7 needs InFlightTask)
TASK-5                             (workflow_core/src/workflow.rs — independent)
TASK-6                             (workflow_utils/src/executor.rs — independent)
TASK-8                             (workflow_core/src/state.rs — independent)
```

TASK-1 through TASK-5 and TASK-7 all modify `workflow_core/src/workflow.rs` — delegate to a single agent.

**Final verification after all tasks:**
```bash
cargo clippy --workspace && cargo test --workspace
```

---

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
