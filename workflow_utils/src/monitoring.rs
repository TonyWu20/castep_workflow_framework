use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::executor::TaskExecutor;

/// A monitoring hook that can be triggered by a task event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringHook {
    /// Human-readable name of the hook.
    pub name: String,
    /// Command to execute when the hook is triggered.
    pub command: String,
    /// The trigger event that activates this hook.
    pub trigger: HookTrigger,
}

impl MonitoringHook {
    /// Creates a new monitoring hook with the specified configuration.
    pub fn new(name: impl Into<String>, command: impl Into<String>, trigger: HookTrigger) -> Self {
        Self { name: name.into(), command: command.into(), trigger }
    }
}

/// The event that triggers a monitoring hook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookTrigger {
    /// Triggered when a task starts.
    OnStart,
    /// Triggered when a task completes successfully.
    OnComplete,
    /// Triggered when a task fails.
    OnFailure,
    /// Triggered periodically at specified intervals.
    Periodic { interval_secs: u64 },
}

/// Context available to monitoring hooks during execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookContext {
    /// ID of the task this hook is associated with.
    pub task_id: String,
    /// Working directory for the task.
    pub workdir: PathBuf,
    /// Current state of the task (running, completed, failed, etc.).
    pub state: String,
    /// Exit code of the task (if available).
    pub exit_code: Option<i32>,
}

/// Result of executing a monitoring hook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    /// Whether the hook execution was successful.
    pub success: bool,
    /// Output from the hook command.
    pub output: String,
}

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
