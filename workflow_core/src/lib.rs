pub mod dag;
pub mod state;
pub mod task;
pub mod workflow;

pub use task::Task;
pub use workflow::Workflow;
pub use state::{TaskStatus, WorkflowState};

/// Initialize default tracing subscriber with env-based filtering.
/// Call once at start of main(). Controlled via RUST_LOG env var.
/// Returns error if already initialized (safe, won't panic).
#[cfg(feature = "default-logging")]
pub fn init_default_logging() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .try_init()
        .map_err(|e| format!("Failed to initialize logging: {}", e).into())
}
