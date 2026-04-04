use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::process::Child;
use async_trait::async_trait;
use anyhow::Result;
use crate::executor::{Executor, JobHandle, JobStatus};

/// Executes jobs locally using `tokio::process::Command`.
pub struct LocalExecutor {
    program: String,
    args: Vec<String>,
    workdir: std::path::PathBuf,
    child: Arc<Mutex<Option<Child>>>,
}

impl LocalExecutor {
    /// Create a new local executor.
    pub fn new(program: impl Into<String>, args: Vec<String>, workdir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            program: program.into(),
            args,
            workdir: workdir.into(),
            child: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait]
impl Executor for LocalExecutor {
    async fn submit(&self) -> Result<JobHandle> {
        let child = tokio::process::Command::new(&self.program)
            .args(&self.args)
            .current_dir(&self.workdir)
            .spawn()?;

        let pid = child.id().ok_or_else(|| anyhow::anyhow!("failed to get child PID"))?;

        let mut stored_child = self.child.lock().await;
        *stored_child = Some(child);

        Ok(JobHandle {
            raw: pid.to_string(),
        })
    }

    async fn poll(&self, _handle: &JobHandle) -> Result<JobStatus> {
        let mut child = self.child.lock().await;

        if let Some(ref mut c) = *child {
            match c.try_wait()? {
                Some(status) => {
                    if status.success() {
                        Ok(JobStatus::Completed)
                    } else {
                        let code = status.code().unwrap_or(-1);
                        Ok(JobStatus::Failed(code))
                    }
                }
                None => Ok(JobStatus::Running),
            }
        } else {
            Ok(JobStatus::Failed(-1))
        }
    }

    async fn cancel(&self, _handle: &JobHandle) -> Result<()> {
        let mut child = self.child.lock().await;

        if let Some(ref mut c) = *child {
            c.kill().await?;
        }

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
        assert!(matches!(status, JobStatus::Failed(_)));
    }
}
