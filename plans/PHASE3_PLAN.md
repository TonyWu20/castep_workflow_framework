# Phase 3: Production Trust

**Goal:** Make the framework trustworthy for overnight HPC cluster runs, not just test-passing.

**Theme:** A researcher can start a 100-task workflow, go home, come back, and have confidence in what happened.

## Part 1: Engine Hardening (library-level, no UX)

### 1. Flip dependency direction

Move `MonitoringHook`, `HookContext`, `HookTrigger`, `HookResult` from `workflow_utils` into `workflow_core`. After this, `workflow_utils` depends on `workflow_core` (not vice versa).

`MonitoringHook::execute()` currently uses `TaskExecutor` (which lives in `workflow_utils`). After the flip, the hook type lives in `workflow_core` but execution logic stays in `workflow_utils` — either as an extension trait or a standalone function.

### 2. Redesign Task model: setup → execution → collect

Drop the opaque `Arc<dyn Fn() -> Result<()>>` closure entirely. No backwards compatibility concerns (crate is unpublished).

Replace with a structured three-phase lifecycle:

```rust
pub struct Task {
    pub id: String,
    pub dependencies: Vec<String>,
    pub workdir: PathBuf,
    pub setup: Option<Arc<dyn Fn(&Path) -> Result<()> + Send + Sync>>,
    pub execution: ExecutionMode,
    pub collect: Option<Arc<dyn Fn(&Path) -> Result<()> + Send + Sync>>,
    pub monitors: Vec<MonitoringHook>,
}

pub enum ExecutionMode {
    Direct {
        command: String,
        args: Vec<String>,
        timeout: Option<Duration>,
    },
    Queued {
        submit_cmd: String,
        poll_cmd: String,
        cancel_cmd: String,
    },
}
```

**Rationale:**
- `setup` (optional closure): prepare input files. Runs after dependencies complete. Receives `&Path` (workdir).
- `execution` (required): the actual computation. Framework-owned — can timeout, kill, cancel.
- `collect` (optional closure): extract results from output files (e.g., grep magnetic moments from `.castep`, write CSV). Named "collect" not "teardown" because it harvests scientific results, not cleanup.
- `Direct` mode: framework spawns the process, monitors it, can enforce timeout. Fully implemented in Phase 3.
- `Queued` mode: for HPC queue systems (SLURM/PBS). Type defined in Phase 3, execution stubbed/unimplemented. Enables downstream code to prepare for it.

### 3. StateStore trait + JSON implementation

Abstract state persistence behind a trait. The one justified trait in the project — state persistence is an I/O boundary.

```rust
pub trait StateStore {
    fn get_status(&self, id: &str) -> Option<TaskStatus>;
    fn set_status(&mut self, id: &str, status: TaskStatus);
    fn tasks_with_status(&self, status: &TaskStatus) -> Vec<String>;
    fn mark_running(&mut self, id: &str);
    fn mark_completed(&mut self, id: &str);
    fn mark_failed(&mut self, id: &str, error: String);
    fn mark_skipped(&mut self, id: &str);
    fn mark_skipped_due_to_dep_failure(&mut self, id: &str);
    fn save(&self) -> Result<()>;
    fn load(path: impl AsRef<Path>) -> Result<Self> where Self: Sized;
    fn all_tasks(&self) -> HashMap<String, TaskStatus>;
    fn summary(&self) -> StateSummary;
}
```

Current `WorkflowState` becomes `JsonStateStore` implementing this trait. `workflow.rs` operates against the trait, not the concrete type.

### 4. Structured errors (thiserror)

Replace `anyhow::Result` in `workflow_core`'s public API with a `#[non_exhaustive]` error enum:

```rust
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum WorkflowError {
    #[error("duplicate task id: {0}")]
    DuplicateTaskId(String),
    #[error("dependency cycle detected")]
    CycleDetected,
    #[error("unknown dependency '{dependency}' in task '{task}'")]
    UnknownDependency { task: String, dependency: String },
    #[error("state file corrupted: {0}")]
    StateCorrupted(String),
    #[error("task '{0}' timed out")]
    TaskTimeout(String),
    #[error("task '{0}' failed: {1}")]
    TaskFailed(String, String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
```

