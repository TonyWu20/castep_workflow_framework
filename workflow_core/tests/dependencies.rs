use std::sync::{Arc, Mutex};
use tempfile::tempdir;
use workflow_core::{Task, Workflow, state::WorkflowState};

#[test]
fn test_diamond_ordering() {
    let dir = tempdir().unwrap();
    let mut workflow = Workflow::resume("diamond", dir.path()).unwrap();

    let log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

    // DAG: a -> b, a -> c, b -> d, c -> d
    let log_a = log.clone();
    let _ = workflow.add_task(Task::new("a", move || {
        log_a.lock().unwrap().push("a".to_string());
        Ok(())
    }));

    let log_b = log.clone();
    let _ = workflow.add_task(Task::new("b", move || {
        log_b.lock().unwrap().push("b".to_string());
        Ok(())
    }).depends_on("a"));

    let log_c = log.clone();
    let _ = workflow.add_task(Task::new("c", move || {
        log_c.lock().unwrap().push("c".to_string());
        Ok(())
    }).depends_on("a"));

    let log_d = log.clone();
    let _ = workflow.add_task(Task::new("d", move || {
        log_d.lock().unwrap().push("d".to_string());
        Ok(())
    }).depends_on("b").depends_on("c"));

    workflow.run().unwrap();

    let entries = log.lock().unwrap();
    let pos = |task: &str| -> usize {
        entries.iter().position(|x| *x == task).unwrap()
    };

    assert!(pos("a") < pos("b"), "a must run before b");
    assert!(pos("a") < pos("c"), "a must run before c");
    assert!(pos("b") < pos("d"), "b must run before d");
    assert!(pos("c") < pos("d"), "c must run before d");
}

#[test]
fn test_failure_propagation() {
    let dir = tempdir().unwrap();
    let mut workflow = Workflow::resume("failure_prop", dir.path()).unwrap();

    // DAG: a -> b, a -> c, b -> d, c -> d
    let _ = workflow.add_task(Task::new("a", || Err(anyhow::anyhow!("failed"))));
    let _ = workflow.add_task(Task::new("b", || Ok::<(), anyhow::Error>(())).depends_on("a"));
    let _ = workflow.add_task(Task::new("c", || Ok::<(), anyhow::Error>(())).depends_on("a"));
    let _ = workflow.add_task(Task::new("d", || Ok::<(), anyhow::Error>(())).depends_on("b").depends_on("c"));

    workflow.run().unwrap();

    // Load state and verify downstream tasks were skipped
    let state_path = dir.path().join(".failure_prop.workflow.json");
    let state = WorkflowState::load(&state_path).unwrap();

    assert!(matches!(state.tasks.get("b"), Some(&workflow_core::state::TaskStatus::SkippedDueToDependencyFailure)));
    assert!(matches!(state.tasks.get("c"), Some(&workflow_core::state::TaskStatus::SkippedDueToDependencyFailure)));
    assert!(matches!(state.tasks.get("d"), Some(&workflow_core::state::TaskStatus::SkippedDueToDependencyFailure)));
}
