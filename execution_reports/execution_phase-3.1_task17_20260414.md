# Execution Report: TASK-17 Timeout Integration Test

**Plan**: `/Users/tony/programming/castep_workflow_framework/plans/phase-3/PHASE3_TASK12-21.md`
**Task**: TASK-17: Timeout integration test
**Started**: 2026-04-14T09:30:00Z
**Completed**: 2026-04-14T09:30:05Z
**Status**: ✓ Passed

## Task Details

- **Scope**: New file `workflow_core/tests/timeout_integration.rs`
- **Acceptance Criteria**:
  - Task A: `sleep 60`, `timeout: Some(Duration::from_millis(200))`
  - Task B: `true`, depends on A
  - `summary.failed` contains A; error contains "timed out"
  - `summary.skipped` contains B
  - Wall time < 1 second

## Implementation

Created file: `workflow_core/tests/timeout_integration.rs`

```rust
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use workflow_core::HookExecutor;
use workflow_core::ProcessRunner;
use workflow_core::state::JsonStateStore;
use workflow_core::task::{ExecutionMode, Task};
use workflow_core::workflow::Workflow;
use workflow_utils::{ShellHookExecutor, SystemProcessRunner};

#[test]
fn timeout_task_fails_and_dependent_skips() {
    let dir = tempfile::tempdir().unwrap();
    let mut wf = Workflow::new("wf_timeout").with_max_parallel(4).unwrap();

    wf.add_task(Task::new("sleeper", ExecutionMode::Direct {
        command: "sleep".into(),
        args: vec!["60".into()],
        env: HashMap::new(),
        timeout: Some(Duration::from_millis(200)),
    })).unwrap();

    wf.add_task(
        Task::new("dependent", ExecutionMode::Direct {
            command: "true".into(),
            args: vec![],
            env: HashMap::new(),
            timeout: None,
        }).depends_on("sleeper"),
    ).unwrap();

    let mut state = JsonStateStore::new("wf_timeout", dir.path().join(".wf_timeout.workflow.json"));
    let wall_start = Instant::now();
    let summary = wf.run(&mut state, Arc::new(SystemProcessRunner) as Arc<dyn ProcessRunner>,
                         Arc::new(ShellHookExecutor) as Arc<dyn HookExecutor>).unwrap();

    assert!(wall_start.elapsed() < Duration::from_secs(1));
    let (_, err) = summary.failed.iter().find(|(id, _)| id == "sleeper").expect("sleeper should fail");
    assert!(err.contains("timed out"), "error was: {}", err);
    assert!(summary.skipped.contains(&"dependent".to_string()));
}
```

## Validation

### LSP Diagnostics
- After code changes: No errors or warnings in `workflow_core/tests/timeout_integration.rs`

### Compilation Check
```bash
$ cargo check -p workflow_core --test timeout_integration
```
**Result**: ✓ Passed (no errors)

### Test Execution
```bash
$ cargo test -p workflow_core --test timeout_integration timeout_task_fails_and_dependent_skips
```
**Result**: ✓ Passed
- Test: `timeout_task_fails_and_dependent_skips`
- Status: OK
- Duration: ~0.2s (well under 1 second wall time)

## Summary

- **Files created**: `workflow_core/tests/timeout_integration.rs`
- **Tests passed**: 1
- **Validation status**: All acceptance criteria met

The test verifies that:
1. Tasks with timeouts properly fail after the timeout expires
2. The error message contains "timed out"
3. Dependent tasks are marked as skipped (not run)
4. Execution completes quickly (< 1 second) rather than waiting for the full 60-second sleep
