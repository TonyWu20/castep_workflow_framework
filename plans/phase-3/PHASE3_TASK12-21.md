# Phase 3.1 — Enriched Implementation Plan (TASK-12 to TASK-21)

## Dependency Graph

```
TASK-12 ──────────────────────────────────────────────────────► TASK-21
TASK-13 ──────────────────────────────────────────────────────► (workspace compile)
TASK-14 ──────────────────────────────────────────────────────► TASK-18
TASK-15 ──────────────────────────────────────────────────────► TASK-16
TASK-16 ──────────────────────────────────────────────────────► TASK-16b
TASK-16b ─────────────────────────────────────────────────────► TASK-17
TASK-18 ──────────────────────────────────────────────────────► TASK-19
TASK-18 ──────────────────────────────────────────────────────► TASK-20
TASK-19 ──────────────────────────────────────────────────────► TASK-21
TASK-20 ──────────────────────────────────────────────────────► TASK-21
```

## Execution Phases

| Phase              | Tasks                              | Notes                                   |
| ------------------ | ---------------------------------- | --------------------------------------- |
| Phase 1 (parallel) | TASK-12, TASK-13, TASK-14, TASK-15 | Disjoint files                          |
| Phase 2 (parallel) | TASK-16, TASK-18                   | After TASK-15 and TASK-14 respectively  |
| Phase 3            | TASK-16b                           | After TASK-16                           |
| Phase 4 (parallel) | TASK-17, TASK-19, TASK-20          | After TASK-16b and TASK-18 respectively |
| Phase 5            | TASK-21                            | After TASK-12, TASK-19, TASK-20         |

---

## TASK-12: Extend `JsonStateStore::load` reset logic

- **Scope**: Widen the `matches!` guard in `JsonStateStore::load` to reset `Failed { .. }` and `SkippedDueToDependencyFailure` to `Pending`.
- **Crate/Module**: `workflow_core/src/state.rs`
- **Depends On**: None
- **Enables**: TASK-21
- **Can Run In Parallel With**: TASK-13, TASK-14, TASK-15

### Acceptance Criteria

- `Failed { .. }` and `SkippedDueToDependencyFailure` reset to `Pending` on load.
- `Skipped` (explicit) and `Completed` are NOT reset.
- Existing test `load_resets_running_to_pending` extended in-place.
- `cargo test -p workflow_core` passes.

### Implementation

**File**: `workflow_core/src/state.rs` — function `JsonStateStore::load`

```rust
// Before:
for status in state.tasks.values_mut() {
    if matches!(status, TaskStatus::Running) {
        *status = TaskStatus::Pending;
    }
}

// After:
for status in state.tasks.values_mut() {
    if matches!(
        status,
        TaskStatus::Running
            | TaskStatus::Failed { .. }
            | TaskStatus::SkippedDueToDependencyFailure
    ) {
        *status = TaskStatus::Pending;
    }
}
```

**Extend** the existing test `load_resets_running_to_pending` — add before `s.save()`:

```rust
s.mark_failed("task3", "boom".into());
s.mark_skipped_due_to_dep_failure("task4");
s.mark_skipped("task5");
```

Add after load assertions:

```rust
assert_eq!(loaded.get_status("task3"), Some(TaskStatus::Pending));
assert_eq!(loaded.get_status("task4"), Some(TaskStatus::Pending));
assert_eq!(loaded.get_status("task5"), Some(TaskStatus::Skipped)); // must NOT reset
```

**Verify**: `cargo test -p workflow_core -- state::tests::load_resets_running_to_pending`

---

## TASK-13: Migrate `hubbard_u_sweep` example

- **Scope**: Rewrite `examples/hubbard_u_sweep/src/main.rs` to use `Workflow::new(...).with_max_parallel(4)?` and `ExecutionMode::Direct` + `.setup()`.
- **Crate/Module**: `examples/hubbard_u_sweep/src/main.rs`
- **Depends On**: None
- **Can Run In Parallel With**: TASK-12, TASK-14, TASK-15

