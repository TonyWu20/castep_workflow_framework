use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use workflow_core::error::WorkflowError;
use workflow_core::process::{OutputLocation, ProcessHandle, ProcessResult};

#[derive(Debug, Clone, Copy)]
pub enum SchedulerKind {
    Slurm,
    Pbs,
}

pub struct QueuedRunner {
    pub scheduler: SchedulerKind,
}

impl QueuedRunner {
    pub fn new(scheduler: SchedulerKind) -> Self {
        Self { scheduler }
    }

    fn build_submit_cmd(&self, script_path: &str, task_id: &str, log_dir: &Path) -> String {
        let stdout_path = log_dir.join(format!("{}.stdout", task_id));
        let stderr_path = log_dir.join(format!("{}.stderr", task_id));
        match self.scheduler {
            SchedulerKind::Slurm => format!(
                "sbatch -o {} -e {} {}",
                stdout_path.display(), stderr_path.display(), script_path
            ),
            SchedulerKind::Pbs => format!(
                "qsub -o {} -e {} {}",
                stdout_path.display(), stderr_path.display(), script_path
            ),
        }
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
        let submit_cmd = self.build_submit_cmd(
            &workdir.join("job.sh").to_string_lossy(), task_id, log_dir
        );
        let output = Command::new("sh")
            .args(["-c", &submit_cmd])
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

        let stdout_path = log_dir.join(format!("{}.stdout", task_id));
        let stderr_path = log_dir.join(format!("{}.stderr", task_id));

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

pub struct QueuedProcessHandle {
    job_id: String,
    poll_cmd: String,
    cancel_cmd: String,
    workdir: PathBuf,
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
