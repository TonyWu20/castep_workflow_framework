# TASK-15: Add `interrupt` field + failing tests

- **Scope**: Add `pub(crate) interrupt: Arc<AtomicBool>` to `Workflow` and write two failing tests.
- **Crate/Module**: `workflow_core/src/workflow.rs`
- **Depends On**: None
- **Enables**: TASK-16
- **Can Run In Parallel With**: TASK-12, TASK-13, TASK-14

### Acceptance Criteria

- `pub(crate) interrupt: Arc<AtomicBool>` added to `Workflow`, initialised to `false` in `Workflow::new`.
- Two tests in inline `#[cfg(test)]` mod compile but FAIL (interrupt check not yet in `run`).
- `WorkflowError::Interrupted` already exists in `error.rs` — do NOT add it again.
- No new public API surface.

### Implementation

**Step 1** — add to `Workflow` struct and `new()`:

```rust
// Add import at top of workflow.rs:
use std::sync::atomic::{AtomicBool, Ordering};

// Add field to struct:
pub struct Workflow {
    pub name: String,
    tasks: HashMap<String, Task>,
    max_parallel: usize,
    pub(crate) interrupt: Arc<AtomicBool>,  // ← add
}

// Initialise in Workflow::new():
Self {
    name: name.into(),
    tasks: HashMap::new(),
    max_parallel,
    interrupt: Arc::new(AtomicBool::new(false)),  // ← add
}
```

**Step 2** — add two failing tests inside the existing `#[cfg(test)] mod tests`:

```rust
#[test]
fn interrupt_before_run_dispatches_nothing() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir().unwrap();
    let mut wf = Workflow::new("wf_interrupt").with_max_parallel(4)?;
    wf.add_task(Task::new("a", ExecutionMode::Direct {
        command: "true".into(), args: vec![], env: HashMap::new(), timeout: None,
    })).unwrap();
    wf.interrupt.store(true, Ordering::SeqCst);
    let mut state = JsonStateStore::new("wf_interrupt", dir.path().join(".wf_interrupt.workflow.json"));
    let result = wf.run(&mut state, Arc::new(SystemProcessRunner), Arc::new(ShellHookExecutor));
    assert_eq!(result.unwrap_err(), WorkflowError::Interrupted);
    assert!(!matches!(state.get_status("a"), Some(TaskStatus::Completed)));
    Ok(())
}

#[test]
fn interrupt_mid_run_stops_dispatch() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir().unwrap();
    let mut wf = Workflow::new("wf_interrupt2").with_max_parallel(4)?;
    let flag = Arc::new(AtomicBool::new(false));
    let flag_clone = Arc::clone(&flag);
    wf.add_task(
        Task::new("a", ExecutionMode::Direct {
            command: "true".into(), args: vec![], env: HashMap::new(), timeout: None,
        })
        .setup(move |_| { flag_clone.store(true, Ordering::SeqCst); Ok(()) }),
    ).unwrap();
    wf.add_task(
        Task::new("b", ExecutionMode::Direct {
            command: "true".into(), args: vec![], env: HashMap::new(), timeout: None,
        }).depends_on("a"),
    ).unwrap();
    wf.interrupt = Arc::clone(&flag);
    let mut state = JsonStateStore::new("wf_interrupt2", dir.path().join(".wf_interrupt2.workflow.json"));
    let result = wf.run(&mut state, Arc::new(SystemProcessRunner), Arc::new(ShellHookExecutor));
    assert_eq!(result.unwrap_err(), WorkflowError::Interrupted);
    Ok(())
}
```

**Verify**: `cargo test -p workflow_core -- tests::interrupt` (expect 2 failures, not compile errors)
