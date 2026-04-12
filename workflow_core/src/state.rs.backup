use std::collections::HashMap;
use std::path::Path;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed { error: String },
    /// Explicitly skipped by the user or workflow logic.
    Skipped,
    /// Skipped because an upstream dependency failed; eligible for retry after fixing upstream.
    SkippedDueToDependencyFailure,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowState {
    pub workflow_name: String,
    pub created_at: String,
    pub last_updated: String,
    pub tasks: HashMap<String, TaskStatus>,
}

impl WorkflowState {
    pub fn new(name: &str) -> Self {
        let now = now_iso8601();
        Self { workflow_name: name.to_owned(), created_at: now.clone(), last_updated: now, tasks: HashMap::new() }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let mut state: Self = serde_json::from_slice(&std::fs::read(path)?)?;
        for status in state.tasks.values_mut() {
            if matches!(status, TaskStatus::Running) {
                *status = TaskStatus::Pending;
            }
        }
        Ok(state)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        std::fs::write(path, serde_json::to_vec_pretty(self)?)?;
        Ok(())
    }

    pub fn is_completed(&self, id: &str) -> bool {
        matches!(self.tasks.get(id), Some(TaskStatus::Completed))
    }

    pub fn mark_running(&mut self, id: &str) {
        self.tasks.insert(id.to_owned(), TaskStatus::Running);
        self.last_updated = now_iso8601();
    }

    pub fn mark_completed(&mut self, id: &str) {
        self.tasks.insert(id.to_owned(), TaskStatus::Completed);
        self.last_updated = now_iso8601();
    }

    pub fn mark_failed(&mut self, id: &str, error: String) {
        self.tasks.insert(id.to_owned(), TaskStatus::Failed { error });
        self.last_updated = now_iso8601();
    }

    pub fn mark_skipped(&mut self, id: &str) {
        self.tasks.insert(id.to_owned(), TaskStatus::Skipped);
        self.last_updated = now_iso8601();
    }

    pub fn mark_skipped_due_to_dep_failure(&mut self, id: &str) {
        self.tasks.insert(id.to_owned(), TaskStatus::SkippedDueToDependencyFailure);
        self.last_updated = now_iso8601();
    }
}

fn now_iso8601() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400;
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn round_trip_json() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("state.json");
        let mut s = WorkflowState::new("test");
        s.mark_completed("a");
        s.save(&path).unwrap();
        let loaded = WorkflowState::load(&path).unwrap();
        assert!(loaded.is_completed("a"));
        assert_eq!(loaded.workflow_name, "test");
    }

    #[test]
    fn load_missing_errors() {
        assert!(WorkflowState::load("/nonexistent/path.json").is_err());
    }

    #[test]
    fn status_transitions() {
        let mut s = WorkflowState::new("w");
        s.mark_running("a");
        assert!(!s.is_completed("a"));
        s.mark_completed("a");
        assert!(s.is_completed("a"));
    }

    #[test]
    fn load_resets_running_to_pending() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("state.json");
        let mut s = WorkflowState::new("test");
        s.mark_running("task1");
        s.mark_completed("task2");
        s.save(&path).unwrap();

        let loaded = WorkflowState::load(&path).unwrap();
        assert_eq!(loaded.tasks.get("task1"), Some(&TaskStatus::Pending));
        assert_eq!(loaded.tasks.get("task2"), Some(&TaskStatus::Completed));
    }
}
