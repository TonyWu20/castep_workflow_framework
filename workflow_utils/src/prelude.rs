//! Convenience re-exports for common types from both workflow_core and workflow_utils.
//!
//! ```
//! use workflow_utils::prelude::*;
//! ```

// Re-export everything from workflow_core::prelude
pub use workflow_core::prelude::*;

// workflow_utils types
pub use crate::{
    copy_file, create_dir, exists, read_file, remove_dir, run_default, write_file,
    QueuedRunner, SchedulerKind, ShellHookExecutor, SystemProcessRunner, JOB_SCRIPT_NAME,
};