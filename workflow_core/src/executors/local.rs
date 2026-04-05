use async_trait::async_trait;
use anyhow::{Result, anyhow};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::path::{Path, PathBuf};
use crate::executor::{Executor, JobHandle, JobStatus};

/// Executes jobs locally using `tokio::process::Command`.
///
/// Stateless after submit: poll and cancel operate on the PID in the handle.
/// On first reap, writes `<workdir>/.exit.<pid>` with the exit code so that
/// subsequent polls (after the zombie is gone) return the correct status even
/// if the OS has recycled the PID.
pub struct LocalExecutor {
    program: String,
    args: Vec<String>,
    workdir: PathBuf,
}

impl LocalExecutor {
    pub fn new(program: impl Into<String>, args: Vec<String>, workdir: impl Into<PathBuf>) -> Self {
        Self { program: program.into(), args, workdir: workdir.into() }
    }
}

fn parse_pid(handle: &JobHandle) -> Result<Pid> {
    let raw: i32 = handle.raw.parse()
        .map_err(|_| anyhow!("invalid PID in handle: {:?}", handle.raw))?;
    Ok(Pid::from_raw(raw))
}

fn sentinel_path(workdir: &Path, pid: Pid) -> PathBuf {
    workdir.join(format!(".exit.{}", pid))
}

async fn write_sentinel(workdir: &Path, pid: Pid, code: i32) {
    let _ = tokio::fs::write(sentinel_path(workdir, pid), code.to_string()).await;
}

async fn read_sentinel(workdir: &Path, pid: Pid) -> Option<JobStatus> {
    let bytes = tokio::fs::read(sentinel_path(workdir, pid)).await.ok()?;
    let code: i32 = std::str::from_utf8(&bytes).ok()?.trim().parse().ok()?;
    Some(if code == 0 { JobStatus::Completed } else { JobStatus::Failed(code) })
}

#[async_trait]
impl Executor for LocalExecutor {
    async fn submit(&self) -> Result<JobHandle> {
        let mut child = tokio::process::Command::new(&self.program)
            .args(&self.args)
            .current_dir(&self.workdir)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;
        let pid = child.id().ok_or_else(|| anyhow!("process exited before PID could be read"))?;
        let workdir = self.workdir.clone();
        // Reap in background and write sentinel so poll() always finds the exit code.
        tokio::spawn(async move {
            if let Ok(status) = child.wait().await {
                let code = status.code().unwrap_or(-1);
                write_sentinel(&workdir, Pid::from_raw(pid as i32), code).await;
            }
        });
        Ok(JobHandle { raw: pid.to_string() })
    }

    async fn poll(&self, handle: &JobHandle) -> Result<JobStatus> {
        let pid = parse_pid(handle)?;
        // Sentinel written by the background reaper in submit().
        if let Some(status) = read_sentinel(&self.workdir, pid).await {
            return Ok(status);
        }
        // No sentinel yet — check if process is still alive via kill(pid, 0).
        match kill(pid, None) {
            Ok(()) => Ok(JobStatus::Running),
            Err(nix::errno::Errno::ESRCH) => {
                // Process gone but sentinel not written yet (tiny race); treat as running
                // and let the next poll cycle find the sentinel.
                Ok(JobStatus::Running)
            }
            Err(e) => Err(anyhow!("kill(0) failed: {e}")),
        }
    }

    async fn cancel(&self, handle: &JobHandle) -> Result<()> {
        let pid = parse_pid(handle)?;
        kill(pid, Signal::SIGTERM).ok();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn local_executor_runs_echo() {
        let exec = LocalExecutor::new("echo", vec!["ok".into()], "/tmp");
        let handle = exec.submit().await.unwrap();
        loop {
            match exec.poll(&handle).await.unwrap() {
                JobStatus::Completed => break,
                JobStatus::Failed(c) => panic!("failed with {c}"),
                JobStatus::Running => tokio::time::sleep(std::time::Duration::from_millis(10)).await,
            }
        }
    }

    #[tokio::test]
    async fn local_executor_cancel_kills_process() {
        let exec = LocalExecutor::new("sleep", vec!["60".into()], "/tmp");
        let handle = exec.submit().await.unwrap();
        exec.cancel(&handle).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let status = exec.poll(&handle).await.unwrap();
        assert!(matches!(status, JobStatus::Failed(_) | JobStatus::Completed));
    }

    /// Resume regression: a fresh executor with only a PID handle can poll a running process.
    #[tokio::test]
    async fn poll_by_pid_survives_executor_drop() {
        let exec = LocalExecutor::new("sleep", vec!["5".into()], "/tmp");
        let handle = exec.submit().await.unwrap();
        drop(exec);

        let exec2 = LocalExecutor::new("sleep", vec![], "/tmp");
        let status = exec2.poll(&handle).await.unwrap();
        assert_eq!(status, JobStatus::Running);

        exec2.cancel(&handle).await.unwrap();
    }

    /// Sentinel regression: double-poll after exit returns correct status, not a guess.
    #[tokio::test]
    async fn double_poll_after_exit_uses_sentinel() {
        let dir = tempfile::tempdir().unwrap();
        let exec = LocalExecutor::new("sh", vec!["-c".into(), "exit 42".into()], dir.path());
        let handle = exec.submit().await.unwrap();

        // Wait for process to exit.
        loop {
            match exec.poll(&handle).await.unwrap() {
                JobStatus::Running => tokio::time::sleep(std::time::Duration::from_millis(10)).await,
                _ => break,
            }
        }
        // Second poll: zombie is gone, sentinel must answer.
        let status = exec.poll(&handle).await.unwrap();
        assert_eq!(status, JobStatus::Failed(42));
    }
}
