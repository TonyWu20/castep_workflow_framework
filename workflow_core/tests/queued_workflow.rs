//! Integration test: `ExecutionMode::Queued` through `Workflow::run`.
//!
//! Uses a stub `QueuedSubmitter` that returns a `ProcessHandle` which
//! immediately reports `is_running() = false` and `wait()` yields exit code 0.

use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use workflow_core::error::WorkflowError;
use workflow_core::process::{
    OutputLocation, ProcessHandle, ProcessResult, ProcessRunner, QueuedSubmitter,
};
use workflow_core::state::{JsonStateStore, StateStore};
use workflow_core::task::{ExecutionMode, Task};
use workflow_core::workflow::Workflow;
use workflow_core::{HookExecutor, HookResult, MonitoringHook};

/// A stub submitter whose handles complete immediately with exit code 0.
struct StubQueuedSubmitter;

impl QueuedSubmitter for StubQueuedSubmitter {
    fn submit(
        &self,
        _workdir: &Path,
        task_id: &str,
        log_dir: &Path,
    ) -> Result<Box<dyn ProcessHandle>, WorkflowError> {
        Ok(Box::new(ImmediateHandle {
            stdout_path: log_dir.join(format!("{}.stdout", task_id)),
            stderr_path: log_dir.join(format!("{}.stderr", task_id)),
            start: Instant::now(),
        }))
    }
}

struct ImmediateHandle {
    stdout_path: std::path::PathBuf,
    stderr_path: std::path::PathBuf,
    start: Instant,
}

impl ProcessHandle for ImmediateHandle {
    fn is_running(&mut self) -> bool {
        false
    }

    fn terminate(&mut self) -> Result<(), WorkflowError> {
        Ok(())
    }

    fn wait(&mut self) -> Result<ProcessResult, WorkflowError> {
        Ok(ProcessResult {
            exit_code: Some(0),
            output: OutputLocation::OnDisk {
                stdout_path: self.stdout_path.clone(),
                stderr_path: self.stderr_path.clone(),
            },
            duration: self.start.elapsed(),
        })
    }
}

/// Stub runner — not used by Queued tasks but required by Workflow::run signature.
struct UnusedRunner;

impl ProcessRunner for UnusedRunner {
    fn spawn(
        &self,
        _workdir: &Path,
        _command: &str,
        _args: &[String],
        _env: &std::collections::HashMap<String, String>,
    ) -> Result<Box<dyn ProcessHandle>, WorkflowError> {
        panic!("UnusedRunner::spawn should not be called for Queued tasks");
    }
}

struct NoopHookExecutor;

impl HookExecutor for NoopHookExecutor {
    fn execute_hook(
        &self,
        _hook: &MonitoringHook,
        _ctx: &workflow_core::HookContext,
    ) -> Result<HookResult, WorkflowError> {
        Ok(HookResult {
            success: true,
            output: String::new(),
        })
    }
}

struct DelayedQueuedSubmitter {
    delay_polls: usize,
}

impl QueuedSubmitter for DelayedQueuedSubmitter {
    fn submit(
        &self,
        _workdir: &Path,
        task_id: &str,
        log_dir: &Path,
    ) -> Result<Box<dyn ProcessHandle>, WorkflowError> {
        Ok(Box::new(DelayedHandle {
            poll_count: std::sync::atomic::AtomicUsize::new(0),
            delay_polls: self.delay_polls,
            stdout_path: log_dir.join(format!("{}.stdout", task_id)),
            stderr_path: log_dir.join(format!("{}.stderr", task_id)),
            start: Instant::now(),
        }))
    }
}

struct DelayedHandle {
    poll_count: std::sync::atomic::AtomicUsize,
    delay_polls: usize,
    stdout_path: std::path::PathBuf,
    stderr_path: std::path::PathBuf,
    start: Instant,
}

impl ProcessHandle for DelayedHandle {
    fn is_running(&mut self) -> bool {
        let count = self.poll_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        count < self.delay_polls
    }

    fn terminate(&mut self) -> Result<(), WorkflowError> {
        Ok(())
    }

    fn wait(&mut self) -> Result<ProcessResult, WorkflowError> {
        Ok(ProcessResult {
            exit_code: Some(0),
            output: OutputLocation::OnDisk {
                stdout_path: self.stdout_path.clone(),
                stderr_path: self.stderr_path.clone(),
            },
            duration: self.start.elapsed(),
        })
    }
}

#[test]
fn queued_task_polls_before_completing() -> Result<(), WorkflowError> {
    let dir = tempfile::tempdir().unwrap();
    let log_dir = dir.path().join("logs");
    std::fs::create_dir_all(&log_dir).unwrap();

    let mut wf = Workflow::new("queued_poll_test")
        .with_max_parallel(4)?
        .with_log_dir(&log_dir)
        .with_queued_submitter(Arc::new(DelayedQueuedSubmitter { delay_polls: 2 }));

    wf.add_task(
        Task::new("queued_delayed", ExecutionMode::Queued)
            .workdir(dir.path().to_path_buf()),
    )?;

    let state_path = dir.path().join(".queued_poll_test.workflow.json");
    let mut state = JsonStateStore::new("queued_poll_test", state_path);

    let summary = wf.run(
        &mut state,
        Arc::new(UnusedRunner),
        Arc::new(NoopHookExecutor),
    )?;

    assert_eq!(summary.succeeded.len(), 1);
    assert!(summary.succeeded.contains(&"queued_delayed".to_string()));
    assert!(summary.failed.is_empty());

    assert!(matches!(
        state.get_status("queued_delayed"),
        Some(workflow_core::state::TaskStatus::Completed)
    ));

    Ok(())
}

#[test]
fn queued_task_completes_via_workflow_run() -> Result<(), WorkflowError> {
    let dir = tempfile::tempdir().unwrap();
    let log_dir = dir.path().join("logs");
    std::fs::create_dir_all(&log_dir).unwrap();

    let mut wf = Workflow::new("queued_test")
        .with_max_parallel(4)?
        .with_log_dir(&log_dir)
        .with_queued_submitter(Arc::new(StubQueuedSubmitter));

    wf.add_task(
        Task::new("queued_a", ExecutionMode::Queued)
            .workdir(dir.path().to_path_buf()),
    )?;

    let state_path = dir.path().join(".queued_test.workflow.json");
    let mut state = JsonStateStore::new("queued_test", state_path);

    let summary = wf.run(
        &mut state,
        Arc::new(UnusedRunner),
        Arc::new(NoopHookExecutor),
    )?;

    assert_eq!(summary.succeeded.len(), 1);
    assert!(summary.succeeded.contains(&"queued_a".to_string()));
    assert!(summary.failed.is_empty());
    assert!(summary.skipped.is_empty());

    assert!(matches!(
        state.get_status("queued_a"),
        Some(workflow_core::state::TaskStatus::Completed)
    ));

    Ok(())
}