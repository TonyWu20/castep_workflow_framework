# Phase 4 Fix Plan (v2)

Seven issues: unused import, shell injection, missing docs, log_dir default, PBS test coverage, QueuedRunner docs, PATH race condition.

## Dependencies

- TASK-1: independent
- TASK-2: independent
- TASK-3: independent
- TASK-4: independent
- TASK-5: independent
- TASK-6: independent
- TASK-7: independent (same file as TASK-1 but different lines)

All tasks can run in parallel. TASK-2 is the highest-risk change, TASK-7 is Blocking.

---

### TASK-1: Remove unused ProcessHandle import in queued integration test

**File:** `workflow_utils/tests/queued_integration.rs`
**Type:** replace

**Before:**

```rust
    use workflow_core::process::{OutputLocation, ProcessHandle};
```

**After:**

```rust
    use workflow_core::process::OutputLocation;
```

**Acceptance:** `cargo clippy -p workflow_utils --tests -- -D unused_imports`; `cargo test -p workflow_utils --test queued_integration`

---

### TASK-2: Eliminate shell injection in QueuedRunner::submit

**File:** `workflow_utils/src/queued.rs`
**Type:** replace

**Change 1:** Remove `build_submit_cmd` method.

**Before:**

```rust
    fn build_submit_cmd(&self, script_path: &str, task_id: &str, log_dir: &Path) -> String {
        let stdout_path = log_dir.join(format!("{}.stdout", task_id));
        let stderr_path = log_dir.join(format!("{}.stderr", task_id));
        match self.scheduler {
            SchedulerKind::Slurm => format!(
                "sbatch -o {} -e {} {}",
                stdout_path.display(), stderr_path.display(), script_path
            ),
            SchedulerKind::Pbs => format!(
                "qsub -o {} -e {} {}",
                stdout_path.display(), stderr_path.display(), script_path
            ),
        }
    }

    fn build_poll_cmd(&self) -> String {
```

**After:**

```rust
    fn build_poll_cmd(&self) -> String {
```

**Change 2:** Replace submit body to use direct Command invocation.

**Before:**

```rust
        let submit_cmd = self.build_submit_cmd(
            &workdir.join("job.sh").to_string_lossy(), task_id, log_dir
        );
        let output = Command::new("sh")
            .args(["-c", &submit_cmd])
            .current_dir(workdir)
            .output()
            .map_err(WorkflowError::Io)?;
```

**After:**

```rust
        let stdout_path = log_dir.join(format!("{}.stdout", task_id));
        let stderr_path = log_dir.join(format!("{}.stderr", task_id));
        let script_path = workdir.join("job.sh");

        let output = match self.scheduler {
            SchedulerKind::Slurm => Command::new("sbatch"),
            SchedulerKind::Pbs => Command::new("qsub"),
        }
        .args(["-o", &stdout_path.to_string_lossy(), "-e", &stderr_path.to_string_lossy()])
        .arg(&script_path)
        .current_dir(workdir)
        .output()
        .map_err(WorkflowError::Io)?;
```

**Acceptance:** `cargo check -p workflow_utils`; `cargo test -p workflow_utils --test queued_integration`

---

### TASK-3: Add doc comments to ProcessHandle trait

**File:** `workflow_core/src/process.rs`
**Type:** replace

**Before:**

```rust
pub trait ProcessHandle: Send {
    fn is_running(&mut self) -> bool;
    fn terminate(&mut self) -> Result<(), WorkflowError>;
    fn wait(&mut self) -> Result<ProcessResult, WorkflowError>;
}
```

**After:**

```rust
/// A handle to a running (or finished) process, used to poll, wait, or terminate it.
///
/// Implementations must be `Send` so handles can be stored across thread boundaries.
pub trait ProcessHandle: Send {
    /// Returns `true` if the process is still running.
    ///
    /// Implementations may cache the result and only re-poll periodically.
    fn is_running(&mut self) -> bool;

    /// Requests termination of the process.
    ///
    /// Best-effort: the process may already have exited.
    fn terminate(&mut self) -> Result<(), WorkflowError>;

    /// Returns the process result once the process has finished.
    ///
    /// For queued (HPC) handles this may return immediately with `OnDisk` output
    /// paths rather than captured output. Callers should ensure `is_running()`
    /// has returned `false` before calling `wait()`, as behaviour when called
    /// on a still-running process is implementation-defined.
    fn wait(&mut self) -> Result<ProcessResult, WorkflowError>;
}
```

**Acceptance:** `cargo doc -p workflow_core --no-deps`; `cargo check -p workflow_core`

---

### TASK-4: Default log_dir to task workdir instead of "."

**File:** `workflow_core/src/workflow.rs`
**Type:** replace

**Before:**

```rust
                                let log_dir = self.log_dir.as_deref()
                                    .unwrap_or_else(|| std::path::Path::new("."));
```

**After:**

```rust
                                let log_dir = self.log_dir.as_deref()
                                    .unwrap_or(task.workdir.as_path());
```

