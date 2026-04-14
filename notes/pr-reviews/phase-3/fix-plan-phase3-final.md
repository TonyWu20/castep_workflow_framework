# Phase 3 Final Fix Plan (2026-04-14)

## Plan Summary

This fix plan addresses 7 issues found in the `phase-3` branch: three blocking bugs (double-dot
temp filenames, `collect` closure never called, OS signals not wired) and four major issues
(`tempfile` in prod deps, stale duplicate test file, unused `bon` dep, misleading test assertion).

---

## TASK-1: Fix double-dot temp filename in `JsonStateStore::save()`

**File:** `workflow_core/src/state.rs`
**Target:** Inside `impl JsonStateStore`, the inherent `pub fn save(&self)` method body
**Before:** `let temp_path = self.path.with_extension(".tmp");`
**After:** `let temp_path = self.path.with_extension("tmp");`
**Verification:** `cargo check -p workflow_core`
**Note:** Do NOT touch the `atomic_save` test — that is TASK-2.

---

## TASK-2: Fix double-dot temp filename in `atomic_save` test

**File:** `workflow_core/src/state.rs`
**Target:** Inside `fn atomic_save()` test, inside `#[cfg(test)] mod tests`
**Before:** `let _temp = std::fs::File::create(path.with_extension(".tmp")).unwrap();`
**After:** `let _temp = std::fs::File::create(path.with_extension("tmp")).unwrap();`
**Depends on:** TASK-1 (same file — apply after TASK-1, or in the same edit session)
**Verification:** `cargo test -p workflow_core -- atomic_save`

---

## TASK-3: Widen `handles` map type to include collect closure slot

**File:** `workflow_core/src/workflow.rs`
**Target:** Inside `pub fn run(...)`, the `let mut handles: HashMap<...>` declaration
**Before:**
```rust
let mut handles: HashMap<
    String,
    (
        Box<dyn ProcessHandle>,
        Instant,
        Vec<crate::monitoring::MonitoringHook>,
    ),
> = HashMap::new();
```
**After:**
```rust
let mut handles: HashMap<
    String,
    (
        Box<dyn ProcessHandle>,
        Instant,
        Vec<crate::monitoring::MonitoringHook>,
        Option<Box<dyn Fn(&Path) -> Result<(), WorkflowError> + Send + Sync>>,
    ),
> = HashMap::new();
```

Also update ALL three destructuring sites in the same function:

1. Interrupt-check loop — `for (_, (handle, _, _)) in handles.iter_mut()`
   → `for (_, (handle, _, _, _)) in handles.iter_mut()`

2. Poll loop — `for (id, (handle, start, _)) in handles.iter_mut()`
   → `for (id, (handle, start, _, _)) in handles.iter_mut()`

3. Finished-task removal — `if let Some((mut handle, start, monitors)) = handles.remove(&id)`
   → `if let Some((mut handle, start, monitors, collect_fn)) = handles.remove(&id)`

Also temporarily change the `handles.insert(...)` call to pass `None` as the fourth element
(TASK-4 will replace this with the real value):
- Before: `handles.insert(id.clone(), (handle, Instant::now(), monitors));`
- After:  `handles.insert(id.clone(), (handle, Instant::now(), monitors, None));`

**Verification:** `cargo check -p workflow_core`

---

## TASK-4: Pass `task.collect` at dispatch time into handles map

**File:** `workflow_core/src/workflow.rs`
**Target:** Inside `pub fn run(...)`, the `handles.insert(id.clone(), ...)` call inside the
`ExecutionMode::Direct` match arm (immediately before closing brace of that arm, just after
OnStart hooks are fired)
**Before:** `handles.insert(id.clone(), (handle, Instant::now(), monitors, None));`
**After:** `handles.insert(id.clone(), (handle, Instant::now(), monitors, task.collect));`
**Depends on:** TASK-3
**Verification:** `cargo check -p workflow_core`

---

## TASK-5: Invoke collect closure after successful task completion

**File:** `workflow_core/src/workflow.rs`
**Target:** Inside `pub fn run(...)`, the `Some(0) =>` arm of the `match process_result.exit_code`
block, after `state.mark_completed(&id);`

**Before:**
```rust
Some(0) => {
    state.mark_completed(&id);
    ("completed", process_result.exit_code)
}
```
**After:**
```rust
Some(0) => {
    state.mark_completed(&id);
    if let Some(ref collect) = collect_fn {
        if let Err(e) = collect(std::path::Path::new(".")) {
            tracing::warn!(
                "Collect closure for task '{}' failed: {}",
                id,
                e
            );
        }
    }
    ("completed", process_result.exit_code)
}
```
**Depends on:** TASK-3, TASK-4
**Note:** `collect_fn` is the variable from the destructuring updated in TASK-3:
`if let Some((mut handle, start, monitors, collect_fn)) = handles.remove(&id)`.
The workdir argument uses `Path::new(".")` — consistent with existing `HookContext` pattern in the
same function.
**Verification:** `cargo test -p workflow_core`

---

## TASK-6: Add `signal-hook` to workspace and `workflow_core` Cargo.toml

**File 1:** `Cargo.toml` (workspace root)
**Target:** `[workspace.dependencies]` section — add after the `clap` line:
```toml
signal-hook = "0.3"
```

**File 2:** `workflow_core/Cargo.toml`
**Target:** `[dependencies]` section — add after `thiserror = "1"` and BEFORE the `[features]`
block (i.e., insert immediately after the `thiserror` line, not after `[features]`):
```toml
signal-hook = { workspace = true }
```
**Note:** Both files can be edited in the same pass. If TASK-10 also edits `Cargo.toml`, combine
both edits in a single edit session to avoid conflicts.
**Verification:** `cargo check -p workflow_core`

