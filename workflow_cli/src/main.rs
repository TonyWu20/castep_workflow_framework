use anyhow::Result;
use tokio_util::sync::CancellationToken;
use workflow_core::{
    executor::ExecutorRegistry,
    pipeline::Pipeline,
    scheduler::Scheduler,
    schema::expand_sweeps,
    state::StateDb,
};
use castep_adapter::CastepFactory;

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let toml_path = args.next()
        .ok_or_else(|| anyhow::anyhow!("usage: workflow <workflow.toml> [--state-db <path>]"))?;

    let state_path = if let Some(flag) = args.next() {
        if flag == "--state-db" {
            args.next()
                .map(std::path::PathBuf::from)
                .ok_or_else(|| anyhow::anyhow!("--state-db requires a path"))?
        } else {
            anyhow::bail!("unknown flag: {flag}");
        }
    } else {
        std::path::Path::new(&toml_path)
            .parent().unwrap_or(std::path::Path::new("."))
            .join(".workflow_state.db")
    };

    let src = std::fs::read_to_string(&toml_path)?;
    let def: workflow_core::schema::WorkflowDef = toml::from_str(&src)?;
    let tasks = expand_sweeps(def)?;
    let pipeline = Pipeline::from_tasks(tasks)?;

    let mut registry = ExecutorRegistry::default();
    registry.register(CastepFactory);

    let state_db = StateDb::open(&state_path).await?;

    let token = CancellationToken::new();
    let token_clone = token.clone();
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        token_clone.cancel();
    });

    Scheduler::new(pipeline, registry, state_db)
        .with_cancellation(token)
        .run()
        .await
}
