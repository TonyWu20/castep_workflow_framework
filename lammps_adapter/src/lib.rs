use anyhow::Result;
use async_trait::async_trait;
use workflow_core::executor::{Executor, ExecutorFactory, JobHandle, JobStatus};
use workflow_core::schema::ConcreteTask;

pub struct LammpsFactory;

impl ExecutorFactory for LammpsFactory {
    fn code_name(&self) -> &'static str { "lammps" }

    fn build(&self, _task: &ConcreteTask) -> Result<Box<dyn Executor>> {
        todo!("LammpsExecutor not yet implemented")
    }
}

struct LammpsExecutor;

#[async_trait]
impl Executor for LammpsExecutor {
    async fn submit(&self) -> Result<JobHandle> { todo!() }
    async fn poll(&self, _handle: &JobHandle) -> Result<JobStatus> { todo!() }
    async fn cancel(&self, _handle: &JobHandle) -> Result<()> { todo!() }
}
