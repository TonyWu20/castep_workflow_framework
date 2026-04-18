# TASK-11 & TASK-12: Concrete Extraction Plan

## Context

TASK-11 and TASK-12 from `plans/pre-phase-4/PLAN.md` extract five free functions from `Workflow::run()` in `workflow_core/src/workflow.rs`:

- TASK-11: `fire_hooks` (unifies two inline hook-firing blocks)
- TASK-12: `poll_finished`, `process_finished`, `propagate_skips`, `build_summary`

The goal is to reduce `run()` from ~300 lines to ~115 lines (loop body under 100), improving readability without changing behavior.

**File:** `workflow_core/src/workflow.rs`

---

## TASK-11: Extract `fire_hooks` as free function

### Deviations from PLAN.md (justified)

| PLAN.md says                        | This plan uses                | Reason                                                                                                                                                          |
| ----------------------------------- | ----------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `workdir: &Path`                    | `workdir: std::path::PathBuf` | `HookContext.workdir` is `PathBuf`; taking `&Path` forces a `.to_path_buf()` clone inside. Both callers already have owned `PathBuf`.                           |
| `state: &mut dyn StateStore` param  | omitted                       | Existing hook-firing code never touches `state`. Including it creates a dead parameter (clippy `unused_variables` warning). `state.save()` stays at call sites. |
| Returns `Result<(), WorkflowError>` | Returns `()`                  | Existing code only `tracing::warn!` on hook errors, never propagates them. Returning `Result` forces dead error handling at call sites.                         |

### The free function

Place after `impl Workflow` closing brace, before `FailedTask` struct.

```rust
/// Fires monitoring hooks that match the given trigger conditions.
///
/// Logs warnings for individual hook failures but does not propagate them.
/// The caller is responsible for calling `state.save()` afterward if needed.
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
        let should_fire = match (&hook.trigger, final_state) {
            (crate::monitoring::HookTrigger::OnStart, "running") => true,
            (crate::monitoring::HookTrigger::OnComplete, "completed") => true,
            (crate::monitoring::HookTrigger::OnFailure, "failed") => true,
            _ => false,
        };
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

**Note**: Site 1 used `"OnStart hook '{}' ..."` vs Site 2 `"Hook '{}' ..."`. Unified to `"Hook '{}' ..."` (hook name identifies the type).

### Call site 1: OnStart hooks (dispatch block)

**BEFORE** (lines 323-344):

```rust
                                // Fire OnStart hooks
                                let ctx = crate::monitoring::HookContext {
                                    task_id: id.clone(),
                                    workdir: task_workdir,
                                    state: "running".to_string(),
                                    exit_code: None,
                                };
                                for hook in &monitors {
                                    if matches!(
                                        hook.trigger,
                                        crate::monitoring::HookTrigger::OnStart
                                    ) {
                                        if let Err(e) = hook_executor.execute_hook(hook, &ctx) {
                                            tracing::warn!(
                                                "OnStart hook '{}' for task '{}' failed: {}",
                                                hook.name,
                                                id,
                                                e
                                            );
                                        }
                                    }
                                }
```

**AFTER**:

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

### Call site 2: OnComplete/OnFailure hooks (process-finished block)

**BEFORE** (lines 205-229):

```rust
                    // Fire OnComplete/OnFailure hooks
                    let ctx = crate::monitoring::HookContext {
                        task_id: id.clone(),
                        workdir: t.workdir,
                        state: final_state.to_string(),
                        exit_code,
                    };
                    for hook in &t.monitors {
                        let should_fire = matches!(
                            (&hook.trigger, final_state),
                            (crate::monitoring::HookTrigger::OnComplete, "completed")
                                | (crate::monitoring::HookTrigger::OnFailure, "failed")
                        );
                        if should_fire {
                            if let Err(e) = hook_executor.execute_hook(hook, &ctx) {
                                tracing::warn!(
                                    "Hook '{}' for task '{}' failed: {}",
                                    hook.name,
                                    id,
                                    e
                                );
                            }
                        }
                    }
                    state.save()?;
