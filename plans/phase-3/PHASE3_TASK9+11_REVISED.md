# TASK-9+11: Task Redesign + Workflow Engine Rewrite (COMBINED, REVISED)

**CRITICAL**: These two tasks MUST land in a single atomic commit. TASK-9 removes `execute_fn` from `Task`, which `workflow.rs` directly accesses. The codebase will not compile if TASK-9 lands without TASK-11.

- **Scope**: Replace the closure-based `Task` struct with `ExecutionMode`-based design (TASK-9), rewrite `Workflow::run()` to use trait-based dependencies (TASK-11), remove `bon` builder, and migrate all tests to the new API.
- **Crate/Module**: `workflow_core/src/task.rs`, `workflow_core/src/workflow.rs`, `workflow_core/src/lib.rs`, `workflow_core/Cargo.toml`
- **Responsible For**: The new Task data model and the core execution engine rewrite.
- **Depends On**: TASK-2d, TASK-3, TASK-5, TASK-7, TASK-8, TASK-10
- **Enables**: TASK-12, TASK-13, TASK-15, TASK-16
- **Can Run In Parallel With**: None (convergence point)

---

## Part A: Task Redesign (TASK-9)

### 1. Define `ExecutionMode` Enum in `workflow_core/src/task.rs`

```rust
use std::collections::HashMap;
use std::time::Duration;

#[derive(Clone)]
pub enum ExecutionMode {
    Direct {
        command: String,
        args: Vec<String>,
        env: HashMap<String, String>,
        timeout: Option<Duration>,
    },
    Queued {
        submit_cmd: String,
        poll_cmd: String,
        cancel_cmd: String,
    },
}
```

**Design decision**: No `Closure` variant. Tests will use real shell commands (`echo`, `true`, `false`, `sh -c`). This framework targets Linux/macOS HPC environments where POSIX commands are available.

---

### 2. Redesign `Task` Struct

```rust
use std::path::{Path, PathBuf};
use crate::error::WorkflowError;
use crate::monitoring::MonitoringHook;

pub struct Task {
    pub id: String,
    pub dependencies: Vec<String>,
    pub workdir: PathBuf,
    pub mode: ExecutionMode,
    pub setup: Option<Box<dyn Fn(&Path) -> Result<(), WorkflowError> + Send + Sync>>,
    pub collect: Option<Box<dyn Fn(&Path) -> Result<(), WorkflowError> + Send + Sync>>,
    pub monitors: Vec<MonitoringHook>,
}
```

**Key changes**:
- `execute_fn: Arc<dyn Fn() -> anyhow::Result<()>>` → REMOVED
- `mode: ExecutionMode` → NEW
- `setup` and `collect` closures → NEW (optional, for pre/post-execution logic)
- `timeout` moved into `ExecutionMode::Direct` (not a field on `Task`)

---

### 3. Update `Task` Constructor and Builder Methods

```rust
impl Task {
    pub fn new(id: impl Into<String>, mode: ExecutionMode) -> Self {
        Self {
            id: id.into(),
            dependencies: Vec::new(),
            workdir: PathBuf::from("."),
            mode,
            setup: None,
            collect: None,
            monitors: Vec::new(),
        }
    }
    
    pub fn depends_on(mut self, id: impl Into<String>) -> Self {
        self.dependencies.push(id.into());
        self
    }
    
    pub fn workdir(mut self, path: impl Into<PathBuf>) -> Self {
        self.workdir = path.into();
        self
    }
    
    pub fn setup<F>(mut self, f: F) -> Self
    where
        F: Fn(&Path) -> Result<(), WorkflowError> + Send + Sync + 'static,
    {
        self.setup = Some(Box::new(f));
        self
    }
    
    pub fn collect<F>(mut self, f: F) -> Self
    where
        F: Fn(&Path) -> Result<(), WorkflowError> + Send + Sync + 'static,
    {
        self.collect = Some(Box::new(f));
        self
    }
    
    pub fn monitors(mut self, hooks: Vec<MonitoringHook>) -> Self {
        self.monitors = hooks;
        self
    }
    
    pub fn add_monitor(mut self, hook: MonitoringHook) -> Self {
        self.monitors.push(hook);
        self
    }
}
```

