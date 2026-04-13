// Integration tests for dependency handling
// These tests verify the DAG execution model works correctly.

use std::collections::HashMap;
use std::sync::Arc;
use tempfile::tempdir;
use workflow_core::{ExecutionMode, JsonStateStore, StateStore, Task, Workflow, state::TaskStatus};

#[test]
fn test_diamond_ancestry() {
    // Verify that a DAG with diamond ancestry (a->b, a->c, b->d, c->d)
    // executes in correct topological order.
    let _dir = tempdir().unwrap();

    // Create workflow with diamond: a -> b, c; b, c -> d
    let mut wf = Workflow::new("diamond_test");

    wf.add_task(Task::new(
        "a",
        ExecutionMode::Direct {
            command: "true".into(),
            args: vec![],
            env: HashMap::new(),
            timeout: None,
        },
    )).unwrap();

    wf.add_task(Task::new(
        "b",
        ExecutionMode::Direct {
            command: "true".into(),
            args: vec![],
            env: HashMap::new(),
            timeout: None,
        },
    ).depends_on("a")).unwrap();

    wf.add_task(Task::new(
        "c",
        ExecutionMode::Direct {
            command: "true".into(),
            args: vec![],
            env: HashMap::new(),
            timeout: None,
        },
    ).depends_on("a")).unwrap();

    wf.add_task(Task::new(
        "d",
        ExecutionMode::Direct {
            command: "true".into(),
            args: vec![],
            env: HashMap::new(),
            timeout: None,
        },
    ).depends_on("b").depends_on("c")).unwrap();

    // Verify dry_run returns valid topological order
    let order = wf.dry_run().unwrap();
    assert_eq!(order, vec!["a", "b", "c", "d"]);
}

#[test]
fn test_failure_skips_downstream() {
    // Verify that when task A fails, tasks B and C (dependent on A) are skipped.
    let dir = tempdir().unwrap();
    let state_path = dir.path().join(".failure_test.workflow.json");

    let mut wf = Workflow::new("failure_test")
        .with_max_parallel(4)
        .unwrap();

    wf.add_task(Task::new(
        "a",
        ExecutionMode::Direct {
            command: "false".into(),  // Failing command
            args: vec![],
            env: HashMap::new(),
            timeout: None,
        },
    )).unwrap();

    wf.add_task(Task::new(
        "b",
        ExecutionMode::Direct {
            command: "true".into(),
            args: vec![],
            env: HashMap::new(),
            timeout: None,
        },
    ).depends_on("a")).unwrap();

    wf.add_task(Task::new(
        "c",
        ExecutionMode::Direct {
            command: "true".into(),
            args: vec![],
            env: HashMap::new(),
            timeout: None,
        },
    ).depends_on("a")).unwrap();

    let runner = Arc::new(workflow_utils::SystemProcessRunner);
    let executor = Arc::new(workflow_utils::ShellHookExecutor);
    let mut state = Box::new(JsonStateStore::new("failure_test", state_path.clone()));

    // Run should complete even though task A failed
    let summary = wf.run(state.as_mut(), runner, executor).unwrap();

    // Task A should be in failed list
    let failed_ids: Vec<_> = summary.failed.iter().map(|(id, _)| id.clone()).collect();
    assert!(failed_ids.iter().any(|id| id == "a"));

    // Tasks B and C should be skipped due to dependency failure
    let state = JsonStateStore::load(&state_path).unwrap();
    // After load, SkippedDueToDependencyFailure resets to Pending for crash recovery
    assert!(matches!(state.get_status("b"), Some(TaskStatus::Pending)));
    assert!(matches!(state.get_status("c"), Some(TaskStatus::Pending)));
}
