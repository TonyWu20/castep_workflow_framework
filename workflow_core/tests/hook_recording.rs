use std::sync::Arc;

use workflow_core::{CollectFailurePolicy, HookExecutor, process::ProcessRunner, state::{JsonStateStore, StateStore, TaskStatus}, Workflow, Task};
use workflow_utils::{ShellHookExecutor, SystemProcessRunner};

mod common;
use common::{RecordingExecutor, direct, direct_with_args};

fn runner() -> Arc<dyn ProcessRunner> { Arc::new(SystemProcessRunner::new()) }

#[test]
fn setup_failure_skips_dependent() {
    let dir = tempfile::tempdir().unwrap();
    let state_path = dir.path().join(".hook_recording.setup.workflow.json");

    let mut wf = Workflow::new("setup_failure_test").with_max_parallel(4).unwrap();

    // Task "a" setup returns error → task status becomes Failed
    wf.add_task(
        Task::new("a", direct("true"))
            .setup(|_| -> Result<(), std::io::Error> { Err(std::io::Error::other("setup failed")) })
    ).unwrap();

    // Task "b" depends on "a"
    wf.add_task(Task::new("b", direct("true")).depends_on("a")).unwrap();

    let mut state = JsonStateStore::new("setup_failure", state_path.clone());
    let summary = wf.run(&mut state, runner(), Arc::new(ShellHookExecutor)).unwrap();

    // Verify "a" is Failed and "b" is SkippedDueToDependencyFailure
    assert!(summary.failed.iter().any(|f| f.id == "a"), "Task a should be in failed summary");
    assert!(summary.skipped.contains(&"b".to_string()), "Task b should be skipped");

    // Verify in-memory state before persisting
    assert!(matches!(state.get_status("a"), Some(TaskStatus::Failed { .. })), "In-memory: Task a should be Failed");
    assert!(matches!(state.get_status("b"), Some(TaskStatus::SkippedDueToDependencyFailure)), "In-memory: Task b should be SkippedDueToDependencyFailure");

    // Verify persisted state after load (Failed and SkippedDueToDependencyFailure reset to Pending for crash recovery)
    let loaded = JsonStateStore::load(&state_path).unwrap();
    assert!(matches!(loaded.get_status("a"), Some(TaskStatus::Pending)), "Persisted: Task a should reset to Pending after load");
    assert!(matches!(loaded.get_status("b"), Some(TaskStatus::Pending)), "Persisted: Task b should reset to Pending after load");
}

#[test]
fn collect_failure_does_not_fail_task() {
    let dir = tempfile::tempdir().unwrap();
    let state_path = dir.path().join(".hook_recording.collect.workflow.json");

    let mut wf = Workflow::new("collect_failure_test").with_max_parallel(4).unwrap();

    wf.add_task(
        Task::new("a", direct("true"))
            .collect_failure_policy(CollectFailurePolicy::WarnOnly)
            .collect(|_| -> Result<(), std::io::Error> { Err(std::io::Error::other("collect failed")) })
    ).unwrap();

    let mut state = JsonStateStore::new("collect_failure", state_path.clone());
    let summary = wf.run(&mut state, runner(), Arc::new(ShellHookExecutor)).unwrap();

    // Verify task is Completed (not Failed) because workflow.rs uses tracing::warn! and doesn't mark failed
    assert!(summary.succeeded.contains(&"a".to_string()));
    assert!(summary.failed.is_empty());

    // Verify persisted state shows Completed
    let loaded = JsonStateStore::load(&state_path).unwrap();
    assert!(matches!(loaded.get_status("a"), Some(TaskStatus::Completed)), "Task a should be Completed");
}

#[test]
fn hooks_fire_on_start_complete_failure() {
    use workflow_core::{HookTrigger, MonitoringHook};

    let dir = tempfile::tempdir().unwrap();
    let state_path = dir.path().join(".hook_recording.hooks.workflow.json");

    // Create RecordingExecutor with shared Arc so tests can read calls
    let executor = RecordingExecutor::new();

    // Create hooks: OnStart, OnComplete, OnFailure
    let start_hook = MonitoringHook::new("onstart", "echo start", HookTrigger::OnStart);
    let complete_hook = MonitoringHook::new("oncomplete", "echo complete", HookTrigger::OnComplete);
    let failure_hook = MonitoringHook::new("onfailure", "echo failure", HookTrigger::OnFailure);

    let mut wf = Workflow::new("hooks_test").with_max_parallel(4).unwrap();

    // Success path: OnStart → process completes → OnComplete fires
    wf.add_task(
        Task::new("success", direct("true"))
            .monitors(vec![start_hook.clone(), complete_hook.clone()])
    ).unwrap();

    // Failure path: OnStart → process fails → OnFailure fires
    wf.add_task(
        Task::new("failure", direct("false"))
            .monitors(vec![start_hook.clone(), failure_hook.clone()])
    ).unwrap();

    let mut state = JsonStateStore::new("hooks_fire", state_path.clone());
    let summary = wf.run(&mut state, runner(), Arc::new(executor.clone()) as Arc<dyn HookExecutor>).unwrap();

    // Verify success: OnStart + OnComplete fired for task "success"
    let calls = executor.calls();

    // 4 hook calls total: 2 per task (cross-task order is non-deterministic)
    assert_eq!(calls.len(), 4);

    // Check success task hooks (OnStart + OnComplete)
    let success_calls: Vec<_> = calls.iter()
        .filter(|(_name, id)| *id == "success")
        .collect();
    assert_eq!(success_calls.len(), 2);
    assert_eq!(success_calls[0].0, "onstart");
    assert_eq!(success_calls[1].0, "oncomplete");

    // Check failure task hooks (OnStart + OnFailure)
    let failure_calls: Vec<_> = calls.iter()
        .filter(|(_name, id)| *id == "failure")
        .collect();
    assert_eq!(failure_calls.len(), 2);
    assert_eq!(failure_calls[0].0, "onstart");
    assert_eq!(failure_calls[1].0, "onfailure");

    // Verify workflow summary
    assert!(summary.succeeded.contains(&"success".to_string()));
    assert!(summary.failed.iter().any(|f| f.id == "failure"));
}

#[test]
fn periodic_hook_fires_during_long_task() {
    use workflow_core::{HookTrigger, MonitoringHook};

    let dir = tempfile::tempdir().unwrap();
    let state_path = dir.path().join(".periodic.workflow.json");

    let executor = RecordingExecutor::new();

    let periodic_hook = MonitoringHook::new(
        "periodic_check", "echo check", HookTrigger::Periodic { interval_secs: 1 }
    );

    let mut wf = Workflow::new("periodic_test").with_max_parallel(4).unwrap();
    wf.add_task(
        Task::new("long_task", direct_with_args("sleep", &["2"]))
            .monitors(vec![periodic_hook])
    ).unwrap();

    let mut state = JsonStateStore::new("periodic", state_path);
    wf.run(&mut state, runner(), Arc::new(executor.clone()) as Arc<dyn HookExecutor>).unwrap();

    let calls = executor.calls();
    let periodic_calls: Vec<_> = calls.iter()
        .filter(|(name, _)| name == "periodic_check")
        .collect();

    // sleep 8 with interval_secs=1 should fire at least once during the task execution.
    // The main loop sleeps 50ms between iterations, so with an 8-second task we should
    // have many loop iterations (at least 80), and the periodic check should trigger.
    assert!(
        !periodic_calls.is_empty(),
        "periodic hook should fire at least once during an 8-second task (got {} calls)",
        periodic_calls.len()
    );
}
