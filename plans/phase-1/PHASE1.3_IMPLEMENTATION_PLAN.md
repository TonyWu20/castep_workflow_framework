# Phase 1.3 Implementation Plan: Integration & Examples

## Context

Phase 1.1 (workflow_utils) and Phase 1.2 (workflow_core revision) are complete. Phase 1.3 validates that the two crates compose correctly under realistic conditions and produces the canonical user-facing example. It also fixes a latent resume bug discovered during planning.

**Goals:**
1. Fix a bug where `Running` tasks from an interrupted run are never re-dispatched on resume
2. Add `examples/hubbard_u_sweep` as a workspace member binary crate (Layer 3 reference implementation)
3. Add three integration tests covering the sweep pattern, resume semantics, and DAG ordering/failure propagation

## Critical Files

**To modify:**
- `/Users/tony/programming/castep_workflow_framework/Cargo.toml` — add `"examples/hubbard_u_sweep"` to `members`
- `/Users/tony/programming/castep_workflow_framework/workflow_core/src/state.rs` — fix `Running → Pending` reset in `load`

**To create:**
- `/Users/tony/programming/castep_workflow_framework/examples/hubbard_u_sweep/Cargo.toml`
- `/Users/tony/programming/castep_workflow_framework/examples/hubbard_u_sweep/src/main.rs`
- `/Users/tony/programming/castep_workflow_framework/workflow_core/tests/resume.rs`
- `/Users/tony/programming/castep_workflow_framework/workflow_core/tests/dependencies.rs`
- `/Users/tony/programming/castep_workflow_framework/workflow_core/tests/hubbard_u_sweep.rs`
- `/Users/tony/programming/castep_workflow_framework/workflow_core/tests/bin/mock_castep`

## Key API Facts (verified against source)

- `workflow_utils` re-exports functions **flat** at crate root: `use workflow_utils::{TaskExecutor, create_dir, write_file};` — the `files` module is private
- `Workflow` uses `bon` builder: `Workflow::builder().name("...").state_dir(".").build()?`
- `Workflow::resume(name, state_dir)` — two args; restores state path only, not closures; tasks must be re-registered via `add_task` after resume
- Task closure signature: `Fn() -> anyhow::Result<()> + Send + Sync + 'static`
- `TaskExecutor::execute()` returns `Result<ExecutionResult>`; must check `result.success()` and bail on non-zero
- `TaskExecutor::env(key, val)` exists and is the correct way to inject PATH in tests (not `std::env::set_var`)
- `tempfile` is already in `workflow_core` dev-dependencies

## Tasks

### TASK-1: Fix `Running → Pending` reset in `WorkflowState::load`

**File:** `workflow_core/src/state.rs`

In `WorkflowState::load`, after `serde_json::from_slice` and before `Ok(state)`, add:
```rust
for status in state.tasks.values_mut() {
    if matches!(status, TaskStatus::Running) {
        *status = TaskStatus::Pending;
    }
}
```

Add a unit test in the existing `#[cfg(test)]` block:
- Create `WorkflowState::new`, set one task to `Running`, save, reload, assert status is `Pending`.

**Verify:** `cargo test -p workflow_core`

---

### TASK-2: Add `examples/hubbard_u_sweep` workspace member

**File:** `/Cargo.toml` — append `"examples/hubbard_u_sweep"` to `members`

**New file:** `examples/hubbard_u_sweep/Cargo.toml`
```toml
[package]
name = "hubbard_u_sweep"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "hubbard_u_sweep"
path = "src/main.rs"

[dependencies]
workflow_core = { path = "../../workflow_core" }
workflow_utils = { path = "../../workflow_utils" }
anyhow = "1"
# castep-cell-io = "0.4.0"  # TODO: add after API verification
```

**Verify:** `cargo metadata --no-deps | grep hubbard_u_sweep`

---

### TASK-3: Implement `examples/hubbard_u_sweep/src/main.rs`

**Depends on:** TASK-2

```rust
use anyhow::Result;
use workflow_core::Workflow;
use workflow_utils::{TaskExecutor, create_dir, write_file};

fn main() -> Result<()> {
    let mut workflow = Workflow::builder()
        .name("hubbard_u_sweep")
        .state_dir(".")
        .build()?;

    for u in [0.0_f64, 1.0, 2.0, 3.0, 4.0, 5.0] {
        let task_id = format!("scf_U{:.1}", u);
        let workdir = format!("runs/U{:.1}", u);

        let task = workflow_core::Task::new(&task_id, move || {
            create_dir(&workdir)?;

            // TODO: replace with castep-cell-io builders
            let cell_content = format!(
                "%BLOCK LATTICE_CART\n  3.25 0.0 0.0\n  0.0 3.25 0.0\n  0.0 0.0 5.21\n%ENDBLOCK LATTICE_CART\n"
            );
            write_file(format!("{}/ZnO.cell", workdir), &cell_content)?;
            write_file(format!("{}/ZnO.param", workdir), "task : SinglePoint\n")?;

            let result = TaskExecutor::new(&workdir)
                .command("castep")
                .arg("ZnO")
                .execute()?;
            if !result.success() {
                anyhow::bail!("castep failed: {:?}\n{}", result.exit_code, result.stderr);
            }
            Ok(())
        });

        workflow.add_task(task)?;
    }

    workflow.run()
}
```

