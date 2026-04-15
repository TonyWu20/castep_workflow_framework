# Fix Plan: phase-3 → main (2026-04-15)

## Execution Order

```
Phase 1 (parallel): TASK-1, 7, 8, 9, 10
Phase 2 (parallel, after TASK-1): TASK-2, 3, 4, 5
Phase 3 (parallel, after TASK-2 + TASK-10): TASK-11, 12, 13, 14
Phase 4 (after TASK-14): TASK-15
Phase 5: cargo test --workspace
```

---

## Phase 1 — Independent fixes (run in parallel)

### TASK-1: Fix `StateStore` trait signatures

**File:** `workflow_core/src/state.rs`
**Target:** `pub trait StateStore: Send + Sync`

Before:

```rust
pub trait StateStore: Send + Sync {
    fn get_status(&self, id: &str) -> Option<&TaskStatus>;
    fn set_status(&mut self, id: &str, status: TaskStatus);
    fn all_tasks(&self) -> &HashMap<String, TaskStatus>;
    fn save(&self) -> Result<(), WorkflowError>;
}
```

After:

```rust
pub trait StateStore: Send + Sync {
    fn get_status(&self, id: &str) -> Option<TaskStatus>;
    fn set_status(&mut self, id: &str, status: TaskStatus);
    fn all_tasks(&self) -> Vec<(String, TaskStatus)>;
    fn save(&self) -> Result<(), WorkflowError>;
}
```

**Verification:** `cargo check -p workflow_core` (caller errors expected — fixed in Phase 2)

---

### TASK-7: Fix `Queued` arm wrong error variant

**File:** `workflow_core/src/workflow.rs`
**Target:** `ExecutionMode::Queued { .. } =>` match arm inside `Workflow::run`

Before:

```rust
ExecutionMode::Queued { .. } => {
    return Err(WorkflowError::Io(std::io::Error::other(
        "queued execution not yet implemented",
    )));
}
```

After:

```rust
ExecutionMode::Queued { .. } => {
    unreachable!("Queued execution mode is not yet implemented");
}
```

**Verification:** `cargo check -p workflow_core`

---

### TASK-8: Guard signal registration with `Once`

**File:** `workflow_core/src/workflow.rs`
**Target:** The first two statements inside the body of `pub fn run` (the two `signal_hook::flag::register` calls)

Before:

```rust
signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&self.interrupt)).ok();
signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&self.interrupt)).ok();
```

After:

```rust
static SIGNAL_INIT: std::sync::Once = std::sync::Once::new();
SIGNAL_INIT.call_once(|| {
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&self.interrupt)).ok();
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&self.interrupt)).ok();
});
```

**Verification:** `cargo check -p workflow_core`

---

### TASK-9: Delete `execute_hook` free function and its re-export

**File 1:** `workflow_utils/src/monitoring.rs`
**Target:** `pub fn execute_hook`

Before:

```rust
pub fn execute_hook(hook: &MonitoringHook, ctx: &HookContext) -> Result<HookResult> {
    let mut parts = hook.command.split_whitespace();
    let cmd = parts.next().unwrap_or_default();
    let args: Vec<String> = parts.map(String::from).collect();
    let result = TaskExecutor::new(&ctx.workdir)
        .command(cmd)
        .args(args)
        .env("TASK_ID", &ctx.task_id)
        .env("TASK_STATE", &ctx.state)
        .env("WORKDIR", ctx.workdir.to_string_lossy().as_ref())
        .env("EXIT_CODE", ctx.exit_code.map(|c| c.to_string()).unwrap_or_default())
        .execute()?;
    Ok(HookResult { success: result.success(), output: result.stdout })
}
```

After: delete the entire function.

**Note:** Run `rg 'execute_hook' --type rust` first. If any caller outside `workflow_utils` uses the free function, update it to `ShellHookExecutor.execute_hook(hook, ctx)` instead.