### Acceptance Criteria

- Constructor is `Workflow::new("hubbard_u_sweep").with_max_parallel(4)?` — no `Workflow::builder()`.
- Tasks use `ExecutionMode::Direct { command: "castep", args: ["ZnO"], env: {}, timeout: None }`.
- File-writing logic stays in `.setup(...)` closure; process execution driven by `ExecutionMode::Direct`.
- `cargo check -p hubbard_u_sweep` passes.

### Notes

- `setup` closure must NOT launch the process — that is `Workflow::run`'s responsibility.
- `create_dir` / `write_file` from `workflow_utils` stay in `setup`.
- Remove `TaskExecutor` import entirely.

### Implementation

Full rewrite of `examples/hubbard_u_sweep/src/main.rs`:

```rust
use anyhow::Result;
use castep_cell_fmt::{format::to_string_many_spaced, parse, ToCellFile};
use castep_cell_io::cell::species::{AtomHubbardU, HubbardU, HubbardUUnit, OrbitalU, Species};
use castep_cell_io::CellDocument;
use std::collections::HashMap;
use std::sync::Arc;
use workflow_core::monitoring::HookExecutor;
use workflow_core::process::ProcessRunner;
use workflow_core::state::JsonStateStore;
use workflow_core::task::{ExecutionMode, Task};
use workflow_core::workflow::Workflow;
use workflow_utils::{ShellHookExecutor, SystemProcessRunner, create_dir, write_file};

fn main() -> Result<()> {
    let seed_cell = include_str!("../seeds/ZnO.cell");
    let seed_param = include_str!("../seeds/ZnO.param");

    let mut workflow = Workflow::new("hubbard_u_sweep").with_max_parallel(4)?;

    for u in [0.0_f64, 1.0, 2.0, 3.0, 4.0, 5.0] {
        let task_id = format!("scf_U{:.1}", u);
        let workdir = std::path::PathBuf::from(format!("runs/U{:.1}", u));
        let seed_cell = seed_cell.to_owned();
        let seed_param = seed_param.to_owned();

        let task = Task::new(
            &task_id,
            ExecutionMode::Direct {
                command: "castep".into(),
                args: vec!["ZnO".into()],
                env: HashMap::new(),
                timeout: None,
            },
        )
        .workdir(workdir.clone())
        .setup(move |workdir| {
            create_dir(workdir.to_str().unwrap())?;

            let mut cell_doc: CellDocument = parse(&seed_cell)
                .map_err(|e| workflow_core::WorkflowError::InvalidConfig(e.to_string()))?;

            let atom_u = AtomHubbardU::builder()
                .species(Species::Symbol("Zn".to_string()))
                .orbitals(vec![OrbitalU::D(u)])
                .build();
            let hubbard_u = HubbardU::builder()
                .unit(HubbardUUnit::ElectronVolt)
                .atom_u_values(vec![atom_u])
                .build();
            cell_doc.hubbard_u = Some(hubbard_u);

            let output = to_string_many_spaced(&cell_doc.to_cell_file());
            write_file(format!("{}/ZnO.cell", workdir.display()), &output)?;
            write_file(format!("{}/ZnO.param", workdir.display()), &seed_param)?;
            Ok(())
        });

        workflow.add_task(task)?;
    }

    let state_path = std::path::PathBuf::from(".hubbard_u_sweep.workflow.json");
    let mut state = JsonStateStore::new("hubbard_u_sweep", state_path);
    let runner: Arc<dyn ProcessRunner> = Arc::new(SystemProcessRunner);
    let executor: Arc<dyn HookExecutor> = Arc::new(ShellHookExecutor);

    workflow.run(&mut state, runner, executor)?;
    Ok(())
}
```

**Verify**: `cargo check -p hubbard_u_sweep`

---

## TASK-14: Remove stale workspace dependencies