```

**AFTER**:

```rust
                    fire_hooks(
                        &t.monitors,
                        t.workdir,
                        final_state,
                        exit_code,
                        &id,
                        hook_executor.as_ref(),
                    );
                    state.save()?;
```

**Ownership**: `t.workdir` is moved (owned) into `fire_hooks`. `t.monitors` is borrowed. After this block, `t` is partially moved and goes out of scope -- no issue.

### Additional TASK-11 change: Remove `Queued => unreachable!` arm

TASK-10 merged the validation loops but left the `Queued => unreachable!` arm in the dispatch `match`. Since TASK-11 is already modifying the dispatch block for `fire_hooks`, also convert the `match &task.mode` to a `let-else` destructure.

**BEFORE** (the entire dispatch match, lines 300-357):

```rust
                        // Spawn process via runner
                        match &task.mode {
                            ExecutionMode::Direct {
                                command,
                                args,
                                env,
                                timeout,
                            } => {
                                // ... Direct body ...
                            }
                            &ExecutionMode::Queued { .. } => {
                                unreachable!("Queued tasks rejected by upfront validation");
                            }
                        }
```

**AFTER**:

```rust
                        let ExecutionMode::Direct {
                            command,
                            args,
                            env,
                            timeout,
                        } = &task.mode;

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

                        fire_hooks(
                            &monitors,
                            task_workdir,
                            "running",
                            None,
                            &id,
                            hook_executor.as_ref(),
                        );

                        handles.insert(id.to_string(), InFlightTask {
                            handle,
                            started_at: Instant::now(),
                            monitors,
                            collect: task.collect,
                            workdir: task.workdir,
                        });
```

**Note**: `let ExecutionMode::Direct { .. } = &task.mode;` compiles because `ExecutionMode` currently has only `Direct` and `Queued`, and the upfront validation rejects `Queued` before this point. If a new variant is added to `ExecutionMode`, this `let` pattern becomes a refutable pattern and the compiler will error, forcing the developer to handle it explicitly. This removes one indentation level from the dispatch body.

**Compiler behavior**: If `ExecutionMode` had `#[non_exhaustive]`, this would fail immediately. Currently it is exhaustive with two variants. The upfront `for (id, task)` loop rejects `Queued`, so only `Direct` reaches here. Rust allows irrefutable `let` on an enum when all other variants are uninhabited or when only one variant exists -- but with two variants, this is actually **refutable**. We need `let-else`:

```rust
                        let ExecutionMode::Direct {
                            command, args, env, timeout,
                        } = &task.mode else {
                            unreachable!("non-Direct tasks rejected by upfront validation");
                        };
```

This still has `unreachable!` but eliminates the match nesting and makes the code flatter.

---

## TASK-12: Extract four free functions

### Pre-condition: Add chain-skip test BEFORE extracting

Add in `mod tests` block of `workflow.rs`:

```rust
    #[test]
    fn three_task_chain_skip_propagation() -> Result<(), WorkflowError> {
        let dir = tempfile::tempdir().unwrap();
        let mut wf = Workflow::new("wf_chain_skip").with_max_parallel(4)?;

        wf.add_task(Task::new("a", ExecutionMode::Direct {
            command: "false".into(), args: vec![], env: HashMap::new(), timeout: None,
        })).unwrap();
        wf.add_task(Task::new("b", ExecutionMode::Direct {
            command: "true".into(), args: vec![], env: HashMap::new(), timeout: None,
        }).depends_on("a")).unwrap();
        wf.add_task(Task::new("c", ExecutionMode::Direct {
            command: "true".into(), args: vec![], env: HashMap::new(), timeout: None,
        }).depends_on("b")).unwrap();

        let runner: Arc<dyn ProcessRunner> = Arc::new(StubRunner);
        let executor: Arc<dyn HookExecutor> = Arc::new(StubHookExecutor);
        let state_path = dir.path().join(".wf_chain_skip.workflow.json");
        let mut state = Box::new(JsonStateStore::new("wf_chain_skip", state_path));

        wf.run(state.as_mut(), runner, executor)?;

        assert!(matches!(state.get_status("a"), Some(TaskStatus::Failed { .. })));
        assert!(matches!(state.get_status("b"), Some(TaskStatus::SkippedDueToDependencyFailure)));
        assert!(matches!(state.get_status("c"), Some(TaskStatus::SkippedDueToDependencyFailure)));
        Ok(())
    }
```

