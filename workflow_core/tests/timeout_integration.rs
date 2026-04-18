// Integration tests for timeout handling
// These tests verify that tasks with timeouts fail properly and dependent tasks are skipped.

mod common;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use workflow_core::HookExecutor;
use workflow_core::ProcessRunner;
use workflow_core::state::JsonStateStore;
use workflow_core::task::{ExecutionMode, Task};
use workflow_core::workflow::Workflow;
use workflow_utils::{ShellHookExecutor, SystemProcessRunner};

use common::direct;

#[test]
fn timeout_task_fails_and_dependent_skips() {
    let dir = tempfile::tempdir().unwrap();
    let mut wf = Workflow::new("wf_timeout").with_max_parallel(4).unwrap();

    wf.add_task(Task::new("sleeper", ExecutionMode::Direct {
        command: "sleep".into(),
        args: vec!["10".into()],
        env: HashMap::new(),
        timeout: Some(Duration::from_millis(100)),
    })).unwrap();

    wf.add_task(Task::new("dependent", direct("true")).depends_on("sleeper")).unwrap();

    let mut state = JsonStateStore::new("wf_timeout", dir.path().join(".wf_timeout.workflow.json"));
    let wall_start = Instant::now();
    let summary = wf.run(&mut state, Arc::new(SystemProcessRunner::new()) as Arc<dyn ProcessRunner>,
                         Arc::new(ShellHookExecutor) as Arc<dyn HookExecutor>).unwrap();

    assert!(wall_start.elapsed() < Duration::from_secs(1));
    let f = summary.failed.iter().find(|f| f.id == "sleeper").expect("sleeper should fail");
    assert!(f.error.contains("timed out"), "error was: {}", f.error);
    assert!(summary.skipped.contains(&"dependent".to_string()));
}

#[test]
fn task_timeout_marks_failed() {
    let dir = tempfile::tempdir().unwrap();
    let mut wf = Workflow::new("wf_timeout_single").with_max_parallel(4).unwrap();

    wf.add_task(Task::new("timeout_task", ExecutionMode::Direct {
        command: "sleep".into(),
        args: vec!["10".into()],
        env: HashMap::new(),
        timeout: Some(Duration::from_millis(100)),
    })).unwrap();

    let mut state = JsonStateStore::new("wf_timeout_single", dir.path().join(".wf_timeout_single.workflow.json"));
    let wall_start = Instant::now();
    let summary = wf.run(&mut state, Arc::new(SystemProcessRunner::new()) as Arc<dyn ProcessRunner>,
                         Arc::new(ShellHookExecutor) as Arc<dyn HookExecutor>).unwrap();

    assert!(wall_start.elapsed() < Duration::from_secs(1));
    let failed = summary.failed.iter().find(|f| f.id == "timeout_task").expect("task should fail");
    assert!(failed.error.contains("timed out"), "error was: {}", failed.error);
}