- **Scope**: Remove `workflow_cli` from workspace members and prune dead deps from `[workspace.dependencies]`.
- **Crate/Module**: `Cargo.toml` (workspace root)
- **Depends On**: None
- **Enables**: TASK-18
- **Can Run In Parallel With**: TASK-12, TASK-13, TASK-15

### Acceptance Criteria

- `workflow_cli` removed from `[workspace.members]` FIRST (before any dep removal).
- `tokio`, `async-trait`, `tokio-rusqlite`, `tokio-util`, `toml` removed from `[workspace.dependencies]`.
- `bon` removed only after `rg '#\[bon\]|bon::' --type rust` returns zero matches.
- `nix` removed only if present.
- `serde`, `serde_json`, `petgraph`, `anyhow`, `tracing`, `tracing-subscriber` remain.
- `cargo check --workspace` and `cargo test --workspace` pass.

### Implementation

Steps in order:

1. Remove `"workflow_cli"` from `[workspace.members]`
2. Run `rg '#\[bon\]|bon::' --type rust` — if zero matches, remove `bon`
3. Remove from `[workspace.dependencies]`: `tokio`, `async-trait`, `tokio-rusqlite`, `tokio-util`, `toml`, `nix` (if present)

**Verify**: `cargo check --workspace && cargo test --workspace`

---

## TASK-15: Add `interrupt` field + failing tests

- **Scope**: Add `pub(crate) interrupt: Arc<AtomicBool>` to `Workflow` and write two failing tests.
- **Crate/Module**: `workflow_core/src/workflow.rs`
- **Depends On**: None
- **Enables**: TASK-16
- **Can Run In Parallel With**: TASK-12, TASK-13, TASK-14

### Acceptance Criteria

- `pub(crate) interrupt: Arc<AtomicBool>` added to `Workflow`, initialised to `false` in `Workflow::new`.
- Two tests in inline `#[cfg(test)]` mod compile but FAIL (interrupt check not yet in `run`).
- `WorkflowError::Interrupted` already exists in `error.rs` — do NOT add it again.
- No new public API surface.

### Implementation

**Step 1** — add to `Workflow` struct and `new()`:

```rust
// Add import at top of workflow.rs:
use std::sync::atomic::{AtomicBool, Ordering};

// Add field to struct:
pub struct Workflow {
    pub name: String,
    tasks: HashMap<String, Task>,
    max_parallel: usize,
    pub(crate) interrupt: Arc<AtomicBool>,  // ← add
}

// Initialise in Workflow::new():
Self {
    name: name.into(),
    tasks: HashMap::new(),
    max_parallel,
    interrupt: Arc::new(AtomicBool::new(false)),  // ← add
}
```

**Step 2** — add two failing tests inside the existing `#[cfg(test)] mod tests`:

```rust
#[test]
fn interrupt_before_run_dispatches_nothing() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir().unwrap();
    let mut wf = Workflow::new("wf_interrupt").with_max_parallel(4)?;
    wf.add_task(Task::new("a", ExecutionMode::Direct {
        command: "true".into(), args: vec![], env: HashMap::new(), timeout: None,
    })).unwrap();
    wf.interrupt.store(true, Ordering::SeqCst);
    let mut state = JsonStateStore::new("wf_interrupt", dir.path().join(".wf_interrupt.workflow.json"));
    let result = wf.run(&mut state, Arc::new(SystemProcessRunner), Arc::new(ShellHookExecutor));
    assert_eq!(result.unwrap_err(), WorkflowError::Interrupted);
    assert!(!matches!(state.get_status("a"), Some(TaskStatus::Completed)));
    Ok(())
}

#[test]
fn interrupt_mid_run_stops_dispatch() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir().unwrap();
    let mut wf = Workflow::new("wf_interrupt2").with_max_parallel(4)?;
    let flag = Arc::new(AtomicBool::new(false));
    let flag_clone = Arc::clone(&flag);
    wf.add_task(
        Task::new("a", ExecutionMode::Direct {
            command: "true".into(), args: vec![], env: HashMap::new(), timeout: None,
        })
        .setup(move |_| { flag_clone.store(true, Ordering::SeqCst); Ok(()) }),
    ).unwrap();
    wf.add_task(
        Task::new("b", ExecutionMode::Direct {
            command: "true".into(), args: vec![], env: HashMap::new(), timeout: None,
        }).depends_on("a"),
    ).unwrap();
    wf.interrupt = Arc::clone(&flag);
    let mut state = JsonStateStore::new("wf_interrupt2", dir.path().join(".wf_interrupt2.workflow.json"));
    let result = wf.run(&mut state, Arc::new(SystemProcessRunner), Arc::new(ShellHookExecutor));
    assert_eq!(result.unwrap_err(), WorkflowError::Interrupted);
    Ok(())
}
```

