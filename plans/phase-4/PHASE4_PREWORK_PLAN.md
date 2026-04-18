# Phase 4 Prework: Code Quality & Test Coverage

**Goal:** Correctness, ergonomics, and maintainability fixes before Phase 4 feature work begins. No observable behavior changes.

---

## Execution Phases

| Phase | Tasks | Notes |
|-------|-------|-------|
| Phase 1 (parallel) | TASK-1, TASK-4 | Independent concerns |
| Phase 2 (parallel) | TASK-2, TASK-3 | Both depend on TASK-1 |

---

## TASK-1: Widen `TaskClosure` error type

**Crate/Module:** `workflow_core/src/task.rs`  
**Depends On:** None

**What to change:**

Change the type alias:
```rust
// Before
pub type TaskClosure = Box<dyn Fn(&Path) -> Result<(), WorkflowError> + Send + Sync>;

// After
pub type TaskClosure = Box<dyn Fn(&Path) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> + Send + Sync>;
```

Update both `setup` and `collect` builder methods to be generic over the error type and box internally:
```rust
pub fn setup<F, E>(mut self, f: F) -> Self
where
    F: Fn(&Path) -> Result<(), E> + Send + Sync + 'static,
    E: std::error::Error + Send + Sync + 'static,
{
    self.setup = Some(Box::new(move |p| {
        f(p).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync + 'static>)
    }));
    self
}
```
Same pattern for `collect`. No changes needed in `workflow.rs` — it already calls `.to_string()` on closure errors.

**LSP guidance:**
- `LSP hover` on `TaskClosure` to confirm current signature before editing
- `mcp__pare-search__search` pattern `TaskClosure` across workspace to find any direct construction sites outside the builder
- Search for `.setup(` and `.collect(` call sites to confirm no caller passes a `WorkflowError`-returning closure directly
- `LSP diagnostics` on `task.rs` and `workflow.rs` after editing, before running cargo

**Acceptance Criteria:**
- `TaskClosure` alias uses `Box<dyn std::error::Error + Send + Sync + 'static>` (note: `+ 'static` required for type unification)
- Both builder methods use `map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync + 'static>)`
- Existing call sites in tests (`.setup(move |_| { ... Ok(()) })`) compile unchanged — no `map_err` needed at call sites
- `cargo check --workspace` passes with zero new errors
- `cargo test -p workflow_core` passes

---

## TASK-2: Write timeout test

**Crate/Module:** `workflow_core/src/workflow.rs` (test module only)  
**Depends On:** TASK-1

Add test `task_timeout_marks_failed` to the existing `mod tests` block. Use `StubRunner` and `StubHookExecutor` already present in the module.

```rust
#[test]
fn task_timeout_marks_failed() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let mut wf = Workflow::new("wf_timeout").with_max_parallel(4)?;
    wf.add_task(Task::new("a", ExecutionMode::Direct {
        command: "sleep".into(),
        args: vec!["10".into()],
        env: HashMap::new(),
        timeout: Some(Duration::from_millis(100)),
    }))?;
    let mut state = JsonStateStore::new("wf_timeout", dir.path().join(".wf_timeout.workflow.json"));
    let summary = wf.run(&mut state, Arc::new(StubRunner), Arc::new(StubHookExecutor))?;
    assert_eq!(summary.failed.len(), 1);
    assert!(matches!(state.get_status("a"), Some(TaskStatus::Failed { .. })));
    Ok(())
}
```

**Acceptance Criteria:**
- `summary.failed.len() == 1` and `state.get_status("a")` is `Some(TaskStatus::Failed { .. })`
- Test completes in under 1 second (verifies the terminate-then-skip-wait path executed — if `wait()` were called on the sleeping process, the test would hang for 10 seconds)
- `cargo test -p workflow_core task_timeout_marks_failed` passes

---

## TASK-3: Setup failure, collect failure, and hook firing tests

**Crate/Module:** `workflow_core/src/workflow.rs` (test module only)  
**Depends On:** TASK-1

Add `RecordingExecutor` and three tests inside `mod tests`.

**`RecordingExecutor` definition** (add once, above the new tests):
```rust
#[derive(Clone)]
struct FiredEvent {
    hook_name: String,
    trigger: crate::monitoring::HookTrigger,
}

struct RecordingExecutor {
    fired: Arc<Mutex<Vec<FiredEvent>>>,
}

impl HookExecutor for RecordingExecutor {
    fn execute_hook(
        &self,
        hook: &crate::monitoring::MonitoringHook,
        _ctx: &crate::monitoring::HookContext,
    ) -> Result<crate::monitoring::HookResult, WorkflowError> {
        self.fired.lock().unwrap().push(FiredEvent {
            hook_name: hook.name.clone(),
            trigger: hook.trigger.clone(),
        });
        Ok(crate::monitoring::HookResult { success: true, output: String::new() })
    }
}
```

