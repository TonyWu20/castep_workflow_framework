# Phase 2.2: Production Readiness - Logging & Periodic Monitoring

## Context

Phase 1 delivered working workflow orchestration with DAG execution, state persistence, and hook infrastructure (OnStart/OnComplete/OnFailure). Gap: framework lacks production readiness for multi-hour CASTEP workflows.

**Problems:**
1. Zero visibility during execution - can't see task progress
2. `HookTrigger::Periodic` defined but never executed
3. Poor error diagnostics - can't distinguish framework bugs from CASTEP input errors

**Why this matters:**
- CASTEP jobs run hours to days
- Researchers need convergence monitoring via periodic hooks (`rg "dE/ion" -A 2 ZnO.castep`)
- When failures occur, need clear blame assignment

**Solution:**
Add structured logging with `tracing`, implement periodic hook execution in background threads with RAII cleanup, enhance error context capture from CASTEP output files, track task/workflow durations.

## Implementation Plan

### 1. Add Tracing Dependencies

**Files:**
- `Cargo.toml` (workspace root)
- `workflow_core/Cargo.toml`

**Changes:**
```toml
# Cargo.toml workspace.dependencies
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }

# workflow_core/Cargo.toml dependencies
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
```

### 2. Make HookContext Clone

**File:** `workflow_utils/src/monitoring.rs`

**Current:** `HookContext` (line 21) not Clone

**Change:** Add `#[derive(Clone)]` to `HookContext` struct

**Verification:** All fields already owned types (String, PathBuf, Option<i32>) - safe to clone

### 3. Add Logging Initialization Helper

**File:** `workflow_core/src/lib.rs`

**Add public function:**
```rust
/// Initialize default tracing subscriber with env-based filtering.
/// Call once at start of main(). Controlled via RUST_LOG env var.
/// Returns error if already initialized (safe, won't panic).
pub fn init_default_logging() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .try_init()
        .map_err(|e| format!("Failed to initialize logging: {}", e).into())
}
```

**Why:** Library shouldn't auto-initialize global subscriber. Provide convenience function for users.

### 4. Add Helper Functions to workflow.rs

**File:** `workflow_core/src/workflow.rs`

**Add three private helpers:**

```rust
fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    let secs = secs % 60;
    
    if hours > 0 {
        format!("{}h {}m {}s", hours, mins, secs)
    } else if mins > 0 {
        format!("{}m {}s", mins, secs)
    } else {
        format!("{}s", secs)
    }
}

fn indent_output(output: &str) -> String {
    if output.is_empty() {
        return "<no output>".to_string();
    }
    output.lines()
        .map(|line| format!("  {}", line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn capture_task_error_context(workdir: &Path, task_id: &str, error: &anyhow::Error) -> String {
    let mut context = format!("Task '{}' failed: {}\n", task_id, error);
    
    // Try reading last 20 lines from CASTEP output
    let castep_file = workdir.join(format!("{}.castep", task_id));
    match std::fs::read_to_string(&castep_file) {
        Ok(content) => {
            let lines: Vec<&str> = content.lines().collect();
            let start = lines.len().saturating_sub(20);
            let last_lines = &lines[start..];
            
            if !last_lines.is_empty() {
                context.push_str("\nLast 20 lines of output:\n");
                for line in last_lines {
                    context.push_str(&format!("  {}\n", line));
                }
            }
        }
        Err(e) => {
            context.push_str(&format!("\nCould not read output file: {}\n", e));
        }
    }
    
    context.push_str(&format!("\nWorkdir: {}\n", workdir.display()));
    context
}
```

### 5. Implement Periodic Hook Thread Management

**File:** `workflow_core/src/workflow.rs`

**Add structs near top (after imports):**

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

struct PeriodicHookHandle {
    thread: std::thread::JoinHandle<()>,
    stop_signal: Arc<AtomicBool>,
}

struct PeriodicHookManager {
    handles: HashMap<String, Vec<PeriodicHookHandle>>,
}

impl PeriodicHookManager {
    fn new() -> Self {
        Self { handles: HashMap::new() }
    }
    
