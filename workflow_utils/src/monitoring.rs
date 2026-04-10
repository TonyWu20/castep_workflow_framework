use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::executor::TaskExecutor;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringHook {
    pub name: String,
    pub command: String,
    pub trigger: HookTrigger,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookTrigger {
    OnStart,
    OnComplete,
    OnFailure,
    Periodic { interval_secs: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookContext {
    pub task_id: String,
    pub workdir: PathBuf,
    pub state: String,
    pub exit_code: Option<i32>,
}

pub struct HookResult {
    pub success: bool,
    pub output: String,
}

impl MonitoringHook {
    pub fn new(name: impl Into<String>, command: impl Into<String>, trigger: HookTrigger) -> Self {
        Self { name: name.into(), command: command.into(), trigger }
    }

    pub fn execute(&self, context: &HookContext) -> Result<HookResult> {
        let mut parts = self.command.split_whitespace();
        let cmd = parts.next().unwrap_or_default();
        let args: Vec<String> = parts.map(String::from).collect();
        let result = TaskExecutor::new(&context.workdir)
            .command(cmd)
            .args(args)
            .env("TASK_ID", &context.task_id)
            .env("TASK_STATE", &context.state)
            .env("WORKDIR", context.workdir.to_string_lossy().as_ref())
            .env("EXIT_CODE", context.exit_code.map(|c| c.to_string()).unwrap_or_default())
            .execute()?;
        Ok(HookResult { success: result.success(), output: result.stdout })
    }
}