Run `cargo test -p workflow_core three_task_chain_skip_propagation` -- must pass before proceeding.

### Function 1: `poll_finished`

```rust
/// Polls in-flight task handles for completion or timeout.
///
/// Returns the IDs of tasks that have finished (either naturally or via timeout).
/// Timed-out tasks are terminated and marked failed before being returned.
fn poll_finished(
    handles: &mut HashMap<String, InFlightTask>,
    task_timeouts: &HashMap<String, Duration>,
    state: &mut dyn StateStore,
) -> Result<Vec<String>, WorkflowError> {
    let mut finished: Vec<String> = Vec::new();
    for (id, t) in handles.iter_mut() {
        if let Some(&timeout) = task_timeouts.get(id) {
            if t.started_at.elapsed() >= timeout {
                t.handle.terminate().ok();
                state.mark_failed(
                    id,
                    WorkflowError::TaskTimeout(id.clone()).to_string(),
                );
                state.save()?;
                finished.push(id.clone());
                continue;
            }
        }
        if !t.handle.is_running() {
            finished.push(id.clone());
        }
    }
    Ok(finished)
}
```

**Note**: Returns `Result` because `state.save()?` in the timeout path can fail. PLAN.md says `Vec<String>` -- that's the success type.

**BEFORE** (lines 148-166 in `run()` loop):

```rust
            let mut finished: Vec<String> = Vec::new();
            for (id, t) in handles.iter_mut() { ... }
```

**AFTER**:

```rust
            let finished = poll_finished(&mut handles, &task_timeouts, state)?;
```

### Function 2: `process_finished`

```rust
/// Processes a single finished task: waits for exit, updates state, runs collect, fires hooks.
///
/// If the task is already marked as Failed (e.g., timed out by `poll_finished`),
/// returns immediately without calling `wait()`.
fn process_finished(
    id: &str,
    mut t: InFlightTask,
    state: &mut dyn StateStore,
    hook_executor: &dyn HookExecutor,
) -> Result<(), WorkflowError> {
    // Guard: skip wait() if already marked failed (e.g., timed out)
    if matches!(state.get_status(id), Some(TaskStatus::Failed { .. })) {
        return Ok(());
    }

    let (final_state, exit_code) = if let Ok(process_result) = t.handle.wait() {
        match process_result.exit_code {
            Some(0) => {
                state.mark_completed(id);
                if let Some(ref collect) = t.collect {
                    if let Err(e) = collect(&t.workdir) {
                        tracing::warn!(
                            "Collect closure for task '{}' failed: {}",
                            id,
                            e
                        );
                    }
                }
                ("completed", process_result.exit_code)
            }
            _ => {
                state.mark_failed(
                    id,
                    format!("exit code {}", process_result.exit_code.unwrap_or(-1)),
                );
                ("failed", process_result.exit_code)
            }
        }
    } else {
        state.mark_failed(id, "process terminated".to_string());
        ("failed", None)
    };

    fire_hooks(
        &t.monitors,
        t.workdir,
        final_state,
        exit_code,
        id,
        hook_executor,
    );
    state.save()?;

    Ok(())
}
```

**Critical risk**: The `Failed` guard MUST be the first check. Calling `wait()` on an already-terminated handle is undefined behavior.

**Ownership**: `collect(&t.workdir)` borrows `t.workdir` temporarily. `fire_hooks(&t.monitors, t.workdir, ...)` then borrows `t.monitors` and moves `t.workdir`. Field-level borrow analysis makes this safe (NLL).