**Verify**: `cargo test -p workflow_core -- tests::interrupt` (expect 2 failures, not compile errors)

---

## TASK-16: Implement interrupt check in `Workflow::run`

- **Scope**: Insert `AtomicBool` check at top of run loop; return `Err(WorkflowError::Interrupted)` on signal.
- **Crate/Module**: `workflow_core/src/workflow.rs`
- **Depends On**: TASK-15
- **Enables**: TASK-16b

### Acceptance Criteria

- `if self.interrupt.load(Ordering::SeqCst) { ... }` at top of poll loop.
- Both TASK-15 tests now pass.
- In-flight tasks marked `Pending` (NOT `Failed`) on interrupt.
- `state.save()` called before returning `Err`.
- All pre-existing tests continue to pass.

### Implementation

At the very top of the `loop` body in `run()`, before the "Poll finished tasks" comment:

```rust
loop {
    // Interrupt check — must be first
    if self.interrupt.load(Ordering::SeqCst) {
        for id in handles.keys() {
            state.set_status(id, TaskStatus::Pending);
        }
        for (_, (handle, _)) in handles.iter_mut() {
            handle.terminate().ok();
        }
        state.save()?;
        return Err(WorkflowError::Interrupted);
    }

    // Poll finished tasks  ← existing code continues here
```

**Verify**: `cargo test -p workflow_core -- tests::interrupt` (both tests now pass)

---

## TASK-16b: Per-task timeout enforcement

- **Scope**: Add `task_timeouts: HashMap<String, Duration>` at dispatch; enforce in poll loop before `is_running()`.
- **Crate/Module**: `workflow_core/src/workflow.rs`
- **Depends On**: TASK-16
- **Enables**: TASK-17

### Acceptance Criteria

- `task_timeouts` populated at dispatch from `ExecutionMode::Direct { timeout: Some(d) }`.
- Timeout check before `is_running()` in poll loop.
- On expiry: `handle.terminate()`, mark `Failed { error: "task '{id}' timed out after {timeout:?}" }`, save, remove from handles.
- Tasks with no timeout entry are unaffected.
- All pre-existing tests pass.

### Implementation

**Change 1** — declare `task_timeouts` just before the `loop`:

```rust
let mut task_timeouts: HashMap<String, Duration> = HashMap::new();
```

**Change 2** — at dispatch, replace `timeout: _` with `timeout`:

```rust
// Before:
ExecutionMode::Direct { command, args, env, timeout: _ } => {
    let handle = runner.spawn(&task.workdir, command, args, env)?;
    handles.insert(id.clone(), (handle, Instant::now()));
}

// After:
ExecutionMode::Direct { command, args, env, timeout } => {
    if let Some(d) = timeout {
        task_timeouts.insert(id.clone(), *d);
    }
    let handle = runner.spawn(&task.workdir, command, args, env)?;
    handles.insert(id.clone(), (handle, Instant::now()));
}
```

**Change 3** — add timeout check at the top of the "Poll finished tasks" loop, before `is_running()`:

```rust
for (id, (handle, start)) in handles.iter_mut() {
    // Timeout check first
    if let Some(&timeout) = task_timeouts.get(id) {
        if start.elapsed() >= timeout {
            handle.terminate().ok();
            state.mark_failed(id, format!("task '{}' timed out after {:?}", id, timeout));
            state.save()?;
            finished.push(id.clone());
            continue;
        }
    }
    if !handle.0.is_running() {
        finished.push(id.clone());
    }
}
```

