# Phase 4: Real HPC Execution — Implementation Plan

## Status

Tasks TASK-1 through TASK-6, TASK-9, TASK-10, and TASK-11 are **already implemented** in the codebase (verified against source files on 2026-04-19). This document contains only the **remaining work** in compilable-plan-spec format.

## Remaining Tasks

| Task | Description | Status |
|------|-------------|--------|
| TASK-7 | `QueuedRunner` must implement `QueuedSubmitter` trait | Partial — `pub fn submit` exists as a direct method, not a trait impl |
| TASK-8 | `task_successors` in `JsonStateStore` + graph-aware `cmd_retry` | Not started |
| TASK-12 | Integration test for queued execution lifecycle | Not started |

## Execution Order

1. TASK-7 (prerequisite for TASK-12)
2. TASK-8 (independent of TASK-7)
3. TASK-12 (depends on TASK-7)

---

### TASK-7: Implement `QueuedSubmitter` trait for `QueuedRunner`

**Type:** replace

**Change 1:**
**File:** `workflow_utils/src/queued.rs`

**Before:**
```rust
    pub fn submit(
        &self,
        workdir: &Path,
        task_id: &str,
        log_dir: &Path,
    ) -> Result<Box<dyn ProcessHandle>, WorkflowError> {
        let submit_cmd = self.build_submit_cmd(
            &workdir.join("job.sh").to_string_lossy(), task_id, log_dir
        );
        let output = Command::new("sh")
            .args(["-c", &submit_cmd])
            .current_dir(workdir)
            .output()
            .map_err(WorkflowError::Io)?;

        if !output.status.success() {
            return Err(WorkflowError::QueueSubmitFailed(
                String::from_utf8_lossy(&output.stderr).into_owned()
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let job_id = self.parse_job_id(&stdout)?;

        let stdout_path = log_dir.join(format!("{}.stdout", task_id));
        let stderr_path = log_dir.join(format!("{}.stderr", task_id));

        Ok(Box::new(QueuedProcessHandle {
            job_id,
            poll_cmd: self.build_poll_cmd(),
            cancel_cmd: self.build_cancel_cmd(),
            workdir: workdir.to_path_buf(),
            stdout_path,
            stderr_path,
            last_poll: Instant::now(),
            poll_interval: Duration::from_secs(15),
            cached_running: true,
            finished_exit_code: None,
            started_at: Instant::now(),
        }))
    }
}
```

**After:**
```rust
}

impl workflow_core::process::QueuedSubmitter for QueuedRunner {
    fn submit(
        &self,
        workdir: &Path,
        task_id: &str,
        log_dir: &Path,
    ) -> Result<Box<dyn ProcessHandle>, WorkflowError> {
        let submit_cmd = self.build_submit_cmd(
            &workdir.join("job.sh").to_string_lossy(), task_id, log_dir
        );
        let output = Command::new("sh")
            .args(["-c", &submit_cmd])
            .current_dir(workdir)
            .output()
            .map_err(WorkflowError::Io)?;

        if !output.status.success() {
            return Err(WorkflowError::QueueSubmitFailed(
                String::from_utf8_lossy(&output.stderr).into_owned()
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let job_id = self.parse_job_id(&stdout)?;

        let stdout_path = log_dir.join(format!("{}.stdout", task_id));
        let stderr_path = log_dir.join(format!("{}.stderr", task_id));

        Ok(Box::new(QueuedProcessHandle {
            job_id,
            poll_cmd: self.build_poll_cmd(),
            cancel_cmd: self.build_cancel_cmd(),
            workdir: workdir.to_path_buf(),
            stdout_path,
            stderr_path,
            last_poll: Instant::now(),
            poll_interval: Duration::from_secs(15),
            cached_running: true,
            finished_exit_code: None,
            started_at: Instant::now(),
        }))
    }
}
```

**Acceptance:** `cargo check --workspace`

---

### TASK-8: Add `task_successors` to `JsonStateStore` and graph-aware `cmd_retry`

**Type:** replace

**Change 1 — `JsonStateStore` struct: add `task_successors` field**
**File:** `workflow_core/src/state.rs`