**BEFORE** (lines 169-231 in `run()` loop):

```rust
            for id in finished {
                if let Some(mut t) = handles.remove(&id) {
                    if matches!(...) { continue; }
                    // ... 60 lines of process + hooks ...
                    state.save()?;
                }
            }
```

**AFTER**:

```rust
            for id in finished {
                if let Some(t) = handles.remove(&id) {
                    process_finished(&id, t, state, hook_executor.as_ref())?;
                }
            }
```

### Function 3: `propagate_skips`

```rust
/// Propagates skip status to tasks whose dependencies have failed or been skipped.
///
/// Runs a fixpoint loop: repeatedly finds Pending tasks with failed/skipped
/// dependencies and marks them SkippedDueToDependencyFailure until stable.
fn propagate_skips(
    dag: &Dag,
    state: &mut dyn StateStore,
    tasks: &HashMap<String, Task>,
) -> Result<(), WorkflowError> {
    let mut changed = true;
    while changed {
        changed = false;
        let to_skip: Vec<String> = dag
            .task_ids()
            .filter(|id| matches!(state.get_status(id), Some(TaskStatus::Pending)))
            .filter(|id| {
                tasks
                    .get(*id)
                    .map(|t| {
                        t.dependencies.iter().any(|dep| {
                            matches!(
                                state.get_status(dep.as_str()),
                                Some(TaskStatus::Failed { .. })
                                    | Some(TaskStatus::Skipped)
                                    | Some(TaskStatus::SkippedDueToDependencyFailure)
                            )
                        })
                    })
                    .unwrap_or(false)
            })
            .cloned()
            .collect();
        if !to_skip.is_empty() {
            changed = true;
            for id in to_skip.iter() {
                state.mark_skipped_due_to_dep_failure(id);
            }
        }
    }
    state.save()?;
    Ok(())
}
```

**Note**: `state.save()?` is INSIDE this function (moved from the call site). The original code had `state.save()?` immediately after the while-loop, logically part of the propagation.

**BEFORE** (lines 234-264):

```rust
            let mut changed = true;
            while changed { ... }
            state.save()?;
```

**AFTER**:

```rust
            propagate_skips(&dag, state, &self.tasks)?;
```

### Function 4: `build_summary`

```rust
/// Builds the workflow execution summary from final task states.
fn build_summary(state: &dyn StateStore, workflow_start: Instant) -> WorkflowSummary {
    let mut succeeded = Vec::new();
    let mut failed = Vec::new();
    let mut skipped = Vec::new();

    for (id, status) in state.all_tasks() {
        match status {
            TaskStatus::Completed => succeeded.push(id),
            TaskStatus::Failed { error } => failed.push(FailedTask { id, error }),
            TaskStatus::Skipped | TaskStatus::SkippedDueToDependencyFailure => {
                skipped.push(id)
            }
            _ => {}
        }
    }

    WorkflowSummary {
        succeeded,
        failed,
        skipped,
        duration: workflow_start.elapsed(),
    }
}
```

**Note**: Takes `&dyn StateStore` (not `&mut`) -- read-only. Returns `WorkflowSummary` directly (infallible, no `state.save()`).

**BEFORE** (lines 380-401):

```rust
        let mut succeeded = Vec::new();
        // ... 20 lines ...
        Ok(WorkflowSummary { succeeded, failed, skipped, duration: workflow_start.elapsed() })
```

**AFTER**:

```rust
        Ok(build_summary(state, workflow_start))
```

---

## Final `run()` loop body (after all extractions)

