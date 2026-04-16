# Fix: TASK-9+11 Unit Test Compilation Failures

## Context

TASK-9+11 rewrote `workflow_core` (new `ExecutionMode`-based `Task`, new `Workflow::run()` with DI). `cargo check -p workflow_core` passes. `cargo test -p workflow_core` fails with compilation errors across five files. These fixes address all of them in one atomic commit.

**Verification that `run()` saves to disk**: `Workflow::run()` calls `state.save()?` at lines 68, 96, 129, and 151 of `workflow.rs`. State is always on disk when the loop exits.

**Verification that `Task::setup` is infallible builder**: `pub fn setup<F>(mut self, f: F) -> Self` — returns `Self`, no `Result`. Chains like `.workdir(...).setup(...).add_monitor(...)` are valid.

---

## Fix 1 — `workflow_utils/src/lib.rs`: Add two missing re-exports

**File**: `workflow_utils/src/lib.rs`

**Why**: Integration tests `resume.rs` and `dependencies.rs` reference `workflow_utils::SystemProcessRunner` and `workflow_utils::ShellHookExecutor`. `SystemProcessRunner` is a `pub struct` in `workflow_utils/src/executor.rs` (lines 114–138) but never re-exported. `ShellHookExecutor` is a `pub struct` in `workflow_utils/src/monitoring.rs` (line 30) that implements `workflow_core::HookExecutor` — the local `workflow_utils` version, not the `workflow_core` version. Re-export the local one.

**Before** (complete file, 10 lines):
```rust
mod executor;
mod files;
mod monitoring;

pub use executor::{ExecutionHandle, ExecutionResult, TaskExecutor};
pub use files::{copy_file, create_dir, exists, read_file, remove_dir, write_file};
// Re-export hook types from workflow_core for backward compatibility
pub use monitoring::{execute_hook};
pub use workflow_core::{HookContext, HookResult, HookTrigger, MonitoringHook};
```

**After** (complete file):
```rust
mod executor;
mod files;
mod monitoring;

pub use executor::{ExecutionHandle, ExecutionResult, TaskExecutor, SystemProcessRunner};
pub use files::{copy_file, create_dir, exists, read_file, remove_dir, write_file};
// Re-export hook types from workflow_core for backward compatibility
pub use monitoring::{execute_hook, ShellHookExecutor};
pub use workflow_core::{HookContext, HookResult, HookTrigger, MonitoringHook};
```

Two changes only:
1. Add `SystemProcessRunner` to the `executor` re-export on line 5
2. Add `ShellHookExecutor` to the `monitoring` re-export on line 8

**No circular dependency**: `workflow_utils` already depends on `workflow_core` as a runtime dependency (confirmed in `workflow_utils/Cargo.toml`). `workflow_core` does not depend on `workflow_utils`.

**Important**: Do NOT write `pub use workflow_core::ShellHookExecutor`. The `workflow_utils::monitoring::ShellHookExecutor` (the local one, which uses `TaskExecutor` internally) is what integration tests need. The `workflow_core::ShellHookExecutor` is a different struct.

---

## Fix 2 — `workflow_core/src/workflow.rs`: Fix test module

**File**: `workflow_core/src/workflow.rs`, `#[cfg(test)] mod tests` block (starts at line 240).

**Why**: The test module uses `SystemProcessRunner` and `ShellHookExecutor` bare without importing them, and uses the invalid `workflow_core::` prefix from inside the crate itself.

**Four changes needed** (in order):

### Change 2a — Add imports after `use super::*;` (line 242)

**Before** (lines 241–244):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::io::Write;
```

**After**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::ShellHookExecutor;
    use crate::process::SystemProcessRunner;
    use std::collections::HashMap;
    use std::io::Write;
```

### Change 2b — Fix `chain_respects_order`: use-after-move on `log_file` (lines 277–323)

`log_file` is created once but moved into the first `move` closure (line 289), then used again in the second `move` closure (line 310) and at line 323. This will not compile.

**Before** (lines 276–278):
```rust
        let dir = tempfile::tempdir().unwrap();
        let log_file = dir.path().join("log.txt");
```

**After**:
```rust
        let dir = tempfile::tempdir().unwrap();
        let log_file = dir.path().join("log.txt");
        let log_for_a = log_file.clone();
        let log_for_b = log_file.clone();
```

Then update the first closure (line 289) to capture `log_for_a`:

