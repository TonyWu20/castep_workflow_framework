use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use anyhow::{Context, Result};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;


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

    pub fn execute(&self) -> Result<ExecutionResult> {
        let start = std::time::Instant::now();
        let output = std::process::Command::new(&self.command)
            .args(&self.args)
            .envs(&self.env)
            .current_dir(&self.workdir)
            .output()
            .with_context(|| format!("failed to execute: {}", self.command))?;
        Ok(ExecutionResult {
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            duration: start.elapsed(),
        })
    }

    pub fn spawn(&self) -> Result<ExecutionHandle> {
        let child = std::process::Command::new(&self.command)
            .args(&self.args)
            .envs(&self.env)
            .current_dir(&self.workdir)
            .spawn()
            .with_context(|| format!("failed to spawn: {}", self.command))?;
        let pid = child.id() as i32;
        Ok(ExecutionHandle { pid, child })
    }
}

pub struct ExecutionResult {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
}

impl ExecutionResult {
    pub fn success(&self) -> bool {
        self.exit_code == Some(0)
    }
}

pub struct ExecutionHandle {
    pid: i32,
    // Held to prevent the child process from being orphaned on drop
    #[allow(dead_code)]
    child: std::process::Child,
}

impl ExecutionHandle {
    pub fn pid(&self) -> i32 {
        self.pid
    }

    pub fn is_running(&self) -> bool {
        use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
        match waitpid(Pid::from_raw(self.pid), Some(WaitPidFlag::WNOHANG)) {
            Ok(WaitStatus::StillAlive) => true,
            _ => false,
        }
    }

    pub fn terminate(&self) -> Result<()> {
        kill(Pid::from_raw(self.pid), Signal::SIGTERM)
            .with_context(|| format!("failed to terminate pid {}", self.pid))
    }
}
