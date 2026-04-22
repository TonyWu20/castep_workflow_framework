use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum WorkflowError {
    #[error("duplicate task id: {0}")]
    DuplicateTaskId(String),

    #[error("dependency cycle detected")]
    CycleDetected,

    #[error("unknown dependency '{dependency}' in task '{task}'")]
    UnknownDependency { task: String, dependency: String },

    #[error("state file corrupted: {0}")]
    StateCorrupted(String),

    #[error("task '{0}' timed out")]
    TaskTimeout(String),

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("I/O error on '{path}': {source}")]
    IoWithPath {
        path: std::path::PathBuf,
        source: std::io::Error,
    },

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("workflow interrupted by signal")]
    Interrupted,

    #[error("failed to submit job to queue: {0}")]
    QueueSubmitFailed(String),
}


impl From<serde_json::Error> for WorkflowError {
    fn from(err: serde_json::Error) -> Self {
        WorkflowError::StateCorrupted(err.to_string())
    }
}

