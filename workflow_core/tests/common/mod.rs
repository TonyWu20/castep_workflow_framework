use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use workflow_core::{HookContext, HookExecutor, HookResult, MonitoringHook, WorkflowError};
use workflow_core::task::ExecutionMode;

/// A test executor that records all hook invocations.
///
/// Stores calls as `(hook_name, task_id)` pairs in a shared `Arc<Mutex<Vec<_>>>`.
/// The `Arc` is exposed via `calls()` so test bodies can verify recorded calls.
pub struct RecordingExecutor {
    /// Shared storage of recorded hook calls: (hook name, task id)
    pub calls: Arc<Mutex<Vec<(String, String)>>>,
}

impl Clone for RecordingExecutor {
    fn clone(&self) -> Self {
        Self {
            calls: Arc::clone(&self.calls),
        }
    }
}

impl RecordingExecutor {
    /// Creates a new `RecordingExecutor` with an empty call log.
    pub fn new() -> Self {
        Self {
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Returns a reference to the recorded calls.
    ///
    /// Each entry is `(hook_name, task_id)` in the order they were executed.
    pub fn calls(&self) -> Vec<(String, String)> {
        self.calls
            .lock()
            .unwrap()
            .clone()
    }
}

impl HookExecutor for RecordingExecutor {
    fn execute_hook(
        &self,
        hook: &MonitoringHook,
        ctx: &HookContext,
    ) -> Result<HookResult, WorkflowError> {
        // Record the hook invocation before execution
        let mut recorded = self.calls.lock().unwrap();
        recorded.push((hook.name.clone(), ctx.task_id.clone()));

        // Execute the hook (mock success for now)
        Ok(HookResult {
            success: true,
            output: String::new(),
        })
    }
}

/// Creates an `ExecutionMode::Direct` executor for test tasks.
///
/// This is a convenience builder that returns an `ExecutionMode` with:
/// - command: the provided shell command
/// - args: empty vector
/// - env: empty HashMap
/// - timeout: None (no timeout)
///
/// Use `common::direct("command")` instead of inline constructions.
pub fn direct(cmd: &str) -> ExecutionMode {
    ExecutionMode::Direct {
        command: cmd.into(),
        args: vec![],
        env: HashMap::new(),
        timeout: None,
    }
}
