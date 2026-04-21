use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use workflow_core::error::WorkflowError;
use workflow_core::process::{OutputLocation, ProcessHandle, ProcessResult};

/// The type of HPC job scheduler to target.
#[derive(Debug, Clone, Copy)]
pub enum SchedulerKind {
    /// SLURM Workload Manager (`sbatch` / `squeue` / `scancel`).
    Slurm,
    /// Portable Batch System (`qsub` / `qstat` / `qdel`).
    Pbs,
}

/// Submits and manages jobs via an HPC batch scheduler.
///
/// Implements [`QueuedSubmitter`](workflow_core::process::QueuedSubmitter) to
/// integrate with the workflow engine's `Queued` execution mode.
pub struct QueuedRunner {
    /// Which scheduler dialect to use for command construction.
    scheduler: SchedulerKind,
}

impl QueuedRunner {
    pub fn new(scheduler: SchedulerKind) -> Self {
        Self { scheduler }
    }

    /// Returns the scheduler kind this runner targets.
    pub fn scheduler(&self) -> SchedulerKind {
        self.scheduler
    }

    fn build_poll_cmd(&self) -> String {
        match self.scheduler {
            SchedulerKind::Slurm => "squeue -j {job_id} -h".into(),
            SchedulerKind::Pbs => "qstat {job_id}".into(),
        }
    }

    fn build_cancel_cmd(&self) -> String {
        match self.scheduler {
            SchedulerKind::Slurm => "scancel {job_id}".into(),
            SchedulerKind::Pbs => "qdel {job_id}".into(),
        }
    }

    fn parse_job_id(&self, stdout: &str) -> Result<String, WorkflowError> {
        match self.scheduler {
            SchedulerKind::Slurm => stdout
                .split_whitespace()
                .last()
                .map(|s| s.to_string())
                .ok_or_else(|| WorkflowError::QueueSubmitFailed(
                    format!("failed to parse SLURM job ID from: {}", stdout)
                )),
            SchedulerKind::Pbs => {
                let trimmed = stdout.trim().to_string();
                if trimmed.is_empty() {
                    Err(WorkflowError::QueueSubmitFailed("empty PBS job ID".into()))
                } else {
                    Ok(trimmed)
                }
            }
        }
    }

}

impl workflow_core::process::QueuedSubmitter for QueuedRunner {
    fn submit(
        &self,
        workdir: &Path,
        task_id: &str,
        log_dir: &Path,
    ) -> Result<Box<dyn ProcessHandle>, WorkflowError> {
        let stdout_path = log_dir.join(format!("{}.stdout", task_id));
        let stderr_path = log_dir.join(format!("{}.stderr", task_id));
        let script_path = workdir.join("job.sh");

        let output = match self.scheduler {
            SchedulerKind::Slurm => Command::new("sbatch"),
            SchedulerKind::Pbs => Command::new("qsub"),
        }
        .args(["-o", &stdout_path.to_string_lossy(), "-e", &stderr_path.to_string_lossy()])
        .arg(&script_path)
        .current_dir(workdir)
        .output()
        .map_err(WorkflowError::Io)?;

        if !output.status.success() {
            return Err(WorkflowError::QueueSubmitFailed(
                String::from_utf8_lossy(&output.stderr).into_owned()
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let job_id = self.parse_job_id(&stdout)?;

        Ok(Box::new(QueuedProcessHandle {
            job_id,
            poll_cmd: self.build_poll_cmd(),
            cancel_cmd: self.build_cancel_cmd(),
            stdout_path,
            stderr_path,
            last_poll: Instant::now(),
            poll_interval: Duration::from_secs(15),
            cached_running: true,
            finished_exit_code: None,
            started_at: Instant::now(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_slurm_job_id_from_submit_output() {
        let runner = QueuedRunner::new(SchedulerKind::Slurm);
        let id = runner.parse_job_id("Submitted batch job 12345").unwrap();
        assert_eq!(id, "12345");
    }

    #[test]
    fn parse_slurm_job_id_single_word() {
        let runner = QueuedRunner::new(SchedulerKind::Slurm);
        let id = runner.parse_job_id("99999").unwrap();
        assert_eq!(id, "99999");
    }

    #[test]
    fn parse_slurm_job_id_empty_fails() {
        let runner = QueuedRunner::new(SchedulerKind::Slurm);
        assert!(runner.parse_job_id("").is_err());
    }

    #[test]
    fn parse_pbs_job_id_typical() {
        let runner = QueuedRunner::new(SchedulerKind::Pbs);
        let id = runner.parse_job_id("1234.pbs-server\n").unwrap();
        assert_eq!(id, "1234.pbs-server");
    }

    #[test]
    fn parse_pbs_job_id_empty_fails() {
        let runner = QueuedRunner::new(SchedulerKind::Pbs);
        assert!(runner.parse_job_id("").is_err());
    }

    #[test]
    fn parse_pbs_job_id_whitespace_only_fails() {
        let runner = QueuedRunner::new(SchedulerKind::Pbs);
        assert!(runner.parse_job_id("   \n  ").is_err());
    }
}

pub struct QueuedProcessHandle {
    job_id: String,
    poll_cmd: String,
    cancel_cmd: String,
    stdout_path: PathBuf,
    stderr_path: PathBuf,
    last_poll: Instant,
    poll_interval: Duration,
    cached_running: bool,
    finished_exit_code: Option<i32>,
    started_at: Instant,
}

impl ProcessHandle for QueuedProcessHandle {
    fn is_running(&mut self) -> bool {
        if self.last_poll.elapsed() < self.poll_interval {
            return self.cached_running;
        }

        let cmd = self.poll_cmd.replace("{job_id}", &self.job_id);
        let result = Command::new("sh")
            .args(["-c", &cmd])
            .output();

        match result {
            Ok(output) => {
                // Non-zero exit or empty stdout = job gone
                let running = output.status.success()
                    && !output.stdout.is_empty();
                self.cached_running = running;
                if !running {
                    self.finished_exit_code = Some(0); // default; accounting query in wait() may refine
                }
            }
            Err(_) => {
                self.cached_running = false;
                self.finished_exit_code = Some(-1);
            }
        }

        self.last_poll = Instant::now();
        self.cached_running
    }

    fn terminate(&mut self) -> Result<(), WorkflowError> {
        let cmd = self.cancel_cmd.replace("{job_id}", &self.job_id);
        Command::new("sh")
            .args(["-c", &cmd])
            .output()
            .map_err(WorkflowError::Io)?;
        Ok(())
    }

    fn wait(&mut self) -> Result<ProcessResult, WorkflowError> {
        Ok(ProcessResult {
            exit_code: self.finished_exit_code,
            output: OutputLocation::OnDisk {
                stdout_path: self.stdout_path.clone(),
                stderr_path: self.stderr_path.clone(),
            },
            duration: self.started_at.elapsed(),
        })
    }
}
