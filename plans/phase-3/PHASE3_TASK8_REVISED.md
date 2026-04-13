# TASK-8: Implement `StateStore` trait and `JsonStateStore` (REVISED)

- **Scope**: Define the `StateStore` trait, `StateStoreExt` extension trait, `StateSummary` struct, and refactor `WorkflowState` into `JsonStateStore` implementing `StateStore`. Update all call sites in `workflow.rs` that are broken by the `save()` signature change. Write tests for the trait contract alongside the implementation.
- **Crate/Module**: `workflow_core/src/state.rs`, `workflow_core/src/lib.rs`, `workflow_core/src/workflow.rs` (3 call sites)
- **Responsible For**: Abstracting state persistence behind a trait boundary, providing the JSON implementation, and verifying the contract with tests.
- **Depends On**: TASK-2d
- **Enables**: TASK-11, TASK-12, TASK-18
- **Can Run In Parallel With**: TASK-7, TASK-9

---

## Critical Breaking Change: `save()` Signature

**Before**: `state.save(&self.state_path)` — path passed by caller
**After**: `state.save()` — path owned by `JsonStateStore`

This breaks 3 call sites in `workflow_core/src/workflow.rs`:
- Line 249: `s.save(&self.state_path)?;`
- Line 284: `s.save(&self.state_path)?;`
- Line 367 (inside final state save): Not present in current code, but will be added

**These must be updated in the same commit as the trait definition.**

---

## Acceptance Criteria

### 1. Define `StateStore` Trait in `workflow_core/src/state.rs`

```rust
use std::collections::HashMap;
use crate::error::WorkflowError;

pub trait StateStore: Send + Sync {
    fn get_status(&self, id: &str) -> Option<&TaskStatus>;
    fn set_status(&mut self, id: &str, status: TaskStatus);
    fn all_tasks(&self) -> &HashMap<String, TaskStatus>;
    fn save(&self) -> Result<(), WorkflowError>;
}
```

**Key design decision**: `save()` takes no path argument. The concrete implementation (`JsonStateStore`) owns the path internally.

---

### 2. Define `StateStoreExt` Extension Trait

```rust
pub trait StateStoreExt: StateStore {
    fn mark_running(&mut self, id: &str) {
        self.set_status(id, TaskStatus::Running);
    }
    
    fn mark_completed(&mut self, id: &str) {
        self.set_status(id, TaskStatus::Completed);
    }
    
    fn mark_failed(&mut self, id: &str, error: String) {
        self.set_status(id, TaskStatus::Failed { error });
    }
    
    fn mark_skipped(&mut self, id: &str) {
        self.set_status(id, TaskStatus::Skipped);
    }
    
    fn mark_skipped_due_to_dep_failure(&mut self, id: &str) {
        self.set_status(id, TaskStatus::SkippedDueToDependencyFailure);
    }
    
    fn is_completed(&self, id: &str) -> bool {
        matches!(self.get_status(id), Some(TaskStatus::Completed))
    }
    
    fn summary(&self) -> StateSummary {
        let mut s = StateSummary {
            pending: 0,
            running: 0,
            completed: 0,
            failed: 0,
            skipped: 0,
        };
        
        for status in self.all_tasks().values() {
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
}

// Blanket implementation for all StateStore types
impl<T: StateStore> StateStoreExt for T {}
```

---

### 3. Define `StateSummary` Struct

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateSummary {
    pub pending: usize,
    pub running: usize,
    pub completed: usize,
    pub failed: usize,
    pub skipped: usize,
}
```

---

### 4. Refactor `WorkflowState` → `JsonStateStore`

**Add `path` field**:

```rust
pub struct JsonStateStore {
    workflow_name: String,
    created_at: String,
    last_updated: String,
    tasks: HashMap<String, TaskStatus>,
    path: PathBuf,  // ← NEW: owned path
}
```

**Update constructors**:

```rust
impl JsonStateStore {
    /// Creates a new state store with the given name and path.
    pub fn new(name: &str, path: PathBuf) -> Self {
        Self {
            workflow_name: name.to_string(),
            created_at: now_iso8601(),
            last_updated: now_iso8601(),
            tasks: HashMap::new(),
            path,
        }
    }
    
