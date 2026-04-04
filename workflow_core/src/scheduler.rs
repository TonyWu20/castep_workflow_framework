//! Scheduler skeleton — drives the pipeline execution loop.

use anyhow::Result;
use crate::executor::ExecutorRegistry;
use crate::pipeline::Pipeline;
use crate::state::StateDb;

/// Drives execution of a [`Pipeline`], persisting state to [`StateDb`].
///
/// The scheduler polls all active jobs on a timer, promotes tasks whose
/// dependencies have completed, and skips tasks whose upstream paths have
/// all failed. On resume, tasks already marked [`crate::state::TaskState::Completed`]
/// in the state DB are skipped automatically.
pub struct Scheduler {
    pub pipeline: Pipeline,
    pub registry: ExecutorRegistry,
    pub state_db: StateDb,
}

impl Scheduler {
    /// Create a new scheduler from a resolved pipeline, executor registry, and state DB.
    pub fn new(pipeline: Pipeline, registry: ExecutorRegistry, state_db: StateDb) -> Self {
        Self { pipeline, registry, state_db }
    }

    /// Run the pipeline to completion (or until all tasks are settled).
    ///
    /// Independent branches continue running when one branch fails.
    /// State is persisted to the DB after every poll cycle for resume support.
    pub async fn run(&mut self) -> Result<()> {
        todo!("scheduler loop not yet implemented")
    }
}