---

### 4. Update Tests in `workflow_core/src/task.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn task_builder() {
        let t = Task::new(
            "my_task",
            ExecutionMode::Direct {
                command: "echo".into(),
                args: vec!["test".into()],
                env: HashMap::new(),
                timeout: None,
            },
        );
        assert_eq!(t.id, "my_task");
        assert!(t.dependencies.is_empty());
        assert!(t.monitors.is_empty());
    }

    #[test]
    fn depends_on_chaining() {
        let t = Task::new(
            "t",
            ExecutionMode::Direct {
                command: "true".into(),
                args: vec![],
                env: HashMap::new(),
                timeout: None,
            },
        )
        .depends_on("a")
        .depends_on("b");
        assert_eq!(t.dependencies, vec!["a", "b"]);
    }
}
```

---

## Part B: Workflow Engine Rewrite (TASK-11)

### 5. Remove `bon` Dependency

**In `workflow_core/Cargo.toml`**:
```toml
# Remove this line:
bon = { workspace = true }
```

**In `workflow_core/src/workflow.rs`**:
```rust
// Remove this line:
use bon::bon;

// Remove these attributes from impl block:
#[bon]
#[builder]
```

---

### 6. Redesign `Workflow` Struct

```rust
use crate::monitoring::{HookExecutor, HookContext, HookTrigger};
use crate::process::{ProcessRunner, ProcessHandle};
use crate::state::StateStore;
use std::sync::Arc;

pub struct Workflow {
    pub name: String,
    tasks: HashMap<String, Task>,
    max_parallel: usize,
}
```

**Key changes**:
- `state_path: PathBuf` → REMOVED (state is now injected via `run()`)
- No `process_runner`, `hook_executor`, `state` fields — these are passed to `run()`

---

### 7. Update `Workflow` Constructor

```rust
impl Workflow {
    pub fn new(name: impl Into<String>) -> Self {
        let max_parallel = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        
        Self {
            name: name.into(),
            tasks: HashMap::new(),
            max_parallel,
        }
    }
    
    pub fn with_max_parallel(mut self, n: usize) -> Result<Self, WorkflowError> {
        if n == 0 {
            return Err(WorkflowError::InvalidConfig(
                "max_parallel must be at least 1".into(),
            ));
        }
        self.max_parallel = n;
        Ok(self)
    }
    
    pub fn add_task(&mut self, task: Task) -> Result<(), WorkflowError> {
        if self.tasks.contains_key(&task.id) {
            return Err(WorkflowError::DuplicateTaskId(task.id.clone()));
        }
        self.tasks.insert(task.id.clone(), task);
        Ok(())
    }
    
