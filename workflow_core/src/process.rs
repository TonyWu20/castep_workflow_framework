use std::collections::HashMap;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
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

/// Concrete implementation of ProcessRunner for system processes.
pub struct SystemProcessRunner;

impl ProcessRunner for SystemProcessRunner {
    fn spawn(
        &self,
        workdir: &Path,
        command: &str,
        args: &[String],
        env: &HashMap<String, String>,
    ) -> Result<Box<dyn ProcessHandle>, WorkflowError> {
        let child = Command::new(command)
            .args(args)
            .envs(env)
            .current_dir(workdir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(WorkflowError::Io)?;

        Ok(Box::new(SystemProcessHandle {
            child: Some(child),
            start: Instant::now(),
        }))
    }
}

/// Handle to a running system process.
pub struct SystemProcessHandle {
    child: Option<Child>,
    start: Instant,
}

impl ProcessHandle for SystemProcessHandle {
    fn is_running(&mut self) -> bool {
        match &mut self.child {
            Some(child) => matches!(child.try_wait(), Ok(None)),
            None => false,
        }
    }

    fn terminate(&mut self) -> Result<(), WorkflowError> {
        match &mut self.child {
            Some(child) => child.kill().map_err(WorkflowError::Io),
            None => Ok(()),
        }
    }

    fn wait(&mut self) -> Result<ProcessResult, WorkflowError> {
        let child = self.child.take()
            .ok_or_else(|| WorkflowError::InvalidConfig("wait() called twice".into()))?;

        let output = child.wait_with_output().map_err(WorkflowError::Io)?;

        Ok(ProcessResult {
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            duration: self.start.elapsed(),
        })
    }
}
