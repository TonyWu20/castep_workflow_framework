use crate::executor::TaskExecutor;
use anyhow::Result;
use workflow_core::{HookContext, HookExecutor, HookResult, MonitoringHook};

/// Executes a monitoring hook with the given context.
/// This is a free function because it needs access to TaskExecutor from workflow_utils.
pub fn execute_hook(hook: &MonitoringHook, ctx: &HookContext) -> Result<HookResult> {
    let mut parts = hook.command.split_whitespace();
    let cmd = parts.next().unwrap_or_default();
    let args: Vec<String> = parts.map(String::from).collect();
    let result = TaskExecutor::new(&ctx.workdir)
        .command(cmd)
        .args(args)
        .env("TASK_ID", &ctx.task_id)
        .env("TASK_STATE", &ctx.state)
        .env("WORKDIR", ctx.workdir.to_string_lossy().as_ref())
        .env(
            "EXIT_CODE",
            ctx.exit_code.map(|c| c.to_string()).unwrap_or_default(),
        )
        .execute()?;
    Ok(HookResult {
        success: result.success(),
        output: result.stdout,
    })
}

/// A concrete implementation of `HookExecutor` that executes hooks via shell commands.
#[derive(Debug)]
pub struct ShellHookExecutor;

impl HookExecutor for ShellHookExecutor {
    fn execute_hook(
        &self,
        hook: &MonitoringHook,
        ctx: &HookContext,
    ) -> Result<HookResult, workflow_core::WorkflowError> {
        let mut parts = hook.command.split_whitespace();
        let cmd = parts.next().unwrap_or_default();
        let args: Vec<String> = parts.map(String::from).collect();
        let result = TaskExecutor::new(&ctx.workdir)
            .command(cmd)
            .args(args)
            .env("TASK_ID", &ctx.task_id)
            .env("TASK_STATE", &ctx.state)
            .env("WORKDIR", ctx.workdir.to_string_lossy().as_ref())
            .env(
                "EXIT_CODE",
                ctx.exit_code.map(|c| c.to_string()).unwrap_or_default(),
            )
            .execute()
            .map_err(|e| workflow_core::WorkflowError::Io(std::io::Error::other(e.to_string())))?;
        Ok(HookResult {
            success: result.success(),
            output: result.stdout,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use workflow_core::HookTrigger;

    #[test]
    fn test_shell_hook_executor_success() {
        let executor = ShellHookExecutor;
        let hook = MonitoringHook::new("test", "echo hello", HookTrigger::OnComplete);
        let ctx = HookContext {
            task_id: "task1".to_string(),
            workdir: std::path::PathBuf::from("/tmp"),
            state: "Completed".to_string(),
            exit_code: Some(0),
        };
        let result = executor.execute_hook(&hook, &ctx).unwrap();
        assert!(result.success);
        assert!(result.output.contains("hello"));
    }

    #[test]
    fn test_shell_hook_executor_with_args() {
        let executor = ShellHookExecutor;
        let hook = MonitoringHook::new("test", "echo arg1 arg2", HookTrigger::OnComplete);
        let ctx = HookContext {
            task_id: "task1".to_string(),
            workdir: std::path::PathBuf::from("/tmp"),
            state: "Completed".to_string(),
            exit_code: Some(0),
        };
        let result = executor.execute_hook(&hook, &ctx).unwrap();
        assert!(result.success);
    }
}