**Acceptance:** `cargo check -p workflow_core`; `cargo test -p workflow_core`

---

### TASK-5: Add unit tests for parse_job_id (SLURM and PBS)

**File:** `workflow_utils/src/queued.rs`
**Type:** replace

Append a test module at the end of the file, after the closing `}` of `impl ProcessHandle for QueuedProcessHandle`.

**Before:**

```rust
            duration: self.started_at.elapsed(),
        })
    }
}
```

(This is the final `}` pair at end of file — the closing of `wait()` and of `impl ProcessHandle for QueuedProcessHandle`.)

**After:**

```rust
            duration: self.started_at.elapsed(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_slurm_job_id_from_submit_output() {
        let runner = QueuedRunner::new(SchedulerKind::Slurm);
        let id = runner.parse_job_id("Submitted batch job 12345").unwrap();
        assert_eq!(id, "12345");
    }

    #[test]
    fn parse_slurm_job_id_single_word() {
        let runner = QueuedRunner::new(SchedulerKind::Slurm);
        let id = runner.parse_job_id("99999").unwrap();
        assert_eq!(id, "99999");
    }

    #[test]
    fn parse_slurm_job_id_empty_fails() {
        let runner = QueuedRunner::new(SchedulerKind::Slurm);
        assert!(runner.parse_job_id("").is_err());
    }

    #[test]
    fn parse_pbs_job_id_typical() {
        let runner = QueuedRunner::new(SchedulerKind::Pbs);
        let id = runner.parse_job_id("1234.pbs-server\n").unwrap();
        assert_eq!(id, "1234.pbs-server");
    }

    #[test]
    fn parse_pbs_job_id_empty_fails() {
        let runner = QueuedRunner::new(SchedulerKind::Pbs);
        assert!(runner.parse_job_id("").is_err());
    }

    #[test]
    fn parse_pbs_job_id_whitespace_only_fails() {
        let runner = QueuedRunner::new(SchedulerKind::Pbs);
        assert!(runner.parse_job_id("   \n  ").is_err());
    }
}
```

**Acceptance:** `cargo test -p workflow_utils -- tests::parse`

---

### TASK-6: Add doc comments to SchedulerKind and QueuedRunner public API

**File:** `workflow_utils/src/queued.rs`
**Type:** replace

**Before:**

```rust
#[derive(Debug, Clone, Copy)]
pub enum SchedulerKind {
    Slurm,
    Pbs,
}

pub struct QueuedRunner {
    pub scheduler: SchedulerKind,
}
```

**After:**

```rust
/// The type of HPC job scheduler to target.
#[derive(Debug, Clone, Copy)]
pub enum SchedulerKind {
    /// SLURM Workload Manager (`sbatch` / `squeue` / `scancel`).
    Slurm,
    /// Portable Batch System (`qsub` / `qstat` / `qdel`).
    Pbs,
}

/// Submits and manages jobs via an HPC batch scheduler.
///
/// Implements [`QueuedSubmitter`](workflow_core::process::QueuedSubmitter) to
/// integrate with the workflow engine's `Queued` execution mode.
pub struct QueuedRunner {
    /// Which scheduler dialect to use for command construction.
    pub scheduler: SchedulerKind,
}
```

**Acceptance:** `cargo doc -p workflow_utils --no-deps`; `cargo check -p workflow_utils`

---

### TASK-7: Add #[serial] to PATH-mutating queued integration tests

**File:** `workflow_utils/Cargo.toml` and `workflow_utils/tests/queued_integration.rs`
**Type:** replace

**Change 0:** Add `serial_test` dev-dependency to `workflow_utils/Cargo.toml`.

**Before:**

```toml
[dev-dependencies]
tempfile = "3"
```

**After:**

```toml
[dev-dependencies]
serial_test = "3"
tempfile = "3"
```

**Change 1:** Add serial_test import at the top of the test file.

**Before:**

```rust
use workflow_core::process::QueuedSubmitter;
use workflow_utils::{QueuedRunner, SchedulerKind};
```

**After:**

```rust
use serial_test::serial;
use workflow_core::process::QueuedSubmitter;
use workflow_utils::{QueuedRunner, SchedulerKind};
```

**Change 2:** Add `#[serial]` to `submit_returns_err_when_sbatch_unavailable`.

**Before:**

```rust
#[test]
fn submit_returns_err_when_sbatch_unavailable() {
```

**After:**

```rust
#[test]
#[serial]
fn submit_returns_err_when_sbatch_unavailable() {
```

**Change 3:** Add `#[serial]` to `submit_with_mock_sbatch_returns_on_disk_handle`.

**Before:**

```rust
#[test]
fn submit_with_mock_sbatch_returns_on_disk_handle() {
```

**After:**

```rust
#[test]
#[serial]
fn submit_with_mock_sbatch_returns_on_disk_handle() {
```

**Acceptance:** `cargo test -p workflow_utils --test queued_integration`

---
