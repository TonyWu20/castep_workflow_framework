## Fix Plan for `phase-4`

### TASK-1: Remove duplicated dead code in `process_finished`

**File:** `workflow_core/src/workflow.rs`
**Type:** replace

**Before:**
```rust
    let (final_state, exit_code) = if let Ok(process_result) = t.handle.wait() {
        match process_result.exit_code {
            Some(0) => {
                state.mark_completed(id);
                if let Some(ref collect) = t.collect {
                    if let Err(e) = collect(&t.workdir) {
                        tracing::warn!(
                            "Collect closure for task '{}' failed: {}",
                            id,
                            e
                        );
                    }
                }
                ("completed", process_result.exit_code)
            }
            _ => {
                state.mark_failed(
                    id,
                    format!("exit code {}", process_result.exit_code.unwrap_or(-1)),
                );
                ("failed", process_result.exit_code)
            }
        }
    } else {
        state.mark_failed(id, "process terminated".to_string());
        ("failed", None)
    };

    let final_state = if exit_code == Some(0) {
        crate::monitoring::TaskPhase::Completed
    } else {
        crate::monitoring::TaskPhase::Failed
    };

    let task_phase = if exit_code == Some(0) {
        crate::monitoring::TaskPhase::Completed
    } else {
        crate::monitoring::TaskPhase::Failed
    };

    fire_hooks(
        &t.monitors,
        &t.workdir,
        task_phase,
        exit_code,
        id,
        hook_executor,
    );
```

**After:**
```rust
    let exit_code = if let Ok(process_result) = t.handle.wait() {
        match process_result.exit_code {
            Some(0) => {
                state.mark_completed(id);
                if let Some(ref collect) = t.collect {
                    if let Err(e) = collect(&t.workdir) {
                        tracing::warn!(
                            "Collect closure for task '{}' failed: {}",
                            id,
                            e
                        );
                    }
                }
                process_result.exit_code
            }
            _ => {
                state.mark_failed(
                    id,
                    format!("exit code {}", process_result.exit_code.unwrap_or(-1)),
                );
                process_result.exit_code
            }
        }
    } else {
        state.mark_failed(id, "process terminated".to_string());
        None
    };

    let task_phase = if exit_code == Some(0) {
        crate::monitoring::TaskPhase::Completed
    } else {
        crate::monitoring::TaskPhase::Failed
    };

    fire_hooks(
        &t.monitors,
        &t.workdir,
        task_phase,
        exit_code,
        id,
        hook_executor,
    );
```

**Acceptance:** `cargo check -p workflow_core`

---

### TASK-2: Add `Copy, PartialEq, Eq` derives to `TaskPhase`

**File:** `workflow_core/src/monitoring.rs`
**Type:** replace

