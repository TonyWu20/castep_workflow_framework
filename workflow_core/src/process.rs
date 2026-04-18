use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use crate::error::WorkflowError;

#[derive(Debug, Clone)]
pub enum OutputLocation {
    Captured { stdout: String, stderr: String },
    OnDisk { stdout_path: PathBuf, stderr_path: PathBuf },
}

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
    pub output: OutputLocation,
    pub duration: Duration,
}

pub trait QueuedSubmitter: Send + Sync {
    fn submit(
        &self,
        workdir: &Path,
        task_id: &str,
        log_dir: &Path,
    ) -> Result<Box<dyn ProcessHandle>, WorkflowError>;
}
