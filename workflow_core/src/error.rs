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

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("workflow interrupted by signal")]
    Interrupted,
}

impl PartialEq for WorkflowError {
    // Note: Io variants compared by ErrorKind only (lossy, sufficient for test assertions).
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::DuplicateTaskId(a), Self::DuplicateTaskId(b)) => a == b,
            (Self::CycleDetected, Self::CycleDetected) => true,
            (Self::UnknownDependency { task: a_task, dependency: a_dep },
             Self::UnknownDependency { task: b_task, dependency: b_dep }) => {
                a_task == b_task && a_dep == b_dep
            }
            (Self::StateCorrupted(a), Self::StateCorrupted(b)) => a == b,
            (Self::TaskTimeout(a), Self::TaskTimeout(b)) => a == b,
            (Self::InvalidConfig(a), Self::InvalidConfig(b)) => a == b,
            (Self::Io(ref e_a), Self::Io(ref e_b)) => {
                // Compare by ErrorKind only (lossy, sufficient for test assertions)
                e_a.kind() == e_b.kind()
            }
            (Self::Interrupted, Self::Interrupted) => true,
            _ => false,
        }
    }
}

impl From<serde_json::Error> for WorkflowError {
    fn from(err: serde_json::Error) -> Self {
        WorkflowError::StateCorrupted(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_error_partial_eq() {
        // Test structural equality for non-Io variants
        let err1 = WorkflowError::DuplicateTaskId("task1".to_string());
        let err2 = WorkflowError::DuplicateTaskId("task1".to_string());
        let err3 = WorkflowError::DuplicateTaskId("task2".to_string());

        assert_eq!(err1, err2);
        assert_ne!(err1, err3);

        // Test Io variant comparison by ErrorKind only
        let io_err1 = std::io::Error::new(std::io::ErrorKind::Other, "test");
        let io_err2 = std::io::Error::new(std::io::ErrorKind::Other, "different");
        let io_err3 = std::io::Error::new(std::io::ErrorKind::Other, "other");

        let io1 = WorkflowError::Io(io_err1);
        let io2 = WorkflowError::Io(io_err2);
        let io3 = WorkflowError::Io(io_err3);

        // Same ErrorKind should be equal
        assert_eq!(io1, io2);

        // Different error content with same ErrorKind should be equal
        assert_eq!(io1, io3);

        // Test InvalidConfig variant
        let invalid1 = WorkflowError::InvalidConfig("config1".to_string());
        let invalid2 = WorkflowError::InvalidConfig("config1".to_string());
        let invalid3 = WorkflowError::InvalidConfig("config2".to_string());

        assert_eq!(invalid1, invalid2);
        assert_ne!(invalid1, invalid3);

        // Test different variants are not equal
        let cycle_err = WorkflowError::CycleDetected;
        assert_ne!(cycle_err, io1);
    }
}
