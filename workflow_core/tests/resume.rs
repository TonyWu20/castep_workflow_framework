use std::collections::HashMap;
use std::sync::Arc;
use tempfile::tempdir;
use workflow_core::{
    ExecutionMode, JsonStateStore, StateStoreExt, Task, Workflow,
    state::{StateStore, TaskStatus},
};

#[test]
fn test_resume_skips_completed_reruns_interrupted() {
    let dir = tempdir().unwrap();
    let state_path = dir.path().join(".test_resume.workflow.json");

    // First run: complete task a, interrupt task b (simulate crash)
    {
        let mut state = JsonStateStore::new("test_resume", state_path.clone());
        state.mark_completed("a");
        state.mark_running("b"); // simulates crash mid-b
        state.save().unwrap();
    }

    // Second run: resume from saved state.
    // JsonStateStore::load resets Running -> Pending (crash recovery).
    // So on load: a=Completed (skip), b=Pending (will run).
    let mut wf = Workflow::new("test_resume");

    // Task a: already Completed in state, will not be dispatched.
    wf.add_task(Task::new(
        "a",
        ExecutionMode::Direct {
            command: "true".into(),
            args: vec![],
            env: HashMap::new(),
            timeout: None,
        },
    ))
    .unwrap();

    // Task b: was interrupted (Running -> Pending on load), will run and succeed.
    wf.add_task(
        Task::new(
            "b",
            ExecutionMode::Direct {
                command: "true".into(),
                args: vec![],
                env: HashMap::new(),
                timeout: None,
            },
        )
        .depends_on("a"),
    )
    .unwrap();

    let runner = Arc::new(workflow_utils::SystemProcessRunner);
    let executor = Arc::new(workflow_utils::ShellHookExecutor);
    let mut state = Box::new(JsonStateStore::load(&state_path).unwrap());

    wf.run(state.as_mut(), runner, executor).unwrap();

    // Reload from disk to get final state (run() saves on every change)
    let final_state = JsonStateStore::load(&state_path).unwrap();
    // "a" was pre-completed and must NOT have been re-run
    assert!(
        matches!(final_state.get_status("a"), Some(TaskStatus::Completed)),
        "task 'a' should remain Completed (not re-run)"
    );
    // "b" was interrupted, reset to Pending on load, then ran and completed
    assert!(
        matches!(final_state.get_status("b"), Some(TaskStatus::Completed)),
        "task 'b' should have run and completed"
    );
}