**Test 1 — `setup_failure_skips_dependent`:**
- Task `"a"` with `.setup(|_| Err("setup failed".into()))` (after TASK-1, `"setup failed".into()` produces `Box<dyn Error + Send + Sync + 'static>`)
- Task `"b"` depends on `"a"`, no setup
- Assert `state.get_status("a")` is `Some(TaskStatus::Failed { .. })`
- Assert `state.get_status("b")` is `Some(TaskStatus::SkippedDueToDependencyFailure)`

**Test 2 — `collect_failure_does_not_fail_task`:**
- Task `"a"` with `command: "true"` and `.collect(|_| Err("collect failed".into()))`
- Assert `state.get_status("a")` is `Some(TaskStatus::Completed)`
- Assert `summary.succeeded.contains(&"a".to_string())` (collect failure must not move task out of succeeded)

**Test 3 — `hooks_fire_on_start_complete_failure`:**
- Create `fired: Arc<Mutex<Vec<FiredEvent>>> = Arc::new(Mutex::new(vec![]))`
- Task `"a"` with `command: "true"`, attach `MonitoringHook` with `HookTrigger::OnStart` and one with `HookTrigger::OnComplete`
- Run with `RecordingExecutor { fired: Arc::clone(&fired) }`
- Assert `OnStart` and `OnComplete` entries present; `OnFailure` absent
- Repeat with `command: "false"` and `OnFailure` hook; assert `OnStart` and `OnFailure` present, `OnComplete` absent

**Notes:**
- `HookTrigger` already derives `Clone` — no change to `monitoring.rs` needed
- Do NOT modify `StubHookExecutor`
- `std::sync::Mutex` — add `use std::sync::Mutex;` inside `mod tests` if not already imported

**Acceptance Criteria:**
- All three tests pass with `cargo test -p workflow_core`
- `collect_failure_does_not_fail_task` asserts both `TaskStatus::Completed` in state AND presence in `summary.succeeded`
- LSP diagnostics on `workflow.rs` show no errors after insertion

---

## TASK-4: Extract private helpers from `Workflow::run`

**Crate/Module:** `workflow_core/src/workflow.rs` (production code only)  
**Depends On:** None

Extract four private helpers. All are free functions (no `&self` receiver) to avoid borrow conflicts with the simultaneous mutable borrows in `run()`.

**Exact signatures:**
```rust
fn poll_finished(
    handles: &mut HashMap<String, InFlightTask>,
    state: &mut dyn StateStore,
    task_timeouts: &HashMap<String, Duration>,
) -> Result<Vec<String>, WorkflowError>

fn process_finished(
    id: &str,
    t: InFlightTask,
    state: &mut dyn StateStore,
    hook_executor: &dyn HookExecutor,
) -> Result<(), WorkflowError>

fn propagate_skips(
    tasks: &HashMap<String, Task>,
    dag: &Dag,
    state: &mut dyn StateStore,
) -> Result<(), WorkflowError>

fn build_summary(state: &dyn StateStore) -> WorkflowSummary
```

**What each covers:**
- `poll_finished`: timeout check + `terminate()` + `mark_failed` + `is_running()` check → returns finished IDs
- `process_finished`: skip-if-already-failed guard, `wait()`, `mark_completed`/`mark_failed`, collect closure, `OnComplete`/`OnFailure` hook firing, `state.save()`
- `propagate_skips`: the `while changed` loop marking `SkippedDueToDependencyFailure`
- `build_summary`: iterates `all_tasks()`, builds `succeeded`/`failed`/`skipped` vecs

**`Workflow::run` loop body after extraction:**
```
interrupt check
→ poll_finished(...)? → for each finished id: process_finished(...)?
→ propagate_skips(...)?
→ dispatch block (UNCHANGED — not extracted)
→ termination check
```

**LSP guidance:**
- `LSP documentSymbol` on `workflow.rs` to enumerate all symbols before editing
- `LSP references` on `mark_failed`, `mark_completed`, `save` to confirm all call sites are captured
- Run `LSP diagnostics` after each extraction step, not all four at once

**Acceptance Criteria:**
- All four helpers exist with exact signatures above; none are `pub`
- `propagate_skips` has no `&self` receiver (free function, takes `tasks` as parameter)
- `poll_finished` and `process_finished` are called sequentially in the loop (not concurrently — borrow checker enforces this)
- Dispatch block (`for id in dag.ready_tasks(...)`) is NOT extracted
- All existing tests pass: `cargo test -p workflow_core`
- LSP diagnostics zero errors

---

## Verification

After each task:
```
cargo check --workspace
cargo clippy --workspace
cargo test --workspace
```