**Change 4** — guard the `wait()` call to skip already-failed tasks (timed out):

```rust
for id in finished {
    if let Some((mut handle, start)) = handles.remove(&id) {
        // Skip wait() if already marked failed (e.g. timed out)
        if matches!(state.get_status(&id), Some(TaskStatus::Failed { .. })) {
            continue;
        }
        // ... existing wait/mark logic unchanged
    }
}
```

**Verify**: `cargo test -p workflow_core`

---

## TASK-17: Timeout integration test

- **Scope**: New file `workflow_core/tests/timeout_integration.rs`.
- **Crate/Module**: `workflow_core/tests/timeout_integration.rs` (NEW)
- **Depends On**: TASK-16b
- **Can Run In Parallel With**: TASK-18, TASK-19, TASK-20

### Acceptance Criteria

- Task A: `sleep 60`, `timeout: Some(Duration::from_millis(200))`.
- Task B: `true`, depends on A.
- `summary.failed` contains A; error contains `"timed out"`.
- `summary.skipped` contains B.
- Wall time < 1 second.

### Implementation

```rust
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use workflow_core::monitoring::HookExecutor;
use workflow_core::process::ProcessRunner;
use workflow_core::state::JsonStateStore;
use workflow_core::task::{ExecutionMode, Task};
use workflow_core::workflow::Workflow;
use workflow_utils::{ShellHookExecutor, SystemProcessRunner};

#[test]
fn timeout_task_fails_and_dependent_skips() {
    let dir = tempfile::tempdir().unwrap();
    let mut wf = Workflow::new("wf_timeout").with_max_parallel(4).unwrap();

    wf.add_task(Task::new("sleeper", ExecutionMode::Direct {
        command: "sleep".into(),
        args: vec!["60".into()],
        env: HashMap::new(),
        timeout: Some(Duration::from_millis(200)),
    })).unwrap();

    wf.add_task(
        Task::new("dependent", ExecutionMode::Direct {
            command: "true".into(), args: vec![], env: HashMap::new(), timeout: None,
        }).depends_on("sleeper"),
    ).unwrap();

    let mut state = JsonStateStore::new("wf_timeout", dir.path().join(".wf_timeout.workflow.json"));
    let wall_start = Instant::now();
    let summary = wf.run(&mut state, Arc::new(SystemProcessRunner) as Arc<dyn ProcessRunner>,
                         Arc::new(ShellHookExecutor) as Arc<dyn HookExecutor>).unwrap();

    assert!(wall_start.elapsed() < Duration::from_secs(1));
    let (_, err) = summary.failed.iter().find(|(id, _)| id == "sleeper").expect("sleeper should fail");
    assert!(err.contains("timed out"), "error was: {}", err);
    assert!(summary.skipped.contains(&"dependent".to_string()));
}
```

**Verify**: `cargo test -p workflow_core -- timeout_integration`

---

## TASK-18: Delete `workflow_cli`, create `workflow-cli` skeleton

- **Scope**: Remove old `workflow_cli/` directory, scaffold new `workflow-cli` binary crate.
- **Crate/Module**: `workflow-cli/` (NEW), `Cargo.toml` (workspace)
- **Depends On**: TASK-14
- **Enables**: TASK-19, TASK-20
- **Can Run In Parallel With**: TASK-17

### Acceptance Criteria

- `workflow_cli/` deleted.
- `"workflow-cli"` in workspace members; `clap = { version = "4", features = ["derive"] }` in `[workspace.dependencies]`.
- `workflow-cli/Cargo.toml`: `name = "workflow-cli"`, `[[bin]]`, deps: `workflow_core` (path), `clap` (workspace), `anyhow` (workspace).
- Skeleton subcommands `Status`, `Retry`, `Inspect` — all `todo!()`.
- `cargo check -p workflow-cli` passes.