    pub fn dry_run(&self) -> Result<Vec<String>, WorkflowError> {
        Ok(self.build_dag()?.topological_order())
    }
}
```

**Key changes**:
- No `state_dir` parameter
- `with_max_parallel` returns `Result` (validation moved here from constructor)
- `resume()` method REMOVED (resume logic moves to caller)

---

### 8. Define `WorkflowSummary` (TASK-10)

Add to `workflow_core/src/workflow.rs`:

```rust
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct WorkflowSummary {
    pub succeeded: Vec<String>,
    pub failed: Vec<(String, String)>,  // (task_id, error_message)
    pub skipped: Vec<String>,
    pub duration: Duration,
}
```

Re-export from `workflow_core/src/lib.rs`:
```rust
pub use workflow::{Workflow, WorkflowSummary};
```

---

### 9. Rewrite `Workflow::run()` Signature

```rust
pub fn run(
    &mut self,
    state: &mut dyn StateStore,
    runner: Arc<dyn ProcessRunner>,
    hook_executor: Arc<dyn HookExecutor>,
) -> Result<WorkflowSummary, WorkflowError>
```

**Key changes**:
- Returns `WorkflowSummary` instead of `()`
- Accepts `state`, `runner`, `hook_executor` as parameters (dependency injection)
- Individual task failures are recorded in `WorkflowSummary::failed`, not returned as `Err`
- `Err(WorkflowError)` is reserved for infrastructure failures (cycle detection, state corruption, signal interruption)

---

### 10. Rewrite `Workflow::run()` Implementation

**High-level structure** (full implementation is ~300 lines, showing key changes):

```rust
pub fn run(
    &mut self,
    state: &mut dyn StateStore,
    runner: Arc<dyn ProcessRunner>,
    hook_executor: Arc<dyn HookExecutor>,
) -> Result<WorkflowSummary, WorkflowError> {
    let dag = self.build_dag()?;
    
    // Initialize state for all tasks
    for id in dag.task_ids() {
        if state.get_status(id).is_none() {
            state.set_status(id, TaskStatus::Pending);
        }
    }
    state.save()?;
    
    let mut handles: HashMap<String, (Box<dyn ProcessHandle>, Instant, String)> = HashMap::new();
    let workflow_start = Instant::now();
    
    loop {
        // Poll finished tasks
        let finished: Vec<String> = handles
            .keys()
            .filter(|id| !handles[*id].0.is_running())
            .cloned()
            .collect();
        
        for id in finished {
            let (mut handle, start, task_id) = handles.remove(&id).unwrap();
            let duration = start.elapsed();
            
            // Execute OnComplete/OnFailure hooks via hook_executor
            let result = handle.wait();
            match result {
                Ok(process_result) if process_result.exit_code == Some(0) => {
                    state.mark_completed(&id);
                    // Execute OnComplete hooks
                }
                Ok(process_result) => {
                    let error = format!("exit code {}", process_result.exit_code.unwrap_or(-1));
                    state.mark_failed(&id, error);
                    // Execute OnFailure hooks
                }
                Err(e) => {
                    state.mark_failed(&id, e.to_string());
                }
            }
            state.save()?;
        }
        
        // Skip propagation (unchanged logic)
        // ...
        
        // Dispatch ready tasks
        let done_set: HashSet<String> = state.all_tasks()
            .iter()
            .filter(|(_, v)| matches!(v, TaskStatus::Completed | TaskStatus::Skipped | TaskStatus::SkippedDueToDependencyFailure))
            .map(|(k, _)| k.clone())
            .collect();
        
        for id in dag.ready_tasks(&done_set) {
            if handles.len() >= self.max_parallel {
                break;
            }
            if matches!(state.get_status(&id), Some(TaskStatus::Pending)) {
                // Take task from HashMap (consume it)
                if let Some(task) = self.tasks.remove(&id) {
                    state.mark_running(&id);
                    
                    // Execute OnStart hooks
                    // ...
                    
                    // Spawn process via runner
                    match &task.mode {
                        ExecutionMode::Direct { command, args, env, timeout } => {
                            // Run setup closure if present
                            if let Some(setup) = &task.setup {
                                if let Err(e) = setup(&task.workdir) {
                                    state.mark_failed(&id, e.to_string());
                                    state.save()?;
                                    continue;
                                }
                            }
                            
                            let handle = runner.spawn(&task.workdir, command, args, env)?;
                            handles.insert(id.clone(), (handle, Instant::now(), id));
                        }
                        ExecutionMode::Queued { .. } => {
                            return Err(WorkflowError::Io(std::io::Error::other(
                                "queued execution not yet implemented"
                            )));
                        }
                    }
                }
            }
        }
        
        // Check if all done
        let all_done = dag.task_ids().all(|id| {
            matches!(
                state.get_status(id),
                Some(TaskStatus::Completed)
                    | Some(TaskStatus::Failed { .. })
                    | Some(TaskStatus::Skipped)
                    | Some(TaskStatus::SkippedDueToDependencyFailure)
            )
        });
        
        if all_done && handles.is_empty() {
            break;
        }
        
        std::thread::sleep(Duration::from_millis(50));
    }
    
    // Build WorkflowSummary
    let mut succeeded = Vec::new();
    let mut failed = Vec::new();
    let mut skipped = Vec::new();
    
    for (id, status) in state.all_tasks() {
        match status {
            TaskStatus::Completed => succeeded.push(id.clone()),
            TaskStatus::Failed { error } => failed.push((id.clone(), error.clone())),
            TaskStatus::Skipped | TaskStatus::SkippedDueToDependencyFailure => {
                skipped.push(id.clone())
            }
            _ => {}
        }
    }
    
    Ok(WorkflowSummary {
        succeeded,
        failed,
        skipped,
        duration: workflow_start.elapsed(),
    })
}
```

**Critical implementation notes**:

1. **Tasks are consumed**: `self.tasks.remove(&id)` takes ownership. The task is no longer in the HashMap after dispatch.

2. **`setup` and `collect` closures**: Run in the main thread (not spawned). If they fail, mark the task as failed immediately.

3. **Hook execution**: Use `hook_executor.execute_hook(&hook, &ctx)` instead of calling `hook.execute()` directly.

4. **`PeriodicHookManager`**: Either implement it properly or remove it entirely. Do not carry a stub through this rewrite.

5. **Error contract**:
   - `Ok(WorkflowSummary)` even if tasks fail
   - `Err(WorkflowError::CycleDetected)` if DAG has cycle
   - `Err(WorkflowError::Io(...))` if state save fails
   - `Err(WorkflowError::Interrupted)` reserved for TASK-15 (signal handling)

---

### 11. Update All Tests in `workflow_core/src/workflow.rs`

**Test migration pattern**:

```rust
// Before (TASK-8 style):
let dir = tempdir().unwrap();
let mut wf = Workflow::builder()
    .name("wf_single".to_string())
    .state_dir(dir.path().to_path_buf())
    .max_parallel(4)
    .build()
    .unwrap();
