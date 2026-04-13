# TASK-12: Extend `JsonStateStore::load` reset logic

- **Scope**: Widen the `matches!` guard in `JsonStateStore::load` to reset `Failed { .. }` and `SkippedDueToDependencyFailure` to `Pending`.
- **Crate/Module**: `workflow_core/src/state.rs`
- **Depends On**: None
- **Enables**: TASK-21
- **Can Run In Parallel With**: TASK-13, TASK-14, TASK-15

### Acceptance Criteria

- `Failed { .. }` and `SkippedDueToDependencyFailure` reset to `Pending` on load.
- `Skipped` (explicit) and `Completed` are NOT reset.
- Existing test `load_resets_running_to_pending` extended in-place.
- `cargo test -p workflow_core` passes.

### Implementation

**File**: `workflow_core/src/state.rs` — function `JsonStateStore::load`

```rust
// Before:
for status in state.tasks.values_mut() {
    if matches!(status, TaskStatus::Running) {
        *status = TaskStatus::Pending;
    }
}

// After:
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
```

**Extend** the existing test `load_resets_running_to_pending` — add before `s.save()`:

```rust
s.mark_failed("task3", "boom".into());
s.mark_skipped_due_to_dep_failure("task4");
s.mark_skipped("task5");
```

Add after load assertions:

```rust
assert_eq!(loaded.get_status("task3"), Some(TaskStatus::Pending));
assert_eq!(loaded.get_status("task4"), Some(TaskStatus::Pending));
assert_eq!(loaded.get_status("task5"), Some(TaskStatus::Skipped)); // must NOT reset
```

**Verify**: `cargo test -p workflow_core -- state::tests::load_resets_running_to_pending`
