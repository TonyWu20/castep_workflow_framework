# TASK-7: Implement `SystemProcessRunner` in `workflow_utils` (REVISED)

- **Scope**: Implement the `ProcessRunner` trait as `SystemProcessRunner` and `ProcessHandle` as `SystemProcessHandle` in `workflow_utils/src/executor.rs`. Write integration tests for the process runner.
- **Crate/Module**: `workflow_utils/src/executor.rs`, `workflow_utils/src/lib.rs`, `workflow_utils/tests/process_tests.rs` (NEW)
- **Responsible For**: Providing the concrete process execution implementation that wraps `std::process::Child`, and verifying its behavior with integration tests.
- **Depends On**: TASK-4, TASK-6
- **Enables**: TASK-11
- **Can Run In Parallel With**: TASK-5, TASK-8, TASK-9

---

## Key Design Decision: `Option<Child>` Pattern

**Problem**: `Child::wait_with_output()` moves `self.child`, but `ProcessHandle::wait(&mut self)` cannot consume the handle. This is a fundamental ownership conflict.

**Solution**: Use `Option<Child>` to allow taking ownership once:

```rust
struct SystemProcessHandle {
    child: Option<std::process::Child>,
    start: std::time::Instant,
}
```

This is the standard Rust idiom for "consume once" resources behind `&mut self`.

---

## Acceptance Criteria

### 1. `SystemProcessRunner` Implementation

Create `workflow_utils/src/executor.rs` (or add to existing file):

```rust
use workflow_core::{ProcessRunner, ProcessHandle, ProcessResult, WorkflowError};
use std::collections::HashMap;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::Instant;

pub struct SystemProcessRunner;

impl ProcessRunner for SystemProcessRunner {
    fn spawn(
        &self,
        workdir: &Path,
        command: &str,
        args: &[String],
        env: &HashMap<String, String>,
    ) -> Result<Box<dyn ProcessHandle>, WorkflowError> {
        let child = Command::new(command)
            .args(args)
            .envs(env)
            .current_dir(workdir)
            .stdout(Stdio::piped())  // ← CRITICAL: pipe stdout/stderr for capture
            .stderr(Stdio::piped())
            .spawn()
            .map_err(WorkflowError::Io)?;
        
        Ok(Box::new(SystemProcessHandle {
            child: Some(child),
            start: Instant::now(),
        }))
    }
}
```

**Why `Stdio::piped()`?** `ProcessResult` has `stdout`/`stderr` fields. Without piping, `wait_with_output()` returns empty strings. This is the production adapter — capturing output is the right default.

---

### 2. `SystemProcessHandle` Implementation

```rust
struct SystemProcessHandle {
    child: Option<Child>,
    start: Instant,
}

impl ProcessHandle for SystemProcessHandle {
    fn is_running(&mut self) -> bool {
        match &mut self.child {
            Some(child) => matches!(child.try_wait(), Ok(None)),
            None => false,  // Already waited
        }
    }

    fn terminate(&mut self) -> Result<(), WorkflowError> {
        match &mut self.child {
            Some(child) => child.kill().map_err(WorkflowError::Io),
            None => Ok(()),  // Idempotent: already terminated/waited
        }
    }

    fn wait(&mut self) -> Result<ProcessResult, WorkflowError> {
        let child = self.child.take()
            .ok_or_else(|| WorkflowError::InvalidConfig("wait() called twice".into()))?;
        
        let output = child.wait_with_output().map_err(WorkflowError::Io)?;
        
        Ok(ProcessResult {
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            duration: self.start.elapsed(),
        })
    }
}
```

**Key points**:
- `wait()` uses `self.child.take()` to move ownership out of the `Option`
- Second call to `wait()` returns `InvalidConfig` error (explicit failure)
- `is_running()` and `terminate()` handle `None` gracefully (return `false`/`Ok(())`)

---

### 3. Re-export from `workflow_utils/src/lib.rs`

```rust
pub use executor::SystemProcessRunner;
```

---

### 4. Integration Tests

