//! Schema types for workflow configuration.

use std::collections::HashMap;

/// Executor definition for a task.
#[derive(Debug, Clone)]
pub enum ExecutorDef {
    /// Execute locally.
    Local {
        /// Number of parallel copies to run.
        parallelism: usize,
    },
    /// Execute via SLURM.
    Slurm {
        /// SLURM partition to use.
        partition: String,
        /// Number of tasks.
        ntasks: usize,
        /// Walltime in seconds.
        walltime: String,
    },
}

impl ExecutorDef {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local { .. } => "local",
            Self::Slurm { .. } => "slurm",
        }
    }
}

/// A task definition in the workflow.
#[derive(Debug, Clone)]
pub struct ConcreteTask {
    /// Unique task identifier.
    pub id: String,
    /// Task code (e.g., "castep", "lammps").
    pub code: String,
    /// Executor identifier (e.g., "local", "slurm").
    pub executor: String,
    /// Working directory for this task.
    pub workdir: String,
    /// Dependencies on other task IDs.
    pub depends_on: Vec<String>,
    /// Task inputs as TOML.
    pub inputs: HashMap<String, toml::Value>,
    /// Executor definition.
    pub executor_def: ExecutorDef,
    /// Wall time in seconds.
    pub wall_time_secs: Option<usize>,
    /// Variable bindings for sweep/parallel execution.
    pub bindings: HashMap<String, String>,
    /// Working directories of dependency tasks.
    pub dep_workdirs: HashMap<String, String>,
}
