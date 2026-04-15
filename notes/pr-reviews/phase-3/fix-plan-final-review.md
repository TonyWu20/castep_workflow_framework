# Fix Plan: phase-3 final review (2026-04-15)

## Issues addressed

13 confirmed issues from the post-fix review. Ordered by dependency.

---

## Execution Order

```
Phase 1 (parallel):  TASK-1, TASK-2, TASK-3, TASK-4, TASK-5
Phase 2 (after TASK-2): TASK-6
Phase 3 (parallel, after TASK-1): TASK-7, TASK-8
Phase 4 (parallel, after TASK-2 + TASK-3): TASK-9, TASK-10, TASK-11
Phase 5 (after all): cargo test --workspace
```

---

## Phase 1 — Independent fixes (run in parallel)

### TASK-1: Fix signal handler to re-register on every `run()` call

**File:** `workflow_core/src/workflow.rs`
**Target:** `static SIGNAL_INIT: std::sync::Once` block inside `pub fn run`
**Severity:** Blocking

**Problem:** `SIGNAL_INIT.call_once(...)` only fires once per process lifetime. The second `Workflow` instance's `interrupt` field is never registered with the signal handler, making it non-interruptible.

**Before** (lines 64–68):

```rust
static SIGNAL_INIT: std::sync::Once = std::sync::Once::new();
SIGNAL_INIT.call_once(|| {
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&self.interrupt)).ok();
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&self.interrupt)).ok();
});
```

**After** (remove the `Once` wrapper entirely; call register directly):

```rust
signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&self.interrupt)).ok();
signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&self.interrupt)).ok();
```

`signal_hook::flag::register` is idempotent for multiple registrations on the same flag and safe to call on every `run()` invocation.

**Verification:** `cargo check -p workflow_core`

---

### TASK-2: Add `JsonStateStore::load_raw()` for CLI read-only access

**File:** `workflow_core/src/state.rs`
**Target:** `impl JsonStateStore`, after the existing `load()` method
**Severity:** Blocking

**Problem:** `load()` resets `Failed`, `Running`, and `SkippedDueToDependencyFailure` to `Pending`. The CLI's `load_state()` calls `load()`, so `status` and `inspect` subcommands always show `Pending` instead of the actual failure state.

**Before:** No `load_raw` method exists.

**After** — within the first `impl JsonStateStore` block (the one starting with `pub fn new`), add this method immediately after the closing `}` of `pub fn load(...)`. The surrounding context to locate the insertion point:

```rust
    pub fn load(path: impl AsRef<Path>) -> Result<Self, WorkflowError> {
        // ... (existing body)
        Ok(state)
    }
    // ← INSERT HERE, before workflow_name() and path() methods

    /// Loads state from disk without applying crash-recovery resets.
    /// Use this for read-only inspection (CLI status/inspect).
    pub fn load_raw(path: impl AsRef<Path>) -> Result<Self, WorkflowError> {
        let state: Self = serde_json::from_slice(&fs::read(path).map_err(WorkflowError::Io)?)?;
        Ok(state)
    }
```

**Verification:** `cargo check -p workflow_core`

---

### TASK-3: Fix `Dag::add_edge` inverted error fields on missing `from` node

**File:** `workflow_core/src/dag.rs`
**Target:** First `ok_or_else` call inside `pub fn add_edge`
**Severity:** Blocking

**Problem:** When the `from` (dependency) node is missing, the error is constructed with `task: from` and `dependency: to`. The message template is `"unknown dependency '{dependency}' in task '{task}'"`, which renders the fields in the wrong roles.

**Before** (lines 39–42):

```rust
.ok_or_else(|| WorkflowError::UnknownDependency {
    task: from.to_string(),
    dependency: to.to_string(),
})?;
```

**After:**

```rust
.ok_or_else(|| WorkflowError::UnknownDependency {
    task: to.to_string(),
    dependency: from.to_string(),
})?;
```

The second `ok_or_else` block (lines 46–49, missing `to` node) is already correct and must not be changed.

**Verification:** `cargo check -p workflow_core`; also run `cargo test -p workflow_core --lib dag::tests`

---

### TASK-4: Remove unused `anyhow` dependency from `workflow_core`

**File:** `workflow_core/Cargo.toml`
**Target:** `anyhow` line in `[dependencies]`
**Severity:** Major

**Problem:** `anyhow` is listed as a dependency but is not used anywhere in `workflow_core` source code.

**Before** (line 7):

```toml
anyhow = { workspace = true }
```

**After:** delete that line entirely.

