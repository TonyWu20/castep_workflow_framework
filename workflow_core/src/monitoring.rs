use serde::{Deserialize, Serialize};

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

/// Context available to monitoring hooks during execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookContext {
    /// ID of the task this hook is associated with.
    pub task_id: String,
    /// Working directory for the task.
    pub workdir: std::path::PathBuf,
    /// Current state of the task (running, completed, failed, etc.).
    pub state: String,
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

    #[test]
    fn test_monitoring_hook_new() {
        let hook = MonitoringHook::new("test-hook", "echo test", HookTrigger::OnComplete);
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
            state: "running".to_string(),
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
}