**Before:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonStateStore {
    workflow_name: String,
    created_at: String,
    last_updated: String,
    tasks: HashMap<String, TaskStatus>,
    path: PathBuf,
}
```

**After:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonStateStore {
    workflow_name: String,
    created_at: String,
    last_updated: String,
    tasks: HashMap<String, TaskStatus>,
    #[serde(default)]
    task_successors: HashMap<String, Vec<String>>,
    path: PathBuf,
}
```

**Change 2 — `JsonStateStore::new`: initialize `task_successors`**
**File:** `workflow_core/src/state.rs`

**Before:**
```rust
        Self {
            workflow_name: name.to_owned(),
            created_at: now.clone(),
            last_updated: now,
            tasks: HashMap::new(),
            path,
        }
```

**After:**
```rust
        Self {
            workflow_name: name.to_owned(),
            created_at: now.clone(),
            last_updated: now,
            tasks: HashMap::new(),
            task_successors: HashMap::new(),
            path,
        }
```

**Change 3 — `StateStore` trait: add `set_task_graph` default method**
**File:** `workflow_core/src/state.rs`

**Before:**
```rust
pub trait StateStore: Send + Sync {
    /// Returns the current status of a task.
    fn get_status(&self, id: &str) -> Option<TaskStatus>;

    /// Sets the status of a task and updates timestamp.
    fn set_status(&mut self, id: &str, status: TaskStatus);

    /// Returns all task IDs and their statuses.
    fn all_tasks(&self) -> Vec<(String, TaskStatus)>;

    /// Persists the current state to disk.
    fn save(&self) -> Result<(), WorkflowError>;
}
```

**After:**
```rust
pub trait StateStore: Send + Sync {
    /// Returns the current status of a task.
    fn get_status(&self, id: &str) -> Option<TaskStatus>;

    /// Sets the status of a task and updates timestamp.
    fn set_status(&mut self, id: &str, status: TaskStatus);

    /// Returns all task IDs and their statuses.
    fn all_tasks(&self) -> Vec<(String, TaskStatus)>;

    /// Persists the current state to disk.
    fn save(&self) -> Result<(), WorkflowError>;

    /// Persists the task dependency graph (successors map) for graph-aware retry.
    /// Default is a no-op; `JsonStateStore` overrides this.
    fn set_task_graph(&mut self, _successors: HashMap<String, Vec<String>>) {}
}
```

**Change 4 — `impl StateStore for JsonStateStore`: add `set_task_graph` override**
**File:** `workflow_core/src/state.rs`

**Before:**
```rust
    fn save(&self) -> Result<(), WorkflowError> {
        self.persist()
    }
}

fn now_iso8601() -> String {
```

**After:**
```rust
    fn save(&self) -> Result<(), WorkflowError> {
        self.persist()
    }

    fn set_task_graph(&mut self, successors: HashMap<String, Vec<String>>) {
        self.task_successors = successors;
    }
}

fn now_iso8601() -> String {
```

**Change 5 — `impl JsonStateStore`: add `task_successors()` getter**
**File:** `workflow_core/src/state.rs`

**Before:**
```rust
    /// Returns the path to the state file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}
```

**After:**
```rust
    /// Returns the path to the state file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the task successor graph persisted from the last workflow run.
    pub fn task_successors(&self) -> &HashMap<String, Vec<String>> {
        &self.task_successors
    }
}
```

**Change 6 — `Workflow::run`: persist successor graph after `build_dag`**
**File:** `workflow_core/src/workflow.rs`

**Before:**
```rust
        let dag = self.build_dag()?;

        // Initialize state for all tasks
        for id in dag.task_ids() {
```

**After:**
```rust
        let dag = self.build_dag()?;

        // Persist task dependency graph for CLI retry
        let successors: HashMap<String, Vec<String>> = dag.task_ids()
            .map(|id| (id.clone(), dag.successors(id)))
            .collect();
        state.set_task_graph(successors);

        // Initialize state for all tasks
        for id in dag.task_ids() {
```

**Change 7 — `workflow-cli/src/main.rs`: add `HashMap` import**
**File:** `workflow-cli/src/main.rs`

**Before:**
```rust
use clap::{Parser, Subcommand};
use workflow_core::state::{JsonStateStore, StateStore, StateStoreExt, TaskStatus};
```