    /// Loads state from the given path, resetting Running → Pending (crash recovery).
    pub fn load(path: impl AsRef<Path>) -> Result<Self, WorkflowError> {
        let path = path.as_ref();
        let data = std::fs::read(path).map_err(WorkflowError::Io)?;
        let mut state: JsonStateStore = serde_json::from_slice(&data)
            .map_err(|e| WorkflowError::StateCorrupted(e.to_string()))?;
        
        // CRITICAL: Preserve crash recovery logic
        for status in state.tasks.values_mut() {
            if matches!(status, TaskStatus::Running) {
                *status = TaskStatus::Pending;
            }
        }
        
        // Store the path for future saves
        state.path = path.to_path_buf();
        Ok(state)
    }
}
```

**CRITICAL**: The `Running → Pending` reset on load must be preserved. This is crash recovery logic that prevents tasks from being stuck in `Running` state after a workflow crash.

---

### 5. Implement `StateStore` Trait

```rust
impl StateStore for JsonStateStore {
    fn get_status(&self, id: &str) -> Option<&TaskStatus> {
        self.tasks.get(id)
    }
    
    fn set_status(&mut self, id: &str, status: TaskStatus) {
        self.last_updated = now_iso8601();
        self.tasks.insert(id.to_string(), status);
    }
    
    fn all_tasks(&self) -> &HashMap<String, TaskStatus> {
        &self.tasks
    }
    
    fn save(&self) -> Result<(), WorkflowError> {
        // Atomic write: write to temp file, then rename
        let tmp = self.path.with_extension("tmp");
        let json = serde_json::to_vec_pretty(self)
            .map_err(|e| WorkflowError::StateCorrupted(e.to_string()))?;
        std::fs::write(&tmp, json).map_err(WorkflowError::Io)?;
        std::fs::rename(&tmp, &self.path).map_err(WorkflowError::Io)?;
        Ok(())
    }
}
```

**Why atomic write?** If the process crashes during `write()`, the original state file remains intact. `rename()` is atomic on POSIX systems.

---

### 6. Backward Compatibility Alias

```rust
/// Backward compatibility alias for existing code.
pub type WorkflowState = JsonStateStore;
```

This allows existing code using `WorkflowState` to continue compiling without changes.

---

### 7. Update `workflow_core/src/lib.rs` Re-exports

```rust
pub use state::{JsonStateStore, StateSummary, StateStore, StateStoreExt, TaskStatus, WorkflowState};
```

---

### 8. Update Call Sites in `workflow_core/src/workflow.rs`

**Line 249** (inside task completion/failure handler):
```rust
// Before:
s.save(&self.state_path)?;

// After:
s.save()?;
```

**Line 284** (inside skip propagation loop):
```rust
// Before:
s.save(&self.state_path)?;

// After:
s.save()?;
```

**Line 134-138** (state initialization):
```rust
// Before:
let mut state = if self.state_path.exists() {
    WorkflowState::load(&self.state_path)?
} else {
    WorkflowState::new(&self.name)
};

// After:
let mut state = if self.state_path.exists() {
    JsonStateStore::load(&self.state_path)?
} else {
    JsonStateStore::new(&self.name, self.state_path.clone())
};
```

**Line 494** (in `failed_task_skips_dependent` test):
```rust
// Before:
let state = WorkflowState::load(dir.path().join(".wf_skip.workflow.json")).unwrap();

// After:
let state = JsonStateStore::load(dir.path().join(".wf_skip.workflow.json")).unwrap();
```

**Line 594** (in `resume_loads_existing_state` test):
```rust
// Before:
let state = WorkflowState::load(dir.path().join(".wf_resume.workflow.json")).unwrap();

// After:
let state = JsonStateStore::load(dir.path().join(".wf_resume.workflow.json")).unwrap();
```

---

### 9. Update Tests in `workflow_core/src/state.rs`

All existing tests that use `WorkflowState::new()` must be updated to pass a path:

```rust
// Before:
let state = WorkflowState::new("test_workflow");