**Before** (line 289):
```rust
            .setup(move |_| {
                let mut f = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_file)?;
```

**After**:
```rust
            .setup(move |_| {
                let mut f = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_for_a)?;
```

And update the second closure (line 307) to capture `log_for_b`:

**Before** (line 307):
```rust
            .setup(move |_| {
                let mut f = std::fs::OpenOptions::new()
                    .append(true)
                    .open(&log_file)?;
```

**After**:
```rust
            .setup(move |_| {
                let mut f = std::fs::OpenOptions::new()
                    .append(true)
                    .open(&log_for_b)?;
```

`log_file` on line 323 (`read_to_string(&log_file)`) remains unchanged — it's the original that was not moved.

### Change 2c — Fix `failed_task_skips_dependent`: use-after-move on `state_path` (lines 357–362)

`JsonStateStore::new` takes `PathBuf` by value (consuming it). Then `JsonStateStore::load(state_path)` reuses the consumed variable.

**Before** (line 358):
```rust
        let mut state = Box::new(JsonStateStore::new("wf_skip", state_path));
```

**After**:
```rust
        let mut state = Box::new(JsonStateStore::new("wf_skip", state_path.clone()));
```

### Change 2d — Fix `resume_loads_existing_state`: invalid `workflow_core::` self-references (lines 490, 504)

**Before** (line 490):
```rust
        wf1.run(state1.as_mut(), Arc::new(workflow_core::SystemProcessRunner), Arc::new(workflow_core::ShellHookExecutor))?;
```

**After**:
```rust
        wf1.run(state1.as_mut(), Arc::new(SystemProcessRunner), Arc::new(ShellHookExecutor))?;
```

**Before** (line 504):
```rust
        wf2.run(state2.as_mut(), Arc::new(workflow_core::SystemProcessRunner), Arc::new(workflow_core::ShellHookExecutor))?;
```

**After**:
```rust
        wf2.run(state2.as_mut(), Arc::new(SystemProcessRunner), Arc::new(ShellHookExecutor))?;
```

Note: `SystemProcessRunner` and `ShellHookExecutor` here resolve to `workflow_core`'s own types (imported via `crate::process::SystemProcessRunner` and `crate::monitoring::ShellHookExecutor` added in Change 2a). These are the types that `workflow_core` owns and can instantiate in its own tests without depending on `workflow_utils`.

---

## Fix 3 — `workflow_core/tests/hubbard_u_sweep.rs`: Full rewrite

**File**: `workflow_core/tests/hubbard_u_sweep.rs`

**Why**: Uses removed APIs: `Workflow::resume()`, closure-based `Task::new(&id, move || {...})`, `workflow.run()` with no args.

**Prerequisite**: Fix 1 must be applied first (`workflow_utils::SystemProcessRunner` and `workflow_utils::ShellHookExecutor` must exist).

**Note on `mock_castep`**: Check whether `workflow_core/tests/bin/mock_castep` exists and is executable before running. If the binary is missing, this test will fail at runtime (not compile time) with a "command not found" error. The binary must be a shell script or executable in that directory that accepts `ZnO` as an argument and creates `ZnO.castep` in the working directory.

**Note on error mapping**: Inside `.setup()` closures, `workflow_utils::{create_dir, write_file}` return `anyhow::Result`. Map them to `WorkflowError` via `std::io::Error::other(e.to_string())`. `std::io::Error::other()` was stabilized in Rust 1.74 — verify your toolchain with `rustc --version`. `write_file` accepts `&str` as its second argument (confirmed: `pub fn write_file(path: impl AsRef<Path>, content: &str) -> Result<()>`), so string literals pass directly.

**Note on `StateStore` import**: The replacement file imports `workflow_core::state::StateStore`. This import is **required** — `get_status()` is defined on the `StateStore` trait, not as an inherent method on `JsonStateStore`. Without this import in scope, `state.get_status(...)` will fail to compile with "method not found". Do not remove it.

**Complete replacement file**:

