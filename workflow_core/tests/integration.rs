mod common;

use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use workflow_core::HookExecutor;
use workflow_core::process::ProcessRunner;
use workflow_core::state::{JsonStateStore, StateStore, TaskStatus};
use workflow_core::task::Task;
use workflow_core::workflow::Workflow;
use workflow_utils::{ShellHookExecutor, SystemProcessRunner};

fn runner() -> Arc<dyn ProcessRunner> { Arc::new(SystemProcessRunner::new()) }
fn executor() -> Arc<dyn HookExecutor> { Arc::new(ShellHookExecutor) }

use common::direct;

#[test]
fn resume_skips_completed_reruns_failed() {
    let dir = tempfile::tempdir().unwrap();
    let state_path = dir.path().join(".integration.workflow.json");
    let a_runs = Arc::new(AtomicUsize::new(0));

    // Run 1: B fails, C skipped
    let a_runs_c = Arc::clone(&a_runs);
    let mut wf1 = Workflow::new("integration").with_max_parallel(4).unwrap();
    wf1.add_task(
        Task::new("a", direct("true"))
            .setup(move |_| -> Result<(), std::convert::Infallible> {
                a_runs_c.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
    ).unwrap();
    wf1.add_task(Task::new("b", direct("false")).depends_on("a")).unwrap();
    wf1.add_task(Task::new("c", direct("true")).depends_on("b")).unwrap();

    let mut state1 = JsonStateStore::new("integration", state_path.clone());
    let summary1 = wf1.run(&mut state1, runner(), executor()).unwrap();
    assert!(summary1.failed.iter().any(|f| f.id == "b"));
    assert!(summary1.skipped.contains(&"c".to_string()));

    // Verify state after Run 1 (B=Failed, C=SkippedDueToDependencyFailure)
    assert!(matches!(state1.get_status("b"), Some(TaskStatus::Failed { .. })));
    assert!(matches!(state1.get_status("c"), Some(TaskStatus::SkippedDueToDependencyFailure)));

    // Run 2: load resets b+c to Pending; b now succeeds; a must NOT re-run
    let mut state2 = JsonStateStore::load(&state_path).unwrap();
    assert!(matches!(state2.get_status("b"), Some(TaskStatus::Pending)));
    assert!(matches!(state2.get_status("c"), Some(TaskStatus::Pending)));

    let mut wf2 = Workflow::new("integration").with_max_parallel(4).unwrap();
    // A uses "false" — would fail if re-dispatched, proving it is skipped
    wf2.add_task(Task::new("a", direct("false"))).unwrap();
    wf2.add_task(Task::new("b", direct("true")).depends_on("a")).unwrap();
    wf2.add_task(Task::new("c", direct("true")).depends_on("b")).unwrap();

    let summary2 = wf2.run(&mut state2, runner(), executor()).unwrap();
    assert!(summary2.succeeded.contains(&"b".to_string()));
    assert!(summary2.succeeded.contains(&"c".to_string()));
    assert_eq!(a_runs.load(Ordering::SeqCst), 1, "A must only run once total");
}
