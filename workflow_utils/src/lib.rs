mod executor;
mod files;
mod monitoring;

pub use executor::{ExecutionHandle, ExecutionResult, SystemProcessHandle, SystemProcessRunner, TaskExecutor};
pub use files::{copy_file, create_dir, exists, read_file, remove_dir, write_file};
// Re-export hook types from workflow_core for backward compatibility
pub use monitoring::execute_hook;
pub use workflow_core::{HookContext, HookResult, HookTrigger, MonitoringHook};