**File 2:** `workflow_utils/src/lib.rs`
**Target:** `pub use monitoring::{execute_hook, ShellHookExecutor};`

Before:

```rust
pub use monitoring::{execute_hook, ShellHookExecutor};
```

After:

```rust
pub use monitoring::ShellHookExecutor;
```

**Verification:** `cargo check --workspace`

---

### TASK-10: Remove `PartialEq` impl from `WorkflowError`

**File:** `workflow_core/src/error.rs`
**Target:** `impl PartialEq for WorkflowError`

Before:

```rust
impl PartialEq for WorkflowError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::DuplicateTaskId(a), Self::DuplicateTaskId(b)) => a == b,
            (Self::CycleDetected, Self::CycleDetected) => true,
            (Self::UnknownDependency { task: a_task, dependency: a_dep },
             Self::UnknownDependency { task: b_task, dependency: b_dep }) => {
                a_task == b_task && a_dep == b_dep
            }
            (Self::StateCorrupted(a), Self::StateCorrupted(b)) => a == b,
            (Self::TaskTimeout(a), Self::TaskTimeout(b)) => a == b,
            (Self::InvalidConfig(a), Self::InvalidConfig(b)) => a == b,
            (Self::Io(ref e_a), Self::Io(ref e_b)) => e_a.kind() == e_b.kind(),
            (Self::Interrupted, Self::Interrupted) => true,
            _ => false,
        }
    }
}
```

After: delete the entire block.

**Verification:** `cargo check -p workflow_core` (test errors expected — fixed in TASK-11)

---

## Phase 2 — Callers of `StateStore` trait (parallel, after TASK-1)

### TASK-2: Fix `JsonStateStore` impl to clone on return

**File:** `workflow_core/src/state.rs`
**Target:** `impl StateStore for JsonStateStore`

Before:

```rust
impl StateStore for JsonStateStore {
    fn get_status(&self, id: &str) -> Option<&TaskStatus> {
        self.tasks.get(id)
    }
    fn set_status(&mut self, id: &str, status: TaskStatus) {
        self.tasks.insert(id.to_owned(), status);
        self.last_updated = now_iso8601();
    }
    fn all_tasks(&self) -> &HashMap<String, TaskStatus> {
        &self.tasks
    }
    fn save(&self) -> Result<(), WorkflowError> {
        self.save()
    }
}
```

After:

```rust
impl StateStore for JsonStateStore {
    fn get_status(&self, id: &str) -> Option<TaskStatus> {
        self.tasks.get(id).cloned()
    }
    fn set_status(&mut self, id: &str, status: TaskStatus) {
        self.tasks.insert(id.to_owned(), status);
        self.last_updated = now_iso8601();
    }
    fn all_tasks(&self) -> Vec<(String, TaskStatus)> {
        self.tasks.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }
    fn save(&self) -> Result<(), WorkflowError> {
        self.save()
    }
}
```

**Verification:** `cargo check -p workflow_core`

---

### TASK-3: Fix `StateStoreExt::summary` iteration

**File:** `workflow_core/src/state.rs`
**Target:** `fn summary` inside `pub trait StateStoreExt`

Before:

```rust
for status in self.all_tasks().values() {
    match status {
```

After:

```rust
for (_, status) in self.all_tasks() {
    match status {
```

**Verification:** `cargo check -p workflow_core`

---

### TASK-4: Fix `done_set` construction in `workflow.rs`

**File:** `workflow_core/src/workflow.rs`
**Target:** `let done_set: HashSet<String> = state`

Before:

```rust
let done_set: HashSet<String> = state
    .all_tasks()
    .iter()
    .filter(|(_, v)| {
        matches!(v, TaskStatus::Completed | TaskStatus::Skipped | TaskStatus::SkippedDueToDependencyFailure)
    })
    .map(|(k, _)| k.clone())
    .collect();
```

After:

