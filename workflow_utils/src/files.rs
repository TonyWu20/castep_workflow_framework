use std::path::Path;
use workflow_core::WorkflowError;

fn io_err(path: &Path, source: std::io::Error) -> WorkflowError {
    WorkflowError::IoWithPath { path: path.to_path_buf(), source }
}

pub fn read_file(path: impl AsRef<Path>) -> Result<String, WorkflowError> {
    let path = path.as_ref();
    std::fs::read_to_string(path).map_err(|e| io_err(path, e))
}

pub fn write_file(path: impl AsRef<Path>, content: &str) -> Result<(), WorkflowError> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| io_err(parent, e))?;
    }
    std::fs::write(path, content).map_err(|e| io_err(path, e))
}

pub fn copy_file(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<(), WorkflowError> {
    let from = from.as_ref();
    let to = to.as_ref();
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent).map_err(|e| io_err(parent, e))?;
    }
    std::fs::copy(from, to).map_err(|e| io_err(from, e))?;
    Ok(())
}

pub fn create_dir(path: impl AsRef<Path>) -> Result<(), WorkflowError> {
    let path = path.as_ref();
    std::fs::create_dir_all(path).map_err(|e| io_err(path, e))
}

pub fn remove_dir(path: impl AsRef<Path>) -> Result<(), WorkflowError> {
    let path = path.as_ref();
    std::fs::remove_dir_all(path).map_err(|e| io_err(path, e))
}

pub fn exists(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}
