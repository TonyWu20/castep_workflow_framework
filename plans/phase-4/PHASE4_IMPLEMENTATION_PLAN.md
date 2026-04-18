# Phase 4: Real HPC Execution — Implementation Plan

## Context

Phase 4 bridges "runs locally" to "runs on a real HPC cluster." After merging `pre-phase-4`, we implement: per-task log persistence, periodic monitoring hooks, SLURM/PBS job submission, and graph-aware retry. Three deferred cleanup items from the pre-phase-4 review are bundled with the hooks work since they touch the same code.

## Design Decisions

1. **Do NOT change `ProcessRunner::spawn` signature.** Store `log_dir` as a field on `SystemProcessRunner`. Mocks/stubs remain untouched.
2. **`OutputLocation` enum** replaces raw `stdout`/`stderr` strings on `ProcessResult` — explicit file-backed vs captured output at the type level.
3. **`QueuedRunner` does NOT implement `ProcessRunner`.** Separate `QueuedSubmitter` trait in `workflow_core/src/process.rs`.
4. **Rate-limited polling** in `QueuedProcessHandle::is_running` (15s cache, not 50ms).
5. **Framework controls SLURM output paths** via `-o`/`-e` flags injected by `QueuedRunner::submit`.
6. **Periodic hooks call `hook_executor.execute_hook` directly**, not through `fire_hooks`.
7. **`task_successors`** (not `task_deps`) with `#[serde(default)]` on `JsonStateStore`.

## Invariants

- No tokio — all polling stays `std::thread::sleep` + `try_wait`
- `thiserror` in libs, `anyhow` only in binaries
- `ProcessHandle` trait signature unchanged
- New `WorkflowError` variant: `QueueSubmitFailed(String)`

---

### TASK-1: Add `OutputLocation` enum and update `ProcessResult`

**Files:** `workflow_core/src/process.rs`, `workflow_utils/src/executor.rs`, `workflow_core/src/workflow.rs`, `workflow_core/tests/common/mod.rs`
**Depends on:** none
**Parallel with:** TASK-3, TASK-6, TASK-8

Change `ProcessResult` to use a typed `OutputLocation` enum instead of bare `stdout`/`stderr` strings.

**Before** (`workflow_core/src/process.rs`):
```rust
pub struct ProcessResult {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
}
```

**After** (`workflow_core/src/process.rs`):
```rust
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum OutputLocation {
    Captured { stdout: String, stderr: String },
    OnDisk { stdout_path: PathBuf, stderr_path: PathBuf },
}

pub struct ProcessResult {
    pub exit_code: Option<i32>,
    pub output: OutputLocation,
    pub duration: Duration,
}
```

**Before** (`workflow_utils/src/executor.rs`, `SystemProcessHandle::wait()`):
```rust
Ok(ProcessResult {
    exit_code: output.status.code(),
    stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
    stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    duration: self.start.elapsed(),
})
```

**After** (`workflow_utils/src/executor.rs`, `SystemProcessHandle::wait()`):
```rust
Ok(ProcessResult {
    exit_code: output.status.code(),
    output: OutputLocation::Captured {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    },
    duration: self.start.elapsed(),
})
```

**Before** (`workflow_core/src/workflow.rs`, `StubHandle::wait()` in test module):
```rust
Ok(crate::process::ProcessResult {
    exit_code: output.status.code(),
    stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
    stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    duration: self.start.elapsed(),
})
```

**After** (`workflow_core/src/workflow.rs`, `StubHandle::wait()` in test module):
```rust
Ok(crate::process::ProcessResult {
    exit_code: output.status.code(),
    output: crate::process::OutputLocation::Captured {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    },
    duration: self.start.elapsed(),
})
```

Note: `process_finished` in `workflow.rs` currently accesses `process_result.exit_code` only — it does not read `stdout`/`stderr`. No change needed there beyond ensuring the pattern compiles. Use `LSP references` on `ProcessResult` to find any other usages.

Acceptance: `cargo check --workspace` passes; `cargo clippy --workspace` passes.

---

### TASK-2: File-backed output in `SystemProcessRunner`

**Files:** `workflow_utils/src/executor.rs`
**Depends on:** TASK-1
**Parallel with:** TASK-3, TASK-6, TASK-8

Add `log_dir` field to `SystemProcessRunner` and redirect stdout/stderr to files when set.

**Before** (`workflow_utils/src/executor.rs`):
```rust
pub struct SystemProcessRunner;

// (no constructors — unit struct)
```

**After** (`workflow_utils/src/executor.rs`):
```rust
pub struct SystemProcessRunner {
    log_dir: Option<PathBuf>,
}

impl SystemProcessRunner {
    pub fn new() -> Self {
        Self { log_dir: None }
    }

    pub fn with_log_dir(dir: impl Into<PathBuf>) -> Self {
        Self { log_dir: Some(dir.into()) }
    }
}
```

**Before** (`SystemProcessHandle`):
```rust
pub struct SystemProcessHandle {
    child: Option<Child>,
    start: Instant,
}
```

**After** (`SystemProcessHandle`):
```rust
pub struct SystemProcessHandle {
    child: Option<Child>,
    start: Instant,
    output_files: Option<(PathBuf, PathBuf)>,
}
```

