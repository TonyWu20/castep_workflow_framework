use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
use crate::error::WorkflowError;

pub trait ProcessRunner: Send + Sync {
    fn spawn(
        &self,
        workdir: &Path,
        command: &str,
        args: &[String],
        env: &HashMap<String, String>,
    ) -> Result<Box<dyn ProcessHandle>, WorkflowError>;
}

pub trait ProcessHandle: Send {
    fn is_running(&mut self) -> bool;
    fn terminate(&mut self) -> Result<(), WorkflowError>;
    fn wait(&mut self) -> Result<ProcessResult, WorkflowError>;
}

pub struct ProcessResult {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
}
