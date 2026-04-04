//! Executor traits and registry for spawning and monitoring HPC jobs.

use std::collections::HashMap;
use anyhow::Result;
use async_trait::async_trait;
use crate::schema::ConcreteTask;

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
///
/// Implement this trait in adapter crates (e.g. `castep_adapter`) to support
/// a new HPC code or execution backend.
#[async_trait]
pub trait Executor: Send {
    /// Submit the job and return a handle for tracking it.
    async fn submit(&self) -> Result<JobHandle>;
    /// Query the current status of a previously submitted job.
    async fn poll(&self, handle: &JobHandle) -> Result<JobStatus>;
    /// Cancel a running job.
    async fn cancel(&self, handle: &JobHandle) -> Result<()>;
}

/// Constructs an [`Executor`] for a specific HPC code from a [`ConcreteTask`].
///
/// Register implementations with [`ExecutorRegistry::register`]. The `code_name`
/// must match the `code = "..."` field in the workflow TOML.
pub trait ExecutorFactory: Send + Sync {
    /// The TOML `code` string this factory handles (e.g. `"castep"`, `"lammps"`).
    fn code_name(&self) -> &'static str;
    /// Build an executor configured for the given task.
    fn build(&self, task: &ConcreteTask) -> Result<Box<dyn Executor>>;
}

/// Maps TOML `code` strings to their [`ExecutorFactory`] implementations.
#[derive(Default)]
pub struct ExecutorRegistry {
    factories: HashMap<String, Box<dyn ExecutorFactory>>,
}

impl ExecutorRegistry {
    /// Register an executor factory. The factory's [`ExecutorFactory::code_name`]
    /// is used as the lookup key.
    pub fn register(&mut self, factory: impl ExecutorFactory + 'static) {
        self.factories.insert(factory.code_name().to_owned(), Box::new(factory));
    }

    /// Build an executor for `task`, dispatching on `task.code`.
    ///
    /// Returns an error if no factory is registered for `task.code`.
    pub fn build(&self, task: &ConcreteTask) -> Result<Box<dyn Executor>> {
        self.factories
            .get(&task.code)
            .ok_or_else(|| anyhow::anyhow!("unknown code: {}", task.code))?
            .build(task)
    }
}
