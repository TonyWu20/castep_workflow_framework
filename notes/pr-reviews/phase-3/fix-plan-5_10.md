# Phase 1 -- Foundational trait changes (no parallelism; each depends on the previous)

These tasks touch `workflow_core/src/state.rs` and must be applied sequentially because each changes the shape of the `StateStore` trait or its impls.

---

## FIX-5: Convert `StateStoreExt` to blanket impl with default methods

- **Task ID**: FIX-5
- **File**: `/Users/tony/programming/castep_workflow_framework/workflow_core/src/state.rs`
- **Target**: `pub trait StateStoreExt` and `impl StateStoreExt for JsonStateStore`

Replace the bodiless trait and its explicit impl with a supertrait that has default method bodies, plus a blanket impl.

- **Before** (the trait definition, lines 45-69):

```rust
/// Extension trait providing convenience methods for state management.
pub trait StateStoreExt {
    /// Marks a task as running and updates the last_updated timestamp.
    fn mark_running(&mut self, id: &str);

    /// Marks a task as completed and updates the last_updated timestamp.
    fn mark_completed(&mut self, id: &str);

    /// Marks a task as failed with the provided error message.
    fn mark_failed(&mut self, id: &str, error: String);

    /// Marks a task as pending and updates the last_updated timestamp.
    fn mark_pending(&mut self, id: &str);

    /// Marks a task as skipped and updates the last_updated timestamp.
    fn mark_skipped(&mut self, id: &str);

    /// Marks a task as skipped due to upstream dependency failure.
    fn mark_skipped_due_to_dep_failure(&mut self, id: &str);

    /// Returns a summary of all task statuses.
    fn summary(&self) -> StateSummary;

    /// Checks if a task is completed.
    fn is_completed(&self, id: &str) -> bool;
}
```

- **After**:

```rust
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

    /// Checks if a task is completed.
    fn is_completed(&self, id: &str) -> bool {
        matches!(self.get_status(id), Some(TaskStatus::Completed))
    }
}

impl<T: StateStore> StateStoreExt for T {}
```

Then **delete** the entire existing `impl StateStoreExt for JsonStateStore { ... }` block. Locate it by its opening line:

```rust
impl StateStoreExt for JsonStateStore {
    fn mark_running(&mut self, id: &str) {
```

Delete from that `impl` line through the matching closing `}`, including all method bodies inside it.

- **Verification**: `cd /Users/tony/programming/castep_workflow_framework && cargo test -p workflow_core --lib state::tests 2>&1 | tail -20`
- **Depends on**: FIX-4c
- **Can run in parallel with**: none

---

## FIX-10: Remove `tasks_mut()` dead code

- **Task ID**: FIX-10
- **File**: `/Users/tony/programming/castep_workflow_framework/workflow_core/src/state.rs`
- **Target**: second `impl JsonStateStore` block containing `tasks_mut`

- **Before**:

```rust
// Internal accessor for workflow module
impl JsonStateStore {
    pub(crate) fn tasks_mut(&mut self) -> &mut HashMap<String, TaskStatus> {
        &mut self.tasks
    }
}
```

- **After**: (delete entirely)
- **Verification**: `cd /Users/tony/programming/castep_workflow_framework && cargo check -p workflow_core 2>&1 | tail -10`
- **Depends on**: FIX-5 (do this after all state.rs trait work is done to avoid merge conflicts)
- **Can run in parallel with**: none
