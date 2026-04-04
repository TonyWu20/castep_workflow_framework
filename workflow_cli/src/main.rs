use anyhow::Result;
use workflow_core::{
    executor::ExecutorRegistry,
    pipeline::Pipeline,
    scheduler::Scheduler,
    schema::expand_sweeps,
    state::StateDb,
};
use castep_adapter::CastepFactory;
use lammps_adapter::LammpsFactory;

#[tokio::main]
async fn main() -> Result<()> {
    let toml_path = std::env::args().nth(1)
        .ok_or_else(|| anyhow::anyhow!("usage: workflow <workflow.toml>"))?;

    let src = std::fs::read_to_string(&toml_path)?;
    let def: workflow_core::schema::WorkflowDef = toml::from_str(&src)?;

    let state_path = std::path::Path::new(&toml_path)
        .parent().unwrap_or(std::path::Path::new("."))
        .join(".workflow_state.db");

    let tasks = expand_sweeps(def)?;
    let pipeline = Pipeline::from_tasks(tasks)?;

    let mut registry = ExecutorRegistry::default();
    registry.register(CastepFactory);
    registry.register(LammpsFactory);

    let state_db = StateDb::open(&state_path).await?;
    let mut scheduler = Scheduler::new(pipeline, registry, state_db);
    scheduler.run().await
}
