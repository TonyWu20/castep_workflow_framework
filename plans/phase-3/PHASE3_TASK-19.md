# TASK-19: Implement `status` and `inspect` subcommands

- **Scope**: Implement `status` and `inspect` arms with exact output format, shared helper, and tests.
- **Crate/Module**: `workflow-cli/src/main.rs`
- **Depends On**: TASK-18
- **Enables**: TASK-21
- **Can Run In Parallel With**: TASK-20, TASK-17

### Acceptance Criteria

- `fn load_state(path: &str) -> anyhow::Result<JsonStateStore>` shared helper.
- `status`: one line per task `<id>: <STATUS>` (failed: `<id>: Failed (<error>)`), final line `Summary: X completed, Y failed, Z skipped, W pending`.
- `inspect <state-file> <task-id>`: `task: <id>`, `status: <STATUS>`, `error: <message>` (if failed).
- `inspect <state-file>` (no id): all tasks.
- Missing file: `error: state file not found: <path>` to stderr, exit non-zero.
- Tests assert exact strings via `assert_eq!`.

### Implementation

Add to `workflow-cli/src/main.rs`:

```rust
use workflow_core::state::{JsonStateStore, StateStore, StateStoreExt, TaskStatus};

fn load_state(path: &str) -> anyhow::Result<JsonStateStore> {
    JsonStateStore::load(path)
        .map_err(|_| anyhow::anyhow!("error: state file not found: {}", path))
}

fn cmd_status(state: &JsonStateStore) -> String {
    let mut tasks: Vec<_> = state.all_tasks().into_iter().collect();
    tasks.sort_by(|a, b| a.0.cmp(&b.0));
    let mut out = String::new();
    for (id, status) in &tasks {
        match status {
            TaskStatus::Failed { error } => out.push_str(&format!("{}: Failed ({})\n", id, error)),
            other => out.push_str(&format!("{}: {:?}\n", id, other)),
        }
    }
    let s = state.summary();
    out.push_str(&format!(
        "Summary: {} completed, {} failed, {} skipped, {} pending",
        s.completed, s.failed, s.skipped, s.pending
    ));
    out
}

fn cmd_inspect(state: &JsonStateStore, task_id: Option<&str>) -> anyhow::Result<String> {
    match task_id {
        Some(id) => match state.get_status(id) {
            None => anyhow::bail!("task '{}' not found", id),
            Some(TaskStatus::Failed { error }) =>
                Ok(format!("task: {}\nstatus: Failed\nerror: {}", id, error)),
            Some(s) => Ok(format!("task: {}\nstatus: {:?}", id, s)),
        },
        None => {
            let mut tasks: Vec<_> = state.all_tasks().into_iter().collect();
            tasks.sort_by(|a, b| a.0.cmp(&b.0));
            Ok(tasks.iter()
                .map(|(id, s)| format!("{}: {:?}", id, s))
                .collect::<Vec<_>>().join("\n"))
        }
    }
}
```

Replace `Commands::Status` and `Commands::Inspect` arms:

```rust
Commands::Status { state_file } => {
    let state = load_state(&state_file)?;
    println!("{}", cmd_status(&state));
    Ok(())
}
Commands::Inspect { state_file, task_id } => {
    let state = load_state(&state_file)?;
    match cmd_inspect(&state, task_id.as_deref()) {
        Ok(out) => { println!("{}", out); Ok(()) }
        Err(e) => { eprintln!("{}", e); std::process::exit(1); }
    }
}
```

Add tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use workflow_core::state::StateStoreExt;

    fn make_state(dir: &std::path::Path) -> JsonStateStore {
        let mut s = JsonStateStore::new("test_wf", dir.join("state.json"));
        s.mark_completed("task_a");
        s.mark_failed("task_b", "exit code 1".into());
        s.mark_skipped_due_to_dep_failure("task_c");
        s.save().unwrap();
        s
    }

    #[test]
    fn status_output_format() {
        let dir = tempfile::tempdir().unwrap();
        let s = make_state(dir.path());
        let out = cmd_status(&s);
        assert!(out.contains("task_a: Completed"));
        assert!(out.contains("task_b: Failed (exit code 1)"));
        assert!(out.contains("Summary: 1 completed, 1 failed, 1 skipped, 0 pending"));
    }

    #[test]
    fn inspect_single_task() {
        let dir = tempfile::tempdir().unwrap();
        let s = make_state(dir.path());
        let out = cmd_inspect(&s, Some("task_b")).unwrap();
        assert_eq!(out, "task: task_b\nstatus: Failed\nerror: exit code 1");
    }

    #[test]
    fn inspect_unknown_task_errors() {
        let dir = tempfile::tempdir().unwrap();
        let s = make_state(dir.path());
        assert!(cmd_inspect(&s, Some("nonexistent")).is_err());
    }
}
```

**Verify**: `cargo test -p workflow-cli`