**Verification:** `cargo check -p workflow_core`

---

### TASK-5: Fix `Queued` arm to return an error instead of panicking

**File:** `workflow_core/src/workflow.rs`
**Target:** `ExecutionMode::Queued { .. } =>` match arm inside `Workflow::run`
**Severity:** Minor

**Problem:** `Queued` is a public enum variant users can construct. Hitting it panics via `unreachable!()` instead of returning a graceful error.

**Before** (lines 304–306):

```rust
ExecutionMode::Queued { .. } => {
    unreachable!("Queued execution mode is not yet implemented");
}
```

**After:**

```rust
ExecutionMode::Queued { .. } => {
    return Err(WorkflowError::InvalidConfig(
        "Queued execution mode is not yet implemented".into(),
    ));
}
```

**Verification:** `cargo check -p workflow_core`

---

## Phase 2 — CLI `load_state` fix (after TASK-2)

### TASK-6: Use `load_raw` in CLI `load_state` for status/inspect commands

**File:** `workflow-cli/src/main.rs`
**Target:** `fn load_state`
**Severity:** Blocking

**Problem:** `load_state` calls `JsonStateStore::load()` (crash-recovery path) for all commands. `status` and `inspect` must see the actual on-disk state, including `Failed` tasks.

**Before** (lines 25–27):

```rust
fn load_state(path: &str) -> anyhow::Result<JsonStateStore> {
    JsonStateStore::load(path)
        .map_err(|_| anyhow::anyhow!("error: state file not found: {}", path))
}
```

**After:**

```rust
fn load_state_raw(path: &str) -> anyhow::Result<JsonStateStore> {
    JsonStateStore::load_raw(path)
        .map_err(|e| anyhow::anyhow!("failed to open state file '{}': {}", path, e))
}

fn load_state_for_resume(path: &str) -> anyhow::Result<JsonStateStore> {
    JsonStateStore::load(path)
        .map_err(|e| anyhow::anyhow!("failed to open state file '{}': {}", path, e))
}
```

Then update `fn main` to use the appropriate loader per command:
- `Commands::Status { state_file }` → `load_state_raw(&state_file)?`
- `Commands::Inspect { state_file, .. }` → `load_state_raw(&state_file)?`
- `Commands::Retry { state_file, .. }` → `load_state_for_resume(&state_file)?`

**Verification:** `cargo check -p workflow-cli`; update the `status_output_format` test to call `load_state_raw` so the bug regression is caught:

Add a new test after `status_output_format`:

```rust
#[test]
fn status_shows_failed_after_load_raw() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = make_state(dir.path());
    s.save().unwrap();
    let loaded = JsonStateStore::load_raw(dir.path().join("state.json").to_str().unwrap()).unwrap();
    let out = cmd_status(&loaded);
    assert!(out.contains("task_b: Failed (exit code 1)"));
}
```

---

## Phase 3 — Style/quality fixes (parallel, after TASK-1 and TASK-2)

### TASK-7: Re-export `TaskStatus` from `workflow_core` crate root

**File:** `workflow_core/src/lib.rs`
**Target:** `pub use state::{...}` line
**Severity:** Minor

**Problem:** `TaskStatus` is the most commonly matched type but must be imported as `workflow_core::state::TaskStatus`. All other key types are re-exported from the root.

**Before** (line 12):

```rust
pub use state::{JsonStateStore, StateStore, StateStoreExt, StateSummary};
```

**After:**

```rust
pub use state::{JsonStateStore, StateStore, StateStoreExt, StateSummary, TaskStatus};
```

**Verification:** `cargo check --workspace`

---

### TASK-8: Fix `TaskTimeout` variant — use it in the timeout code path

**File:** `workflow_core/src/workflow.rs`
**Target:** The timeout branch inside the `for (id, (handle, start, _, _)) in handles.iter_mut()` loop
**Severity:** Major

**Problem:** `WorkflowError::TaskTimeout(String)` is defined but never constructed. The timeout path uses `state.mark_failed(id, format!("task '{}' timed out after {:?}", id, timeout))`. Downstream code cannot programmatically distinguish timeouts from other failures by error type.

The `TaskStatus::Failed { error }` string is what the summary exposes, so change the error string passed to `mark_failed` to include a sentinel that is also reflected in `WorkflowError::TaskTimeout`. The cleanest fix is to keep `mark_failed` (it goes into the persisted state) but also surface the `TaskTimeout` variant in the `WorkflowSummary` by changing `WorkflowSummary::failed` to carry enough info. However, a minimal fix consistent with current design: change the error string so it matches the `TaskTimeout` display format.