```rust
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::tempdir;
use workflow_core::{
    ExecutionMode, JsonStateStore, Task, Workflow, WorkflowError,
    state::{TaskStatus, StateStore},
};
use workflow_utils::{create_dir, write_file};

#[test]
fn test_hubbard_u_sweep_with_mock_castep() {
    let dir = tempdir().unwrap();
    let state_path = dir.path().join(".hubbard_u.workflow.json");

    let bin_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/bin");
    let path_val = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    let mut wf = Workflow::new("hubbard_u");

    for u in [0.0_f64, 1.0, 2.0] {
        let task_id = format!("scf_U{:.1}", u);
        let abs_workdir = dir.path().join(format!("runs/U{:.1}", u));
        let workdir_for_setup = abs_workdir.clone();
        let path_clone = path_val.clone();

        let mut env = HashMap::new();
        env.insert("PATH".to_string(), path_clone);

        wf.add_task(
            Task::new(
                &task_id,
                ExecutionMode::Direct {
                    command: "mock_castep".into(),
                    args: vec!["ZnO".into()],
                    env,
                    timeout: None,
                },
            )
            .workdir(abs_workdir.clone())
            .setup(move |_| {
                create_dir(&workdir_for_setup).map_err(|e| {
                    WorkflowError::Io(std::io::Error::other(e.to_string()))
                })?;
                write_file(
                    workdir_for_setup.join("ZnO.cell"),
                    "%BLOCK LATTICE_CART\n  3.25 0.0 0.0\n  0.0 3.25 0.0\n  0.0 0.0 5.21\n%ENDBLOCK LATTICE_CART\n",
                )
                .map_err(|e| WorkflowError::Io(std::io::Error::other(e.to_string())))?;
                write_file(
                    workdir_for_setup.join("ZnO.param"),
                    "task : SinglePoint\n",
                )
                .map_err(|e| WorkflowError::Io(std::io::Error::other(e.to_string())))?;
                Ok(())
            }),
        )
        .unwrap();
    }

    let runner = Arc::new(workflow_utils::SystemProcessRunner);
    let executor = Arc::new(workflow_utils::ShellHookExecutor);
    let mut state = Box::new(JsonStateStore::new("hubbard_u", state_path.clone()));

    wf.run(state.as_mut(), runner, executor).unwrap();

    // Reload from disk to verify final state (run() saves on every status change)
    let state = JsonStateStore::load(&state_path).unwrap();
    for u in [0.0_f64, 1.0, 2.0] {
        let task_id = format!("scf_U{:.1}", u);
        assert!(
            matches!(state.get_status(&task_id), Some(TaskStatus::Completed)),
            "Expected {task_id} to be Completed"
        );
        let castep_file = dir.path().join(format!("runs/U{:.1}/ZnO.castep", u));
        assert!(castep_file.exists(), "Expected {castep_file:?} to exist");
    }
}
```

---

## Fix 4 — Delete `workflow_core/tests/periodic_hooks.rs`

All 4 tests use removed APIs. User confirmed: delete.

```bash
rm workflow_core/tests/periodic_hooks.rs
```

No other files reference this test file. Cargo auto-discovers integration tests in `tests/`, so no `Cargo.toml` change is needed.

---

## Fix 5 — `workflow_core/tests/resume.rs`: Full replacement

**File**: `workflow_core/tests/resume.rs`

**Why**: The test has a dead `log: Arc<Mutex<Vec<String>>>` that is never populated (tasks use shell commands, not closures). The assertions check the always-empty vec, so they pass vacuously — the test proves nothing. Also, commands like `"echo a"` are passed to `Command::new` which treats the entire string as the binary name (no shell splitting), causing a "file not found" error at runtime.

**Disk-flush confirmed**: `Workflow::run()` calls `state.save()?` on every status change, so the state file is always up-to-date when `run()` returns.

**Complete replacement file**:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::tempdir;
use workflow_core::{
    ExecutionMode, JsonStateStore, StateStoreExt, Task, Workflow,
    state::{StateStore, TaskStatus},
};

