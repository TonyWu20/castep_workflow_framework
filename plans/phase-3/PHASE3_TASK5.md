# TASK-5: Implement `HookExecutor` trait in `workflow_core`

- **Scope**: Define the `HookExecutor` trait in `workflow_core/src/monitoring.rs` and re-export it. Implement it as `ShellHookExecutor` in `workflow_utils`. Write mock-based tests for the trait contract.
- **Crate/Module**: `workflow_core/src/monitoring.rs`, `workflow_core/src/lib.rs`, `workflow_utils/src/monitoring.rs`
- **Responsible For**: Defining the trait abstraction for hook execution and providing the shell-based concrete implementation.
- **Depends On**: TASK-3
- **Enables**: TASK-11
- **Can Run In Parallel With**: TASK-4, TASK-6, TASK-7
- **Acceptance Criteria**:
  - The trait is defined exactly as:
    ```rust
    pub trait HookExecutor: Send + Sync {
        fn execute_hook(&self, hook: &MonitoringHook, ctx: &HookContext) -> Result<HookResult, WorkflowError>;
    }
    ```
  - `workflow_core/src/lib.rs` re-exports `HookExecutor`.
  - `workflow_utils/src/monitoring.rs` contains a concrete impl:

    ```rust
    pub struct ShellHookExecutor;

    impl workflow_core::HookExecutor for ShellHookExecutor {
        fn execute_hook(&self, hook: &MonitoringHook, ctx: &HookContext) -> Result<HookResult, WorkflowError> {
            let mut parts = hook.command.split_whitespace();
            let cmd = parts.next().unwrap_or_default();
            let args: Vec<String> = parts.map(String::from).collect();
            let result = TaskExecutor::new(&ctx.workdir)
                .command(cmd)
                .args(args)
                .env("TASK_ID", &ctx.task_id)
                .env("TASK_STATE", &ctx.state)
                .env("WORKDIR", ctx.workdir.to_string_lossy().as_ref())
                .env("EXIT_CODE", ctx.exit_code.map(|c| c.to_string()).unwrap_or_default())
                .execute()
                .map_err(|e| WorkflowError::Io(std::io::Error::other(e.to_string())))?;
            Ok(HookResult { success: result.success(), output: result.stdout })
        }
    }
    ```

  - Tests for the `HookExecutor` trait contract written alongside implementation: a mock executor that succeeds, and one that returns an error.
  - `cargo test -p workflow_core` and `cargo test -p workflow_utils` both pass.

- **Notes for Subagent**: The `ShellHookExecutor` replaces the old `MonitoringHook::execute()` method. Write mock tests in `workflow_core/src/monitoring.rs` using local structs that implement `HookExecutor` -- no external process needed for the trait tests. Use two mocks:

  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;

      struct MockOkExecutor;
      impl HookExecutor for MockOkExecutor {
          fn execute_hook(&self, _hook: &MonitoringHook, _ctx: &HookContext) -> Result<HookResult, WorkflowError> {
              Ok(HookResult { success: true, output: String::new() })
          }
      }

      struct MockErrExecutor;
      impl HookExecutor for MockErrExecutor {
          fn execute_hook(&self, _hook: &MonitoringHook, _ctx: &HookContext) -> Result<HookResult, WorkflowError> {
              Err(WorkflowError::Io(std::io::Error::other("mock failure")))
          }
      }

      #[test]
      fn mock_ok_executor_returns_success() {
          let hook = MonitoringHook { command: "echo hi".into(), ..Default::default() };
          let ctx = HookContext { task_id: "t1".into(), state: "running".into(), workdir: ".".into(), exit_code: None };
          let result = MockOkExecutor.execute_hook(&hook, &ctx).unwrap();
          assert!(result.success);
      }

      #[test]
      fn mock_err_executor_returns_error() {
          let hook = MonitoringHook { command: "echo hi".into(), ..Default::default() };
          let ctx = HookContext { task_id: "t1".into(), state: "running".into(), workdir: ".".into(), exit_code: None };
          assert!(matches!(MockErrExecutor.execute_hook(&hook, &ctx), Err(WorkflowError::Io(_))));
      }
  }
  ```

  Note: adjust `MonitoringHook` and `HookContext` field construction to match their actual definitions in `monitoring.rs`.
