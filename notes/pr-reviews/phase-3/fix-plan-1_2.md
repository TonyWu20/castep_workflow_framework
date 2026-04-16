# Phase 2 -- Remove concrete impls from `workflow_core` and update re-exports (parallel pair)

These two tasks touch different files and can run in parallel.

---

## FIX-1: Remove `SystemProcessRunner` / `SystemProcessHandle` from `workflow_core/src/process.rs`

- **Task ID**: FIX-1
- **File**: `/Users/tony/programming/castep_workflow_framework/workflow_core/src/process.rs`
- **Target**: Everything after `pub struct ProcessResult { ... }`

**Step 1**: Delete everything from the comment `/// Concrete implementation of ProcessRunner for system processes.` through the end of the file. The block to remove starts with:

```rust
/// Concrete implementation of ProcessRunner for system processes.
pub struct SystemProcessRunner;

impl ProcessRunner for SystemProcessRunner {
```

and ends with the closing `}` of `impl ProcessHandle for SystemProcessHandle`. Everything before this block (the three trait/struct definitions — `ProcessRunner`, `ProcessHandle`, `ProcessResult`) must be kept.

**Step 2**: Update the imports at the top of the file.

- **Before** (lines 1-4):

```rust
use std::collections::HashMap;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
```

- **After**:

```rust
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
```

**Step 3**: Update re-exports in `workflow_core/src/lib.rs`.

- **Before** (line 11):

```rust
pub use process::{ProcessHandle, ProcessResult, ProcessRunner, SystemProcessRunner};
```

- **After**:

```rust
pub use process::{ProcessHandle, ProcessResult, ProcessRunner};
```

- **Verification**: `cd /Users/tony/programming/castep_workflow_framework && cargo check -p workflow_core 2>&1 | head -20` (expect errors from `workflow.rs` tests -- fixed in FIX-2b)
- **Depends on**: FIX-10 (all state.rs work done)
- **Can run in parallel with**: FIX-2a

---

## FIX-2a: Remove `ShellHookExecutor` from `workflow_core/src/monitoring.rs` and update re-exports

- **Task ID**: FIX-2a
- **File**: `/Users/tony/programming/castep_workflow_framework/workflow_core/src/monitoring.rs`
- **Target**: The `ShellHookExecutor` struct and its `impl HookExecutor` block

**Step 1**: Delete lines 74-105 (the doc comment, the struct, and the impl block).

- **Before** (lines 74-105):

```rust
/// Concrete implementation of HookExecutor that executes hooks via shell commands.
pub struct ShellHookExecutor;

impl HookExecutor for ShellHookExecutor {
    fn execute_hook(
        &self,
        hook: &MonitoringHook,
        ctx: &HookContext,
    ) -> Result<HookResult, WorkflowError> {
        let mut parts = hook.command.split_whitespace();
        let cmd = parts.next().unwrap_or_default();
        let args: Vec<String> = parts.map(String::from).collect();

        let output = std::process::Command::new(cmd)
            .args(&args)
            .env("TASK_ID", &ctx.task_id)
            .env("TASK_STATE", &ctx.state)
            .env("WORKDIR", ctx.workdir.to_string_lossy().as_ref())
            .env(
                "EXIT_CODE",
                ctx.exit_code.map(|c| c.to_string()).unwrap_or_default(),
            )
            .current_dir(&ctx.workdir)
            .output()
            .map_err(WorkflowError::Io)?;

        Ok(HookResult {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).into_owned(),
        })
    }
}
```

- **After**: (delete entirely -- the `[cfg(test)] mod tests` block that follows should remain)

**Step 2**: Update re-exports in `workflow_core/src/lib.rs`.

- **Before** (line 10):

```rust
pub use monitoring::{HookContext, HookExecutor, HookResult, HookTrigger, MonitoringHook, ShellHookExecutor};
```

- **After**:

```rust
pub use monitoring::{HookContext, HookExecutor, HookResult, HookTrigger, MonitoringHook};
```

- **Verification**: `cd /Users/tony/programming/castep_workflow_framework && cargo check -p workflow_core 2>&1 | head -20` (expect errors from `workflow.rs` tests -- fixed in FIX-2b)
- **Depends on**: FIX-10 (all state.rs work done)
- **Can run in parallel with**: FIX-1

---
