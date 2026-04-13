# TASK-4: Update `ExecutionHandle` to own `Child` and remove `nix` PID usage

- **Scope**: Refactor `ExecutionHandle` in `workflow_utils/src/executor.rs` to use the owned `Child` handle for process management instead of raw PID + nix signals.
- **Crate/Module**: `workflow_utils/src/executor.rs`, `workflow_utils/Cargo.toml`
- **Responsible For**: Making process management safe and portable by using `std::process::Child` directly.
- **Depends On**: TASK-2d, TASK-3
- **Can Run In Parallel With**: TASK-5, TASK-6
- **Acceptance Criteria**:
  - `ExecutionHandle` fields become:
    ```rust
    pub struct ExecutionHandle {
        child: Child,  // std::process::Child -- NOT Option<Child>
    }
    ```
  - `pid()` method extracts PID from the owned child: `self.child.id() as i32`.
  - `is_running()` uses `self.child.try_wait()` instead of `nix::sys::wait::waitpid`:
    ```rust
    pub fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }
    ```
    Note: `is_running` now takes `&mut self` because `try_wait` requires `&mut self`.
  - `terminate()` uses `self.child.kill()`:
    ```rust
    pub fn terminate(&mut self) -> Result<(), WorkflowError> {
        self.child.kill().map_err(WorkflowError::Io)
    }
    ```
  - `spawn()` method in `TaskExecutor` stores the `Child` directly:
    ```rust
    pub fn spawn(&self) -> Result<ExecutionHandle, WorkflowError> {
        let child = std::process::Command::new(&self.command)
            .args(&self.args)
            .envs(&self.env)
            .current_dir(&self.workdir)
            .spawn()
            .map_err(WorkflowError::Io)?;
        Ok(ExecutionHandle { child })
    }
    ```
    PID is extracted from the child on demand via `pid()`, not cached at spawn time.
  - `nix` is removed from `workflow_utils/Cargo.toml` dependencies.
  - `nix` is removed from workspace `Cargo.toml` `[workspace.dependencies]`.
  - All existing tests pass.
- **Notes for Subagent**: The current code extracts `pid` at spawn time and stores it alongside `child`. After this change, only `child` is stored. The `pid()` accessor calls `self.child.id()` which is always available. The `is_running()` signature changes from `&self` to `&mut self` -- update all callers. `Child::kill()` sends SIGKILL, which is acceptable for now (graceful SIGTERM is handled at the workflow level in TASK-15).
