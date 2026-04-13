# Phase 1 -- Foundational trait changes (no parallelism; each depends on the previous)

These tasks touch `workflow_core/src/state.rs` and must be applied sequentially because each changes the shape of the `StateStore` trait or its impls.

---

## FIX-4a: Add `Send + Sync` to `StateStore`, return references, remove `load()` from trait

- **Task ID**: FIX-4a
- **File**: `/Users/tony/programming/castep_workflow_framework/workflow_core/src/state.rs`
- **Target**: `pub trait StateStore`
- **Before**:

```rust
pub trait StateStore {
    /// Returns the current status of a task.
    fn get_status(&self, id: &str) -> Option<TaskStatus>;

    /// Sets the status of a task and updates timestamp.
    fn set_status(&mut self, id: &str, status: TaskStatus);

    /// Returns all task IDs and their statuses.
    fn all_tasks(&self) -> HashMap<String, TaskStatus>;

    /// Persists the current state to disk.
    /// The path is owned by JsonStateStore and not passed explicitly.
    fn save(&self) -> Result<(), WorkflowError>;

    /// Loads state from disk. Running tasks are reset to Pending for crash recovery.
    fn load(path: impl AsRef<Path>) -> Result<JsonStateStore, WorkflowError>
    where
        Self: Sized;
}
```

- **After**:

```rust
pub trait StateStore: Send + Sync {
    /// Returns the current status of a task.
    fn get_status(&self, id: &str) -> Option<&TaskStatus>;

    /// Sets the status of a task and updates timestamp.
    fn set_status(&mut self, id: &str, status: TaskStatus);

    /// Returns all task IDs and their statuses.
    fn all_tasks(&self) -> &HashMap<String, TaskStatus>;

    /// Persists the current state to disk.
    /// The path is owned by JsonStateStore and not passed explicitly.
    fn save(&self) -> Result<(), WorkflowError>;
}
```

- **Verification**: `cd /Users/tony/programming/castep_workflow_framework && cargo check -p workflow_core 2>&1 | head -40` (expect errors from callers -- those are fixed in FIX-4b)
- **Depends on**: none
- **Can run in parallel with**: none (all FIX-4/5 tasks are sequential)

---

## FIX-4b: Update `impl StateStore for JsonStateStore` to match new signatures

- **Task ID**: FIX-4b
- **File**: `/Users/tony/programming/castep_workflow_framework/workflow_core/src/state.rs`
- **Target**: `impl StateStore for JsonStateStore`
- **Before**:

```rust
impl StateStore for JsonStateStore {
    fn get_status(&self, id: &str) -> Option<TaskStatus> {
        self.tasks.get(id).cloned()
    }

    fn set_status(&mut self, id: &str, status: TaskStatus) {
        self.tasks.insert(id.to_owned(), status);
        self.last_updated = now_iso8601();
    }

    fn all_tasks(&self) -> HashMap<String, TaskStatus> {
        self.tasks.clone()
    }

    fn save(&self) -> Result<(), WorkflowError> {
        self.save()
    }

    fn load(path: impl AsRef<Path>) -> Result<JsonStateStore, WorkflowError> {
        Self::load(path)
    }
}
```

- **After**:

```rust
impl StateStore for JsonStateStore {
    fn get_status(&self, id: &str) -> Option<&TaskStatus> {
        self.tasks.get(id)
    }

    fn set_status(&mut self, id: &str, status: TaskStatus) {
        self.tasks.insert(id.to_owned(), status);
        self.last_updated = now_iso8601();
    }

    fn all_tasks(&self) -> &HashMap<String, TaskStatus> {
        &self.tasks
    }

    fn save(&self) -> Result<(), WorkflowError> {
        self.save()
    }
}
```

- **Verification**: `cd /Users/tony/programming/castep_workflow_framework && cargo check -p workflow_core 2>&1 | head -40` (expect errors from callers in `state.rs` tests -- those are fixed next)
- **Depends on**: FIX-4a
- **Can run in parallel with**: none

---

## FIX-4c: Fix `state.rs` test assertions for reference returns

- **Task ID**: FIX-4c
- **File**: `/Users/tony/programming/castep_workflow_framework/workflow_core/src/state.rs`
- **Target**: `mod tests` (multiple test functions)

After FIX-4a/4b, `get_status()` returns `Option<&TaskStatus>` instead of `Option<TaskStatus>`. Callers using `==` comparisons need updating.

- **Before** (in `round_trip_json`):

```rust
        assert!(loaded.get_status("a") == Some(TaskStatus::Completed));
```

- **After**:

```rust
        assert!(matches!(loaded.get_status("a"), Some(TaskStatus::Completed)));
```

- **Before** (in `status_transitions`):

```rust
        assert!(s.get_status("a") == Some(TaskStatus::Running));
        s.mark_completed("a");
        assert!(s.get_status("a") == Some(TaskStatus::Completed));
```

- **After**:

```rust
        assert!(matches!(s.get_status("a"), Some(TaskStatus::Running)));
        s.mark_completed("a");
        assert!(matches!(s.get_status("a"), Some(TaskStatus::Completed)));
```

- **Before** (in `load_resets_running_to_pending`):

```rust
        assert_eq!(loaded.get_status("task1"), Some(TaskStatus::Pending));
        assert_eq!(loaded.get_status("task2"), Some(TaskStatus::Completed));
        // Add these three lines
        assert_eq!(loaded.get_status("task3"), Some(TaskStatus::Pending));
        assert_eq!(loaded.get_status("task4"), Some(TaskStatus::Pending));
        assert_eq!(loaded.get_status("task5"), Some(TaskStatus::Skipped)); // must NOT reset
```

- **After**:

```rust
        assert!(matches!(loaded.get_status("task1"), Some(TaskStatus::Pending)));
        assert!(matches!(loaded.get_status("task2"), Some(TaskStatus::Completed)));
        assert!(matches!(loaded.get_status("task3"), Some(TaskStatus::Pending)));
        assert!(matches!(loaded.get_status("task4"), Some(TaskStatus::Pending)));
        assert!(matches!(loaded.get_status("task5"), Some(TaskStatus::Skipped)));
```

- **Before** (in `atomic_save`):

```rust
        assert!(loaded.get_status("test") == Some(TaskStatus::Completed));
```

- **After**:

```rust
        assert!(matches!(loaded.get_status("test"), Some(TaskStatus::Completed)));
```

- **Before** (in `save_load_roundtrip`):

```rust
        assert_eq!(s1.get_status("t1"), s2.get_status("t1"));
        assert_eq!(s1.get_status("t2"), s2.get_status("t2"));
        assert_eq!(s1.get_status("t3"), s2.get_status("t3"));
```

- **After**:

```rust
        assert_eq!(s1.get_status("t1").cloned(), s2.get_status("t1").cloned());
        assert_eq!(s1.get_status("t2").cloned(), s2.get_status("t2").cloned());
        assert_eq!(s1.get_status("t3").cloned(), s2.get_status("t3").cloned());
```

- **Verification**: `cd /Users/tony/programming/castep_workflow_framework && cargo test -p workflow_core --lib state::tests 2>&1 | tail -20`
- **Depends on**: FIX-4b
- **Can run in parallel with**: none

---
