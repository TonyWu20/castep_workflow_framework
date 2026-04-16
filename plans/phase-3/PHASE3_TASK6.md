# TASK-6: Define `ProcessRunner`, `ProcessHandle`, `ProcessResult` traits/types in `workflow_core`
- **Scope**: Define the process abstraction traits and the `ProcessResult` struct in `workflow_core/src/process.rs` (NEW file).
- **Crate/Module**: `workflow_core/src/process.rs` (NEW), `workflow_core/src/lib.rs`
- **Responsible For**: Defining the abstraction boundary for process execution so the workflow engine does not depend on a specific process runner.
- **Depends On**: TASK-1
- **Enables**: TASK-7, TASK-11
- **Can Run In Parallel With**: TASK-2a, TASK-3, TASK-4, TASK-5
- **Acceptance Criteria**:
  - `workflow_core/src/process.rs` contains exactly:
    ```rust
    use std::collections::HashMap;
    use std::path::Path;
    use std::time::Duration;
    use crate::error::WorkflowError;

    pub trait ProcessRunner: Send + Sync {
        fn spawn(
            &self,
            workdir: &Path,
            command: &str,
            args: &[String],
            env: &HashMap<String, String>,
        ) -> Result<Box<dyn ProcessHandle>, WorkflowError>;
    }

    pub trait ProcessHandle: Send {
        fn is_running(&mut self) -> bool;
        fn terminate(&mut self) -> Result<(), WorkflowError>;
        fn wait(&mut self) -> Result<ProcessResult, WorkflowError>;
    }

    pub struct ProcessResult {
        pub exit_code: Option<i32>,
        pub stdout: String,
        pub stderr: String,
        pub duration: Duration,
    }
    ```
  - `workflow_core/src/lib.rs` has `pub mod process;` and re-exports `ProcessRunner`, `ProcessHandle`, `ProcessResult`.
  - `cargo check -p workflow_core` succeeds.
- **Notes for Subagent**: These are trait-only definitions -- no implementations in `workflow_core`. The concrete `SystemProcessRunner` will be implemented in `workflow_utils` (TASK-7). `ProcessHandle` is `Send` but not `Sync` -- a process handle is used from one thread at a time. `ProcessResult` is deliberately not a trait -- it's a plain data struct.

