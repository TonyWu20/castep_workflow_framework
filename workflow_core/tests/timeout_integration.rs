// Integration tests for timeout handling
// These tests verify that tasks with timeouts fail properly and dependent tasks are skipped.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use workflow_core::HookExecutor;
use workflow_core::ProcessRunner;
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
            command: "true".into(),
            args: vec![],
            env: HashMap::new(),
            timeout: None,
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
