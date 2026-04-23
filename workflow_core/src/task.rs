use crate::monitoring::MonitoringHook;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// A closure used for task setup or result collection.
pub type TaskClosure = Box<dyn Fn(&Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static>;

#[derive(Debug, Clone)]
pub enum ExecutionMode {
    Direct {
        command: String,
        args: Vec<String>,
        env: HashMap<String, String>,
        timeout: Option<Duration>,
    },
    /// Queued execution via an HPC scheduler (SLURM/PBS).
    /// The actual submit/poll/cancel commands are owned by the `QueuedSubmitter`
    /// implementation set via `Workflow::with_queued_submitter()`.
    Queued,
}

impl ExecutionMode {
    /// Convenience constructor for `Direct` mode with no env vars or timeout.
    ///
    /// # Examples
    /// ```
    /// # use workflow_core::task::ExecutionMode;
    /// let mode = ExecutionMode::direct("castep", &["ZnO"]);
    /// ```
    pub fn direct(command: impl Into<String>, args: &[&str]) -> Self {
        Self::Direct {
            command: command.into(),
            args: args.iter().map(|s| (*s).to_owned()).collect(),
            env: HashMap::new(),
            timeout: None,
        }
    }
}

pub struct Task {
    pub id: String,
    pub dependencies: Vec<String>,
    pub workdir: PathBuf,
    pub mode: ExecutionMode,
    pub setup: Option<TaskClosure>,
    pub collect: Option<TaskClosure>,
    pub monitors: Vec<MonitoringHook>,
}

impl Task {
    pub fn new(id: impl Into<String>, mode: ExecutionMode) -> Self {
        Self {
            id: id.into(),
            dependencies: Vec::new(),
            workdir: PathBuf::from("."),
            mode,
            setup: None,
            collect: None,
            monitors: Vec::new(),
        }
    }

    pub fn depends_on(mut self, id: impl Into<String>) -> Self {
        self.dependencies.push(id.into());
        self
    }

    pub fn workdir(mut self, path: impl Into<PathBuf>) -> Self {
        self.workdir = path.into();
        self
    }

    pub fn setup<F, E>(mut self, f: F) -> Self
    where
        F: Fn(&Path) -> Result<(), E> + Send + Sync + 'static,
        E: std::error::Error + Send + Sync + 'static,
    {
        self.setup = Some(Box::new(move |path| -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            f(path).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        }));
        self
    }

    pub fn collect<F, E>(mut self, f: F) -> Self
    where
        F: Fn(&Path) -> Result<(), E> + Send + Sync + 'static,
        E: std::error::Error + Send + Sync + 'static,
    {
        self.collect = Some(Box::new(move |path| -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            f(path).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        }));
        self
    }

    pub fn monitors(mut self, hooks: Vec<MonitoringHook>) -> Self {
        self.monitors = hooks;
        self
    }

    pub fn add_monitor(mut self, hook: MonitoringHook) -> Self {
        self.monitors.push(hook);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_builder() {
        let t = Task::new("my_task", ExecutionMode::direct("echo", &["test"]));
        assert_eq!(t.id, "my_task");
        assert!(t.dependencies.is_empty());
        assert!(t.monitors.is_empty());
    }

    #[test]
    fn direct_constructor_fields() {
        let mode = ExecutionMode::direct("castep", &["ZnO", "--flag"]);
        match mode {
            ExecutionMode::Direct { command, args, env, timeout } => {
                assert_eq!(command, "castep");
                assert_eq!(args, vec!["ZnO".to_string(), "--flag".to_string()]);
                assert!(env.is_empty());
                assert!(timeout.is_none());
            }
            _ => panic!("expected Direct variant"),
        }
    }

    #[test]
    fn execution_mode_debug() {
        let mode = ExecutionMode::direct("echo", &[]);
        let dbg = format!("{:?}", mode);
        assert!(dbg.contains("Direct"));
    }

    #[test]
    fn depends_on_chaining() {
        let t = Task::new("t", ExecutionMode::direct("true", &[]))
            .depends_on("a")
            .depends_on("b");
        assert_eq!(t.dependencies, vec!["a", "b"]);
    }
}
