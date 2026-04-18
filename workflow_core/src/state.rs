use crate::error::WorkflowError;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Task status enum.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed {
        error: String,
    },
    /// Explicitly skipped by the user or workflow logic.
    Skipped,
    /// Skipped because an upstream dependency failed; eligible for retry after fixing upstream.
    SkippedDueToDependencyFailure,
}

/// State management interface for workflow execution.
///
/// This trait defines the contract for persisting and retrieving task status during
/// live workflow runs. Implementations handle runtime mutation of task states as
/// the workflow progresses, ensuring durability through periodic saves.
///
/// Workflow Summary:
/// The `summary` method on `StateStoreExt` aggregates all task statuses into a
/// concise overview (pending, running, completed, failed, skipped counts) suitable
/// for progress reporting and export.
pub trait StateStore: Send + Sync {
    /// Returns the current status of a task.
    fn get_status(&self, id: &str) -> Option<TaskStatus>;

    /// Sets the status of a task and updates timestamp.
    fn set_status(&mut self, id: &str, status: TaskStatus);

    /// Returns all task IDs and their statuses.
    fn all_tasks(&self) -> Vec<(String, TaskStatus)>;

    /// Persists the current state to disk.
    fn save(&self) -> Result<(), WorkflowError>;
}

/// Extension trait providing convenience methods for state management.
pub trait StateStoreExt: StateStore {
    /// Marks a task as running and updates the last_updated timestamp.
    fn mark_running(&mut self, id: &str) {
        self.set_status(id, TaskStatus::Running);
    }

    /// Marks a task as completed and updates the last_updated timestamp.
    fn mark_completed(&mut self, id: &str) {
        self.set_status(id, TaskStatus::Completed);
    }

    /// Marks a task as failed with the provided error message.
    fn mark_failed(&mut self, id: &str, error: String) {
        self.set_status(id, TaskStatus::Failed { error });
    }

    /// Marks a task as pending and updates the last_updated timestamp.
    fn mark_pending(&mut self, id: &str) {
        self.set_status(id, TaskStatus::Pending);
    }

    /// Marks a task as skipped and updates the last_updated timestamp.
    fn mark_skipped(&mut self, id: &str) {
        self.set_status(id, TaskStatus::Skipped);
    }

    /// Marks a task as skipped due to upstream dependency failure.
    fn mark_skipped_due_to_dep_failure(&mut self, id: &str) {
        self.set_status(id, TaskStatus::SkippedDueToDependencyFailure);
    }

    /// Returns a summary of all task statuses.
    fn summary(&self) -> StateSummary {
        let mut s = StateSummary {
            pending: 0,
            running: 0,
            completed: 0,
            failed: 0,
            skipped: 0,
        };
        for (_id, status) in self.all_tasks() {
            match status {
                TaskStatus::Pending => s.pending += 1,
                TaskStatus::Running => s.running += 1,
                TaskStatus::Completed => s.completed += 1,
                TaskStatus::Failed { .. } => s.failed += 1,
                TaskStatus::Skipped | TaskStatus::SkippedDueToDependencyFailure => s.skipped += 1,
            }
        }
        s
    }

    /// Checks if a task is completed.
    fn is_completed(&self, id: &str) -> bool {
        matches!(self.get_status(id), Some(TaskStatus::Completed))
    }
}

impl<T: ?Sized + StateStore> StateStoreExt for T {}

/// Summary of workflow state.
#[derive(Debug, Clone)]
pub struct StateSummary {
    /// Number of pending tasks.
    pub pending: usize,
    /// Number of running tasks.
    pub running: usize,
    /// Number of completed tasks.
    pub completed: usize,
    /// Number of failed tasks.
    pub failed: usize,
    /// Number of skipped tasks.
    pub skipped: usize,
}

