mod executor;
mod files;
mod monitoring;
pub mod prelude;
mod queued;

pub use executor::{ExecutionHandle, ExecutionResult, OutputLocation, TaskExecutor, SystemProcessRunner};
pub use files::{copy_file, create_dir, exists, read_file, remove_dir, write_file};
// Re-export hook types from workflow_core for backward compatibility
pub use monitoring::ShellHookExecutor;
pub use queued::{QueuedRunner, SchedulerKind, JOB_SCRIPT_NAME};
pub use workflow_core::{HookContext, HookResult, HookTrigger, MonitoringHook};

use std::sync::Arc;
use workflow_core::state::StateStore;
use workflow_core::workflow::{Workflow, WorkflowSummary};
use workflow_core::{HookExecutor, ProcessRunner, WorkflowError};

/// Runs a workflow with the default `SystemProcessRunner` and `ShellHookExecutor`.
///
/// Eliminates the repeated `Arc` wiring boilerplate in every binary that uses
/// direct (non-queued) process execution.
///
/// # Example
/// ```ignore
/// let mut workflow = Workflow::new("my_workflow");
/// // ... add tasks ...
/// let mut state = JsonStateStore::new("my_workflow", path);
/// let summary = workflow_utils::run_default(&mut workflow, &mut state)?;
/// ```
pub fn run_default(
    workflow: &mut Workflow,
    state: &mut dyn StateStore,
) -> Result<WorkflowSummary, WorkflowError> {
    let runner: Arc<dyn ProcessRunner> = Arc::new(SystemProcessRunner::new());
    let hook_executor: Arc<dyn HookExecutor> = Arc::new(ShellHookExecutor);
    workflow.run(state, runner, hook_executor)
}
