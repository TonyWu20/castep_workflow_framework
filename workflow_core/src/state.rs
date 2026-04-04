//! Task state machine and SQLite-backed persistence.

use anyhow::Result;
use tokio_rusqlite::Connection;
use rusqlite::params;
use crate::executor::JobHandle;

/// Lifecycle state of a single task in the pipeline.
#[derive(Debug, Clone, PartialEq)]
pub enum TaskState {
    /// Waiting for dependencies to complete.
    Pending,
    /// All dependencies completed; ready to submit.
    Ready,
    /// Submitted to the executor backend; awaiting confirmation it is running.
    Submitted,
    /// Confirmed running on the backend.
    Running,
    /// Finished with exit code 0.
    Completed,
    /// Finished with a non-zero exit code.
    Failed(i32),
    /// Skipped because all upstream paths failed or were skipped.
    Skipped,
}

impl TaskState {
    fn to_db(&self) -> (String, Option<i32>) {
        match self {
            TaskState::Pending    => ("Pending".into(), None),
            TaskState::Ready      => ("Ready".into(), None),
            TaskState::Submitted  => ("Submitted".into(), None),
            TaskState::Running    => ("Running".into(), None),
            TaskState::Completed  => ("Completed".into(), None),
            TaskState::Failed(c)  => ("Failed".into(), Some(*c)),
            TaskState::Skipped    => ("Skipped".into(), None),
        }
    }

    fn from_db(s: &str, code: Option<i32>) -> Self {
        match s {
            "Ready"     => TaskState::Ready,
            "Submitted" => TaskState::Submitted,
            "Running"   => TaskState::Running,
            "Completed" => TaskState::Completed,
            "Failed"    => TaskState::Failed(code.unwrap_or(-1)),
            "Skipped"   => TaskState::Skipped,
            _           => TaskState::Pending,
        }
    }
}

/// A snapshot of a task's runtime state, including its executor handle.
#[derive(Debug, Clone)]
pub struct TaskRecord {
    /// Unique task ID matching the expanded TOML definition.
    pub id: String,
    /// Current lifecycle state.
    pub state: TaskState,
    /// Executor handle, present once the task has been submitted.
    pub handle: Option<JobHandle>,
}

/// SQLite-backed store for [`TaskRecord`]s.
///
/// Written to `.workflow_state.db` alongside the workflow TOML. On resume,
/// tasks already in [`TaskState::Completed`] are skipped automatically.
pub struct StateDb {
    conn: Connection,
}

impl StateDb {
    /// Open (or create) the state database at `path`, initialising the schema.
    pub async fn open(path: &std::path::Path) -> Result<Self> {
        let conn = Connection::open(path.to_path_buf()).await?;
        conn.call(|c| {
            c.execute_batch(
                "CREATE TABLE IF NOT EXISTS tasks (
                    id TEXT PRIMARY KEY,
                    state TEXT NOT NULL,
                    exit_code INTEGER,
                    handle TEXT
                );"
            )?;
            Ok(())
        }).await?;
        Ok(Self { conn })
    }

    /// Load all persisted task records.
    pub async fn load(&self) -> Result<Vec<TaskRecord>> {
        self.conn.call(|c| {
            let mut stmt = c.prepare(
                "SELECT id, state, exit_code, handle FROM tasks"
            )?;
            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<i32>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            })?;
            rows.map(|r| {
                let (id, state_str, code, handle_raw) = r?;
                Ok(TaskRecord {
                    id,
                    state: TaskState::from_db(&state_str, code),
                    handle: handle_raw.map(|raw| JobHandle { raw }),
                })
            }).collect()
        }).await.map_err(|e| anyhow::anyhow!(e))
    }

    /// Insert or update a task record.
    pub async fn upsert(&self, record: &TaskRecord) -> Result<()> {
        let (state_str, code) = record.state.to_db();
        let id = record.id.clone();
        let handle_raw = record.handle.as_ref().map(|h| h.raw.clone());
        self.conn.call(move |c| {
            c.execute(
                "INSERT INTO tasks (id, state, exit_code, handle)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(id) DO UPDATE SET state=?2, exit_code=?3, handle=?4",
                params![
                    id,
                    state_str,
                    code,
                    handle_raw,
                ],
            )?;
            Ok(())
        }).await.map_err(|e| anyhow::anyhow!(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_db_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<StateDb>();
    }
}
