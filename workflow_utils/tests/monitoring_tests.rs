use workflow_core::HookExecutor;
use workflow_utils::{HookContext, HookTrigger, MonitoringHook, ShellHookExecutor};

#[test]
fn test_hook_executes() {
    let hook = MonitoringHook::new("test", "echo hook_output", HookTrigger::OnComplete);
    let ctx = HookContext {
        task_id: "task1".into(),
        workdir: std::path::PathBuf::from("/tmp"),
        state: "Completed".into(),
        exit_code: Some(0),
    };
    let executor = ShellHookExecutor;
    let result = executor.execute_hook(&hook, &ctx).unwrap();
    assert!(result.success);
    assert!(result.output.contains("hook_output"));
}

#[test]
fn test_hook_receives_context() {
    let hook = MonitoringHook::new("test", "sh -c echo $TASK_ID", HookTrigger::OnComplete);
    let ctx = HookContext {
        task_id: "mytask".into(),
        workdir: std::path::PathBuf::from("/tmp"),
        state: "Completed".into(),
        exit_code: Some(0),
    };
    let executor = ShellHookExecutor;
    let result = executor.execute_hook(&hook, &ctx).unwrap();
    assert!(result.success);
}