**After:**
```rust
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use workflow_core::state::{JsonStateStore, StateStore, StateStoreExt, TaskStatus};
```

**Change 8 — `cmd_retry`: graph-aware implementation**
**File:** `workflow-cli/src/main.rs`

**Before:**
```rust
fn cmd_retry(state: &mut dyn StateStore, task_ids: &[String]) -> anyhow::Result<()> {
    for id in task_ids {
        if state.get_status(id).is_none() {
            eprintln!("warn: task '{}' not found", id);
        } else {
            state.mark_pending(id);
        }
    }
    // Reset all dependency-failure-skipped tasks globally (not just those downstream
    // of `task_ids`). Intentional for v0.1 simplicity — a graph-aware retry would
    // require DAG access that the CLI does not have.
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

**After:**
```rust
fn downstream_tasks(
    start: &[String],
    successors: &HashMap<String, Vec<String>>,
) -> std::collections::HashSet<String> {
    let mut visited = std::collections::HashSet::new();
    let mut queue: std::collections::VecDeque<String> = start.iter().cloned().collect();
    while let Some(id) = queue.pop_front() {
        if let Some(deps) = successors.get(&id) {
            for dep in deps {
                if visited.insert(dep.clone()) {
                    queue.push_back(dep.clone());
                }
            }
        }
    }
    visited
}

fn cmd_retry(state: &mut JsonStateStore, task_ids: &[String]) -> anyhow::Result<()> {
    for id in task_ids {
        if state.get_status(id).is_none() {
            eprintln!("warn: task '{}' not found", id);
        } else {
            state.mark_pending(id);
        }
    }

    let successors = state.task_successors().clone();
    if successors.is_empty() {
        eprintln!("warn: state file lacks dependency info; falling back to global reset");
        let to_reset: Vec<String> = state
            .all_tasks()
            .into_iter()
            .filter(|(_, s)| matches!(s, TaskStatus::SkippedDueToDependencyFailure))
            .map(|(id, _)| id)
            .collect();
        for id in to_reset {
            state.mark_pending(&id);
        }
    } else {
        let downstream = downstream_tasks(task_ids, &successors);
        let to_reset: Vec<String> = state
            .all_tasks()
            .into_iter()
            .filter(|(id, s)| {
                matches!(s, TaskStatus::SkippedDueToDependencyFailure)
                    && downstream.contains(id)
            })
            .map(|(id, _)| id)
            .collect();
        for id in to_reset {
            state.mark_pending(&id);
        }
    }

    state.save().map_err(|e| anyhow::anyhow!("failed to save state: {}", e))?;
    Ok(())
}
```

**Acceptance:** `cargo check --workspace`; `cargo test --workspace`

Note: The existing `retry_resets_failed_and_skipped_dep` test still passes because old state files have an empty `task_successors` map (via `#[serde(default)]`), triggering the global-reset fallback path. Add a new test `retry_graph_aware_resets_only_downstream` to `workflow-cli/src/main.rs` after implementing, verifying that tasks NOT downstream of the retried task are NOT reset.

---

### TASK-12: Integration test for queued execution lifecycle

**File:** `workflow_utils/tests/queued_integration.rs`
**Type:** create