### Implementation

**`workflow-cli/Cargo.toml`**:

```toml
[package]
name = "workflow-cli"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "workflow-cli"
path = "src/main.rs"

[dependencies]
workflow_core = { path = "../workflow_core" }
clap = { workspace = true }
anyhow = { workspace = true }
```

**`workflow-cli/src/main.rs`**:

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "workflow-cli", about = "Workflow state inspection tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Status { state_file: String },
    Retry {
        state_file: String,
        #[arg(required = true)]
        task_ids: Vec<String>,
    },
    Inspect {
        state_file: String,
        task_id: Option<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Status { state_file } => todo!(),
        Commands::Retry { state_file, task_ids } => todo!(),
        Commands::Inspect { state_file, task_id } => todo!(),
    }
}
```

**Verify**: `cargo check -p workflow-cli`

---

## TASK-19: Implement `status` and `inspect` subcommands

- **Scope**: Implement `status` and `inspect` arms with exact output format, shared helper, and tests.
- **Crate/Module**: `workflow-cli/src/main.rs`
- **Depends On**: TASK-18
- **Enables**: TASK-21
- **Can Run In Parallel With**: TASK-20, TASK-17

### Acceptance Criteria

- `fn load_state(path: &str) -> anyhow::Result<JsonStateStore>` shared helper.
- `status`: one line per task `<id>: <STATUS>` (failed: `<id>: Failed (<error>)`), final line `Summary: X completed, Y failed, Z skipped, W pending`.
- `inspect <state-file> <task-id>`: `task: <id>`, `status: <STATUS>`, `error: <message>` (if failed).
- `inspect <state-file>` (no id): all tasks.
- Missing file: `error: state file not found: <path>` to stderr, exit non-zero.
- Tests assert exact strings via `assert_eq!`.

### Implementation

Add to `workflow-cli/src/main.rs`:

```rust
use workflow_core::state::{JsonStateStore, StateStore, StateStoreExt, TaskStatus};

fn load_state(path: &str) -> anyhow::Result<JsonStateStore> {
    JsonStateStore::load(path)
        .map_err(|_| anyhow::anyhow!("error: state file not found: {}", path))
}

fn cmd_status(state: &JsonStateStore) -> String {
    let mut tasks: Vec<_> = state.all_tasks().into_iter().collect();
    tasks.sort_by(|a, b| a.0.cmp(&b.0));
    let mut out = String::new();
    for (id, status) in &tasks {
        match status {
            TaskStatus::Failed { error } => out.push_str(&format!("{}: Failed ({})\n", id, error)),
            other => out.push_str(&format!("{}: {:?}\n", id, other)),
        }
    }
    let s = state.summary();
    out.push_str(&format!(
        "Summary: {} completed, {} failed, {} skipped, {} pending",
        s.completed, s.failed, s.skipped, s.pending
    ));
    out
}