Create `workflow_utils/tests/process_tests.rs`:

```rust
use workflow_utils::SystemProcessRunner;
use workflow_core::{ProcessRunner, ProcessHandle};
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn test_system_process_runner_echo() {
    let runner = SystemProcessRunner;
    let mut handle = runner.spawn(
        &PathBuf::from("/tmp"),
        "echo",
        &["hello".to_string()],
        &HashMap::new(),
    ).unwrap();
    
    let result = handle.wait().unwrap();
    assert_eq!(result.exit_code, Some(0));
    assert!(result.stdout.contains("hello"));
}

#[test]
fn test_is_running_transitions() {
    let runner = SystemProcessRunner;
    let mut handle = runner.spawn(
        &PathBuf::from("/tmp"),
        "sleep",
        &["0.1".to_string()],
        &HashMap::new(),
    ).unwrap();
    
    // Immediately after spawn, should be running
    assert!(handle.is_running());
    
    // After wait, should not be running
    handle.wait().unwrap();
    assert!(!handle.is_running());
}

#[test]
fn test_terminate_long_running_process() {
    let runner = SystemProcessRunner;
    let mut handle = runner.spawn(
        &PathBuf::from("/tmp"),
        "sleep",
        &["60".to_string()],
        &HashMap::new(),
    ).unwrap();
    
    assert!(handle.is_running());
    handle.terminate().unwrap();
    
    // After terminate, wait should succeed (process is dead)
    let result = handle.wait().unwrap();
    assert!(result.exit_code.is_some());  // Killed processes have exit codes
}

#[test]
fn test_wait_called_twice_errors() {
    let runner = SystemProcessRunner;
    let mut handle = runner.spawn(
        &PathBuf::from("/tmp"),
        "echo",
        &["test".to_string()],
        &HashMap::new(),
    ).unwrap();
    
    handle.wait().unwrap();
    
    // Second wait should error
    let result = handle.wait();
    assert!(result.is_err());
}

#[test]
fn test_terminate_idempotent() {
    let runner = SystemProcessRunner;
    let mut handle = runner.spawn(
        &PathBuf::from("/tmp"),
        "echo",
        &["test".to_string()],
        &HashMap::new(),
    ).unwrap();
    
    handle.wait().unwrap();
    
    // Terminate after wait should succeed (idempotent)
    assert!(handle.terminate().is_ok());
}
```

---

### 5. Verification Commands

```bash
# Check compilation
cargo check -p workflow_utils

# Run integration tests
cargo test -p workflow_utils --test process_tests

# Verify all workspace tests still pass
cargo test --workspace
```

---

## Notes for Implementer

1. **Do NOT modify `TaskExecutor` or `ExecutionHandle`** — these are lower-level utilities that coexist with `SystemProcessRunner`. They serve different purposes:
   - `TaskExecutor`: Builder-pattern utility for one-shot command execution
   - `SystemProcessRunner`: Trait-based adapter for the workflow engine

2. **`SystemProcessHandle` is private** — only `SystemProcessRunner` is public. The trait `ProcessHandle` is the public interface.

3. **Platform assumption**: Tests use `echo` and `sleep`, which are POSIX commands. This framework targets Linux/macOS HPC environments. Document this explicitly if Windows support is ever needed.

4. **Error on double-wait is intentional** — calling `wait()` twice is a programming error, not a recoverable condition. The `InvalidConfig` variant makes this explicit.

5. **Idempotent terminate** — calling `terminate()` after `wait()` is safe and returns `Ok(())`. This simplifies cleanup logic in the workflow engine.

---

## Relationship to TASK-4

TASK-4 added `.wait()` to `ExecutionHandle` but kept `child: Child` (not `Option<Child>`). This works for `ExecutionHandle` because it doesn't implement `ProcessHandle` — it's a standalone utility. `SystemProcessHandle` needs `Option<Child>` specifically because it implements the `ProcessHandle` trait, which requires `&mut self` for `wait()`.

These are two different types serving different purposes. No conflict.
