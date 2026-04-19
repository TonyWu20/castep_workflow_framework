use crate::WorkflowError;
use serde::{Deserialize, Serialize};

/// Executor trait for monitoring hooks.
///
/// This trait abstracts the execution of monitoring hooks, allowing different
/// backend implementations (e.g., shell-based, in-process, etc.).
pub trait HookExecutor: Send + Sync {
    /// Executes a monitoring hook with the given context.
    fn execute_hook(
        &self,
        hook: &MonitoringHook,
        ctx: &HookContext,
    ) -> Result<HookResult, WorkflowError>;
}

/// A monitoring hook that can be triggered by a task event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringHook {
    /// Human-readable name of the hook.
    pub name: String,
    /// Command to execute when the hook is triggered.
    pub command: String,
    /// The trigger event that activates this hook.
    pub trigger: HookTrigger,
}

impl MonitoringHook {
    /// Creates a new monitoring hook with the specified configuration.
    pub fn new(name: impl Into<String>, command: impl Into<String>, trigger: HookTrigger) -> Self {
        Self {
            name: name.into(),
            command: command.into(),
            trigger,
        }
    }
}

/// The event that triggers a monitoring hook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookTrigger {
    /// Triggered when a task starts.
    OnStart,
    /// Triggered when a task completes successfully.
    OnComplete,
    /// Triggered when a task fails.
    OnFailure,
    /// Triggered periodically at specified intervals.
    Periodic { interval_secs: u64 },
}

/// The phase of a task in the workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskPhase {
    /// Task is currently running.
    Running,
    /// Task has completed successfully.
    Completed,
    /// Task has failed.
    Failed,
}

impl std::fmt::Display for TaskPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskPhase::Running => write!(f, "Running"),
            TaskPhase::Completed => write!(f, "Completed"),
            TaskPhase::Failed => write!(f, "Failed"),
        }
    }
}

/// Context available to monitoring hooks during execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookContext {
    /// ID of the task this hook is associated with.
    pub task_id: String,
    /// Working directory for the task.
    pub workdir: std::path::PathBuf,
    /// Current phase of the task.
    pub phase: TaskPhase,
    /// Exit code of the task (if available).
    pub exit_code: Option<i32>,
}

/// Result of executing a monitoring hook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    /// Whether the hook execution was successful.
    pub success: bool,
    /// Output from the hook command.
    pub output: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A mock executor that always succeeds.
    struct MockOkExecutor;

    impl HookExecutor for MockOkExecutor {
        fn execute_hook(
            &self,
            _hook: &MonitoringHook,
            _ctx: &HookContext,
        ) -> Result<HookResult, WorkflowError> {
            Ok(HookResult {
                success: true,
                output: String::new(),
            })
        }
    }

    /// A mock executor that always returns an error.
    struct MockErrExecutor;

    impl HookExecutor for MockErrExecutor {
        fn execute_hook(
            &self,
            _hook: &MonitoringHook,
            _ctx: &HookContext,
        ) -> Result<HookResult, WorkflowError> {
            Err(WorkflowError::Io(std::io::Error::other("mock failure")))
        }
    }

    #[test]
    fn test_monitoring_hook_new() {
        let hook = MonitoringHook {
            name: "test-hook".into(),
            command: "echo test".into(),
            trigger: HookTrigger::OnComplete,
        };
        assert_eq!(hook.name, "test-hook");
    }

    #[test]
    fn test_hook_trigger_variant() {
        assert!(matches!(HookTrigger::OnStart, HookTrigger::OnStart));
    }

    #[test]
    fn test_hook_context() {
        let ctx = HookContext {
            task_id: "task-1".to_string(),
            workdir: std::path::PathBuf::from("."),
            phase: TaskPhase::Running,
            exit_code: None,
        };
        assert_eq!(ctx.task_id, "task-1");
    }

    #[test]
    fn test_hook_result() {
        let result = HookResult {
            success: true,
            output: "output".to_string(),
        };
        assert!(result.success);
    }

    #[test]
    fn mock_ok_executor_returns_success() {
        let hook = MonitoringHook {
            name: String::new(),
            command: "echo hi".into(),
            trigger: HookTrigger::OnStart,
        };
        let ctx = HookContext {
            task_id: "t1".into(),
            phase: TaskPhase::Running,
            workdir: std::path::PathBuf::from("."),
            exit_code: None,
        };
        let result = MockOkExecutor.execute_hook(&hook, &ctx).unwrap();
        assert!(result.success);
    }

    #[test]
    fn mock_err_executor_returns_error() {
        let hook = MonitoringHook {
            name: String::new(),
            command: "echo hi".into(),
            trigger: HookTrigger::OnStart,
        };
        let ctx = HookContext {
            task_id: "t1".into(),
            phase: TaskPhase::Running,
            workdir: std::path::PathBuf::from("."),
            exit_code: None,
        };
        assert!(matches!(
            MockErrExecutor.execute_hook(&hook, &ctx),
            Err(WorkflowError::Io(_))
        ));
    }
}