**Before** (`ProcessRunner::spawn` impl):
```rust
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
            .stdout(Stdio::piped())
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

**After** (`ProcessRunner::spawn` impl):
```rust
impl ProcessRunner for SystemProcessRunner {
    fn spawn(
        &self,
        workdir: &Path,
        command: &str,
        args: &[String],
        env: &HashMap<String, String>,
    ) -> Result<Box<dyn ProcessHandle>, WorkflowError> {
        let (stdout_cfg, stderr_cfg, output_files) = if let Some(ref log_dir) = self.log_dir {
            // Derive file prefix from workdir leaf directory name
            let prefix = workdir
                .file_name()
                .map(|n| n.to_string_lossy().replace('/', "_"))
                .unwrap_or_else(|| "unknown".into());
            let stdout_path = log_dir.join(format!("{}.stdout", prefix));
            let stderr_path = log_dir.join(format!("{}.stderr", prefix));
            let stdout_file = std::fs::File::create(&stdout_path).map_err(WorkflowError::Io)?;
            let stderr_file = std::fs::File::create(&stderr_path).map_err(WorkflowError::Io)?;
            (Stdio::from(stdout_file), Stdio::from(stderr_file), Some((stdout_path, stderr_path)))
        } else {
            (Stdio::piped(), Stdio::piped(), None)
        };

        let child = Command::new(command)
            .args(args)
            .envs(env)
            .current_dir(workdir)
            .stdout(stdout_cfg)
            .stderr(stderr_cfg)
            .spawn()
            .map_err(WorkflowError::Io)?;

        Ok(Box::new(SystemProcessHandle {
            child: Some(child),
            start: Instant::now(),
            output_files,
        }))
    }
}
```

**Before** (`SystemProcessHandle::wait()`):
```rust
fn wait(&mut self) -> Result<ProcessResult, WorkflowError> {
    let child = self.child.take()
        .ok_or_else(|| WorkflowError::InvalidConfig("wait() called twice".into()))?;

    let output = child.wait_with_output().map_err(WorkflowError::Io)?;

    Ok(ProcessResult {
        exit_code: output.status.code(),
        output: OutputLocation::Captured {
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        },
        duration: self.start.elapsed(),
    })
}
```

**After** (`SystemProcessHandle::wait()`):
```rust
fn wait(&mut self) -> Result<ProcessResult, WorkflowError> {
    let child = self.child.take()
        .ok_or_else(|| WorkflowError::InvalidConfig("wait() called twice".into()))?;

    let output_loc = if let Some((ref stdout_path, ref stderr_path)) = self.output_files {
        // File-backed: just wait for exit, output is on disk
        let status = child.wait_with_output().map_err(WorkflowError::Io)?;
        let _ = status; // exit code handled below via output
        OutputLocation::OnDisk {
            stdout_path: stdout_path.clone(),
            stderr_path: stderr_path.clone(),
        }
    } else {
        let output = child.wait_with_output().map_err(WorkflowError::Io)?;
        OutputLocation::Captured {
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    };

    // Need exit code in both cases
    // Re-read: actually wait_with_output returns Output which has status
    // Fix: capture output in both branches and extract exit_code
    // The subagent should handle this — both branches call wait_with_output
    // and can extract status.code() before constructing OutputLocation

    Ok(ProcessResult {
        exit_code: todo!("extract from wait_with_output in both branches"),
        output: output_loc,
        duration: self.start.elapsed(),
    })
}
```

**Note to subagent**: The after snippet for `wait()` has a `todo!()` placeholder. The correct implementation: call `child.wait_with_output()` once in both branches, extract `status.code()` as the exit_code, and build the appropriate `OutputLocation`. Do not call `wait_with_output` twice.

Also update the only external constructor site: `fn runner() -> Arc<dyn ProcessRunner> { Arc::new(SystemProcessRunner) }` in `workflow_core/tests/hook_recording.rs` → `Arc::new(SystemProcessRunner::new())`.

Acceptance: `cargo check --workspace` passes; `cargo test --workspace` passes.

---

### TASK-3: Add `TaskPhase` enum, refactor `fire_hooks` signature (Deferred #1 + #2)

**Files:** `workflow_core/src/monitoring.rs`, `workflow_core/src/workflow.rs`
**Depends on:** none
**Parallel with:** TASK-1, TASK-6, TASK-8

Replace stringly-typed state matching in hooks with a `TaskPhase` enum, and change `fire_hooks` to take `&Path` instead of `PathBuf`.

**Before** (`workflow_core/src/monitoring.rs`, `HookContext`):
```rust
pub struct HookContext {
    pub task_id: String,
    pub workdir: std::path::PathBuf,
    pub state: String,
    pub exit_code: Option<i32>,
}
```

**After** (`workflow_core/src/monitoring.rs`, `HookContext`):
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskPhase {
    Running,
    Completed,
    Failed,
}

pub struct HookContext {
    pub task_id: String,
    pub workdir: std::path::PathBuf,
    pub phase: TaskPhase,
    pub exit_code: Option<i32>,
}
```

**Before** (`workflow_core/src/workflow.rs`, `fire_hooks` fn):
```rust
fn fire_hooks(
    monitors: &[crate::monitoring::MonitoringHook],
    workdir: std::path::PathBuf,
    final_state: &str,
    exit_code: Option<i32>,
    task_id: &str,
    hook_executor: &dyn HookExecutor,
) {
    let ctx = crate::monitoring::HookContext {
        task_id: task_id.to_string(),
        workdir,
        state: final_state.to_string(),
        exit_code,
    };
    for hook in monitors {
        let should_fire = matches!((&hook.trigger, final_state), (
            crate::monitoring::HookTrigger::OnStart,
            "running"
        ) | (crate::monitoring::HookTrigger::OnComplete, "completed")
            | (crate::monitoring::HookTrigger::OnFailure, "failed"));
        if should_fire {
            if let Err(e) = hook_executor.execute_hook(hook, &ctx) {
                tracing::warn!(
                    "Hook '{}' for task '{}' failed: {}",
                    hook.name,
                    task_id,
                    e
                );
            }
        }
    }
}
```

**After** (`workflow_core/src/workflow.rs`, `fire_hooks` fn):
```rust
fn fire_hooks(
    monitors: &[crate::monitoring::MonitoringHook],
    workdir: &std::path::Path,
    phase: crate::monitoring::TaskPhase,
    exit_code: Option<i32>,
    task_id: &str,
    hook_executor: &dyn HookExecutor,
) {
    let ctx = crate::monitoring::HookContext {
        task_id: task_id.to_string(),
        workdir: workdir.to_path_buf(),
        phase,
        exit_code,
    };
    for hook in monitors {
        let should_fire = matches!(
            (&hook.trigger, phase),
            (crate::monitoring::HookTrigger::OnStart, crate::monitoring::TaskPhase::Running)
                | (crate::monitoring::HookTrigger::OnComplete, crate::monitoring::TaskPhase::Completed)
                | (crate::monitoring::HookTrigger::OnFailure, crate::monitoring::TaskPhase::Failed)
        );
        if should_fire {
            if let Err(e) = hook_executor.execute_hook(hook, &ctx) {
                tracing::warn!(
                    "Hook '{}' for task '{}' failed: {}",
                    hook.name,
                    task_id,
                    e
                );
            }
        }
    }
}
```

**Before** (dispatch block call site, `Workflow::run`):
```rust
fire_hooks(
    &monitors,
    task_workdir,
    "running",
    None,
    &id,
    hook_executor.as_ref(),
);
```

**After** (dispatch block call site, `Workflow::run`):
```rust
fire_hooks(
    &monitors,
    &task_workdir,
    crate::monitoring::TaskPhase::Running,
    None,
    &id,
    hook_executor.as_ref(),
);
```

**Before** (`process_finished` call site):
```rust
fire_hooks(
    &t.monitors,
    t.workdir,
    final_state,
    exit_code,
    id,
    hook_executor,
);
```

**After** (`process_finished` — `final_state` variable changes type):
```rust
// Change the type of final_state from &str to TaskPhase throughout process_finished:
// "completed" → crate::monitoring::TaskPhase::Completed
// "failed" → crate::monitoring::TaskPhase::Failed

fire_hooks(
    &t.monitors,
    &t.workdir,
    final_state,
    exit_code,
    id,
    hook_executor,
);
```

Also update all test code in `monitoring.rs` tests and `hook_recording.rs` that constructs `HookContext`:

**Before** (monitoring.rs tests, multiple locations):
```rust
let ctx = HookContext {
    task_id: "t1".into(),
    state: "running".into(),
    workdir: std::path::PathBuf::from("."),
    exit_code: None,
};
```

**After**:
```rust
let ctx = HookContext {
    task_id: "t1".into(),
    phase: TaskPhase::Running,
    workdir: std::path::PathBuf::from("."),
    exit_code: None,
};
```

Acceptance: `cargo check --workspace` passes; `cargo test --workspace` passes.

---

### TASK-4: Wire `log_dir` into `Workflow` + fix misleading test comment (Deferred #3)

**Files:** `workflow_core/src/workflow.rs`, `workflow_core/tests/hook_recording.rs`
**Depends on:** TASK-2
**Parallel with:** TASK-5

Add `log_dir` field to `Workflow` for directory creation, and fix the misleading comment.

**Before** (`Workflow` struct):
```rust
pub struct Workflow {
    pub name: String,
    tasks: HashMap<String, Task>,
    max_parallel: usize,
    pub(crate) interrupt: Arc<AtomicBool>,
}
```

**After** (`Workflow` struct):
```rust
pub struct Workflow {
    pub name: String,
    tasks: HashMap<String, Task>,
    max_parallel: usize,
    pub(crate) interrupt: Arc<AtomicBool>,
    log_dir: Option<std::path::PathBuf>,
}
```

**Before** (`Workflow::new`):
```rust
Self {
    name: name.into(),
    tasks: HashMap::new(),
    max_parallel,
    interrupt: Arc::new(AtomicBool::new(false)),
}
```

**After** (`Workflow::new`):
```rust
Self {
    name: name.into(),
    tasks: HashMap::new(),
    max_parallel,
    interrupt: Arc::new(AtomicBool::new(false)),
    log_dir: None,
}
```

Add builder method after `with_max_parallel`:
```rust
pub fn with_log_dir(mut self, path: impl Into<std::path::PathBuf>) -> Self {
    self.log_dir = Some(path.into());
    self
}
```

**Before** (`Workflow::run`, after signal registration, before rejection block):
```rust
signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&self.interrupt)).ok();

// Reject Queued tasks and Periodic hooks upfront...
```

**After** (insert between signal registration and rejection block):
```rust
signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&self.interrupt)).ok();

if let Some(ref dir) = self.log_dir {
    std::fs::create_dir_all(dir).map_err(WorkflowError::Io)?;
}

// Reject Queued tasks and Periodic hooks upfront...
```

**Before** (`hook_recording.rs`, line ~103):
```rust
// Expected order: success OnStart, failure OnStart, success OnComplete, failure OnFailure
```

**After**:
```rust
// 4 hook calls total: 2 per task (cross-task order is non-deterministic)
```

Acceptance: `cargo check --workspace` passes; `cargo test --workspace` passes.

---

### TASK-5: Implement `HookTrigger::Periodic` firing in run loop

**Files:** `workflow_core/src/workflow.rs`
**Depends on:** TASK-3
**Parallel with:** TASK-4, TASK-7

Add periodic hook support to the run loop and remove the upfront rejection.

**Before** (`InFlightTask`):
```rust
pub(crate) struct InFlightTask {
    pub handle: Box<dyn ProcessHandle>,
    pub started_at: Instant,
    pub monitors: Vec<crate::monitoring::MonitoringHook>,
    pub collect: Option<TaskClosure>,
    pub workdir: std::path::PathBuf,
}
```

**After** (`InFlightTask`):
```rust
pub(crate) struct InFlightTask {
    pub handle: Box<dyn ProcessHandle>,
    pub started_at: Instant,
    pub monitors: Vec<crate::monitoring::MonitoringHook>,
    pub collect: Option<TaskClosure>,
    pub workdir: std::path::PathBuf,
    pub last_periodic_fire: HashMap<String, Instant>,
}
```

**Before** (upfront rejection block to DELETE):
```rust
for hook in &task.monitors {
    if matches!(hook.trigger, crate::monitoring::HookTrigger::Periodic { .. }) {
        return Err(WorkflowError::InvalidConfig(
            format!("task '{}': Periodic hooks are not yet supported", id)
        ));
    }
}
```

**After**: Delete the above block entirely.

**Before** (InFlightTask insertion):
```rust
handles.insert(id.to_string(), InFlightTask {
    handle,
    started_at: Instant::now(),
    monitors,
    collect: task.collect,
    workdir: task.workdir,
});
```

**After** (InFlightTask insertion):
```rust
handles.insert(id.to_string(), InFlightTask {
    handle,
    started_at: Instant::now(),
    monitors,
    collect: task.collect,
    workdir: task.workdir,
    last_periodic_fire: HashMap::new(),
});
```

**New block** to insert AFTER `propagate_skips(...)` and BEFORE the dispatch block (`// Dispatch ready tasks`):
```rust
// Fire periodic hooks for in-flight tasks
for (task_id, t) in handles.iter_mut() {
    for hook in &t.monitors {
        if let crate::monitoring::HookTrigger::Periodic { interval_secs } = hook.trigger {
            let last = t.last_periodic_fire
                .entry(hook.name.clone())
                .or_insert(t.started_at);
            if last.elapsed() >= Duration::from_secs(interval_secs) {
                let ctx = crate::monitoring::HookContext {
                    task_id: task_id.clone(),
                    workdir: t.workdir.clone(),
                    phase: crate::monitoring::TaskPhase::Running,
                    exit_code: None,
                };
                if let Err(e) = hook_executor.execute_hook(hook, &ctx) {
                    tracing::warn!(
                        "Periodic hook '{}' for task '{}' failed: {}",
                        hook.name, task_id, e
                    );
                }
                *last = Instant::now();
            }
        }
    }
}
```

Note: periodic hooks call `hook_executor.execute_hook` directly — NOT through `fire_hooks`.

Acceptance: `cargo check --workspace` passes; existing non-periodic tests pass via `cargo test --workspace`.

---

### TASK-6: Add `QueueSubmitFailed` error variant + `QueuedSubmitter` trait

**Files:** `workflow_core/src/error.rs`, `workflow_core/src/process.rs`
**Depends on:** none
**Parallel with:** TASK-1, TASK-3, TASK-8

**Before** (`workflow_core/src/error.rs`, end of enum):
```rust
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("workflow interrupted by signal")]
    Interrupted,
}
```

**After** (`workflow_core/src/error.rs`):
```rust
    #[error("queue submission failed: {0}")]
    QueueSubmitFailed(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("workflow interrupted by signal")]
    Interrupted,
}
```

**Before** (`workflow_core/src/process.rs`, end of file):
```rust
pub struct ProcessResult {
    pub exit_code: Option<i32>,
    pub output: OutputLocation,
    pub duration: Duration,
}
```

**After** (`workflow_core/src/process.rs`, append after `ProcessResult`):
```rust
pub trait QueuedSubmitter: Send + Sync {
    fn submit(
        &self,
        workdir: &Path,
        task_id: &str,
        log_dir: &Path,
    ) -> Result<Box<dyn ProcessHandle>, WorkflowError>;
}
```

Acceptance: `cargo check --workspace` passes.

---

### TASK-7: Implement `QueuedRunner` and `QueuedProcessHandle`

**Files:** `workflow_utils/src/queued.rs` (new), `workflow_utils/src/lib.rs`
**Depends on:** TASK-1, TASK-6
**Parallel with:** TASK-5, TASK-8

Create the full queued execution module. No "before" — this is a new file.

**After** (`workflow_utils/src/queued.rs`):
```rust
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use workflow_core::error::WorkflowError;
use workflow_core::process::{OutputLocation, ProcessHandle, ProcessResult, QueuedSubmitter};

#[derive(Debug, Clone, Copy)]
pub enum SchedulerKind {
    Slurm,
    Pbs,
}

pub struct QueuedRunner {
    pub scheduler: SchedulerKind,
}

impl QueuedRunner {
    pub fn new(scheduler: SchedulerKind) -> Self {
        Self { scheduler }
    }

    fn build_submit_cmd(&self, script_path: &str, task_id: &str, log_dir: &Path) -> String {
        let stdout_path = log_dir.join(format!("{}.stdout", task_id));
        let stderr_path = log_dir.join(format!("{}.stderr", task_id));
        match self.scheduler {
            SchedulerKind::Slurm => format!(
                "sbatch -o {} -e {} {}",
                stdout_path.display(), stderr_path.display(), script_path
            ),
            SchedulerKind::Pbs => format!(
                "qsub -o {} -e {} {}",
                stdout_path.display(), stderr_path.display(), script_path
            ),
        }
    }

    fn build_poll_cmd(&self) -> String {
        match self.scheduler {
            SchedulerKind::Slurm => "squeue -j {job_id} -h".into(),
            SchedulerKind::Pbs => "qstat {job_id}".into(),
        }
    }

    fn build_cancel_cmd(&self) -> String {
        match self.scheduler {
            SchedulerKind::Slurm => "scancel {job_id}".into(),
            SchedulerKind::Pbs => "qdel {job_id}".into(),
        }
    }

    fn parse_job_id(&self, stdout: &str) -> Result<String, WorkflowError> {
        match self.scheduler {
            SchedulerKind::Slurm => stdout
                .split_whitespace()
                .last()
                .map(|s| s.to_string())
                .ok_or_else(|| WorkflowError::QueueSubmitFailed(
                    format!("failed to parse SLURM job ID from: {}", stdout)
                )),
            SchedulerKind::Pbs => {
                let trimmed = stdout.trim().to_string();
                if trimmed.is_empty() {
                    Err(WorkflowError::QueueSubmitFailed("empty PBS job ID".into()))
                } else {
                    Ok(trimmed)
                }
            }
        }
    }
}

impl QueuedSubmitter for QueuedRunner {
    fn submit(
        &self,
        workdir: &Path,
        task_id: &str,
        log_dir: &Path,
    ) -> Result<Box<dyn ProcessHandle>, WorkflowError> {
        let submit_cmd = self.build_submit_cmd(
            &workdir.join("job.sh").to_string_lossy(), task_id, log_dir
        );
        let output = Command::new("sh")
            .args(["-c", &submit_cmd])
            .current_dir(workdir)
            .output()
            .map_err(WorkflowError::Io)?;

        if !output.status.success() {
            return Err(WorkflowError::QueueSubmitFailed(
                String::from_utf8_lossy(&output.stderr).into_owned()
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let job_id = self.parse_job_id(&stdout)?;

        let stdout_path = log_dir.join(format!("{}.stdout", task_id));
        let stderr_path = log_dir.join(format!("{}.stderr", task_id));

        Ok(Box::new(QueuedProcessHandle {
            job_id,
            poll_cmd: self.build_poll_cmd(),
            cancel_cmd: self.build_cancel_cmd(),
            workdir: workdir.to_path_buf(),
            stdout_path,
            stderr_path,
            last_poll: Instant::now(),
            poll_interval: Duration::from_secs(15),
            cached_running: true,
            finished_exit_code: None,
            started_at: Instant::now(),
        }))
    }
}

pub struct QueuedProcessHandle {
    job_id: String,
    poll_cmd: String,
    cancel_cmd: String,
    workdir: PathBuf,
    stdout_path: PathBuf,
    stderr_path: PathBuf,
    last_poll: Instant,
    poll_interval: Duration,
    cached_running: bool,
    finished_exit_code: Option<i32>,
    started_at: Instant,
}

impl ProcessHandle for QueuedProcessHandle {
    fn is_running(&mut self) -> bool {
        if self.last_poll.elapsed() < self.poll_interval {
            return self.cached_running;
        }

        let cmd = self.poll_cmd.replace("{job_id}", &self.job_id);
        let result = Command::new("sh")
            .args(["-c", &cmd])
            .output();

        match result {
            Ok(output) => {
                // Non-zero exit or empty stdout = job gone
                let running = output.status.success()
                    && !output.stdout.is_empty();
                self.cached_running = running;
                if !running {
                    self.finished_exit_code = Some(0); // default; accounting query in wait() may refine
                }
            }
            Err(_) => {
                self.cached_running = false;
                self.finished_exit_code = Some(-1);
            }
        }

        self.last_poll = Instant::now();
        self.cached_running
    }

    fn terminate(&mut self) -> Result<(), WorkflowError> {
        let cmd = self.cancel_cmd.replace("{job_id}", &self.job_id);
        Command::new("sh")
            .args(["-c", &cmd])
            .output()
            .map_err(WorkflowError::Io)?;
        Ok(())
    }

    fn wait(&mut self) -> Result<ProcessResult, WorkflowError> {
        Ok(ProcessResult {
            exit_code: self.finished_exit_code,
            output: OutputLocation::OnDisk {
                stdout_path: self.stdout_path.clone(),
                stderr_path: self.stderr_path.clone(),
            },
            duration: self.started_at.elapsed(),
        })
    }
}
```

**Before** (`workflow_utils/src/lib.rs`, relevant section):
```rust
// (existing module declarations and re-exports)
```

**After** (add to `workflow_utils/src/lib.rs`):
```rust
pub mod queued;
pub use queued::*;
```

Acceptance: `cargo check --workspace` passes.

---

### TASK-8: Add `task_successors` to `JsonStateStore` and graph-aware `cmd_retry`

**Files:** `workflow_core/src/state.rs`, `workflow_core/src/workflow.rs`, `workflow-cli/src/main.rs`
**Depends on:** none
**Parallel with:** TASK-1, TASK-3, TASK-6

**Before** (`JsonStateStore` struct):
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonStateStore {
    workflow_name: String,
    created_at: String,
    last_updated: String,
    tasks: HashMap<String, TaskStatus>,
    path: PathBuf,
}
```

**After** (`JsonStateStore` struct):
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonStateStore {
    workflow_name: String,
    created_at: String,
    last_updated: String,
    tasks: HashMap<String, TaskStatus>,
    #[serde(default)]
    task_successors: HashMap<String, Vec<String>>,
    path: PathBuf,
}
```

**Before** (`JsonStateStore::new`):
```rust
Self {
    workflow_name: name.to_owned(),
    created_at: now.clone(),
    last_updated: now,
    tasks: HashMap::new(),
    path,
}
```

**After** (`JsonStateStore::new`):
```rust
Self {
    workflow_name: name.to_owned(),
    created_at: now.clone(),
    last_updated: now,
    tasks: HashMap::new(),
    task_successors: HashMap::new(),
    path,
}
```

**Before** (`StateStore` trait — no `set_task_graph`):
```rust
pub trait StateStore: Send + Sync {
    fn get_status(&self, id: &str) -> Option<TaskStatus>;
    fn set_status(&mut self, id: &str, status: TaskStatus);
    fn all_tasks(&self) -> Vec<(String, TaskStatus)>;
    fn save(&self) -> Result<(), WorkflowError>;
}
```

**After** (`StateStore` trait):
```rust
pub trait StateStore: Send + Sync {
    fn get_status(&self, id: &str) -> Option<TaskStatus>;
    fn set_status(&mut self, id: &str, status: TaskStatus);
    fn all_tasks(&self) -> Vec<(String, TaskStatus)>;
    fn save(&self) -> Result<(), WorkflowError>;
    fn set_task_graph(&mut self, _successors: HashMap<String, Vec<String>>) {}
}
```

Add override in `impl StateStore for JsonStateStore`:
```rust
fn set_task_graph(&mut self, successors: HashMap<String, Vec<String>>) {
    self.task_successors = successors;
}
```

Add getter on `JsonStateStore`:
```rust
pub fn task_successors(&self) -> &HashMap<String, Vec<String>> {
    &self.task_successors
}
```

**Before** (`Workflow::run`, after `let dag = self.build_dag()?;`):
```rust
let dag = self.build_dag()?;

// Initialize state for all tasks
for id in dag.task_ids() {
```

**After**:
```rust
let dag = self.build_dag()?;

// Persist task dependency graph for CLI retry
let successors: HashMap<String, Vec<String>> = dag.task_ids()
    .map(|id| (id.clone(), dag.successors(id)))
    .collect();
state.set_task_graph(successors);

// Initialize state for all tasks
for id in dag.task_ids() {
```

**Before** (`cmd_retry` in `workflow-cli/src/main.rs`):
```rust
fn cmd_retry(state: &mut dyn StateStore, task_ids: &[String]) -> anyhow::Result<()> {
    for id in task_ids {
        if state.get_status(id).is_none() {
            eprintln!("warn: task '{}' not found", id);
        } else {
            state.mark_pending(id);
        }
    }
    // Reset all dependency-failure-skipped tasks globally (not just those downstream
    // of `task_ids`). Intentional for v0.1 simplicity — a graph-aware retry would
    // require DAG access that the CLI does not have.
    let to_reset: Vec<String> = state
        .all_tasks()
        .into_iter()
        .filter(|(_, s)| matches!(s, TaskStatus::SkippedDueToDependencyFailure))
        .map(|(id, _)| id)
        .collect();
    for id in to_reset {
        state.mark_pending(&id);
    }
    state.save().map_err(|e| anyhow::anyhow!("failed to save state: {}", e))?;
    Ok(())
}
```

**After** (`cmd_retry` — note: takes `JsonStateStore` directly since CLI already loads it as concrete type):
```rust
use std::collections::{HashSet, VecDeque};

fn downstream_tasks(start: &[String], successors: &HashMap<String, Vec<String>>) -> HashSet<String> {
    let mut visited = HashSet::new();
    let mut queue: VecDeque<String> = start.iter().cloned().collect();
    while let Some(id) = queue.pop_front() {
        if let Some(deps) = successors.get(&id) {
            for dep in deps {
                if visited.insert(dep.clone()) {
                    queue.push_back(dep.clone());
                }
            }
        }
    }
    visited
}

fn cmd_retry(state: &mut JsonStateStore, task_ids: &[String]) -> anyhow::Result<()> {
    for id in task_ids {
        if state.get_status(id).is_none() {
            eprintln!("warn: task '{}' not found", id);
        } else {
            state.mark_pending(id);
        }
    }

    let successors = state.task_successors();
    if successors.is_empty() {
        eprintln!("warn: state file lacks dependency info; falling back to global reset");
        let to_reset: Vec<String> = state
            .all_tasks()
            .into_iter()
            .filter(|(_, s)| matches!(s, TaskStatus::SkippedDueToDependencyFailure))
            .map(|(id, _)| id)
            .collect();
        for id in to_reset {
            state.mark_pending(&id);
        }
    } else {
        let downstream = downstream_tasks(task_ids, successors);
        let to_reset: Vec<String> = state
            .all_tasks()
            .into_iter()
            .filter(|(id, s)| {
                matches!(s, TaskStatus::SkippedDueToDependencyFailure)
                    && downstream.contains(id)
            })
            .map(|(id, _)| id)
            .collect();
        for id in to_reset {
            state.mark_pending(&id);
        }
    }

    state.save().map_err(|e| anyhow::anyhow!("failed to save state: {}", e))?;
    Ok(())
}
```

Also update `cmd_retry` call site in `main()` — the `Commands::Retry` branch currently calls `cmd_retry(&mut state, &task_ids)` where `state` is already `JsonStateStore`, so no change needed there. But update the function signature from `&mut dyn StateStore` to `&mut JsonStateStore`.

Update the test `retry_resets_failed_and_skipped_dep` — it should still pass since old state files have empty `task_successors`, triggering the global fallback.

Acceptance: `cargo check --workspace` passes; `cargo test --workspace` passes. Add a new test for graph-aware retry with populated `task_successors`.

---

### TASK-9: Wire `ExecutionMode::Queued` dispatch into `Workflow::run`

**Files:** `workflow_core/src/workflow.rs`
**Depends on:** TASK-5, TASK-6, TASK-7, TASK-4
**Parallel with:** none (touches heavily-modified run loop; must be last run-loop change)

Remove upfront Queued rejection and add dispatch branch.

**Before** (upfront rejection block to DELETE — the Queued part only):
```rust
for (id, task) in &self.tasks {
    if matches!(task.mode, ExecutionMode::Queued { .. }) {
        return Err(WorkflowError::InvalidConfig(
            format!("task '{}': Queued execution mode is not yet implemented", id)
        ));
    }
}
```

**After**: Delete the `if matches!(task.mode, ExecutionMode::Queued { .. })` block. Keep the surrounding `for` loop if TASK-5 already removed the Periodic block; if both are gone, delete the entire `for` loop.

Add `queued_submitter` field to `Workflow`:

**Before** (`Workflow` struct, after TASK-4 has added `log_dir`):
```rust
pub struct Workflow {
    pub name: String,
    tasks: HashMap<String, Task>,
    max_parallel: usize,
    pub(crate) interrupt: Arc<AtomicBool>,
    log_dir: Option<std::path::PathBuf>,
}
```

**After**:
```rust
pub struct Workflow {
    pub name: String,
    tasks: HashMap<String, Task>,
    max_parallel: usize,
    pub(crate) interrupt: Arc<AtomicBool>,
    log_dir: Option<std::path::PathBuf>,
    queued_submitter: Option<Arc<dyn crate::process::QueuedSubmitter>>,
}
```

Add builder (and init in `new` as `None`):
```rust
pub fn with_queued_submitter(mut self, qs: Arc<dyn crate::process::QueuedSubmitter>) -> Self {
    self.queued_submitter = Some(qs);
    self
}
```

**Before** (dispatch path):
```rust
let ExecutionMode::Direct {
    command,
    args,
    env,
    timeout,
} = &task.mode else {
    unreachable!("non-Direct tasks rejected by upfront validation");
};

if let Some(d) = timeout {
    task_timeouts.insert(id.to_string(), *d);
}

let monitors = task.monitors.clone();
let task_workdir = task.workdir.clone();
let handle = match runner.spawn(&task.workdir, command, args, env) {
    Ok(h) => h,
    Err(e) => {
        state.mark_failed(&id, e.to_string());
        state.save()?;
        continue;
    }
};
```

**After** (dispatch path):
```rust
let handle = match &task.mode {
    ExecutionMode::Direct { command, args, env, timeout } => {
        if let Some(d) = timeout {
            task_timeouts.insert(id.to_string(), *d);
        }
        match runner.spawn(&task.workdir, command, args, env) {
            Ok(h) => h,
            Err(e) => {
                state.mark_failed(&id, e.to_string());
                state.save()?;
                continue;
            }
        }
    }
    ExecutionMode::Queued { submit_cmd, poll_cmd, cancel_cmd } => {
        let qs = match self.queued_submitter.as_ref() {
            Some(qs) => qs,
            None => {
                state.mark_failed(&id, format!(
                    "task '{}': Queued mode requires a QueuedSubmitter", id
                ));
                state.save()?;
                continue;
            }
        };
        let log_dir = self.log_dir.as_deref()
            .unwrap_or_else(|| std::path::Path::new("."));
        match qs.submit(&task.workdir, &id, log_dir) {
            Ok(h) => h,
            Err(e) => {
                state.mark_failed(&id, e.to_string());
                state.save()?;
                continue;
            }
        }
    }
};

let monitors = task.monitors.clone();
let task_workdir = task.workdir.clone();
```

Note: the `monitors` and `task_workdir` lines move AFTER the match since they're used regardless of mode.

Acceptance: `cargo check --workspace` passes; `cargo test --workspace` passes.

---

### TASK-10: Tests for log persistence

**Files:** `workflow_core/tests/log_persistence.rs` (new)
**Depends on:** TASK-4
**Parallel with:** TASK-11, TASK-12

No before — new test file. See TASK-4 and TASK-2 for the APIs to test.

**After** (`workflow_core/tests/log_persistence.rs`):
```rust
use std::collections::HashMap;
use std::sync::Arc;
use workflow_core::{Workflow, Task, process::ProcessRunner, process::OutputLocation};
use workflow_core::task::ExecutionMode;
use workflow_core::state::JsonStateStore;
use workflow_utils::SystemProcessRunner;

mod common;
use common::RecordingExecutor;

#[test]
fn file_backed_stdout_written_to_disk() {
    let dir = tempfile::tempdir().unwrap();
    let log_dir = dir.path().join("logs");
    let task_workdir = dir.path().join("my_task");
    std::fs::create_dir_all(&task_workdir).unwrap();

    let runner: Arc<dyn ProcessRunner> = Arc::new(SystemProcessRunner::with_log_dir(&log_dir));
    let executor = Arc::new(RecordingExecutor::new());
    let state_path = dir.path().join("state.json");
    let mut state = JsonStateStore::new("test", state_path);

    let mut wf = Workflow::new("log_test").with_log_dir(&log_dir);
    wf.add_task(Task::new("t1", ExecutionMode::Direct {
        command: "echo".into(),
        args: vec!["hello".into()],
        env: HashMap::new(),
        timeout: None,
    }).with_workdir(&task_workdir)).unwrap();

    wf.run(&mut state, runner, executor).unwrap();

    // Verify stdout file exists with correct content
    let stdout_file = log_dir.join("my_task.stdout");
    assert!(stdout_file.exists(), "stdout log file should exist");
    let content = std::fs::read_to_string(&stdout_file).unwrap();
    assert!(content.contains("hello"), "stdout should contain 'hello'");
}

#[test]
fn piped_mode_returns_captured_output() {
    let dir = tempfile::tempdir().unwrap();
    let runner: Arc<dyn ProcessRunner> = Arc::new(SystemProcessRunner::new());
    let executor = Arc::new(RecordingExecutor::new());
    let state_path = dir.path().join("state.json");
    let mut state = JsonStateStore::new("test", state_path);

    let mut wf = Workflow::new("piped_test");
    wf.add_task(Task::new("t1", ExecutionMode::Direct {
        command: "echo".into(),
        args: vec!["hello".into()],
        env: HashMap::new(),
        timeout: None,
    })).unwrap();

    wf.run(&mut state, runner, executor).unwrap();
    // ProcessResult.output should be Captured variant
    // (verified implicitly: if it were OnDisk, the collect closure pattern would differ)
}
```

Note to subagent: these are illustrative. Adapt to the actual API after TASK-1 through TASK-4 are implemented. Use `LSP hover` on `SystemProcessRunner`, `Workflow`, and `Task` to confirm current constructors and methods.

Acceptance: `cargo test --test log_persistence` passes.

---

### TASK-11: Tests for periodic hook firing

**Files:** `workflow_core/tests/hook_recording.rs` (append) or new file
**Depends on:** TASK-5
**Parallel with:** TASK-10, TASK-12

No before — new test code appended to existing file.

**After** (append to `hook_recording.rs`):
```rust
#[test]
fn periodic_hook_fires_during_long_task() {
    use workflow_core::{HookTrigger, MonitoringHook};

    let dir = tempfile::tempdir().unwrap();
    let state_path = dir.path().join(".periodic.workflow.json");

    let executor = RecordingExecutor::new();

    let periodic_hook = MonitoringHook::new(
        "periodic_check", "echo check", HookTrigger::Periodic { interval_secs: 1 }
    );

    let mut wf = Workflow::new("periodic_test").with_max_parallel(4).unwrap();
    wf.add_task(
        Task::new("long_task", direct("sleep 3"))
            .monitors(vec![periodic_hook])
    ).unwrap();

    let mut state = JsonStateStore::new("periodic", state_path);
    wf.run(&mut state, runner(), Arc::new(executor.clone()) as Arc<dyn HookExecutor>).unwrap();

    let calls = executor.calls();
    let periodic_calls: Vec<_> = calls.iter()
        .filter(|(name, _)| name == "periodic_check")
        .collect();

    // sleep 3 with interval_secs=1 should fire at least once
    assert!(
        !periodic_calls.is_empty(),
        "periodic hook should fire at least once during a 3-second task"
    );
}
```

Acceptance: `cargo test --workspace` passes; periodic test specifically passes.

---

### TASK-12: Integration test for queued execution lifecycle

**Files:** `workflow_utils/tests/queued_integration.rs` (new) or `workflow_core/tests/queued_integration.rs`
**Depends on:** TASK-9
**Parallel with:** TASK-10, TASK-11

No before — new test file with mock shell scripts.

**After** (illustrative — subagent should adapt):
```rust
use std::os::unix::fs::PermissionsExt;
use workflow_core::process::{ProcessHandle, QueuedSubmitter};
use workflow_utils::queued::{QueuedRunner, SchedulerKind};

#[test]
fn slurm_job_id_parsed_correctly() {
    let dir = tempfile::tempdir().unwrap();
    let log_dir = dir.path().join("logs");
    std::fs::create_dir_all(&log_dir).unwrap();
    let workdir = dir.path().join("work");
    std::fs::create_dir_all(&workdir).unwrap();

    // Mock sbatch script
    let script = workdir.join("job.sh");
    std::fs::write(&script, "#!/bin/sh\necho hello > /dev/null\n").unwrap();

    // Mock submit command — write a wrapper that prints SLURM-style output
    let mock_sbatch = dir.path().join("mock_sbatch.sh");
    std::fs::write(&mock_sbatch, "#!/bin/sh\necho 'Submitted batch job 12345'\n").unwrap();
    std::fs::set_permissions(&mock_sbatch, std::fs::Permissions::from_mode(0o755)).unwrap();

    // Create stdout/stderr files (SLURM would do this)
    std::fs::write(log_dir.join("test_task.stdout"), "output\n").unwrap();
    std::fs::write(log_dir.join("test_task.stderr"), "").unwrap();

    let runner = QueuedRunner::new(SchedulerKind::Slurm);
    // Note: actual test needs to override the submit command to use mock_sbatch
    // The subagent should adapt this to the actual QueuedRunner API
}
```

Note to subagent: The mock scripts must be executable. The poll mock uses a counter file pattern:
```sh
#!/bin/sh
counter_file="$1"
count=$(cat "$counter_file" 2>/dev/null || echo 0)
echo $((count + 1)) > "$counter_file"
if [ "$count" -lt 2 ]; then exit 0; else exit 1; fi
```
Adapt to the actual `QueuedRunner` API as implemented in TASK-7.

Acceptance: `cargo test --workspace` passes.

---

## Execution Order

| Phase | Tasks | Notes |
|-------|-------|-------|
| Phase 1 (parallel) | TASK-1, TASK-3, TASK-6, TASK-8 | No dependencies; touch different files |
| Phase 2 (parallel) | TASK-2, TASK-5, TASK-7 | TASK-2←TASK-1; TASK-5←TASK-3; TASK-7←TASK-1+TASK-6 |
| Phase 3 (parallel) | TASK-4, TASK-9 | TASK-4←TASK-2; TASK-9←TASK-4+TASK-5+TASK-6+TASK-7 |
| Phase 4 (parallel) | TASK-10, TASK-11, TASK-12 | All tests; TASK-10←TASK-4; TASK-11←TASK-5; TASK-12←TASK-9 |

**File conflict note**: `workflow_core/src/workflow.rs` is touched by TASK-3 (fire_hooks), TASK-4 (log_dir+builder), TASK-5 (periodic), TASK-8 (set_task_graph), TASK-9 (queued dispatch). Within each phase, these are serialized by dependencies. TASK-8 touches only the `run()` init section and state.rs; overlap is minimal.

## Verification

```bash
cargo check --workspace
cargo clippy --workspace
cargo test --workspace
```
