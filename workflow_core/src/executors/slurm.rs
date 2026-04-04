use async_trait::async_trait;
use anyhow::Result;
use crate::executor::{Executor, JobHandle, JobStatus};
use std::path::PathBuf;

/// Injectable command runner — real impl calls tokio::process::Command,
/// test impl returns canned output.
#[async_trait]
pub trait CommandRunner: Send + Sync {
    async fn run(&self, program: &str, args: &[&str]) -> Result<String>;
}

/// Real runner using tokio::process::Command.
pub struct ProcessRunner;

#[async_trait]
impl CommandRunner for ProcessRunner {
    async fn run(&self, program: &str, args: &[&str]) -> Result<String> {
        let out = tokio::process::Command::new(program)
            .args(args)
            .output()
            .await?;
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }
}

/// Submits jobs via sbatch, polls via squeue, cancels via scancel.
pub struct SlurmExecutor {
    jobscript: String,
    workdir: PathBuf,
    runner: Box<dyn CommandRunner>,
}

impl SlurmExecutor {
    /// Create a new SLURM executor with an injectable command runner.
    pub fn new(
        jobscript: impl Into<String>,
        workdir: impl Into<PathBuf>,
        runner: Box<dyn CommandRunner>,
    ) -> Self {
        Self {
            jobscript: jobscript.into(),
            workdir: workdir.into(),
            runner,
        }
    }
}

#[async_trait]
impl Executor for SlurmExecutor {
    async fn submit(&self) -> Result<JobHandle> {
        let script_path = self.workdir.join("job.sh");
        std::fs::write(&script_path, &self.jobscript)?;

        let output = self.runner.run("sbatch", &[script_path.to_str().unwrap()]).await?;
        let job_id = output
            .lines()
            .find_map(|line| line.strip_prefix("Submitted batch job "))
            .map(|s| s.trim())
            .ok_or_else(|| anyhow::anyhow!("failed to parse job ID from sbatch output"))?;

        Ok(JobHandle {
            raw: job_id.to_string(),
        })
    }

    async fn poll(&self, handle: &JobHandle) -> Result<JobStatus> {
        let output = self
            .runner
            .run("squeue", &["--job", &handle.raw, "--noheader", "--format=%T"])
            .await?;

        let status_line = output.trim();
        if status_line.is_empty() {
            return Ok(JobStatus::Completed);
        }

        match status_line {
            "RUNNING" | "PENDING" => Ok(JobStatus::Running),
            "COMPLETED" => Ok(JobStatus::Completed),
            "FAILED" | "CANCELLED" | "TIMEOUT" | "NODE_FAIL" => Ok(JobStatus::Failed(-1)),
            _ => Ok(JobStatus::Running),
        }
    }

    async fn cancel(&self, handle: &JobHandle) -> Result<()> {
        self.runner.run("scancel", &[&handle.raw]).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockRunner {
        responses: Mutex<Vec<String>>,
    }

    impl MockRunner {
        fn new(responses: Vec<&str>) -> Self {
            Self {
                responses: Mutex::new(responses.into_iter().map(String::from).collect()),
            }
        }
    }

    #[async_trait]
    impl CommandRunner for MockRunner {
        async fn run(&self, _program: &str, _args: &[&str]) -> Result<String> {
            Ok(self
                .responses
                .lock()
                .unwrap()
                .remove(0))
        }
    }

    #[tokio::test]
    async fn submit_parses_job_id() {
        let runner = MockRunner::new(vec!["Submitted batch job 42\n"]);
        let exec = SlurmExecutor::new("#!/bin/bash\necho hi", "/tmp", Box::new(runner));
        let handle = exec.submit().await.unwrap();
        assert_eq!(handle.raw, "42");
    }

    #[tokio::test]
    async fn poll_maps_slurm_states() {
        let h = JobHandle {
            raw: "42".into(),
        };

        let exec = SlurmExecutor::new("", "/tmp", Box::new(MockRunner::new(vec!["RUNNING\n"])));
        assert_eq!(exec.poll(&h).await.unwrap(), JobStatus::Running);

        let exec2 = SlurmExecutor::new("", "/tmp", Box::new(MockRunner::new(vec![""])));
        assert_eq!(exec2.poll(&h).await.unwrap(), JobStatus::Completed);

        let exec3 = SlurmExecutor::new("", "/tmp", Box::new(MockRunner::new(vec!["FAILED\n"])));
        assert_eq!(exec3.poll(&h).await.unwrap(), JobStatus::Failed(-1));
    }
}
