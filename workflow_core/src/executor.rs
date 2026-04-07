//! Executor traits and registry for spawning and monitoring HPC jobs.

use anyhow::Result;
use async_trait::async_trait;

/// An opaque handle to a submitted job — a SLURM job ID, local PID, etc.
#[derive(Debug, Clone)]
pub struct JobHandle {
    /// Raw string representation (e.g. `"12345"` for a SLURM job ID).
    pub raw: String,
}

/// The observed status of a submitted job.
#[derive(Debug, Clone, PartialEq)]
pub enum JobStatus {
    /// Job is still executing.
    Running,
    /// Job finished successfully (exit code 0).
    Completed,
    /// Job finished with a non-zero exit code.
    Failed(i32),
}

/// Submits, polls, and cancels a single job on a specific backend.
#[async_trait]
pub trait Executor: Send + Sync {
    async fn submit(&self) -> Result<JobHandle>;
    async fn poll(&self, handle: &JobHandle) -> Result<JobStatus>;
    async fn cancel(&self, handle: &JobHandle) -> Result<()>;
}
