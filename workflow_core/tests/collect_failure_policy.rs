use std::collections::HashMap;
use std::sync::Arc;

use workflow_core::error::WorkflowError;
use workflow_core::prelude::*;
use workflow_core::process::{ProcessHandle, ProcessResult};
use workflow_core::state::JsonStateStore;
use workflow_core::{HookExecutor, HookResult, ProcessRunner};

struct StubRunner;
impl ProcessRunner for StubRunner {
    fn spawn(
        &self,
        workdir: &std::path::Path,
        command: &str,
        args: &[String],
        env: &HashMap<String, String>,
    ) -> Result<Box<dyn ProcessHandle>, WorkflowError> {
        let child = std::process::Command::new(command)
            .args(args)
            .envs(env)
            .current_dir(workdir)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(WorkflowError::Io)?;
        Ok(Box::new(StubHandle {
            child: Some(child),
            start: std::time::Instant::now(),
        }))
    }
}

struct StubHandle {
    child: Option<std::process::Child>,
    start: std::time::Instant,
}

impl ProcessHandle for StubHandle {
    fn is_running(&mut self) -> bool {
        match &mut self.child {
            Some(child) => child.try_wait().ok().flatten().is_none(),
            None => false,
        }
    }
    fn terminate(&mut self) -> Result<(), WorkflowError> {
        match &mut self.child {
            Some(child) => child.kill().map_err(WorkflowError::Io),
            None => Ok(()),
        }
    }
    fn wait(&mut self) -> Result<ProcessResult, WorkflowError> {
        let child = self
            .child
            .take()
            .ok_or_else(|| WorkflowError::InvalidConfig("wait() called twice".into()))?;
        let output = child.wait_with_output().map_err(WorkflowError::Io)?;
        Ok(ProcessResult {
            exit_code: output.status.code(),
            output: workflow_core::process::OutputLocation::Captured {
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            },
            duration: self.start.elapsed(),
        })
    }
}

struct StubHookExecutor;
impl HookExecutor for StubHookExecutor {
    fn execute_hook(
        &self,
        _hook: &workflow_core::MonitoringHook,
        _ctx: &workflow_core::HookContext,
    ) -> Result<HookResult, WorkflowError> {
        Ok(HookResult {
            success: true,
            output: String::new(),
        })
    }
}

#[test]
fn collect_failure_with_failtask_marks_failed() -> Result<(), WorkflowError> {
    let dir = tempfile::tempdir().unwrap();
    let mut wf = Workflow::new("wf_collect_fail").with_max_parallel(4)?;

    wf.add_task(
        Task::new(
            "a",
            ExecutionMode::Direct {
                command: "true".into(),
                args: vec![],
                env: HashMap::new(),
                timeout: None,
            },
        )
        .collect_failure_policy(CollectFailurePolicy::FailTask)
        .collect(|_workdir| -> Result<(), std::io::Error> {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "collect boom"))
        }),
    )
    .unwrap();

    let runner: Arc<dyn ProcessRunner> = Arc::new(StubRunner);
    let executor: Arc<dyn HookExecutor> = Arc::new(StubHookExecutor);
    let state_path = dir.path().join(".wf_collect_fail.workflow.json");
    let mut state = Box::new(JsonStateStore::new("wf_collect_fail", state_path));

    wf.run(state.as_mut(), runner, executor)?;

    assert!(matches!(
        state.get_status("a"),
        Some(TaskStatus::Failed { .. })
    ));
    Ok(())
}

#[test]
fn collect_failure_with_warnonly_marks_completed() -> Result<(), WorkflowError> {
    let dir = tempfile::tempdir().unwrap();
    let mut wf = Workflow::new("wf_collect_warn").with_max_parallel(4)?;

    wf.add_task(
        Task::new(
            "a",
            ExecutionMode::Direct {
                command: "true".into(),
                args: vec![],
                env: HashMap::new(),
                timeout: None,
            },
        )
        .collect_failure_policy(CollectFailurePolicy::WarnOnly)
        .collect(|_workdir| -> Result<(), std::io::Error> {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "collect warning"))
        }),
    )
    .unwrap();

    let runner: Arc<dyn ProcessRunner> = Arc::new(StubRunner);
    let executor: Arc<dyn HookExecutor> = Arc::new(StubHookExecutor);
    let state_path = dir.path().join(".wf_collect_warn.workflow.json");
    let mut state = Box::new(JsonStateStore::new("wf_collect_warn", state_path));

    wf.run(state.as_mut(), runner, executor)?;

    assert!(matches!(
        state.get_status("a"),
        Some(TaskStatus::Completed)
    ));
    Ok(())
}