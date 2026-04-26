pub mod dag;
pub mod error;
mod monitoring;
pub mod prelude;
pub mod process;
pub mod state;
pub mod task;
pub mod workflow;

pub use error::WorkflowError;
pub use monitoring::{HookContext, HookExecutor, HookResult, HookTrigger, MonitoringHook, TaskPhase};
pub use process::{OutputLocation, ProcessHandle, ProcessResult, ProcessRunner, QueuedSubmitter};
pub use state::{JsonStateStore, StateStore, StateStoreExt, StateSummary, TaskStatus, TaskSuccessors};
pub use task::{CollectFailurePolicy, ExecutionMode, Task, TaskClosure};
pub use workflow::{FailedTask, Workflow, WorkflowSummary};

// Returns Box<dyn Error> rather than WorkflowError because tracing_subscriber's
// SetGlobalDefaultError is not convertible to any WorkflowError variant without
// introducing a logging-specific variant that doesn't belong in the domain error type.
/// Initialize default tracing subscriber with env-based filtering.
/// Call once at start of main(). Controlled via RUST_LOG env var.
/// Returns error if already initialized (safe, won't panic).
#[cfg(feature = "default-logging")]
pub fn init_default_logging() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .try_init()
        .map_err(|e| format!("Failed to initialize logging: {e}").into())
}
