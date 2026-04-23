//! Convenience re-exports for common workflow_core types.
//!
//! ```
//! use workflow_core::prelude::*;
//! ```

pub use crate::error::WorkflowError;
pub use crate::state::{JsonStateStore, StateStore, StateStoreExt, TaskStatus};
pub use crate::task::{ExecutionMode, Task};
pub use crate::workflow::{Workflow, WorkflowSummary};
pub use crate::{HookExecutor, ProcessRunner};