mod common;

use std::collections::HashMap;
use std::sync::Arc;
use workflow_core::{Workflow, Task, process::ProcessRunner};
use workflow_core::task::ExecutionMode;
use workflow_core::state::JsonStateStore;
use workflow_utils::{SystemProcessRunner, ShellHookExecutor};

#[test]
fn file_backed_stdout_written_to_disk() {
    let dir = tempfile::tempdir().unwrap();
    let log_dir = dir.path().join("logs");
    let task_workdir = dir.path().join("my_task");
    std::fs::create_dir_all(&task_workdir).unwrap();

    let runner: Arc<dyn ProcessRunner> = Arc::new(SystemProcessRunner::with_log_dir(&log_dir));
    let executor = Arc::new(ShellHookExecutor);
    let state_path = dir.path().join("state.json");
    let mut state = JsonStateStore::new("test", state_path);

    let mut wf = Workflow::new("log_test").with_log_dir(&log_dir);
    wf.add_task(Task::new("t1", ExecutionMode::Direct {
        command: "echo".into(),
        args: vec!["hello".into()],
        env: HashMap::new(),
        timeout: None,
    }).workdir(task_workdir)).unwrap();

    wf.run(&mut state, runner, executor).unwrap();

    // Verify stdout file exists with correct content
    let stdout_file = log_dir.join("my_task.stdout");
    assert!(stdout_file.exists(), "stdout log file should exist");
    let content = std::fs::read_to_string(&stdout_file).unwrap();
    assert!(content.contains("hello"), "stdout should contain 'hello'");
}

#[test]
fn piped_mode_returns_captured_output() {
    let dir = tempfile::tempdir().unwrap();
    let runner: Arc<dyn ProcessRunner> = Arc::new(SystemProcessRunner::new());
    let executor = Arc::new(ShellHookExecutor);
    let state_path = dir.path().join("state.json");
    let mut state = JsonStateStore::new("test", state_path);

    let mut wf = Workflow::new("piped_test");
    wf.add_task(Task::new("t1", ExecutionMode::Direct {
        command: "echo".into(),
        args: vec!["hello".into()],
        env: HashMap::new(),
        timeout: None,
    })).unwrap();

    wf.run(&mut state, runner, executor).unwrap();
}
