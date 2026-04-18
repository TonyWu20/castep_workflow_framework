use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Instant;

// Re-exported so consumers that only depend on `workflow_utils` can access
// the core process/error types without a direct `workflow_core` dependency.
pub use workflow_core::WorkflowError;
pub use workflow_core::{ProcessHandle, ProcessResult, ProcessRunner};
pub use workflow_core::process::OutputLocation;

pub struct TaskExecutor {
    workdir: PathBuf,
    command: String,
    args: Vec<String>,
    env: HashMap<String, String>,
}

impl TaskExecutor {
    pub fn new(workdir: impl Into<PathBuf>) -> Self {
        Self {
            workdir: workdir.into(),
            command: String::new(),
            args: Vec::new(),
            env: HashMap::new(),
        }
    }

    pub fn command(mut self, cmd: impl Into<String>) -> Self {
        self.command = cmd.into();
        self
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn args(mut self, args: Vec<String>) -> Self {
        self.args.extend(args);
        self
    }

    pub fn env(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.env.insert(key.into(), val.into());
        self
    }

    pub fn execute(&self) -> Result<ExecutionResult, WorkflowError> {
        let start = std::time::Instant::now();
        let output = std::process::Command::new(&self.command)
            .args(&self.args)
            .envs(&self.env)
            .current_dir(&self.workdir)
            .output()
            .map_err(WorkflowError::Io)?;
        Ok(ExecutionResult {
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            duration: start.elapsed(),
        })
    }

    pub fn spawn(&self) -> Result<ExecutionHandle, WorkflowError> {
        let child = std::process::Command::new(&self.command)
            .args(&self.args)
            .envs(&self.env)
            .current_dir(&self.workdir)
            .spawn()
            .map_err(WorkflowError::Io)?;
        Ok(ExecutionHandle { child })
    }
}

pub struct ExecutionResult {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration: std::time::Duration,
}

impl ExecutionResult {
    pub fn success(&self) -> bool {
        self.exit_code == Some(0)
    }
}

pub struct ExecutionHandle {
    child: std::process::Child,
}

impl ExecutionHandle {
    pub fn pid(&self) -> i32 {
        self.child.id() as i32
    }

    pub fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    pub fn terminate(&mut self) -> Result<(), WorkflowError> {
        self.child.kill().map_err(WorkflowError::Io)
    }
}

/// Concrete implementation of the ProcessRunner trait for system processes.
/// Wraps `std::process::Child` with output capture and timing.
pub struct SystemProcessRunner {
    log_dir: Option<PathBuf>,
}

impl SystemProcessRunner {
    pub fn new() -> Self {
        Self { log_dir: None }
    }

    pub fn with_log_dir(dir: impl Into<PathBuf>) -> Self {
        Self { log_dir: Some(dir.into()) }
    }
}

impl ProcessRunner for SystemProcessRunner {
    fn spawn(
        &self,
        workdir: &Path,
        command: &str,
        args: &[String],
        env: &HashMap<String, String>,
    ) -> Result<Box<dyn ProcessHandle>, WorkflowError> {
        let (stdout_cfg, stderr_cfg, output_files) = if let Some(ref log_dir) = self.log_dir {
            // Derive file prefix from workdir leaf directory name
            let prefix = workdir
                .file_name()
                .map(|n| n.to_string_lossy().replace('/', "_"))
                .unwrap_or_else(|| "unknown".into());
            let stdout_path = log_dir.join(format!("{}.stdout", prefix));
            let stderr_path = log_dir.join(format!("{}.stderr", prefix));
            let stdout_file = std::fs::File::create(&stdout_path).map_err(WorkflowError::Io)?;
            let stderr_file = std::fs::File::create(&stderr_path).map_err(WorkflowError::Io)?;
            (Stdio::from(stdout_file), Stdio::from(stderr_file), Some((stdout_path, stderr_path)))
        } else {
            (Stdio::piped(), Stdio::piped(), None)
        };

        let child = Command::new(command)
            .args(args)
            .envs(env)
            .current_dir(workdir)
            .stdout(stdout_cfg)
            .stderr(stderr_cfg)
            .spawn()
            .map_err(WorkflowError::Io)?;

        Ok(Box::new(SystemProcessHandle {
            child: Some(child),
            start: Instant::now(),
            output_files,
        }))
    }
}

/// Handle to a running system process.
/// Uses `Option<Child>` to allow consuming the child once via `wait()`.
pub struct SystemProcessHandle {
    child: Option<Child>,
    start: Instant,
    output_files: Option<(PathBuf, PathBuf)>,
}

impl ProcessHandle for SystemProcessHandle {
    fn is_running(&mut self) -> bool {
        match &mut self.child {
            Some(child) => matches!(child.try_wait(), Ok(None)),
            None => false,  // Already waited/terminated
        }
    }

    fn terminate(&mut self) -> Result<(), WorkflowError> {
        match &mut self.child {
            Some(child) => child.kill().map_err(WorkflowError::Io),
            None => Ok(()),  // Idempotent: already terminated/waited
        }
    }

    fn wait(&mut self) -> Result<ProcessResult, WorkflowError> {
        let child = self.child.take()
            .ok_or_else(|| WorkflowError::InvalidConfig("wait() called twice".into()))?;

        let (output_loc, exit_code) = if let Some((ref stdout_path, ref stderr_path)) = self.output_files {
            // File-backed: just wait for exit, output is on disk
            let status = child.wait_with_output().map_err(WorkflowError::Io)?;
            (OutputLocation::OnDisk {
                stdout_path: stdout_path.clone(),
                stderr_path: stderr_path.clone(),
            }, status.status.code())
        } else {
            let output = child.wait_with_output().map_err(WorkflowError::Io)?;
            (OutputLocation::Captured {
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            }, output.status.code())
        };

        Ok(ProcessResult {
            exit_code,
            output: output_loc,
            duration: self.start.elapsed(),
        })
    }
}
