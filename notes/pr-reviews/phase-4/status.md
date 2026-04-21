# Branch Status: `phase-4` — 2026-04-21

## Last Fix Round

- **Fix document**: `notes/pr-reviews/phase-4/fix-plan.toml` (v3, 9 tasks)
- **Applied**: 2026-04-21
- **Tasks**: 9 total — 9 passed, 0 failed, 0 blocked
- **Post-fix corrections**: 2 (compile errors and missing import from plan gaps)

## Files Modified This Round

- `workflow_core/src/state.rs` — added `task_successors` field to `JsonStateStore`, `set_task_graph` default method to `StateStore` trait, `task_successors()` getter, and `set_task_graph` override on `JsonStateStore`
- `workflow_core/src/workflow.rs` — persist successor graph via `state.set_task_graph()` after `build_dag()` in `Workflow::run`
- `workflow-cli/src/main.rs` — `downstream_tasks` BFS helper + graph-aware `cmd_retry` (falls back to global reset when state file pre-dates the graph field); `cmd_retry` signature narrowed from `&mut dyn StateStore` to `&mut JsonStateStore`
- `workflow_core/src/lib.rs` — added `QueuedSubmitter` to the `pub use process::{...}` re-export line
- `workflow_utils/src/queued.rs` — removed `build_poll_cmd`/`build_cancel_cmd` shell-injection methods; replaced `poll_cmd`/`cancel_cmd` String fields with `scheduler: SchedulerKind` field in `QueuedProcessHandle`; direct `Command::new` construction in `is_running()` and `terminate()`; made `scheduler` field private with `scheduler()` getter; removed duplicate `stdout_path`/`stderr_path` computation in `submit()`; moved `#[cfg(test)] mod tests` to end of file; mapped spawn `Io` errors to `QueueSubmitFailed` in `submit()`
- `workflow_utils/src/monitoring.rs` — set `TASK_STATE` env var alongside `TASK_PHASE` for backwards compatibility with existing hook scripts
- `workflow_utils/tests/queued_integration.rs` — replaced `println!` with `panic!` in `submit_returns_err_when_sbatch_unavailable` error arm
- `workflow_core/tests/queued_workflow.rs` — new integration test: `queued_task_completes_via_workflow_run` using `StubQueuedSubmitter` + `ImmediateHandle`

## Outstanding Issues

None — all tasks passed.

## Build Status

- **cargo check**: Passed
- **cargo clippy**: Passed (0 warnings, `-D warnings`)
- **cargo test**: Passed (84/84)

## Branch Summary

Phase-4 v3 fix plan fully applied. All 9 tasks from the PR review round passed. The branch now has graph-aware CLI retry, shell injection eliminated from `QueuedRunner`, `QueuedSubmitter` properly re-exported from `workflow_core`, backwards-compatible `TASK_STATE` env var, and an integration test verifying `ExecutionMode::Queued` through `Workflow::run`. Ready for merge review.

## Diff Snapshot

### `workflow_core/src/state.rs`

```diff
diff --git a/workflow_core/src/state.rs b/workflow_core/src/state.rs
index b4a995f..16f0621 100644
--- a/workflow_core/src/state.rs
+++ b/workflow_core/src/state.rs
@@ -42,6 +42,10 @@ pub trait StateStore: Send + Sync {
     /// Persists the current state to disk.
     fn save(&self) -> Result<(), WorkflowError>;
+
+    /// Persists the task dependency graph (successors map) for graph-aware retry.
+    /// Default is a no-op; `JsonStateStore` overrides this.
+    fn set_task_graph(&mut self, _successors: HashMap<String, Vec<String>>) {}
 }

@@ -146,6 +150,8 @@ pub struct JsonStateStore {
     created_at: String,
     last_updated: String,
     tasks: HashMap<String, TaskStatus>,
+    #[serde(default)]
+    task_successors: HashMap<String, Vec<String>>,
     path: PathBuf,
 }

@@ -158,6 +164,7 @@ impl JsonStateStore {
             created_at: now.clone(),
             last_updated: now,
             tasks: HashMap::new(),
+            task_successors: HashMap::new(),
             path,
         }
     }

@@ -204,6 +211,11 @@ impl JsonStateStore {
     pub fn path(&self) -> &Path {
         &self.path
     }
+
+    /// Returns the task successor graph persisted from the last workflow run.
+    pub fn task_successors(&self) -> &HashMap<String, Vec<String>> {
+        &self.task_successors
+    }
 }

+    fn set_task_graph(&mut self, successors: HashMap<String, Vec<String>>) {
+        self.task_successors = successors;
+    }
```