```rust
        loop {
            // Interrupt check
            if self.interrupt.load(Ordering::SeqCst) {
                for id in handles.keys() {
                    state.set_status(id, TaskStatus::Pending);
                }
                for (_, t) in handles.iter_mut() {
                    t.handle.terminate().ok();
                }
                state.save()?;
                return Err(WorkflowError::Interrupted);
            }

            let finished = poll_finished(&mut handles, &task_timeouts, state)?;
            for id in finished {
                if let Some(t) = handles.remove(&id) {
                    process_finished(&id, t, state, hook_executor.as_ref())?;
                }
            }

            propagate_skips(&dag, state, &self.tasks)?;

            // Dispatch ready tasks
            let done_set: HashSet<String> = state
                .all_tasks()
                .into_iter()
                .filter(|(_, v)| {
                    matches!(
                        v,
                        TaskStatus::Completed
                            | TaskStatus::Skipped
                            | TaskStatus::SkippedDueToDependencyFailure
                    )
                })
                .map(|(k, _)| k)
                .collect();

            for id in dag.ready_tasks(&done_set) {
                if handles.len() >= self.max_parallel {
                    break;
                }
                if matches!(state.get_status(&id), Some(TaskStatus::Pending)) {
                    if let Some(task) = self.tasks.remove(&id) {
                        state.mark_running(&id);

                        if let Some(setup) = &task.setup {
                            if let Err(e) = setup(&task.workdir) {
                                state.mark_failed(&id, e.to_string());
                                state.save()?;
                                continue;
                            }
                        }

                        let ExecutionMode::Direct {
                            command, args, env, timeout,
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

                        fire_hooks(
                            &monitors,
                            task_workdir,
                            "running",
                            None,
                            &id,
                            hook_executor.as_ref(),
                        );

                        handles.insert(id.to_string(), InFlightTask {
                            handle,
                            started_at: Instant::now(),
                            monitors,
                            collect: task.collect,
                            workdir: task.workdir,
                        });
                    }
                }
            }

            let all_done = dag.task_ids().all(|id| {
                matches!(
                    state.get_status(id),
                    Some(TaskStatus::Completed)
                        | Some(TaskStatus::Failed { .. })
                        | Some(TaskStatus::Skipped)
                        | Some(TaskStatus::SkippedDueToDependencyFailure)
                )
            });

            if all_done && handles.is_empty() {
                break;
            }

            std::thread::sleep(Duration::from_millis(50));
        }
```

**Note**: The `let-else` destructure replaces the previous `match &task.mode` + `Queued => unreachable!` pattern, removing one indentation level from the dispatch body.

---

## Function placement in file

After `impl Workflow` closing brace, before `FailedTask`:

```
} // end impl Workflow

fn fire_hooks(...) { ... }
fn poll_finished(...) -> Result<Vec<String>, WorkflowError> { ... }
fn process_finished(...) -> Result<(), WorkflowError> { ... }
fn propagate_skips(...) -> Result<(), WorkflowError> { ... }
fn build_summary(...) -> WorkflowSummary { ... }

#[derive(Debug, Clone)]
pub struct FailedTask { ... }
```

No `pub` visibility. No `&self`. No new imports needed.

---

## Execution order

1. Add `three_task_chain_skip_propagation` test, verify passes
2. Add `fire_hooks` free function, replace both call sites (TASK-11)
3. Verify: `cargo test -p workflow_core && cargo clippy -p workflow_core -- -D warnings`
4. Add `poll_finished`, replace inline block
5. Verify
6. Add `process_finished`, replace inline block
7. Verify
8. Add `propagate_skips`, replace inline block
9. Verify
10. Add `build_summary`, replace inline block
11. Final: `cargo check --workspace && cargo clippy --workspace -- -D warnings && cargo test --workspace`
12. Confirm `run()` loop body is under 100 lines

---

## Risk flags

- **`process_finished` early-return guard**: Must be first check. `wait()` on terminated handle is undefined.
- **`t.workdir` borrow-then-move in `process_finished`**: `collect(&t.workdir)` borrows, then `fire_hooks(... t.workdir ...)` moves. NLL field-level analysis handles this, but reordering would break it.
- **`fire_hooks` log message change**: Site 1 said `"OnStart hook '{}' ..."`, unified to `"Hook '{}' ..."`. Acceptable -- hook name identifies type.
- **`propagate_skips` includes `state.save()`**: Moved from the call site into the function. This means the save happens even if no skips occurred (unchanged from current behavior).