```rust
let done_set: HashSet<String> = state
    .all_tasks()
    .into_iter()
    .filter(|(_, v)| {
        matches!(v, TaskStatus::Completed | TaskStatus::Skipped | TaskStatus::SkippedDueToDependencyFailure)
    })
    .map(|(k, _)| k)
    .collect();
```

**Verification:** `cargo check -p workflow_core`

---

### TASK-5: Fix `WorkflowSummary` loop in `workflow.rs`

**File:** `workflow_core/src/workflow.rs`
**Target:** `for (id, status) in state.all_tasks().iter()`

Before:

```rust
for (id, status) in state.all_tasks().iter() {
    match status {
        TaskStatus::Completed => succeeded.push(id.clone()),
        TaskStatus::Failed { error } => failed.push((id.clone(), error.clone())),
        TaskStatus::Skipped | TaskStatus::SkippedDueToDependencyFailure => skipped.push(id.clone()),
        _ => {}
    }
}
```

After:

```rust
for (id, status) in state.all_tasks() {
    match status {
        TaskStatus::Completed => succeeded.push(id),
        TaskStatus::Failed { error } => failed.push((id, error)),
        TaskStatus::Skipped | TaskStatus::SkippedDueToDependencyFailure => skipped.push(id),
        _ => {}
    }
}
```

**Verification:** `cargo check -p workflow_core`

---

## Phase 3 — CLI fixes + test updates (parallel, after TASK-2 and TASK-10)

### TASK-11: Replace `assert_eq!` on `WorkflowError` with `matches!`

**File 1:** `workflow_core/src/workflow.rs`
**Target:** `assert_eq!(result.unwrap_err(), WorkflowError::Interrupted)` inside `interrupt_before_run_dispatches_nothing`

Before:

```rust
assert_eq!(result.unwrap_err(), WorkflowError::Interrupted);
```

After:

```rust
assert!(matches!(result.unwrap_err(), WorkflowError::Interrupted));
```

**File 2:** `workflow_core/src/workflow.rs`
**Target:** `assert_eq!` block inside `duplicate_task_id_errors`

Before:

```rust
assert_eq!(
    wf.add_task(Task::new(
        "a",
        ExecutionMode::Direct {
            command: "true".into(),
            args: vec![],
            env: HashMap::new(),
            timeout: None,
        },
    )),
    Err(WorkflowError::DuplicateTaskId("a".to_string()))
);
```

After:

```rust
assert!(matches!(
    wf.add_task(Task::new(
        "a",
        ExecutionMode::Direct {
            command: "true".into(),
            args: vec![],
            env: HashMap::new(),
            timeout: None,
        },
    )),
    Err(WorkflowError::DuplicateTaskId(_))
));
```

**File 3:** `workflow_core/src/error.rs`
**Target:** `fn test_workflow_error_partial_eq` — delete the entire test function.

**Verification:** `cargo test -p workflow_core`

---

### TASK-12: Fix `cmd_status` to use trait

**File:** `workflow-cli/src/main.rs`
**Target:** `fn cmd_status`

Before:

```rust
fn cmd_status(state: &JsonStateStore) -> String {
    let mut tasks: Vec<(String, &TaskStatus)> = state.all_tasks().iter()
        .map(|(k, v)| (k.clone(), v))
        .collect();
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
```

After:

```rust
fn cmd_status(state: &dyn StateStore) -> String {
    let mut tasks: Vec<(String, TaskStatus)> = state.all_tasks();
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
```

**Verification:** `cargo check -p workflow-cli`

---

### TASK-13: Fix `cmd_inspect` to use trait

**File:** `workflow-cli/src/main.rs`
**Target:** `fn cmd_inspect`

Before:

```rust
fn cmd_inspect(state: &JsonStateStore, task_id: Option<&str>) -> anyhow::Result<String> {
    match task_id {
        Some(id) => match state.get_status(id) {
            None => anyhow::bail!("task '{}' not found", id),
            Some(TaskStatus::Failed { error }) =>
                Ok(format!("task: {}\nstatus: Failed\nerror: {}", id, error)),
            Some(s) => Ok(format!("task: {}\nstatus: {:?}", id, s)),
        },
        None => {
            let mut tasks: Vec<(String, &TaskStatus)> = state.all_tasks().iter()
                .map(|(k, v)| (k.clone(), v))
                .collect();
            tasks.sort_by(|a, b| a.0.cmp(&b.0));
            Ok(tasks.iter()
                .map(|(id, s)| format!("{}: {:?}", id, s))
                .collect::<Vec<_>>()
                .join("\n"))
        }
    }
}
```

After:

```rust
fn cmd_inspect(state: &dyn StateStore, task_id: Option<&str>) -> anyhow::Result<String> {
    match task_id {
        Some(id) => match state.get_status(id) {
            None => anyhow::bail!("task '{}' not found", id),
            Some(TaskStatus::Failed { error }) =>
                Ok(format!("task: \nstatus: Failed\nerror: {}", id, error)),
            Some(s) => Ok(format!("task: {}\nstatus: {:?}", id, s)),
        },
        None => {
            let mut tasks: Vec<(String, TaskStatus)> = state.all_tasks();
            tasks.sort_by(|a, b| a.0.cmp(&b.0));
            Ok(tasks.iter()
                .map(|(id, s)| format!("{}: {:?}", id, s))
                .collect::<Vec<_>>()
                .join("\n"))
        }
    }
}
```

**Verification:** `cargo check -p workflow-cli`

---

### TASK-14: Fix `cmd_retry` signature and error handling

**File:** `workflow-cli/src/main.rs`
**Target:** `fn cmd_retry`

Before:

```rust
fn cmd_retry(state: &mut JsonStateStore, task_ids: &[String]) {
    for id in task_ids {
        if state.get_status(id).is_none() {
            eprintln!("warn: task '{}' not found", id);
        } else {
            state.mark_pending(id);
        }
    }
    let to_reset: Vec<String> = state
        .all_tasks()
        .iter()
        .filter(|(_, s)| matches!(s, TaskStatus::SkippedDueToDependencyFailure))
        .map(|(id, _)| id.clone())
        .collect();
    for id in to_reset {
        state.mark_pending(&id);
    }
    state.save().unwrap();
}
```

After:

```rust
fn cmd_retry(state: &mut dyn StateStore, task_ids: &[String]) -> anyhow::Result<()> {
    for id in task_ids {
        if state.get_status(id).is_none() {
            eprintln!("warn: task '{}' not found", id);
        } else {
            state.mark_pending(id);
        }
    }
    let to_reset: Vec<String> = state
        .all_tasks()
        .into_iter()
        .filter(|(_, s)| matches!(s, TaskStatus::SkippedDueToDependencyFailure))
        .map(|(id, _)| id)
        .collect();
    for id in to_reset {
        state.mark_pending(&id);
    }
    state.save().map_err(|e| anyhow::anyhow!("failed to save state: {}", e))?;
    Ok(())
}
```

**Verification:** `cargo check -p workflow-cli`

---

## Phase 4 — Call site update (after TASK-14)

### TASK-15: Update `cmd_retry` call site in `main`

**File:** `workflow-cli/src/main.rs`
**Target:** `Commands::Retry` arm in `fn main`

Before:

```rust
Commands::Retry { state_file, task_ids } => {
    let mut state = load_state(&state_file)?;
    cmd_retry(&mut state, &task_ids);
    Ok(())
}
```

After:

```rust
Commands::Retry { state_file, task_ids } => {
    let mut state = load_state(&state_file)?;
    cmd_retry(&mut state, &task_ids)?;
    Ok(())
}
```

**Verification:** `cargo check -p workflow-cli`

---

## Phase 5 — Final

```
cargo test --workspace
```