**Before** (lines 115–118):

```rust
state.mark_failed(
    id,
    format!("task '{}' timed out after {:?}", id, timeout),
);
```

**After:**

```rust
state.mark_failed(
    id,
    WorkflowError::TaskTimeout(id.clone()).to_string(),
);
```

This makes the stored error message match the `#[error("task '{0}' timed out")]` display string, giving consistent wording and making `TaskTimeout` the canonical source of the message.

**Verification:** `cargo test -p workflow_core --test timeout_integration`; verify the test assertion `assert!(err.contains("timed out"))` still passes.

---

## Phase 4 — Clippy/lint fixes (parallel, after TASK-2 and TASK-3)

### TASK-9: Fix type_complexity warnings in `task.rs`

**File:** `workflow_core/src/task.rs`
**Target:** `pub setup` and `pub collect` fields on `pub struct Task`
**Severity:** Minor

**Problem:** `clippy::type_complexity` on both closure fields.

**Before** (lines 28–29):

```rust
pub setup: Option<Box<dyn Fn(&Path) -> Result<(), WorkflowError> + Send + Sync>>,
pub collect: Option<Box<dyn Fn(&Path) -> Result<(), WorkflowError> + Send + Sync>>,
```

**After** — add a type alias before the `Task` struct definition:

```rust
/// A closure used for task setup or result collection.
pub type TaskClosure = Box<dyn Fn(&Path) -> Result<(), WorkflowError> + Send + Sync>;
```

Then change the fields to:

```rust
pub setup: Option<TaskClosure>,
pub collect: Option<TaskClosure>,
```

Also update `workflow_core/src/workflow.rs` line 87 to use the alias:

**Before** (line 87):

```rust
Option<Box<dyn Fn(&Path) -> Result<(), WorkflowError> + Send + Sync>>,
```

**After:**

```rust
Option<crate::task::TaskClosure>,
```

**Verification:** `cargo clippy -p workflow_core 2>&1 | grep type_complexity` should return no matches.

---

### TASK-10: Fix unused import in `executor_tests.rs`

**File:** `workflow_utils/tests/executor_tests.rs`
**Target:** `use` statement on line 2
**Severity:** Minor

**Problem:** `ExecutionHandle` is imported but not explicitly used (type is inferred from `.spawn()`).

**Before** (line 2):

```rust
use workflow_utils::{TaskExecutor, ExecutionHandle};
```

**After:**

```rust
use workflow_utils::TaskExecutor;
```

**Verification:** `cargo clippy -p workflow_utils --tests 2>&1 | grep unused_imports` should return no matches.

---

### TASK-11: Fix `partialeq_to_none` lint in `process_tests.rs`

**File:** `workflow_utils/tests/process_tests.rs`
**Target:** Line 55 inside `test_terminate_long_running_process`
**Severity:** Minor

**Problem:** `result.exit_code == None` triggers `clippy::partialeq_to_none`.

**Before** (line 55):

```rust
assert!(result.exit_code.is_some() || result.exit_code == None);  // Either has code or was killed by signal
```

**After:**

```rust
assert!(result.exit_code.is_some() || result.exit_code.is_none());  // Either has code or was killed by signal
```

**Verification:** `cargo clippy -p workflow_utils --tests 2>&1 | grep partialeq_to_none` should return no matches.

---

## Phase 5 — Final

```bash
cargo test --workspace
cargo clippy --workspace --all-targets 2>&1 | grep -E "^error"
```

Both must exit 0.

---

## Dependency Graph

```
TASK-1 ─────────────────────────────────────────────────────────────┐
TASK-2 ──────────────────────────────────────────┬─── TASK-6        │
TASK-3 ──────────────────────────────────────────┤                   │
TASK-4 (independent)                             │                   │
TASK-5 (independent)                             │                   │
                                                 ├─── TASK-9         │
                                                 ├─── TASK-10        │
                                                 └─── TASK-11        │
                                      TASK-7 (after TASK-1) ────────┤
                                      TASK-8 (after TASK-1) ────────┤
                                                                     ▼
                                                               cargo test
```

| Phase | Tasks | Dependency |
|---|---|---|
| 1 (parallel) | TASK-1, TASK-2, TASK-3, TASK-4, TASK-5 | none |
| 2 | TASK-6 | TASK-2 |
| 3 (parallel) | TASK-7, TASK-8 | TASK-1 |
| 4 (parallel) | TASK-9, TASK-10, TASK-11 | TASK-2, TASK-3 |
| 5 | `cargo test --workspace` | all |