// After:
let state = JsonStateStore::new("test_workflow", PathBuf::from("/tmp/test.json"));
```

Tests that use `WorkflowState::load()` and `state.save(&path)` must be updated:

```rust
// Before:
state.save(&path)?;
let loaded = WorkflowState::load(&path)?;

// After:
state.save()?;
let loaded = JsonStateStore::load(&path)?;
```

---

### 10. Add New Tests for Trait Contract

Add to `workflow_core/src/state.rs`:

```rust
#[cfg(test)]
mod trait_tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_state_store_get_set_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.json");
        let mut store = JsonStateStore::new("test", path);
        
        store.set_status("task1", TaskStatus::Running);
        assert_eq!(store.get_status("task1"), Some(&TaskStatus::Running));
    }
    
    #[test]
    fn test_state_store_ext_mark_methods() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.json");
        let mut store = JsonStateStore::new("test", path);
        
        store.mark_running("t1");
        assert_eq!(store.get_status("t1"), Some(&TaskStatus::Running));
        
        store.mark_completed("t1");
        assert!(store.is_completed("t1"));
        
        store.mark_failed("t2", "error".into());
        assert!(matches!(store.get_status("t2"), Some(TaskStatus::Failed { .. })));
    }
    
    #[test]
    fn test_summary_counts() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.json");
        let mut store = JsonStateStore::new("test", path);
        
        store.mark_pending("t1");
        store.mark_running("t2");
        store.mark_completed("t3");
        store.mark_failed("t4", "err".into());
        store.mark_skipped("t5");
        
        let summary = store.summary();
        assert_eq!(summary.pending, 1);
        assert_eq!(summary.running, 1);
        assert_eq!(summary.completed, 1);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.skipped, 1);
    }
    
    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.json");
        
        let mut store = JsonStateStore::new("test", path.clone());
        store.mark_completed("task1");
        store.save().unwrap();
        
        let loaded = JsonStateStore::load(&path).unwrap();
        assert!(loaded.is_completed("task1"));
    }
    
    #[test]
    fn test_load_resets_running_to_pending() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.json");
        
        let mut store = JsonStateStore::new("test", path.clone());
        store.mark_running("task1");
        store.save().unwrap();
        
        let loaded = JsonStateStore::load(&path).unwrap();
        assert_eq!(loaded.get_status("task1"), Some(&TaskStatus::Pending));
    }
}
```

**Note**: `mark_pending` is not in `StateStoreExt` — add it:

```rust
fn mark_pending(&mut self, id: &str) {
    self.set_status(id, TaskStatus::Pending);
}
```

---

### 11. Verification Commands

```bash
# Check compilation
cargo check -p workflow_core

# Run state tests
cargo test -p workflow_core -- state

# Run workflow tests (verify call site updates)
cargo test -p workflow_core -- workflow

# Verify all workspace tests pass
cargo test --workspace
```

---

## Notes for Implementer

1. **This is a single atomic commit** — do not land the trait definition without updating the call sites in `workflow.rs`. The codebase will not compile in between.

2. **The `path` field is private** — external users cannot access it directly. This is intentional — the path is an implementation detail of `JsonStateStore`.

3. **The `WorkflowState` type alias preserves backward compatibility** — external crates using `WorkflowState` will continue to work. Internal code in `workflow_core` should migrate to `JsonStateStore` for clarity.

4. **Crash recovery logic must be preserved** — the `Running → Pending` reset in `load()` is critical for resuming workflows after crashes. Do not remove it.

5. **Atomic write pattern** — the temp file + rename pattern prevents state corruption if the process crashes during save. This is a production-critical feature.

6. **`StateStoreExt` is a blanket impl** — any type implementing `StateStore` automatically gets the extension methods. This is the standard Rust pattern for trait extension.

---

## Sequencing with TASK-11

TASK-11 will change `Workflow` to accept `Box<dyn StateStore>` instead of owning a `state_path`. After TASK-8, `workflow.rs` still owns `state_path` and constructs `JsonStateStore` internally. TASK-11 will move that construction to the caller. This is the correct sequencing — TASK-8 makes the trait work, TASK-11 inverts the dependency.