wf.add_task(Task::new("a", || Ok::<_, anyhow::Error>(()))).unwrap();
wf.run().unwrap();

// After (TASK-9+11 style):
let dir = tempdir().unwrap();
let state_path = dir.path().join(".wf_single.workflow.json");
let mut state = Box::new(JsonStateStore::new("wf_single", state_path));
let runner = Arc::new(workflow_utils::SystemProcessRunner);
let executor = Arc::new(workflow_utils::ShellHookExecutor);

let mut wf = Workflow::new("wf_single")
    .with_max_parallel(4)
    .unwrap();

wf.add_task(Task::new(
    "a",
    ExecutionMode::Direct {
        command: "true".into(),
        args: vec![],
        env: HashMap::new(),
        timeout: None,
    },
)).unwrap();

let summary = wf.run(state.as_mut(), runner, executor).unwrap();
assert_eq!(summary.succeeded.len(), 1);
```

---

### 12. Specific Test Migrations

**`single_task_completes`** (line 435):
- Use `ExecutionMode::Direct { command: "true", ... }`
- Use `setup` closure to set the flag: `.setup(move |_| { *flag2.lock().unwrap() = true; Ok(()) })`

**`chain_respects_order`** (line 455):
- **CRITICAL REDESIGN REQUIRED**: Cannot use `Arc<Mutex<Vec<String>>>` with shell commands
- Solution: Use a temp file that each task appends to
```rust
let log_file = dir.path().join("log.txt");
let log1 = log_file.clone();
let log2 = log_file.clone();

wf.add_task(
    Task::new("a", ExecutionMode::Direct { command: "true".into(), ... })
        .setup(move |_| {
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log1)?;
            writeln!(f, "a")?;
            Ok(())
        })
).unwrap();

wf.add_task(
    Task::new("b", ExecutionMode::Direct { command: "true".into(), ... })
        .depends_on("a")
        .setup(move |_| {
            let mut f = std::fs::OpenOptions::new()
                .append(true)
                .open(&log2)?;
            writeln!(f, "b")?;
            Ok(())
        })
).unwrap();

wf.run(...).unwrap();