### `workflow_core/src/workflow.rs`

```diff
@@ -109,6 +109,12 @@ impl Workflow {
         let dag = self.build_dag()?;

+        // Persist task dependency graph for CLI retry
+        let successors: HashMap<String, Vec<String>> = dag.task_ids()
+            .map(|id| (id.clone(), dag.successors(id)))
+            .collect();
+        state.set_task_graph(successors);
+
         // Initialize state for all tasks
```

### `workflow-cli/src/main.rs`

```diff
+use std::collections::HashMap;
+
+fn downstream_tasks(
+    start: &[String],
+    successors: &HashMap<String, Vec<String>>,
+) -> std::collections::HashSet<String> { ... BFS traversal ... }
+
-fn cmd_retry(state: &mut dyn StateStore, task_ids: &[String]) -> anyhow::Result<()> {
+fn cmd_retry(state: &mut JsonStateStore, task_ids: &[String]) -> anyhow::Result<()> {
     // graph-aware: uses state.task_successors() when available,
     // falls back to global reset for old state files
-        let _ = cmd_retry(&mut s, &["task_b".to_string()]);
+        cmd_retry(&mut s, &["task_b".to_string()]).unwrap();
```

### `workflow_utils/src/queued.rs`

```diff
-    pub scheduler: SchedulerKind,
+    scheduler: SchedulerKind,

-    fn build_poll_cmd(&self) -> String { ... }
-    fn build_cancel_cmd(&self) -> String { ... }
+    pub fn scheduler(&self) -> SchedulerKind { self.scheduler }

 // QueuedProcessHandle: poll_cmd/cancel_cmd String fields replaced by scheduler field
-    poll_cmd: String,
-    cancel_cmd: String,
+    scheduler: SchedulerKind,

 // submit: spawn errors now map to QueueSubmitFailed instead of Io
-    .map_err(WorkflowError::Io)?;
+    .map_err(|e| WorkflowError::QueueSubmitFailed(e.to_string()))?;

 // submit: duplicate path computation removed
-    let stdout_path = log_dir.join(format!("{}.stdout", task_id));
-    let stderr_path = log_dir.join(format!("{}.stderr", task_id));

 // is_running/terminate: direct Command construction, no sh -c
-    let cmd = self.poll_cmd.replace("{job_id}", &self.job_id);
-    let result = Command::new("sh").args(["-c", &cmd]).output();
+    let mut cmd = match self.scheduler { SchedulerKind::Slurm => { let mut c = Command::new("squeue"); ... } ... };
+    let result = cmd.output();

 // test module moved to end of file
```

### `workflow_utils/src/monitoring.rs`

```diff
+            // Deprecated: TASK_STATE is the old name for TASK_PHASE.
+            // Kept for backwards compatibility with existing hook scripts.
+            .env("TASK_STATE", ctx.phase.to_string().as_str())
```

### `workflow_utils/tests/queued_integration.rs`

```diff
-        Err(e) => println!("expected QueueSubmitFailed, got {:?}", e),
+        Err(e) => panic!("expected QueueSubmitFailed, got {:?}", e),
```

### `workflow_core/tests/queued_workflow.rs`

```diff
+(new file — integration test for ExecutionMode::Queued through Workflow::run)
+// StubQueuedSubmitter, ImmediateHandle, UnusedRunner, NoopHookExecutor
+fn queued_task_completes_via_workflow_run() -> Result<(), WorkflowError> { ... }
```