**After:**
```rust
//! Integration tests for `QueuedRunner` and `QueuedProcessHandle`.
//!
//! These tests verify that `QueuedRunner` correctly implements `QueuedSubmitter`,
//! handles scheduler unavailability gracefully, and that `QueuedProcessHandle`
//! satisfies the `ProcessHandle` trait contract.

use workflow_core::process::QueuedSubmitter;
use workflow_utils::queued::{QueuedRunner, SchedulerKind};

/// Compile-time verification that `QueuedRunner` implements `QueuedSubmitter`.
#[test]
fn queued_runner_implements_queued_submitter_slurm() {
    let runner = QueuedRunner::new(SchedulerKind::Slurm);
    let _: &dyn QueuedSubmitter = &runner;
}

#[test]
fn queued_runner_implements_queued_submitter_pbs() {
    let runner = QueuedRunner::new(SchedulerKind::Pbs);
    let _: &dyn QueuedSubmitter = &runner;
}

/// When `sbatch` is not installed, `submit` must return `QueueSubmitFailed`,
/// not panic or produce an `Io` error from `Command::output()`.
///
/// This relies on `sh -c "sbatch ..."` exiting non-zero when `sbatch` is absent,
/// which triggers the `!output.status.success()` branch.
#[test]
fn submit_returns_err_when_sbatch_unavailable() {
    use workflow_core::error::WorkflowError;

    let dir = tempfile::tempdir().unwrap();
    let log_dir = dir.path().join("logs");
    std::fs::create_dir_all(&log_dir).unwrap();
    let workdir = dir.path().join("work");
    std::fs::create_dir_all(&workdir).unwrap();
    std::fs::write(workdir.join("job.sh"), "#!/bin/sh\necho hello\n").unwrap();

    // Restrict PATH to an empty directory so `sbatch` cannot be found.
    let empty_bin = dir.path().join("empty_bin");
    std::fs::create_dir_all(&empty_bin).unwrap();

    // Set PATH for this process (tests run sequentially within this file due to
    // the PATH mutation; mark with #[serial] if the suite is parallelised).
    let original = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", empty_bin.display().to_string());

    let runner = QueuedRunner::new(SchedulerKind::Slurm);
    let result = runner.submit(&workdir, "task_a", &log_dir);

    std::env::set_var("PATH", original);

    assert!(
        result.is_err(),
        "submit should fail when sbatch is not on PATH"
    );
    assert!(
        matches!(result.unwrap_err(), WorkflowError::QueueSubmitFailed(_)),
        "error should be QueueSubmitFailed"
    );
}

/// Verify that a successful mock submission returns a `ProcessHandle` whose
/// `wait()` produces an `OnDisk` `OutputLocation` pointing to the expected paths.
#[cfg(unix)]
#[test]
fn submit_with_mock_sbatch_returns_on_disk_handle() {
    use std::os::unix::fs::PermissionsExt;
    use workflow_core::process::{OutputLocation, ProcessHandle};

    let dir = tempfile::tempdir().unwrap();
    let log_dir = dir.path().join("logs");
    std::fs::create_dir_all(&log_dir).unwrap();
    let workdir = dir.path().join("work");
    std::fs::create_dir_all(&workdir).unwrap();
    std::fs::write(workdir.join("job.sh"), "#!/bin/sh\necho hello\n").unwrap();

    // Mock `sbatch` that prints a SLURM-style submission line and exits 0.
    let mock_dir = dir.path().join("mock_bin");
    std::fs::create_dir_all(&mock_dir).unwrap();
    let mock_sbatch = mock_dir.join("sbatch");
    std::fs::write(&mock_sbatch, "#!/bin/sh\necho 'Submitted batch job 99999'\n").unwrap();
    std::fs::set_permissions(&mock_sbatch, std::fs::Permissions::from_mode(0o755)).unwrap();

    let original = std::env::var("PATH").unwrap_or_default();
    std::env::set_var(
        "PATH",
        format!("{}:{}", mock_dir.display(), original),
    );

    let runner = QueuedRunner::new(SchedulerKind::Slurm);
    let mut handle = runner
        .submit(&workdir, "task_a", &log_dir)
        .expect("submit should succeed with mock sbatch");

    std::env::set_var("PATH", original);

    // `wait()` on a QueuedProcessHandle returns immediately with OnDisk paths.
    let result = handle.wait().expect("wait should succeed");
    assert!(
        matches!(result.output, OutputLocation::OnDisk { .. }),
        "output should be OnDisk for queued handles"
    );
    if let OutputLocation::OnDisk { stdout_path, stderr_path } = result.output {
        assert_eq!(
            stdout_path,
            log_dir.join("task_a.stdout"),
            "stdout path should follow <log_dir>/<task_id>.stdout convention"
        );
        assert_eq!(
            stderr_path,
            log_dir.join("task_a.stderr"),
            "stderr path should follow <log_dir>/<task_id>.stderr convention"
        );
    }
}
```

**Acceptance:** `cargo test -p workflow_utils --test queued_integration`

---

## Verification

```bash
cargo check --workspace
cargo clippy --workspace
cargo test --workspace
```
