use anyhow::Result;
use crate::executor::TaskExecutor;
use workflow_core::{HookContext, HookResult, MonitoringHook};

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
        .env("EXIT_CODE", ctx.exit_code.map(|c| c.to_string()).unwrap_or_default())
        .execute()?;
    Ok(HookResult { success: result.success(), output: result.stdout })
}
