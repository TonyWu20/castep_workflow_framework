use crate::monitoring::MonitoringHook;

use std::path::PathBuf;
use std::sync::Arc;

pub struct Task {
    pub id: String,
    pub dependencies: Vec<String>,
    pub execute_fn: Arc<dyn Fn() -> anyhow::Result<()> + Send + Sync>,
    pub monitors: Vec<MonitoringHook>,
    pub workdir: PathBuf,
}

impl Task {
    pub fn new<F>(id: impl Into<String>, f: F) -> Self
    where
        F: Fn() -> anyhow::Result<()> + Send + Sync + 'static,
    {
        Self {
            id: id.into(),
            dependencies: Vec::new(),
            execute_fn: Arc::new(f),
            monitors: Vec::new(),
            workdir: PathBuf::from("."),
        }
    }

    pub fn depends_on(mut self, id: impl Into<String>) -> Self {
        self.dependencies.push(id.into());
        self
    }

    pub fn add_monitor(mut self, hook: MonitoringHook) -> Self {
        self.monitors.push(hook);
        self
    }

    pub fn workdir(mut self, path: impl Into<PathBuf>) -> Self {
        self.workdir = path.into();
        self
    }

    pub fn monitors(mut self, hooks: Vec<MonitoringHook>) -> Self {
        self.monitors = hooks;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_builder() {
        let t = Task::new("my_task", || Ok(()));
        assert_eq!(t.id, "my_task");
        assert!(t.dependencies.is_empty());
        assert!(t.monitors.is_empty());
    }

    #[test]
    fn depends_on_chaining() {
        let t = Task::new("t", || Ok(()))
            .depends_on("a")
            .depends_on("b");
        assert_eq!(t.dependencies, vec!["a", "b"]);
    }
}