fn cmd_inspect(state: &JsonStateStore, task_id: Option<&str>) -> anyhow::Result<String> {
    match task_id {
        Some(id) => match state.get_status(id) {
            None => anyhow::bail!("task '{}' not found", id),
            Some(TaskStatus::Failed { error }) =>
                Ok(format!("task: {}\nstatus: Failed\nerror: {}", id, error)),
            Some(s) => Ok(format!("task: {}\nstatus: {:?}", id, s)),
        },
        None => {
            let mut tasks: Vec<_> = state.all_tasks().into_iter().collect();
            tasks.sort_by(|a, b| a.0.cmp(&b.0));
            Ok(tasks.iter()
                .map(|(id, s)| format!("{}: {:?}", id, s))
                .collect::<Vec<_>>().join("\n"))
        }
    }
}
```

Replace `Commands::Status` and `Commands::Inspect` arms:

```rust
Commands::Status { state_file } => {
    let state = load_state(&state_file)?;
    println!("{}", cmd_status(&state));
    Ok(())
}
Commands::Inspect { state_file, task_id } => {
    let state = load_state(&state_file)?;
    match cmd_inspect(&state, task_id.as_deref()) {
        Ok(out) => { println!("{}", out); Ok(()) }
        Err(e) => { eprintln!("{}", e); std::process::exit(1); }
    }
}
```

Add tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use workflow_core::state::StateStoreExt;

    fn make_state(dir: &std::path::Path) -> JsonStateStore {
        let mut s = JsonStateStore::new("test_wf", dir.join("state.json"));
        s.mark_completed("task_a");
        s.mark_failed("task_b", "exit code 1".into());
        s.mark_skipped_due_to_dep_failure("task_c");
        s.save().unwrap();
        s
    }

    #[test]
    fn status_output_format() {
        let dir = tempfile::tempdir().unwrap();
        let s = make_state(dir.path());
        let out = cmd_status(&s);
        assert!(out.contains("task_a: Completed"));
        assert!(out.contains("task_b: Failed (exit code 1)"));
        assert!(out.contains("Summary: 1 completed, 1 failed, 1 skipped, 0 pending"));
    }

    #[test]
    fn inspect_single_task() {
        let dir = tempfile::tempdir().unwrap();
        let s = make_state(dir.path());
        let out = cmd_inspect(&s, Some("task_b")).unwrap();
        assert_eq!(out, "task: task_b\nstatus: Failed\nerror: exit code 1");
    }

    #[test]
    fn inspect_unknown_task_errors() {
        let dir = tempfile::tempdir().unwrap();
        let s = make_state(dir.path());
        assert!(cmd_inspect(&s, Some("nonexistent")).is_err());
    }
}
```

**Verify**: `cargo test -p workflow-cli`

---

## TASK-20: Implement `retry` subcommand

- **Scope**: Implement `retry` arm with state mutation and tests.
- **Crate/Module**: `workflow-cli/src/main.rs`
- **Depends On**: TASK-18
- **Enables**: TASK-21
- **Can Run In Parallel With**: TASK-19, TASK-17

### Acceptance Criteria

- Each named `task_id` set to `Pending`; warn via `eprintln!` if not found.
- All `SkippedDueToDependencyFailure` tasks reset to `Pending`.
- `TaskStatus::Skipped` (explicit) NOT reset.
- State saved after all mutations.
- At least one test covering happy path and not-found warning.

### Implementation

Add function and replace `Commands::Retry` arm:

```rust
fn cmd_retry(state: &mut JsonStateStore, task_ids: &[String]) {
    for id in task_ids {
        if state.get_status(id).is_none() {
            eprintln!("warn: task '{}' not found", id);
        } else {
            state.mark_pending(id);
        }
    }
    let to_reset: Vec<String> = state.all_tasks()
        .into_iter()
        .filter(|(_, s)| matches!(s, TaskStatus::SkippedDueToDependencyFailure))
        .map(|(id, _)| id)
        .collect();
    for id in to_reset {
        state.mark_pending(&id);
    }
    state.save().unwrap();
}
```

```rust
Commands::Retry { state_file, task_ids } => {
    let mut state = load_state(&state_file)?;
    cmd_retry(&mut state, &task_ids);
    Ok(())
}
```

Add to `#[cfg(test)]` block:

```rust
#[test]
fn retry_resets_failed_and_skipped_dep() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = make_state(dir.path());
    // task_b=Failed, task_c=SkippedDueToDependencyFailure, task_a=Completed
    cmd_retry(&mut s, &["task_b".to_string()]);
    assert_eq!(s.get_status("task_b"), Some(TaskStatus::Pending));
    assert_eq!(s.get_status("task_c"), Some(TaskStatus::Pending));
    assert_eq!(s.get_status("task_a"), Some(TaskStatus::Completed)); // unchanged
}
```

**Verify**: `cargo test -p workflow-cli`

---

## TASK-21: End-to-end integration test

