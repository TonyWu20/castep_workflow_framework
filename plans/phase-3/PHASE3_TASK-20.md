# TASK-20: Implement `retry` subcommand

- **Scope**: Implement `retry` arm with state mutation and tests.
- **Crate/Module**: `workflow-cli/src/main.rs`
- **Depends On**: TASK-18
- **Enables**: TASK-21
- **Can Run In Parallel With**: TASK-19, TASK-17

### Acceptance Criteria

- Each named `task_id` set to `Pending`; warn via `eprintln!` if not found.
- All `SkippedDueToDependencyFailure` tasks reset to `Pending`.
- `TaskStatus::Skipped` (explicit) NOT reset.
- State saved after all mutations.
- At least one test covering happy path and not-found warning.

### Implementation

Add function and replace `Commands::Retry` arm:

```rust
fn cmd_retry(state: &mut JsonStateStore, task_ids: &[String]) {
    for id in task_ids {
        if state.get_status(id).is_none() {
            eprintln!("warn: task '{}' not found", id);
        } else {
            state.mark_pending(id);
        }
    }
    let to_reset: Vec<String> = state.all_tasks()
        .into_iter()
        .filter(|(_, s)| matches!(s, TaskStatus::SkippedDueToDependencyFailure))
        .map(|(id, _)| id)
        .collect();
    for id in to_reset {
        state.mark_pending(&id);
    }
    state.save().unwrap();
}
```

```rust
Commands::Retry { state_file, task_ids } => {
    let mut state = load_state(&state_file)?;
    cmd_retry(&mut state, &task_ids);
    Ok(())
}
```

Add to `#[cfg(test)]` block:

```rust
#[test]
fn retry_resets_failed_and_skipped_dep() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = make_state(dir.path());
    // task_b=Failed, task_c=SkippedDueToDependencyFailure, task_a=Completed
    cmd_retry(&mut s, &["task_b".to_string()]);
    assert_eq!(s.get_status("task_b"), Some(TaskStatus::Pending));
    assert_eq!(s.get_status("task_c"), Some(TaskStatus::Pending));
    assert_eq!(s.get_status("task_a"), Some(TaskStatus::Completed)); // unchanged
}
```

**Verify**: `cargo test -p workflow-cli`
