use crate::monitoring::MonitoringHook;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// A closure used for task setup or result collection.
pub type TaskClosure = Box<dyn Fn(&Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static>;

#[derive(Clone)]
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
    use std::collections::HashMap;

    #[test]
    fn task_builder() {
        let t = Task::new(
            "my_task",
            ExecutionMode::Direct {
                command: "echo".into(),
                args: vec!["test".into()],
                env: HashMap::new(),
                timeout: None,
            },
        );
        assert_eq!(t.id, "my_task");
        assert!(t.dependencies.is_empty());
        assert!(t.monitors.is_empty());
    }

    #[test]
    fn depends_on_chaining() {
        let t = Task::new(
            "t",
            ExecutionMode::Direct {
                command: "true".into(),
                args: vec![],
                env: HashMap::new(),
                timeout: None,
            },
        )
        .depends_on("a")
        .depends_on("b");
        assert_eq!(t.dependencies, vec!["a", "b"]);
    }
}
