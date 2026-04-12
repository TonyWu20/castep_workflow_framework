use std::sync::{Arc, Mutex};
use tempfile::tempdir;
use workflow_core::{Task, Workflow, state::{StateStoreExt, JsonStateStore}, StateStore};

#[test]
fn test_resume_skips_completed_reruns_interrupted() {
    let dir = tempdir().unwrap();
    let state_path = dir.path().join(".test_resume.workflow.json");
    let mut state = JsonStateStore::new("test_resume", state_path.clone());
    state.mark_completed("a");
    state.mark_running("b"); // simulates crash mid-b
    state.save().unwrap();

    let log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

    let mut wf = Workflow::resume("test_resume", dir.path()).unwrap();

    let log_a = log.clone();
    wf.add_task(Task::new("a", move || { log_a.lock().unwrap().push("a".into()); Ok(()) })).unwrap();

    let log_b = log.clone();
    wf.add_task(Task::new("b", move || { log_b.lock().unwrap().push("b".into()); Ok(()) })).unwrap();

    wf.run().unwrap();

    let ran = log.lock().unwrap();
    assert!(!ran.contains(&"a".to_string()), "task 'a' should not re-run");
    assert_eq!(ran.iter().filter(|x| *x == "b").count(), 1, "task 'b' should run exactly once");
}
