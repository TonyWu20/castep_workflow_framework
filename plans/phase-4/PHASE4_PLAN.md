# Phase 4: Real HPC Execution

**Goal:** Tasks that actually run on a SLURM/PBS cluster instead of locally.

**Theme:** A researcher writes a workflow, submits it to the cluster, and the framework handles job submission, polling, log capture, and graceful cancellation — with no manual `sbatch` scripting.

---

## Scope

**In (Phase 4):**
1. Per-task log persistence — stdout/stderr to disk; prerequisite for queued jobs
2. `HookTrigger::Periodic` — walltime/convergence monitoring during long jobs
3. `ExecutionMode::Queued` — real SLURM/PBS submission + polling
4. Graph-aware retry in CLI — correctness fix before queued jobs make it matter

**Deferred (Phase 5+):**
- Config-driven TOML/YAML workflow definition
- Result aggregation helpers
- TUI / live progress
- Multi-workflow orchestration

---

## Part 1: Per-task log persistence

**Files:**
- `workflow_core/src/process.rs`
- `workflow_utils/src/executor.rs`
- `workflow_core/src/workflow.rs`

Extend `ProcessRunner::spawn` to accept `log_dir: Option<&Path>`. When `Some`, redirect stdout/stderr to `<log_dir>/<task_id>.stdout` / `.stderr` via `Stdio::from(file)`. When `None`, keep current piped behaviour.

`ProcessResult.stdout`/`stderr` return empty strings for the file-backed case — the `collect` closure reads from disk.

No new types or crate dependencies.

---

## Part 2: `HookTrigger::Periodic`

**File:** `workflow_core/src/workflow.rs`

Add `last_periodic_fire: HashMap<String, Instant>` to `InFlightTask`. In the 50 ms poll loop, after the finished-task block, iterate in-flight tasks and fire any `Periodic { interval_secs }` hooks whose elapsed time exceeds the interval. Update `last_periodic_fire` after firing.

Remove the upfront `InvalidConfig` rejection of `Periodic` hooks.

No new types or crate dependencies.

---

## Part 3: `ExecutionMode::Queued` (SLURM/PBS)

**New file:** `workflow_utils/src/queued.rs`  
**Modified:** `workflow_core/src/task.rs`, `workflow_core/src/workflow.rs`, `workflow_utils/src/lib.rs`

### New types in `workflow_utils/src/queued.rs`

```rust
pub enum SchedulerKind { Slurm, Pbs }

pub struct QueuedProcessHandle {
    job_id: String,
    scheduler: SchedulerKind,
    workdir: PathBuf,
    stdout_path: PathBuf,
    stderr_path: PathBuf,
    finished: Option<i32>,
}

pub struct QueuedRunner {
    pub scheduler: SchedulerKind,
}
```

`QueuedProcessHandle` implements `ProcessHandle`:
- `is_running()` — `squeue -j <id> -h` (SLURM) or `qstat <id>` (PBS); non-zero exit = job gone
- `wait()` — `sacct -j <id> --format=ExitCode -n -P` (SLURM) or `qstat -x` XML (PBS); falls back to 0 if accounting unavailable
- `terminate()` — `scancel <id>` or `qdel <id>`

`QueuedRunner` implements `ProcessRunner`: submits via `sbatch`/`qsub`, parses job ID from stdout, returns `QueuedProcessHandle`.

### Layering note

`workflow_core` stays scheduler-agnostic. `ExecutionMode::Queued` holds raw `submit_cmd`/`poll_cmd`/`cancel_cmd` strings. `workflow_utils::QueuedRunner` builds those from `SchedulerKind`. This preserves the existing dependency direction.

Remove the `unreachable!()` / `InvalidConfig` rejection guard in `workflow.rs`.

---

## Part 4: Graph-aware retry in CLI

**Files:** `workflow_core/src/state.rs`, `workflow_core/src/workflow.rs`, `workflow-cli/src/main.rs`

Add `task_deps: HashMap<String, Vec<String>>` to `JsonStateStore` (backward-compatible: missing field deserializes as empty vec). Populate it in `Workflow::run` during initialization.

Update `cmd_retry` to BFS/DFS from the retried task IDs through `task_deps` (reversed: successors) and reset only downstream `SkippedDueToDependencyFailure` tasks, not all of them globally.

---

## Sequencing

```
Part 1 (log persistence)   — isolated, no risk; do first
Part 4 (graph-aware retry) — isolated, touches state + CLI only
Part 2 (Periodic hooks)    — run loop change, well-contained
Part 3 (Queued execution)  — main effort; depends on Part 1 for log files
```

---

## Invariants to preserve

- No tokio — all polling stays `std::thread::sleep` + `try_wait`
- `thiserror` in libs, `anyhow` only in binaries
- `ProcessHandle` trait signature unchanged — `QueuedProcessHandle` implements as-is
- Run loop structure in `workflow.rs` unchanged except removing two rejection guards + ~15 lines for periodic firing
- New `WorkflowError` variant at most: `QueueSubmitFailed(String)`

---

## Verification

After each part:
```
cargo check --workspace
cargo clippy --workspace
cargo test --workspace
```

After Part 3: write a minimal integration test in `tests/` that constructs a `Queued` task with a mock `sbatch` script and verifies `QueuedRunner::spawn` parses a job ID correctly.
