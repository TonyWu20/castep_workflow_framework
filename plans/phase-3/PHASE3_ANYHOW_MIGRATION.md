# Plan: Replace `anyhow` in `workflow_utils/src/files.rs` with `WorkflowError`

## Context

TASK-1 in the v7 fix plan incorrectly assumed `anyhow` was unused in `workflow_utils`. In fact, `workflow_utils/src/files.rs` actively uses `anyhow::{Context, Result}` for all file I/O helpers. Simply removing the dependency would break the build.

The correct fix is to migrate `files.rs` to use `WorkflowError` (from `workflow_core`), preserving path context by adding a new `IoWithPath` variant.

## Files to modify

1. `workflow_core/src/error.rs` — add `IoWithPath` variant
2. `workflow_utils/src/files.rs` — replace anyhow with WorkflowError
3. `workflow_utils/Cargo.toml` — remove `anyhow` dependency

---

## TASK-1: Add `IoWithPath` variant to `WorkflowError`

**File:** `workflow_core/src/error.rs`

Insert before the existing `Io` variant:

```rust
    #[error("I/O error on '{path}': {source}")]
    IoWithPath {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
```

The existing `Io(#[from] std::io::Error)` variant stays — other code may use it via `?`.

---

## TASK-2: Rewrite `workflow_utils/src/files.rs`

**File:** `workflow_utils/src/files.rs`

Full replacement:

```rust
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
```

Notes:

- `io_err` is a private helper — not exported, not an abstraction, just deduplication.
- `copy_file` reports the `from` path on error (that's where the OS error originates).

---

## TASK-3: Remove `anyhow` from `workflow_utils/Cargo.toml`

**File:** `workflow_utils/Cargo.toml`

Remove:

```toml
anyhow = { workspace = true }
```

---

## Execution order

TASK-1 must complete before TASK-2 (files.rs references the new variant). TASK-3 is independent but logically last.

Sequential: TASK-1 → TASK-2 → TASK-3

---

## Verification

```
cargo check --workspace && cargo clippy --workspace
```