**Verify:** `cargo check -p hubbard_u_sweep`

---

### TASK-4: Integration test — resume semantics

**File:** `workflow_core/tests/resume.rs`  
**Depends on:** TASK-1

```rust
use std::sync::{Arc, Mutex};
use tempfile::tempdir;
use workflow_core::{Task, Workflow, state::{TaskStatus, WorkflowState}};

#[test]
fn test_resume_skips_completed_reruns_interrupted() {
    let dir = tempdir().unwrap();
    let mut state = WorkflowState::new("test_resume");
    state.tasks.insert("a".into(), TaskStatus::Completed);
    state.tasks.insert("b".into(), TaskStatus::Running); // simulates crash mid-b
    state.save(dir.path()).unwrap();

    let log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

    let mut wf = Workflow::resume("test_resume", dir.path()).unwrap();

    let log_a = log.clone();
    wf.add_task(Task::new("a", move || { log_a.lock().unwrap().push("a".into()); Ok(()) })).unwrap();

    let log_b = log.clone();
    wf.add_task(Task::new("b", move || { log_b.lock().unwrap().push("b".into()); Ok(()) })).unwrap();

    wf.run().unwrap();

    let ran = log.lock().unwrap();
    assert!(!ran.contains(&"a".to_string()), "task 'a' should not re-run");
    assert_eq!(ran.iter().filter(|x| *x == "b").count(), 1, "task 'b' should run exactly once");
}
```

**Verify:** `cargo test -p workflow_core --test resume`

---

### TASK-5: Integration test — diamond dependency and failure propagation

**File:** `workflow_core/tests/dependencies.rs`

Two `#[test]` functions:

1. **`test_diamond_ordering`**: DAG `a→b, a→c, b→d, c→d`. Shared `Arc<Mutex<Vec<String>>>` log. After `run()`, assert `pos("a") < pos("b")`, `pos("a") < pos("c")`, `pos("b") < pos("d")`, `pos("c") < pos("d")`. Do not assert "b" vs "c".

2. **`test_failure_propagation`**: Same DAG. Task "a" returns `Err`. After `run()`, call `WorkflowState::load(state_path)` and assert "b", "c", "d" are all `TaskStatus::SkippedDueToDependencyFailure`.

Both use `tempfile::tempdir()` for state isolation.

**Verify:** `cargo test -p workflow_core --test dependencies`

---

### TASK-6: Integration test — Hubbard-U sweep with mock CASTEP

**New script:** `workflow_core/tests/bin/mock_castep`
```sh
#!/bin/sh
echo "mock castep: $@" > "${1}.castep"
exit 0
```
Must be executable (`chmod +x workflow_core/tests/bin/mock_castep`).

**File:** `workflow_core/tests/hubbard_u_sweep.rs`

- Registers 3 tasks (U=0.0, 1.0, 2.0) for test brevity
- Workdirs: `tempdir.path().join(format!("runs/U{:.1}", u))` (absolute paths)
- PATH injection:
  ```rust
  let bin_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/bin");
  let path = format!("{}:{}", bin_dir.display(), std::env::var("PATH").unwrap_or_default());
  // pass via TaskExecutor::env("PATH", &path) inside each closure
  ```
- After `workflow.run()`, load `WorkflowState` and assert all 3 task statuses are `Completed`
- Assert each `tempdir.path().join(format!("runs/U{:.1}/ZnO.castep", u))` exists

**Verify:** `cargo test -p workflow_core --test hubbard_u_sweep`

---

## Execution Phases

| Phase | Tasks | Notes |
|-------|-------|-------|
| 1 (parallel) | TASK-1, TASK-2, TASK-5, TASK-6 | No inter-dependencies |
| 2 (parallel) | TASK-3, TASK-4 | TASK-3 needs TASK-2; TASK-4 needs TASK-1 |

## Verification

After all tasks:
1. `cargo test --workspace` — all tests pass
2. `cargo check -p hubbard_u_sweep` — example compiles clean
3. `cargo test -p workflow_core --test resume` — resume bug fix verified
4. `cargo test -p workflow_core --test dependencies` — DAG ordering and failure propagation verified
5. `cargo test -p workflow_core --test hubbard_u_sweep` — end-to-end sweep with mock CASTEP verified

## Pipeline Summary

- **Rust-architect elaboration**: Identified 3 compilation errors in the original plan's example code (wrong `Workflow::new` call, unverified castep-cell-io API, missing exit-code check), the `Running→Pending` resume bug, and the need for a workspace-member example crate.
- **Reviewer iteration 1**: Flagged Cargo test discovery ambiguity (`tests/integration/*.rs` not auto-discovered) and missing `files::` import path. Resolved by switching to top-level `tests/*.rs` and specifying flat imports.
- **Architect final review**: Flagged `files::create_dir` (private module — use flat imports), missing re-registration of tasks after `resume` in TASK-4, and ambiguous "three tasks" in TASK-6. All three corrected.
- **Reviewer iteration 2**: All tasks CLEAR. Verdict: **Ready to Implement**.