    fn spawn_for_task(
        &mut self,
        task_id: String,
        hooks: Vec<workflow_utils::MonitoringHook>,
        ctx: HookContext,
    ) {
        let mut task_handles = Vec::new();
        
        for hook in hooks {
            if let HookTrigger::Periodic { interval_secs } = hook.trigger {
                let stop = Arc::new(AtomicBool::new(false));
                let stop_clone = stop.clone();
                let hook_clone = hook.clone();
                let ctx_clone = ctx.clone();
                
                let thread = std::thread::spawn(move || {
                    while !stop_clone.load(Ordering::Relaxed) {
                        std::thread::sleep(Duration::from_secs(interval_secs));
                        if stop_clone.load(Ordering::Relaxed) {
                            break;
                        }
                        
                        match hook_clone.execute(&ctx_clone) {
                            Ok(result) => {
                                tracing::info!(
                                    hook_name = %hook_clone.name,
                                    task_id = %ctx_clone.task_id,
                                    "Hook output:\n{}",
                                    indent_output(&result.output)
                                );
                            }
                            Err(e) => {
                                tracing::warn!(
                                    hook_name = %hook_clone.name,
                                    error = %e,
                                    "Hook failed (task continues)"
                                );
                            }
                        }
                    }
                });
                
                task_handles.push(PeriodicHookHandle {
                    thread,
                    stop_signal: stop,
                });
            }
        }
        
        if !task_handles.is_empty() {
            self.handles.insert(task_id, task_handles);
        }
    }
    
    fn stop_for_task(&mut self, task_id: &str) {
        if let Some(handles) = self.handles.remove(task_id) {
            for handle in handles {
                handle.stop_signal.store(true, Ordering::Relaxed);
                let _ = handle.thread.join();
            }
        }
    }
}

impl Drop for PeriodicHookManager {
    fn drop(&mut self) {
        for (task_id, handles) in self.handles.drain() {
            tracing::debug!("Stopping periodic hooks for task: {}", task_id);
            for handle in handles {
                handle.stop_signal.store(true, Ordering::Relaxed);
                let _ = handle.thread.join();
            }
        }
    }
}
```

**Design notes:**
- One thread per periodic hook (simple, no shared scheduler complexity)
- If hook execution > interval, next run delayed (no overlap/queueing)
- Hook panics logged as warnings, thread continues (resilient)
- RAII via Drop ensures cleanup on panic/early return

### 6. Add Logging and Duration Tracking to run()

**File:** `workflow_core/src/workflow.rs` in `run()` method

**At start of run() (after line 66):**
```rust
let workflow_start = Instant::now();
let mut task_start_times: HashMap<String, Instant> = HashMap::new();
let mut periodic_manager = PeriodicHookManager::new();

tracing::info!(
    workflow_name = %self.name,
    total_tasks = self.tasks.len(),
    max_parallel = self.max_parallel,
    "Starting workflow"
);
```

**When task starts (after line 207, after mark_running):**
```rust
task_start_times.insert(id.clone(), Instant::now());

tracing::info!(
    task_id = %id,
    workdir = %task_workdirs[&id].display(),
    "Task started"
);

// Spawn periodic hooks
if let Some(hooks) = monitors.get(&id) {
    let periodic_hooks: Vec<_> = hooks
        .iter()
        .filter(|h| matches!(h.trigger, HookTrigger::Periodic { .. }))
        .cloned()
        .collect();
    
    if !periodic_hooks.is_empty() {
        let ctx = HookContext {
            task_id: id.clone(),
            workdir: task_workdirs[&id].clone(),
            state: "running".to_string(),
            exit_code: None,
        };
        periodic_manager.spawn_for_task(id.clone(), periodic_hooks, ctx);
    }
}
```

**When task completes (after line 113, in Ok branch):**
```rust
// Stop periodic hooks first
periodic_manager.stop_for_task(&id);

let duration = task_start_times.remove(&id)
    .map(|start| start.elapsed())
    .unwrap_or(Duration::from_secs(0));

tracing::info!(
    task_id = %id,
    duration_secs = duration.as_secs(),
    "Task completed in {}",
    format_duration(duration)
);
```

**When task fails (replace line 132 error handling):**
```rust
// Stop periodic hooks first
periodic_manager.stop_for_task(&id);

let duration = task_start_times.remove(&id)
    .map(|start| start.elapsed())
    .unwrap_or(Duration::from_secs(0));

let error_context = capture_task_error_context(&task_workdirs[&id], &id, &e);
tracing::error!("{}", error_context);

tracing::error!(
    task_id = %id,
    duration_secs = duration.as_secs(),
    "Task failed after {}",
    format_duration(duration)
);

s.mark_failed(&id, e.to_string());
```

**At end of run() (before line 241 return):**
```rust
let total_duration = workflow_start.elapsed();
let final_state = state.lock().unwrap();
let succeeded = final_state.tasks.values()
    .filter(|s| matches!(s, TaskStatus::Completed))
    .count();
