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

/// A handle to a running (or finished) process, used to poll, wait, or terminate it.
///
/// Implementations must be `Send` so handles can be stored across thread boundaries.
pub trait ProcessHandle: Send {
    /// Returns `true` if the process is still running.
    ///
    /// Implementations may cache the result and only re-poll periodically.
    fn is_running(&mut self) -> bool;

    /// Requests termination of the process.
    ///
    /// Best-effort: the process may already have exited.
    fn terminate(&mut self) -> Result<(), WorkflowError>;

    /// Returns the process result once the process has finished.
    ///
    /// For queued (HPC) handles this may return immediately with `OnDisk` output
    /// paths rather than captured output. Callers should ensure `is_running()`
    /// has returned `false` before calling `wait()`, as behaviour when called
    /// on a still-running process is implementation-defined.
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