/// JSON-based state store implementation.
///
/// # Crash Recovery and Resume
///
/// When loading via [`JsonStateStore::load`], any tasks marked as `Running`, `Failed`, or
/// `SkippedDueToDependencyFailure` are automatically reset to `Pending`. This ensures
/// that incomplete or failed runs can be safely resumed without stale state blocking
/// progress. Note that `Skipped` and `SkippedDueToDependencyFailure` (when not in
/// a failed context) are preserved as-is.
///
/// # Read-Only Inspection
///
/// For read-only status inspection (e.g., CLI display, `workflow inspect` commands),
/// use [`JsonStateStore::load_raw`]. Unlike `load`, this method does not apply crash
/// recovery resets and returns the state exactly as persisted to disk.
///
/// # Persistence Semantics
///
/// State is persisted to disk via atomic writes (temp file + rename). See [`JsonStateStore::load`]
/// and [`JsonStateStore::load_raw`] for details on crash recovery behavior.
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonStateStore {
    workflow_name: String,
    created_at: String,
    last_updated: String,
    tasks: HashMap<String, TaskStatus>,
    path: PathBuf,
}

impl JsonStateStore {
    /// Creates a new empty state store.
    pub fn new(name: &str, path: PathBuf) -> Self {
        let now = now_iso8601();
        Self {
            workflow_name: name.to_owned(),
            created_at: now.clone(),
            last_updated: now,
            tasks: HashMap::new(),
            path,
        }
    }

    /// Saves state atomically using temp file + rename pattern.
    fn persist(&self) -> Result<(), WorkflowError> {
        let temp_path = self.path.with_extension("tmp");
        let json = serde_json::to_vec_pretty(self)
            .map_err(|e| WorkflowError::StateCorrupted(e.to_string()))?;
        fs::write(&temp_path, json).map_err(WorkflowError::Io)?;
        fs::rename(&temp_path, &self.path).map_err(WorkflowError::Io)?;
        Ok(())
    }

    /// Loads state from disk, resetting Running tasks to Pending for crash recovery.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, WorkflowError> {
        let mut state: Self = serde_json::from_slice(&fs::read(path).map_err(WorkflowError::Io)?)?;
        for status in state.tasks.values_mut() {
            if matches!(
                status,
                TaskStatus::Running
                    | TaskStatus::Failed { .. }
                    | TaskStatus::SkippedDueToDependencyFailure
            ) {
                *status = TaskStatus::Pending;
            }
        }
        Ok(state)
    }

    /// Loads state from disk without applying crash-recovery resets.
    /// Use this for read-only inspection (CLI status/inspect).
    pub fn load_raw(path: impl AsRef<Path>) -> Result<Self, WorkflowError> {
        let state: Self = serde_json::from_slice(&fs::read(path).map_err(WorkflowError::Io)?)?;
        Ok(state)
    }

    /// Returns the workflow name.
    pub fn workflow_name(&self) -> &str {
        &self.workflow_name
    }

    /// Returns the path to the state file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl StateStore for JsonStateStore {
    fn get_status(&self, id: &str) -> Option<TaskStatus> {
        self.tasks.get(id).cloned()
    }

    fn set_status(&mut self, id: &str, status: TaskStatus) {
        self.tasks.insert(id.to_owned(), status);
        self.last_updated = now_iso8601();
    }

    fn all_tasks(&self) -> Vec<(String, TaskStatus)> {
        self.tasks.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    fn save(&self) -> Result<(), WorkflowError> {
        self.persist()
    }
}