#[test]
fn test_resume_skips_completed_reruns_interrupted() {
    let dir = tempdir().unwrap();
    let state_path = dir.path().join(".test_resume.workflow.json");

    // First run: complete task a, interrupt task b (simulate crash)
    {
        let mut state = JsonStateStore::new("test_resume", state_path.clone());
        state.mark_completed("a");
        state.mark_running("b"); // simulates crash mid-b
        state.save().unwrap();
    }

    // Second run: resume from saved state.
    // JsonStateStore::load resets Running -> Pending (crash recovery).
    // So on load: a=Completed (skip), b=Pending (will run).
    let mut wf = Workflow::new("test_resume");

    // Task a: already Completed in state, will not be dispatched.
    wf.add_task(Task::new(
        "a",
        ExecutionMode::Direct {
            command: "true".into(),
            args: vec![],
            env: HashMap::new(),
            timeout: None,
        },
    ))
    .unwrap();

    // Task b: was interrupted (Running -> Pending on load), will run and succeed.
    wf.add_task(
        Task::new(
            "b",
            ExecutionMode::Direct {
                command: "true".into(),
                args: vec![],
                env: HashMap::new(),
                timeout: None,
            },
        )
        .depends_on("a"),
    )
    .unwrap();

    let runner = Arc::new(workflow_utils::SystemProcessRunner);
    let executor = Arc::new(workflow_utils::ShellHookExecutor);
    let mut state = Box::new(JsonStateStore::load(&state_path).unwrap());

    wf.run(state.as_mut(), runner, executor).unwrap();

    // Reload from disk to get final state (run() saves on every change)
    let final_state = JsonStateStore::load(&state_path).unwrap();
    // "a" was pre-completed and must NOT have been re-run
    assert!(
        matches!(final_state.get_status("a"), Some(TaskStatus::Completed)),
        "task 'a' should remain Completed (not re-run)"
    );
    // "b" was interrupted, reset to Pending on load, then ran and completed
    assert!(
        matches!(final_state.get_status("b"), Some(TaskStatus::Completed)),
        "task 'b' should have run and completed"
    );
}
```

**Note on `StateStore` import in Fix 5**: The replacement file imports `workflow_core::state::{StateStore, TaskStatus}`. `StateStore` is **required** — `get_status()` is a trait method, not an inherent method on `JsonStateStore`. Without it in scope, `final_state.get_status("a")` will fail to compile. `StateStoreExt` is also imported and required for `mark_completed` and `mark_running` on the manually-constructed state object in the setup block.

**Note on `Workflow::run()` parameter type**: The current signature takes `&mut JsonStateStore` (the concrete type), not `&mut dyn StateStore`. Passing `state.as_mut()` where `state: Box<JsonStateStore>` correctly yields `&mut JsonStateStore`. If `run()` is later refactored to take `&mut dyn StateStore`, these callers will need updating.

Key changes from the original:
- Remove `use std::sync::Mutex;` (no longer needed; keep `Arc` for runner/executor)
- Remove `let log: Arc<Mutex<Vec<String>>> = ...` (line 19)
- Remove `let ran = log.lock().unwrap();` and both assertions (lines 52–54)
- Change task commands from `"echo a"` / `"echo b"` to `"true"` (avoids `Command::new` arg-splitting bug)
- Add `use workflow_core::state::{StateStore, TaskStatus};`
- Add state-based assertions using `JsonStateStore::load` on the saved file

---

## Critical Files

| File | Fix | Change type |
|------|-----|-------------|
| `workflow_utils/src/lib.rs` | 1 | 2-symbol re-export addition |
| `workflow_core/src/workflow.rs` | 2 (a/b/c/d) | 4 targeted edits in test block |
| `workflow_core/tests/hubbard_u_sweep.rs` | 3 | Complete rewrite |
| `workflow_core/tests/periodic_hooks.rs` | 4 | Delete |
| `workflow_core/tests/resume.rs` | 5 | Complete rewrite |

---

## Order of Operations

1. **Fix 1** (`workflow_utils/src/lib.rs`) — unblocks integration test imports
2. **Fix 4** (delete `periodic_hooks.rs`) — no dependencies, do early to reduce noise
3. **Fix 2** (`workflow_core/src/workflow.rs`) — all 4 sub-changes together
4. **Fix 3** (`hubbard_u_sweep.rs`) — requires Fix 1 done first
5. **Fix 5** (`resume.rs`) — requires Fix 1 done first

---

## Verification

```bash
# 1. Check library still compiles (no regressions)
cargo check -p workflow_core
cargo check -p workflow_utils

# 2. Run unit tests (inline test blocks)
cargo test -p workflow_core -- task::tests
cargo test -p workflow_core -- workflow::tests

# 3. Run integration tests individually
cargo test -p workflow_core --test resume
cargo test -p workflow_core --test dependencies
cargo test -p workflow_core --test hubbard_u_sweep

# 4. Full workspace
cargo test --workspace
```

If `hubbard_u_sweep` fails with "command not found: mock_castep", verify that `workflow_core/tests/bin/mock_castep` exists and is executable (`chmod +x`).
