# Phase 1.1 Implementation Plan: Create workflow_utils Crate

## Context

This implements Phase 1.1 of the workflow_core revision plan documented in `PHASE1_IMPLEMENTATION_PLAN.md`. The goal is to create a new `workflow_utils` crate (Layer 2) that provides generic utilities for process execution, file I/O, and monitoring hooks. This replaces the trait-based executor pattern with a utilities-based architecture.

**Why this change:** The current architecture uses TOML-driven, trait-based executors (ExecutorFactory + Executor traits). The target architecture is Rust-first with closures, where users write Task::new(closure) instead of TOML definitions. The utilities layer provides the building blocks for these closures.

**Outcome:** A standalone crate with TaskExecutor, file I/O functions, and MonitoringHook that can be used in Task closures. This enables the Rust-first workflow API planned for Phase 1.2.

## Critical Design Decision: Sync API with Async Implementation

**Problem:** The plan says "start with sync" but the existing codebase is heavily async (tokio-based scheduler, async executors).

**Solution:** Provide a **synchronous public API** that internally uses async tokio code, bridged with `tokio::runtime::Handle::current().block_on()`.

**Benefits:**
- Simpler closures in Task::new() - no async/await complexity
- Compatible with existing tokio-based scheduler
- Clear upgrade path to async variants in Phase 2
- Avoids complex lifetime bounds (Send + Sync + 'static) in Phase 1

## File Structure

```
workflow_utils/
├── Cargo.toml
├── src/
│   ├── lib.rs           # Public API exports
│   ├── executor.rs      # TaskExecutor implementation
│   ├── files.rs         # File I/O utilities
│   └── monitoring.rs    # MonitoringHook system
└── tests/
    ├── executor_tests.rs
    ├── files_tests.rs
    └── monitoring_tests.rs
```

## Implementation Details

### 1. Cargo.toml

```toml
[package]
name = "workflow_utils"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { workspace = true }
anyhow = { workspace = true }
serde = { workspace = true }
nix = { workspace = true }

[dev-dependencies]
tempfile = "3"
```

**Note:** All dependencies already exist in workspace, so no new external dependencies needed.

### 2. TaskExecutor (executor.rs)

**Reusable patterns from existing code:**
- Process spawning pattern from `workflow_core/src/executors/local.rs` (lines 49-64)
- PID tracking and signal handling using `nix::sys::signal::kill()`
- Uses `tokio::process::Command` for async spawning

**API:**
```rust
pub struct TaskExecutor {
    workdir: PathBuf,
    command: String,
    args: Vec<String>,
    env: HashMap<String, String>,
}

impl TaskExecutor {
    pub fn new(workdir: impl Into<PathBuf>) -> Self;
    pub fn command(mut self, cmd: impl Into<String>) -> Self;
    pub fn arg(mut self, arg: impl Into<String>) -> Self;
    pub fn args(mut self, args: Vec<String>) -> Self;
    pub fn env(mut self, key: impl Into<String>, val: impl Into<String>) -> Self;
    
    // Sync API (bridges to async internally)
    pub fn execute(&self) -> Result<ExecutionResult>;
    pub fn spawn(&self) -> Result<ExecutionHandle>;
}

pub struct ExecutionResult {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
}

impl ExecutionResult {
    pub fn success(&self) -> bool { 
        self.exit_code == Some(0) 
    }
}

pub struct ExecutionHandle {
    pid: i32,
    workdir: PathBuf,
}

impl ExecutionHandle {
    pub fn pid(&self) -> i32;
    pub fn is_running(&self) -> bool;  // Uses nix::sys::signal::kill(pid, None)
    pub fn terminate(&self) -> Result<()>;  // Sends SIGTERM
}
```

**Implementation notes:**
- `execute()`: Use `.output().await` to capture stdout/stderr, measure duration with `Instant::now()`
- `spawn()`: Use `.spawn()` with null stdio for background execution, return PID-based handle
- Bridge sync/async: `tokio::runtime::Handle::current().block_on(self.execute_async())`
- Signal handling: `nix::sys::signal::kill(Pid::from_raw(pid), Signal::SIGTERM)`

**Tests (tests/executor_tests.rs):**
```rust
#[test]
fn test_executor_basic() // echo hello
fn test_executor_with_args() // echo hello world
fn test_executor_exit_code() // sh -c "exit 42"
fn test_executor_spawn() // sleep 1, check is_running()
fn test_executor_terminate() // sleep 60, terminate(), check stopped
fn test_executor_with_env() // sh -c "echo $VAR"
fn test_executor_stdout_stderr() // Capture both streams
```

### 3. Files Module (files.rs)

**Implementation:**
- Pure `std::fs` - no async needed
- Auto-create parent directories in `write_file()` and `copy_file()`
- Rich error context using `.with_context()`

**API:**
```rust
pub fn read_file(path: impl AsRef<Path>) -> Result<String>;
pub fn write_file(path: impl AsRef<Path>, content: &str) -> Result<()>;
pub fn copy_file(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()>;
pub fn create_dir(path: impl AsRef<Path>) -> Result<()>;
pub fn remove_dir(path: impl AsRef<Path>) -> Result<()>;
pub fn exists(path: impl AsRef<Path>) -> bool;
```

**Implementation notes:**
- `write_file`: Call `fs::create_dir_all(parent)` before `fs::write()`
- `copy_file`: Call `fs::create_dir_all(parent)` before `fs::copy()`
- `create_dir`: Use `fs::create_dir_all()` (creates parents automatically)
- `remove_dir`: Use `fs::remove_dir_all()` (recursive)
- Add `.with_context(|| format!("failed to read file: {}", path.display()))` to all operations

**Tests (tests/files_tests.rs):**
```rust
#[test]
fn test_read_write_file() // Round-trip
fn test_write_creates_parent_dirs() // a/b/c/test.txt
fn test_copy_file() // Copy and verify
fn test_create_remove_dir() // Create, check exists, remove
fn test_file_not_found() // Error handling
```

### 4. MonitoringHook (monitoring.rs)

**Implementation:**
- Serializable with serde (can be stored in workflow state)
- Context passed via environment variables
- Uses TaskExecutor internally for execution

**API:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringHook {
    pub name: String,
    pub command: String,
    pub trigger: HookTrigger,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookTrigger {
    OnStart,
    OnComplete,
    OnFailure,
    Periodic { interval_secs: u64 },
}

impl MonitoringHook {
    pub fn new(name: impl Into<String>, command: impl Into<String>, trigger: HookTrigger) -> Self;
    pub fn execute(&self, context: &HookContext) -> Result<HookResult>;
}

pub struct HookContext {
    pub task_id: String,
    pub workdir: PathBuf,
    pub state: String,
    pub exit_code: Option<i32>,
}

pub struct HookResult {
    pub success: bool,
    pub output: String,
}
```

**Implementation notes:**
- Parse command: `command.split_whitespace().collect::<Vec<_>>()`
- First element is command, rest are args
- Build TaskExecutor with parsed command/args
- Add context as env vars: `TASK_ID`, `TASK_STATE`, `WORKDIR`, `EXIT_CODE`
- Execute and return result

**Tests (tests/monitoring_tests.rs):**
```rust
#[test]
fn test_hook_on_complete() // echo success
fn test_hook_on_failure() // Different trigger
fn test_hook_receives_context() // sh -c "echo $TASK_ID"
fn test_hook_context() // Verify all env vars passed
```

### 5. Public API (lib.rs)

```rust
//! Generic utilities for workflow execution
//!
//! This crate provides building blocks for workflow tasks:
//! - `TaskExecutor`: Execute processes with captured output
//! - File I/O functions: Read, write, copy files and directories
//! - `MonitoringHook`: Execute external monitoring commands

mod executor;
mod files;
mod monitoring;

pub use executor::{TaskExecutor, ExecutionResult, ExecutionHandle};
pub use monitoring::{MonitoringHook, HookTrigger, HookContext, HookResult};
pub use files::{read_file, write_file, copy_file, create_dir, remove_dir, exists};
```

## Implementation Order

**Day 1: Files Module**
1. Create crate structure: `mkdir -p workflow_utils/src workflow_utils/tests`
2. Create `Cargo.toml` with dependencies
3. Implement `files.rs` (simplest, no dependencies on other modules)
4. Write `tests/files_tests.rs`
5. Verify: `cargo test -p workflow_utils files`

**Day 2: Executor and Monitoring**
6. Implement `executor.rs` (core functionality)
7. Write `tests/executor_tests.rs`
8. Verify: `cargo test -p workflow_utils executor`
9. Implement `monitoring.rs` (uses executor internally)
10. Write `tests/monitoring_tests.rs`
11. Create `lib.rs` with public exports
12. Verify: `cargo test -p workflow_utils`

## Integration Steps

1. **Add to workspace:** Edit root `Cargo.toml`, add `"workflow_utils"` to `[workspace] members`
2. **Verify workspace:** `cargo build -p workflow_utils`
3. **Run all tests:** `cargo test -p workflow_utils`
4. **Check warnings:** `cargo clippy -p workflow_utils`

## Usage Example (Validation)

```rust
use workflow_utils::{TaskExecutor, files, MonitoringHook, HookTrigger, HookContext};

fn example_task() -> anyhow::Result<()> {
    // Create workdir
    files::create_dir("runs/task1")?;
    
    // Write input file
    files::write_file("runs/task1/input.txt", "data")?;
    
    // Execute process
    let result = TaskExecutor::new("runs/task1")
        .command("echo")
        .arg("hello")
        .env("OMP_NUM_THREADS", "4")
        .execute()?;
    
    // Check result
    if !result.success() {
        anyhow::bail!("Task failed with code {:?}", result.exit_code);
    }
    
    // Run monitoring hook
    let hook = MonitoringHook::new(
        "check_output",
        "echo Hook executed",
        HookTrigger::OnComplete,
    );
    
    let hook_result = hook.execute(&HookContext {
        task_id: "task1".into(),
        workdir: "runs/task1".into(),
        state: "Completed".into(),
        exit_code: result.exit_code,
    })?;
    
    println!("Hook output: {}", hook_result.output);
    Ok(())
}
```

## Critical Files

**To create:**
- `/Users/tony/Documents/programming/castep_workflow_framework/workflow_utils/Cargo.toml`
- `/Users/tony/Documents/programming/castep_workflow_framework/workflow_utils/src/lib.rs`
- `/Users/tony/Documents/programming/castep_workflow_framework/workflow_utils/src/executor.rs`
- `/Users/tony/Documents/programming/castep_workflow_framework/workflow_utils/src/files.rs`
- `/Users/tony/Documents/programming/castep_workflow_framework/workflow_utils/src/monitoring.rs`
- `/Users/tony/Documents/programming/castep_workflow_framework/workflow_utils/tests/executor_tests.rs`
- `/Users/tony/Documents/programming/castep_workflow_framework/workflow_utils/tests/files_tests.rs`
- `/Users/tony/Documents/programming/castep_workflow_framework/workflow_utils/tests/monitoring_tests.rs`

**To modify:**
- `/Users/tony/Documents/programming/castep_workflow_framework/Cargo.toml` (add workflow_utils to workspace members)

## Reusable Code References

**From workflow_core/src/executors/local.rs:**
- Lines 49-64: Process spawning with tokio::process::Command
- Lines 32-44: Sentinel file pattern (optional for Phase 2)
- Lines 85-89: Signal handling with nix::sys::signal::kill()

**From workflow_core/src/scheduler.rs:**
- Lines 178-203: Timeout enforcement pattern (for Phase 2)

**Testing patterns from workflow_core:**
- Inline tests in `#[cfg(test)] mod tests` blocks
- Use `tempfile::tempdir()` for isolated file I/O
- Real process execution tests (echo, sleep, sh -c)
- `#[tokio::test]` for async tests (if needed)

## Verification

After implementation, verify:

1. **Build succeeds:** `cargo build -p workflow_utils`
2. **All tests pass:** `cargo test -p workflow_utils`
3. **No warnings:** `cargo clippy -p workflow_utils`
4. **Example compiles:** Create example in `workflow_utils/examples/basic.rs` and run `cargo run -p workflow_utils --example basic`
5. **API matches spec:** Review public API in generated docs: `cargo doc -p workflow_utils --open`

## Success Criteria

- [ ] All unit tests pass (minimum 15 tests total)
- [ ] No compiler warnings or clippy issues
- [ ] API matches target specification from PHASE1_IMPLEMENTATION_PLAN.md
- [ ] Documentation comments on all public items
- [ ] Example compiles and runs successfully
- [ ] Ready for Phase 1.2 (workflow_core integration)

## Deferred to Phase 2

- Async variants (`execute_async()`, `spawn_async()`)
- Shell-style command parsing for hooks (currently simple whitespace split)
- Timeout support in TaskExecutor (handled by Workflow layer in Phase 1)
- Sentinel files for exit code persistence across restarts
- Stream-based output capture (currently buffers all output)
- SLURM backend support (ExecutorBackend enum)