let failed = final_state.tasks.values()
    .filter(|s| matches!(s, TaskStatus::Failed { .. }))
    .count();

tracing::info!(
    workflow_name = %self.name,
    duration_secs = total_duration.as_secs(),
    succeeded = succeeded,
    failed = failed,
    "Workflow completed in {}",
    format_duration(total_duration)
);
```

**Add DEBUG logging:**
- After `build_dag()`: `tracing::debug!("DAG execution order: {:?}", dag.topological_order());`
- In `add_task()`: `tracing::debug!("Registered task '{}' with {} dependencies", task.id, task.dependencies.len());`
- After state save: `tracing::debug!("Saved state to {}", self.state_path.display());`

### 7. Update Example

**File:** `examples/hubbard_u_sweep/src/main.rs`

**At start of main():**
```rust
// Initialize logging (control level with RUST_LOG env var)
workflow_core::init_default_logging()
    .expect("Failed to initialize logging");
```

**Add periodic hook to one task:**
```rust
// In task creation loop, add to one task:
.add_monitor(workflow_utils::MonitoringHook::new(
    "convergence",
    "rg 'dE/ion' -A 2 ZnO.castep",
    workflow_utils::HookTrigger::Periodic { interval_secs: 30 }
))
```

### 8. Add Tests

**File:** `workflow_core/tests/periodic_hooks.rs` (new)

**Test 1: Execution count**
```rust
#[test]
fn test_periodic_hook_executes_multiple_times() {
    // Task sleeps 3s, hook interval 1s
    // Expect 2-4 executions (fuzzy timing)
    // Hook writes to file, count lines after
}
```

**Test 2: Cleanup on completion**
```rust
#[test]
fn test_periodic_hook_stops_on_completion() {
    // Task completes quickly
    // Wait 2s after completion
    // Verify no new hook executions
}
```

**Test 3: RAII cleanup**
```rust
#[test]
fn test_periodic_manager_drop_stops_threads() {
    // Create manager, spawn hooks, drop
    // Verify threads stopped
}
```

## Critical Files

**To modify:**
- `Cargo.toml` - Add tracing workspace deps
- `workflow_core/Cargo.toml` - Reference tracing deps
- `workflow_core/src/lib.rs` - Add init_default_logging()
- `workflow_core/src/workflow.rs` - Add logging, periodic hooks, duration tracking
- `workflow_utils/src/monitoring.rs` - Add Clone to HookContext
- `examples/hubbard_u_sweep/src/main.rs` - Add logging init and periodic hook

**To create:**
- `workflow_core/tests/periodic_hooks.rs` - Integration tests

## Verification

**Manual testing:**
```bash
cd examples/hubbard_u_sweep
RUST_LOG=info cargo run
```

Expected output:
```
[INFO] Starting workflow 'hubbard_u_sweep' (6 tasks, max_parallel=4)
[INFO] Task started: scf_U0.0 (workdir: runs/U0.0)
[INFO] Hook output (convergence):
  dE/ion   |   1.198345E-003 |   5.000000E-005 |         eV | No
[INFO] Task completed: scf_U0.0 in 45m 23s
[INFO] Workflow completed in 2h 15m (6 succeeded, 0 failed)
```

**With DEBUG:**
```bash
RUST_LOG=debug cargo run
```

Expected additional:
```
[DEBUG] Registered task 'scf_U0.0' with 0 dependencies
[DEBUG] DAG execution order: ["scf_U0.0", "scf_U1.0", ...]
[DEBUG] Saved state to .hubbard_u_sweep.workflow.json
```

**Automated tests:**
```bash
cargo test --all
```

## Design Decisions

**Tracing over log:** Structured logging, zero-cost when disabled, env filter built-in, industry standard

**One thread per periodic hook:** Simple, no scheduler complexity. Acceptable overhead for monitoring use case.

**Sleep after execution:** Interval = time between executions, not fixed period. Acceptable for convergence monitoring.

**Error context from files:** Avoids memory bloat from storing full output. Reads CASTEP file on failure only.

**Library/application boundary:** Library emits events, application initializes subscriber. Prevents panic on multiple workflows.

**RAII cleanup:** Drop ensures threads stopped even on panic/early return. Critical for production use.

**No retry logic:** CASTEP failures mostly permanent (bad input). Resume handles transient failures. Retry deferred to future phase if needed.