let log = std::fs::read_to_string(&log_file).unwrap();
assert_eq!(log.lines().collect::<Vec<_>>(), vec!["a", "b"]);
```

**`failed_task_skips_dependent`** (line 483):
- Use `ExecutionMode::Direct { command: "false".into(), ... }` for the failing task
- `false` is a POSIX command that exits with code 1

**`dry_run_returns_topo_order`** (line 503):
- No changes needed (doesn't call `run()`)

**`duplicate_task_id_errors`** (line 521):
- Update task construction only

**`valid_dependency_add`** (line 538):
- Update task construction only

**`builder_with_custom_max_parallel`** (line 551):
- Replace `.builder().max_parallel(4).build()` with `.new("test").with_max_parallel(4).unwrap()`

**`builder_validation_zero_parallelism`** (line 562):
- Replace `.builder().max_parallel(0).build()` with `.new("test").with_max_parallel(0)` (returns `Err`)

**`resume_uses_builder`** (line 572):
- **REMOVE THIS TEST** — `resume()` method no longer exists

**`resume_loads_existing_state`** (line 579):
- Redesign: create two workflows, run first, load state, run second
```rust
let dir = tempdir().unwrap();
let state_path = dir.path().join(".wf_resume.workflow.json");

// First run
let mut state1 = Box::new(JsonStateStore::new("wf_resume", state_path.clone()));
let mut wf1 = Workflow::new("wf_resume");
wf1.add_task(Task::new("a", ExecutionMode::Direct { command: "true".into(), ... })).unwrap();
wf1.run(state1.as_mut(), runner.clone(), executor.clone()).unwrap();

// Second run (resume)
let mut state2 = Box::new(JsonStateStore::load(&state_path).unwrap());
let mut wf2 = Workflow::new("wf_resume");
wf2.add_task(Task::new("a", ExecutionMode::Direct { command: "false".into(), ... })).unwrap();
wf2.run(state2.as_mut(), runner, executor).unwrap();

// Task "a" should still be Completed (not re-run)
assert!(state2.is_completed("a"));
```

---

### 13. Add `workflow_utils` as Dev-Dependency

**In `workflow_core/Cargo.toml`**:
```toml
[dev-dependencies]
workflow_utils = { path = "../workflow_utils" }
tempfile = { workspace = true }
```

---

### 14. Verification Commands

```bash
# Check compilation
cargo check -p workflow_core

# Run task tests
cargo test -p workflow_core -- task::tests

# Run workflow tests
cargo test -p workflow_core -- workflow::tests

# Verify all workspace tests pass
cargo test --workspace
```

---

## Call Site Reference Table

| File | Line(s) | Change Required |
|------|---------|-----------------|
| `workflow_core/src/task.rs` | 9 | Remove `execute_fn` field |
| `workflow_core/src/task.rs` | 15-26 | Replace `new()` signature |
| `workflow_core/src/task.rs` | 54-67 | Update tests |
| `workflow_core/src/workflow.rs` | 6 | Remove `use bon::bon;` |
| `workflow_core/src/workflow.rs` | 77-78 | Remove `#[bon]` and `#[builder]` |
| `workflow_core/src/workflow.rs` | 70-75 | Remove `state_path` field |
| `workflow_core/src/workflow.rs` | 79-105 | Replace constructor |
| `workflow_core/src/workflow.rs` | 120-128 | Remove `resume()` method |
| `workflow_core/src/workflow.rs` | 130-388 | Rewrite `run()` method |
| `workflow_core/src/workflow.rs` | 435-597 | Update all 10 tests |
| `workflow_core/Cargo.toml` | - | Remove `bon` dependency |
| `workflow_core/src/lib.rs` | - | Add `WorkflowSummary` re-export |

---

## Notes for Implementer

1. **This is a single atomic commit** — do not split TASK-9 and TASK-11. The codebase will not compile in between.

2. **Test redesign is non-trivial** — `chain_respects_order` requires file I/O instead of in-process side effects. This is a meaningful test design change.

3. **Platform assumption** — Tests use `true`, `false`, `echo`, `sh -c`. These are POSIX commands available on Linux/macOS. Document this explicitly.

4. **`PeriodicHookManager` decision** — Either implement it properly in this task or remove it entirely. Do not carry a stub.

5. **`bon` removal** — After this task, `bon` can be removed from workspace `Cargo.toml` (TASK-14).

6. **Resume logic moves to caller** — The `Workflow::resume()` method is removed. Callers must construct `JsonStateStore::load(path)` and pass it to `run()`.

7. **Error contract is critical** — Individual task failures return `Ok(WorkflowSummary { failed: [...] })`, NOT `Err(WorkflowError)`. Only infrastructure failures return `Err`.