fn now_iso8601() -> String {
    use time::format_description::well_known::Rfc3339;
    time::OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn round_trip_json() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("state.json");
        let mut s = JsonStateStore::new("test", path.clone());
        s.mark_completed("a");
        s.save().unwrap();
        let loaded = JsonStateStore::load(&path).unwrap();
        assert!(matches!(
            loaded.get_status("a"),
            Some(TaskStatus::Completed)
        ));
        assert_eq!(loaded.workflow_name(), "test");
    }

    #[test]
    fn status_transitions() {
        let mut s = JsonStateStore::new("w", PathBuf::from("/tmp"));
        s.mark_running("a");
        assert!(matches!(s.get_status("a"), Some(TaskStatus::Running)));
        s.mark_completed("a");
        assert!(matches!(s.get_status("a"), Some(TaskStatus::Completed)));
    }

    #[test]
    fn load_resets_running_to_pending() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("state.json");
        let mut s = JsonStateStore::new("test", path.clone());
        s.mark_running("task1");
        s.mark_completed("task2");
        // Add these three lines
        s.mark_failed("task3", "boom".into());
        s.mark_skipped_due_to_dep_failure("task4");
        s.mark_skipped("task5"); // must NOT reset
        s.save().unwrap();

        let loaded = JsonStateStore::load(&path).unwrap();
        assert!(matches!(
            loaded.get_status("task1"),
            Some(TaskStatus::Pending)
        ));
        assert!(matches!(
            loaded.get_status("task2"),
            Some(TaskStatus::Completed)
        ));
        // Add these three lines
        assert!(matches!(
            loaded.get_status("task3"),
            Some(TaskStatus::Pending)
        ));
        assert!(matches!(
            loaded.get_status("task4"),
            Some(TaskStatus::Pending)
        ));
        assert!(matches!(
            loaded.get_status("task5"),
            Some(TaskStatus::Skipped)
        )); // must NOT reset
    }

    #[test]
    fn load_corrupted_json_errors() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("corrupted.json");
        fs::write(&path, b"not json at all").unwrap();

        let result = JsonStateStore::load(&path);
        assert!(matches!(
            result.unwrap_err(),
            WorkflowError::StateCorrupted(_)
        ));
    }

    #[test]
    fn load_missing_errors() {
        let result = JsonStateStore::load("/nonexistent/path.json");
        assert!(matches!(result.unwrap_err(), WorkflowError::Io(_)));
    }

    #[test]
    fn summary_counts() {
        let mut s = JsonStateStore::new("test", PathBuf::from("/tmp"));
        s.mark_pending("a");
        s.mark_running("b");
        s.mark_completed("c");
        s.mark_failed("d", "error".to_string());
        s.mark_skipped("e");

        let summary = s.summary();
        assert_eq!(summary.pending, 1);
        assert_eq!(summary.running, 1);
        assert_eq!(summary.completed, 1);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.skipped, 1); // Only mark_skipped was called
    }

    #[test]
    fn all_tasks() {
        let mut s = JsonStateStore::new("test", PathBuf::from("/tmp"));
        s.mark_completed("a");
        s.mark_running("b");
        assert_eq!(s.all_tasks().len(), 2);
    }


    #[test]
    fn save_load_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("roundtrip.json");
        let mut s1 = JsonStateStore::new("roundtrip", path.clone());
        s1.mark_pending("t1"); // Use Pending instead of Running to avoid crash recovery reset
        s1.mark_completed("t2");
        s1.mark_pending("t3"); // Use Pending instead of Failed to avoid crash recovery reset
        s1.save().unwrap();

        let s2 = JsonStateStore::load(&path).unwrap();
        assert_eq!(s1.get_status("t1"), s2.get_status("t1"));
        assert_eq!(s1.get_status("t2"), s2.get_status("t2"));
        assert_eq!(s1.get_status("t3"), s2.get_status("t3"));
    }

    #[test]
    fn atomic_save() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("atomic.json");
        let mut s = JsonStateStore::new("atomic", path.clone());
        s.mark_completed("test");
        // Force actual file write by checking temp path exists
        let _temp = std::fs::File::create(path.with_extension("tmp")).unwrap();
        s.save().expect("save should succeed on second attempt");
        let loaded = JsonStateStore::load(&path).unwrap();
        assert!(matches!(
            loaded.get_status("test"),
            Some(TaskStatus::Completed)
        ));
    }

    #[test]
    fn workflow_name_preserved() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("name.json");
        let s = JsonStateStore::new("my_workflow", path.clone());
        s.save().unwrap();
        let loaded = JsonStateStore::load(&path).unwrap();
        assert_eq!(loaded.workflow_name(), "my_workflow");
    }
}