`#[non_exhaustive]` ensures adding new variants is not a breaking change.

### 5. Atomic state writes

Change `JsonStateStore::save()` to write to a temp file in the same directory, then `std::fs::rename` into place. Prevents corrupted state on SIGKILL/power loss.

### 6. `run()` → `Result<WorkflowSummary>`

```rust
pub struct WorkflowSummary {
    pub succeeded: Vec<String>,
    pub failed: Vec<(String, String)>,  // (task_id, error_message)
    pub skipped: Vec<String>,
    pub duration: Duration,
}
```

The caller can programmatically detect partial failure instead of the current silent `Ok(())`.

### 7. Signal handling (SIGTERM)

Catch SIGTERM (sent by HPC batch schedulers before SIGKILL). On receipt:
1. Set a shared `AtomicBool` flag
2. The run loop checks this flag each iteration
3. Mark all currently running tasks as `Pending` in state
4. Save state
5. Exit gracefully

This enables clean resume after HPC job preemption.

### 8. Resume resets Failed → Pending

When loading state for resume, reset `Failed` and `SkippedDueToDependencyFailure` to `Pending` (in addition to the existing `Running` → `Pending` reset). Default behavior, no opt-out flag.

### 9. Direct mode timeout

For `ExecutionMode::Direct`: spawn the process via `TaskExecutor::spawn()`, poll `is_running()` with a deadline. If `timeout` is `Some(d)` and elapsed > d, call `terminate()` and mark the task as `TaskTimeout`. This works because the framework owns the process handle.

### 10. Clean up dead workspace

Remove unused workspace dependencies: `tokio`, `async-trait`, `tokio-rusqlite`, `tokio-util`, `toml`. Clean up any stale references.

## Part 2: workflow-cli binary

A new `workflow-cli` crate in the workspace. Depends on `workflow_core`.

### Subcommands

- `workflow-cli status <state-file>` — show task status summary (completed/failed/skipped counts, failed task IDs)
- `workflow-cli retry <state-file> <task-id>...` — mark specific tasks + their downstream dependents as Pending
- `workflow-cli inspect <state-file> [task-id]` — detailed task info

### Design

Retry logic lives in `workflow_core` as a library method on the `StateStore` (or a free function). The CLI is a thin wrapper using `clap`.

## Implementation Order

The dependency flip is foundational — everything else builds on it. Suggested order:

1. Flip dependency direction (hooks → core)
2. StateStore trait + JSON impl + atomic writes (items 3, 5)
3. Structured errors (item 4)
4. Task model redesign (item 2) — this is the biggest change
5. run() → WorkflowSummary (item 6)
6. Resume resets Failed → Pending (item 8)
7. Signal handling (item 7)
8. Direct mode timeout (item 9)
9. workflow-cli (Part 2)
10. Workspace cleanup (item 10) — alongside other work

## Files to Modify

- `workflow_core/src/task.rs` — new Task struct, ExecutionMode enum
- `workflow_core/src/state.rs` — StateStore trait, JsonStateStore, atomic writes
- `workflow_core/src/workflow.rs` — new run loop for ExecutionMode, WorkflowSummary, signal handling
- `workflow_core/src/lib.rs` — re-exports, error module
- `workflow_core/src/monitoring.rs` — NEW, moved from workflow_utils
- `workflow_core/src/error.rs` — NEW, WorkflowError enum
- `workflow_utils/src/monitoring.rs` — reduce to execution extension only
- `workflow_utils/src/lib.rs` — update re-exports
- `workflow_utils/Cargo.toml` — add dep on workflow_core
- `workflow_core/Cargo.toml` — remove dep on workflow_utils, add thiserror
- `Cargo.toml` (workspace) — add workflow-cli member, clean unused deps
- `workflow-cli/` — NEW crate
- `examples/hubbard_u_sweep/src/main.rs` — update to new Task API
- All tests — update to new Task model
