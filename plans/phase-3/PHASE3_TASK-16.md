# TASK-16: Implement interrupt check in `Workflow::run`

- **Scope**: Insert `AtomicBool` check at top of run loop; return `Err(WorkflowError::Interrupted)` on signal.
- **Crate/Module**: `workflow_core/src/workflow.rs`
- **Depends On**: TASK-15
- **Enables**: TASK-16b

### Acceptance Criteria

- `if self.interrupt.load(Ordering::SeqCst) { ... }` at top of poll loop.
- Both TASK-15 tests now pass.
- In-flight tasks marked `Pending` (NOT `Failed`) on interrupt.
- `state.save()` called before returning `Err`.
- All pre-existing tests continue to pass.

### Implementation

At the very top of the `loop` body in `run()`, before the "Poll finished tasks" comment:

```rust
loop {
    // Interrupt check — must be first
    if self.interrupt.load(Ordering::SeqCst) {
        for id in handles.keys() {
            state.set_status(id, TaskStatus::Pending);
        }
        for (_, (handle, _)) in handles.iter_mut() {
            handle.terminate().ok();
        }
        state.save()?;
        return Err(WorkflowError::Interrupted);
    }

    // Poll finished tasks  ← existing code continues here
```

**Verify**: `cargo test -p workflow_core -- tests::interrupt` (both tests now pass)