**Before:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskPhase {
```

**After:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskPhase {
```

**Acceptance:** `cargo check -p workflow_core`

---

### TASK-3: Remove `.clone()` on `TaskPhase` in `fire_hooks`

**File:** `workflow_core/src/workflow.rs`
**Type:** replace

**Change 1:**

**Before:**
```rust
fn fire_hooks(
    monitors: &[crate::monitoring::MonitoringHook],
    workdir: &std::path::Path,
    phase: crate::monitoring::TaskPhase,
    exit_code: Option<i32>,
    task_id: &str,
    hook_executor: &dyn HookExecutor,
) {
    let ctx = crate::monitoring::HookContext {
        task_id: task_id.to_string(),
        workdir: workdir.to_path_buf(),
        phase: phase.clone(),
        exit_code,
    };
    for hook in monitors {
        let should_fire = matches!(
            (&hook.trigger, phase.clone()),
            (crate::monitoring::HookTrigger::OnStart, crate::monitoring::TaskPhase::Running)
                | (crate::monitoring::HookTrigger::OnComplete, crate::monitoring::TaskPhase::Completed)
                | (crate::monitoring::HookTrigger::OnFailure, crate::monitoring::TaskPhase::Failed)
        );
```

**After:**
```rust
fn fire_hooks(
    monitors: &[crate::monitoring::MonitoringHook],
    workdir: &std::path::Path,
    phase: crate::monitoring::TaskPhase,
    exit_code: Option<i32>,
    task_id: &str,
    hook_executor: &dyn HookExecutor,
) {
    let ctx = crate::monitoring::HookContext {
        task_id: task_id.to_string(),
        workdir: workdir.to_path_buf(),
        phase,
        exit_code,
    };
    for hook in monitors {
        let should_fire = matches!(
            (&hook.trigger, phase),
            (crate::monitoring::HookTrigger::OnStart, crate::monitoring::TaskPhase::Running)
                | (crate::monitoring::HookTrigger::OnComplete, crate::monitoring::TaskPhase::Completed)
                | (crate::monitoring::HookTrigger::OnFailure, crate::monitoring::TaskPhase::Failed)
        );
```

**Acceptance:** `cargo check -p workflow_core`

---

### TASK-4: Simplify `ExecutionMode::Queued` to unit-like variant

**File:** `workflow_core/src/task.rs`
**Type:** replace

**Before:**
```rust
    /// Not yet implemented. Constructing a task with this mode will cause
    /// `Workflow::run()` to return `Err(WorkflowError::InvalidConfig)`.
    /// Reserved for future HPC queue integration (SLURM/PBS).
    Queued {
        submit_cmd: String,
        poll_cmd: String,
        cancel_cmd: String,
    },
```

**After:**
```rust
    /// Queued execution via an HPC scheduler (SLURM/PBS).
    /// The actual submit/poll/cancel commands are owned by the `QueuedSubmitter`
    /// implementation set via `Workflow::with_queued_submitter()`.
    Queued,
```

**Acceptance:** `cargo check --workspace`

---

### TASK-5: Update `Queued` match arm in `workflow.rs` for unit variant

**File:** `workflow_core/src/workflow.rs`
**Type:** replace

**Before:**
```rust
                            ExecutionMode::Queued { submit_cmd, poll_cmd, cancel_cmd } => {
```

**After:**
```rust
                            ExecutionMode::Queued => {
```

**Acceptance:** `cargo check --workspace`

---

### TASK-6: Replace `pub use queued::*` with explicit re-exports

**File:** `workflow_utils/src/lib.rs`
**Type:** replace

**Before:**
```rust
pub use queued::*;
```

**After:**
```rust
pub use queued::{QueuedRunner, SchedulerKind};
```

**Acceptance:** `cargo check --workspace`

---

### TASK-7: Remove dead `workdir` field from `QueuedProcessHandle`

**File:** `workflow_utils/src/queued.rs`
**Type:** replace

**Change 1: Remove field declaration**

**Before:**
```rust
pub struct QueuedProcessHandle {
    job_id: String,
    poll_cmd: String,
    cancel_cmd: String,
    workdir: PathBuf,
    stdout_path: PathBuf,
```

**After:**
```rust
pub struct QueuedProcessHandle {
    job_id: String,
    poll_cmd: String,
    cancel_cmd: String,
    stdout_path: PathBuf,
```

**Change 2: Remove field in constructor**

**Before:**
```rust
        Ok(Box::new(QueuedProcessHandle {
            job_id,
            poll_cmd: self.build_poll_cmd(),
            cancel_cmd: self.build_cancel_cmd(),
            workdir: workdir.to_path_buf(),
            stdout_path,
```

**After:**
```rust
        Ok(Box::new(QueuedProcessHandle {
            job_id,
            poll_cmd: self.build_poll_cmd(),
            cancel_cmd: self.build_cancel_cmd(),
            stdout_path,
```

**Acceptance:** `cargo check -p workflow_utils`

---

### TASK-8: Add `#[derive(Default)]` to `SystemProcessRunner`

**File:** `workflow_utils/src/executor.rs`
**Type:** replace

**Before:**
```rust
/// Concrete implementation of the ProcessRunner trait for system processes.
/// Wraps `std::process::Child` with output capture and timing.
pub struct SystemProcessRunner {
```

**After:**
```rust
/// Concrete implementation of the ProcessRunner trait for system processes.
/// Wraps `std::process::Child` with output capture and timing.
#[derive(Default)]
pub struct SystemProcessRunner {
```

**Acceptance:** `cargo check -p workflow_utils`

---

### TASK-9: Reduce periodic hook test sleep from 8s to 2s

**File:** `workflow_core/tests/hook_recording.rs`
**Type:** replace

**Before:**
```rust
    wf.add_task(
        Task::new("long_task", direct_with_args("sleep", &["8"]))
            .monitors(vec![periodic_hook])
    ).unwrap();
```

**After:**
```rust
    wf.add_task(
        Task::new("long_task", direct_with_args("sleep", &["2"]))
            .monitors(vec![periodic_hook])
    ).unwrap();
```

**Acceptance:** `cargo test -p workflow_core --test hook_recording periodic_hook_fires_during_long_task`

---

### TASK-10: Implement TASK-8 from phase-4 plan (task_successors + graph-aware retry)

**File:** `workflow_core/src/state.rs`
**Type:** replace

This task requires implementing the full TASK-8 spec from `plans/phase-4/PHASE4_IMPLEMENTATION_PLAN.md`:
1. Add `task_successors: HashMap<String, Vec<String>>` with `#[serde(default)]` to `JsonStateStore`
2. Initialize it in `new()`
3. Add `set_task_graph()` to `StateStore` trait (default no-op)
4. Implement `set_task_graph()` for `JsonStateStore`
5. Add `task_successors()` getter to `JsonStateStore`
6. Persist successor graph in `Workflow::run()` after `build_dag()`
7. Implement `downstream_tasks()` and graph-aware `cmd_retry` in `workflow-cli/src/main.rs`

**Acceptance:** `cargo check --workspace`; `cargo test --workspace`

---

### TASK-11: Fix and commit TASK-12 integration test

**File:** `workflow_utils/tests/queued_integration.rs`
**Type:** replace

The test `submit_with_mock_sbatch_returns_on_disk_handle` fails because the mock sbatch script's PATH isn't inherited properly by the `sh -c` subprocess. Additionally, add `serial_test` as a dev-dependency and `#[serial]` to tests that mutate PATH.

**Acceptance:** `cargo test -p workflow_utils --test queued_integration`

---

## Dependency Graph

- TASK-1: independent
- TASK-2: independent
- TASK-3: depends on TASK-2
- TASK-4: independent
- TASK-5: depends on TASK-4
- TASK-6: independent
- TASK-7: independent
- TASK-8: independent
- TASK-9: independent
- TASK-10: independent (but large — implements missing feature)
- TASK-11: depends on TASK-6 (explicit re-exports)

## Parallel Groups

- Group A (independent): TASK-1, TASK-2, TASK-4, TASK-6, TASK-7, TASK-8, TASK-9, TASK-10
- Group B (after Group A): TASK-3, TASK-5, TASK-11