- **Scope**: New file `workflow_core/tests/integration.rs`. Two-run test verifying resume, failure propagation, and non-re-execution of completed tasks.
- **Crate/Module**: `workflow_core/tests/integration.rs` (NEW)
- **Depends On**: TASK-12, TASK-19, TASK-20

### Acceptance Criteria

- Run 1: B=`"false"`. `summary.failed` contains B; `summary.skipped` contains C.
- Persisted state: B=`Failed`, C=`SkippedDueToDependencyFailure`.
- Run 2: `JsonStateStore::load` resets B+C to `Pending` (TASK-12). B=`"true"`. All complete.
- A does NOT re-run: `Arc<AtomicUsize>` counter == 1 total after both runs.
- `tempfile::tempdir()` for isolation.

### Implementation

```rust
use std::collections::HashMap;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use workflow_core::monitoring::HookExecutor;
use workflow_core::process::ProcessRunner;
use workflow_core::state::{JsonStateStore, StateStore, TaskStatus};
use workflow_core::task::{ExecutionMode, Task};
use workflow_core::workflow::Workflow;
use workflow_utils::{ShellHookExecutor, SystemProcessRunner};

fn runner() -> Arc<dyn ProcessRunner> { Arc::new(SystemProcessRunner) }
fn executor() -> Arc<dyn HookExecutor> { Arc::new(ShellHookExecutor) }
fn direct(cmd: &str) -> ExecutionMode {
    ExecutionMode::Direct { command: cmd.into(), args: vec![], env: HashMap::new(), timeout: None }
}

#[test]
fn resume_skips_completed_reruns_failed() {
    let dir = tempfile::tempdir().unwrap();
    let state_path = dir.path().join(".integration.workflow.json");
    let a_runs = Arc::new(AtomicUsize::new(0));

    // Run 1: B fails, C skipped
    let a_runs_c = Arc::clone(&a_runs);
    let mut wf1 = Workflow::new("integration").with_max_parallel(4).unwrap();
    wf1.add_task(
        Task::new("a", direct("true"))
            .setup(move |_| { a_runs_c.fetch_add(1, Ordering::SeqCst); Ok(()) })
    ).unwrap();
    wf1.add_task(Task::new("b", direct("false")).depends_on("a")).unwrap();
    wf1.add_task(Task::new("c", direct("true")).depends_on("b")).unwrap();

    let mut state1 = JsonStateStore::new("integration", state_path.clone());
    let summary1 = wf1.run(&mut state1, runner(), executor()).unwrap();
    assert!(summary1.failed.iter().any(|(id, _)| id == "b"));
    assert!(summary1.skipped.contains(&"c".to_string()));

    // Verify persisted state
    let persisted = JsonStateStore::load(&state_path).unwrap();
    assert!(matches!(persisted.get_status("b"), Some(TaskStatus::Failed { .. })));
    assert!(matches!(persisted.get_status("c"), Some(TaskStatus::SkippedDueToDependencyFailure)));

    // Run 2: load resets b+c to Pending; b now succeeds; a must NOT re-run
    let mut state2 = JsonStateStore::load(&state_path).unwrap();
    assert_eq!(state2.get_status("b"), Some(TaskStatus::Pending));
    assert_eq!(state2.get_status("c"), Some(TaskStatus::Pending));

    let mut wf2 = Workflow::new("integration").with_max_parallel(4).unwrap();
    // A uses "false" — would fail if re-dispatched, proving it is skipped
    wf2.add_task(Task::new("a", direct("false"))).unwrap();
    wf2.add_task(Task::new("b", direct("true")).depends_on("a")).unwrap();
    wf2.add_task(Task::new("c", direct("true")).depends_on("b")).unwrap();

    let summary2 = wf2.run(&mut state2, runner(), executor()).unwrap();
    assert!(summary2.succeeded.contains(&"b".to_string()));
    assert!(summary2.succeeded.contains(&"c".to_string()));
    assert_eq!(a_runs.load(Ordering::SeqCst), 1, "A must only run once total");
}
```

**Verify**: `cargo test -p workflow_core -- integration`
