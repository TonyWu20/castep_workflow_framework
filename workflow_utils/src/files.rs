use anyhow::{Context, Result};
use std::path::Path;

pub fn read_file(path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();
    std::fs::read_to_string(path).with_context(|| format!("failed to read file: {}", path.display()))
}

pub fn write_file(path: impl AsRef<Path>, content: &str) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create parent directories for: {}", path.display()))?;
    }
    std::fs::write(path, content).with_context(|| format!("failed to write file: {}", path.display()))
}

pub fn copy_file(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create parent directories for: {}", to.display()))?;
    }
    std::fs::copy(from, to)
        .with_context(|| format!("failed to copy file from {} to {}", from.display(), to.display()))?;
    Ok(())
}

pub fn create_dir(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    std::fs::create_dir_all(path).with_context(|| format!("failed to create directory: {}", path.display()))
}

pub fn remove_dir(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    std::fs::remove_dir_all(path).with_context(|| format!("failed to remove directory: {}", path.display()))
}

pub fn exists(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}