---

## TASK-7: Wire OS signal handlers in `Workflow::run()`

**File:** `workflow_core/src/workflow.rs`
**Target:** Inside `pub fn run(...)`, at the very beginning of the function body, before
`let dag = self.build_dag()?;`

Insert these two lines immediately after the opening brace of the `run()` function body:
```rust
signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&self.interrupt)).ok();
signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&self.interrupt)).ok();
```
No new `use` statement is needed — use fully-qualified paths as shown.

**Depends on:** TASK-6 (signal-hook must be in Cargo.toml first)
**Verification:** `cargo test -p workflow_core`

---

## TASK-8: Move `tempfile` to dev-dependencies in `workflow-cli/Cargo.toml`

**File:** `workflow-cli/Cargo.toml`
**Before:**
```toml
[dependencies]
workflow_core = { path = "../workflow_core" }
clap = { workspace = true }
anyhow = { workspace = true }
tempfile = "3.27.0"
```
**After:**
```toml
[dependencies]
workflow_core = { path = "../workflow_core" }
clap = { workspace = true }
anyhow = { workspace = true }

[dev-dependencies]
tempfile = "3.27.0"
```
**Verification:** `cargo check -p workflow-cli && cargo test -p workflow-cli`

---

## TASK-9: Consolidate stale test file `executor_tests_updated.rs`

**IMPORTANT ordering:** Do Step B (delete) BEFORE Step A (add). If Step A is applied first, both
files exist simultaneously with duplicate test function names, causing a compile error.

**Step B — Delete the stale file first:**
Delete the file: `workflow_utils/tests/executor_tests_updated.rs`

**Step A — Add unique test and import to `executor_tests.rs`:**
File: `workflow_utils/tests/executor_tests.rs`

Update the import line:
- Before: `use workflow_utils::TaskExecutor;`
- After: `use workflow_utils::{TaskExecutor, ExecutionHandle};`

Append at end of file:
```rust

#[test]
fn test_execution_handle_pid() {
    let handle = TaskExecutor::new("/tmp")
        .command("echo")
        .arg("hello")
        .spawn()
        .unwrap();
    let pid = handle.pid();
    assert!(pid > 0);
}
```

**Verification:** `cargo test -p workflow_utils && cargo check --all-targets -p workflow_utils`

---

## TASK-10: Remove unused `bon` workspace dependency

**File:** `Cargo.toml` (workspace root)
**Target:** `[workspace.dependencies]` section
**Remove the line:** `bon = "3.9.1"`
**Note:** If TASK-6 also edits this file (adding `signal-hook`), do both edits in one pass.
**Verification:** `cargo check --workspace`

---

## TASK-11: Fix `failed_task_skips_dependent` test to assert in-memory skip state

**File:** `workflow_core/src/workflow.rs`
**Target:** Test function `fn failed_task_skips_dependent`, after `wf.run(state.as_mut(), runner,
executor)?;` and before `let state = JsonStateStore::load(state_path).unwrap();`

**Before (the block after `wf.run(...)`):**
```rust
    wf.run(state.as_mut(), runner, executor)?;

    let state = JsonStateStore::load(state_path).unwrap();
    // After load, SkippedDueToDependencyFailure resets to Pending for crash recovery
    assert!(matches!(state.get_status("b"), Some(TaskStatus::Pending)));
```
**After:**
```rust
    wf.run(state.as_mut(), runner, executor)?;

    // Verify in-memory state shows skip propagation actually worked
    assert!(matches!(
        state.get_status("b"),
        Some(TaskStatus::SkippedDueToDependencyFailure)
    ));

    let state = JsonStateStore::load(state_path).unwrap();
    // After load, SkippedDueToDependencyFailure resets to Pending for crash recovery
    assert!(matches!(state.get_status("b"), Some(TaskStatus::Pending)));
```
**Verification:** `cargo test -p workflow_core -- failed_task_skips_dependent`

---

## Execution Order

| Phase | Tasks | Notes |
|-------|-------|-------|
| Phase 1 (parallel) | TASK-8, TASK-9 | Different files from all other tasks |
| Phase 1 (parallel) | TASK-1 + TASK-2 merged | Same file `state.rs` — do in one session |
| Phase 1 (parallel) | TASK-6 + TASK-10 merged | Same file root `Cargo.toml` — do in one session |
| Phase 2 (sequential) | TASK-3 → TASK-4 → TASK-5 | All touch `workflow.rs`; TASK-7 and TASK-11 can be interleaved after TASK-3 |
| Phase 2 (parallel with above) | TASK-7, TASK-11 | Touch `workflow.rs`; must not overlap with each other or 3/4/5 |
| Final | — | `cargo test --workspace` |

## Dependency Graph

```
TASK-1 → TASK-2
TASK-3 → TASK-4 → TASK-5
TASK-6 → TASK-7
TASK-8, TASK-9, TASK-10, TASK-11: independent
```

## Risk Flags

- **`workflow.rs` file conflicts**: TASK-3, TASK-4, TASK-5, TASK-7, TASK-11 all touch
  `workflow_core/src/workflow.rs`. Assign to a single subagent or enforce strict serialization.
- **Root `Cargo.toml` conflicts**: TASK-6 and TASK-10 both edit root `Cargo.toml`. Merge into one
  edit pass.
- **TASK-5 workdir**: The collect closure receives `Path::new(".")` rather than the task's actual
  workdir (which is consumed at dispatch time). A more precise fix would stash `task.workdir` as a
  fifth tuple element in `handles`, but that is out of scope for this plan.
