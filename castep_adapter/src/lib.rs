use anyhow::Result;
use async_trait::async_trait;
use workflow_core::executor::{Executor, ExecutorFactory, JobHandle, JobStatus};
use workflow_core::schema::ConcreteTask;

pub struct CastepFactory;

impl ExecutorFactory for CastepFactory {
    fn code_name(&self) -> &'static str { "castep" }

    fn build(&self, _task: &ConcreteTask) -> Result<Box<dyn Executor>> {
        todo!("CastepExecutor not yet implemented")
    }
}

struct CastepExecutor;

#[async_trait]
impl Executor for CastepExecutor {
    async fn submit(&self) -> Result<JobHandle> { todo!() }
    async fn poll(&self, _handle: &JobHandle) -> Result<JobStatus> { todo!() }
    async fn cancel(&self, _handle: &JobHandle) -> Result<()> { todo!() }
}
