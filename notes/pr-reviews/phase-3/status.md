# Branch Status: `phase-3` — 2026-04-15

## Last Fix Round
- **Fix document**: `notes/pr-reviews/phase-3/fix-plan.md` (v4)
- **Applied**: 2026-04-15
- **Tasks**: 8 total — 8 passed, 0 failed, 0 blocked

## Files Modified This Round
- `workflow_core/src/workflow.rs` — Replaced bare tuple `TaskHandle` with named `InFlightTask` struct; added upfront validation for Queued tasks and Periodic hooks; graceful spawn error handling
- `workflow_utils/src/executor.rs` — Consolidated imports (moved `Path`, `Child`, `Command`, `Stdio`, `Instant` to top; removed duplicates)
- `workflow_core/src/state.rs` — Removed redundant `all_task_statuses()` method; updated test to use existing `all_tasks()`

## Outstanding Issues
None — all tasks passed.

## Build Status
- **cargo check**: Passed
- **cargo clippy**: Passed
- **cargo test**: 51 tests passed

## Branch Summary
Phase 3 v4 fix round complete. Eliminated orphan process hazard by moving Queued validation before DAG build; improved code ergonomics with named struct fields replacing 5-element tuples; cleaned up redundant imports and methods.

## Diff Snapshot

### `workflow_core/src/workflow.rs`
```diff
+use std::time::Instant;
+
 use crate::dag::Dag;
@@ -6,17 +8,18 @@ use crate::task::{ExecutionMode, Task, TaskClosure};
 use crate::HookExecutor;
 
 /// A handle to a running task with metadata.
-pub(crate) type TaskHandle = (
-    Box<dyn ProcessHandle>,
-    Instant,
-    Vec<crate::monitoring::MonitoringHook>,
-    Option<TaskClosure>,
-    std::path::PathBuf,
-);
+pub(crate) struct InFlightTask {
+    pub handle: Box<dyn ProcessHandle>,
+    pub started_at: Instant,
+    pub monitors: Vec<crate::monitoring::MonitoringHook>,
+    pub collect: Option<TaskClosure>,
+    pub workdir: std::path::PathBuf,
+}
+
 use std::collections::{HashMap, HashSet};
 use std::sync::atomic::{AtomicBool, Ordering};
 use std::sync::Arc;
-use std::time::{Duration, Instant};
+use std::time::Duration;
 
 pub struct Workflow {
     pub name: String,
@@ -82,8 +85,28 @@ impl Workflow {
         signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&self.interrupt)).ok();
         signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&self.interrupt)).ok();
 
+        // Reject Queued tasks upfront — before any processes are spawned — so we never orphan handles.
+        for (id, task) in &self.tasks {
+            if matches!(task.mode, ExecutionMode::Queued { .. }) {
+                return Err(WorkflowError::InvalidConfig(
+                    format!("task '{}': Queued execution mode is not yet implemented", id)
+                ));
+            }
+        }
+
         let dag = self.build_dag()?;
 
+        // Reject Periodic hooks upfront — not yet implemented in the run loop.
+        for (id, task) in &self.tasks {
+            for hook in &task.monitors {
+                if matches!(hook.trigger, crate::monitoring::HookTrigger::Periodic { .. }) {
+                    return Err(WorkflowError::InvalidConfig(
+                        format!("task '{}': Periodic hooks are not yet supported", id)
+                    ));
+                }
+            }
+        }
+
         // Initialize state for all tasks
@@ -92,7 +115,7 @@ impl Workflow {
         }
         state.save()?;
 
-        let mut handles: HashMap<String, TaskHandle> = HashMap::new();
+        let mut handles: HashMap<String, InFlightTask> = HashMap::new();
         let workflow_start = Instant::now();
 
         // Task timeout tracking
@@ -104,8 +127,8 @@ impl Workflow {
                 for id in handles.keys() {
                     state.set_status(id, TaskStatus::Pending);
                 }
-                for (_, (handle, _start, _monitors, _collect_fn, _workdir)) in handles.iter_mut() {
-                    handle.terminate().ok();
+                for (_, t) in handles.iter_mut() {
+                    t.handle.terminate().ok();
                 }
                 state.save()?;
                 return Err(WorkflowError::Interrupted);
@@ -113,11 +136,11 @@ impl Workflow {
 
             // Poll finished tasks
             let mut finished: Vec<String> = Vec::new();
-            for (id, (handle, start, _monitors, _collect_fn, _workdir)) in handles.iter_mut() {
+            for (id, t) in handles.iter_mut() {
                 // Timeout check first
                 if let Some(&timeout) = task_timeouts.get(id) {
-                    if start.elapsed() >= timeout {
-                        handle.terminate().ok();
+                    if t.started_at.elapsed() >= timeout {
+                        t.handle.terminate().ok();
                         state.mark_failed(
                             id,
@@ -127,28 +150,26 @@ impl Workflow {
                         continue;
                     }
                 }
-                if !handle.is_running() {
+                if !t.handle.is_running() {
                     finished.push(id.clone());
                 }
             }
 
             // Remove and process finished tasks
             for id in finished {
-                if let Some((mut handle, start, monitors, collect_fn, workdir)) = handles.remove(&id) {
+                if let Some(mut t) = handles.remove(&id) {
                     // Skip wait() if already marked failed (e.g., timed out)
                     if matches!(state.get_status(&id), Some(TaskStatus::Failed { .. })) {
                         continue;
                     }
 
-                    let _duration = start.elapsed();
-
                     // Execute the process and handle result
-                    let (final_state, exit_code) = if let Ok(process_result) = handle.wait() {
+                    let (final_state, exit_code) = if let Ok(process_result) = t.handle.wait() {
                         match process_result.exit_code {
                             Some(0) => {
                                 state.mark_completed(&id);
-                                if let Some(ref collect) = collect_fn {
-                                    if let Err(e) = collect(&workdir) {
+                                if let Some(ref collect) = t.collect {
+                                    if let Err(e) = collect(&t.workdir) {
                                         tracing::warn!(
                                             "Collect closure for task '{}' failed: {}",
                                             id,
@@ -174,11 +195,11 @@ impl Workflow {
                     // Fire OnComplete/OnFailure hooks
                     let ctx = crate::monitoring::HookContext {
                         task_id: id.clone(),
-                        workdir,
+                        workdir: t.workdir,
                         state: final_state.to_string(),
                         exit_code,
                     };
-                    for hook in &monitors {
+                    for hook in &t.monitors {
                         let should_fire = matches!(
                             (&hook.trigger, final_state),
@@ -280,7 +301,14 @@ impl Workflow {
 
                                 let monitors = task.monitors.clone();
                                 let task_workdir = task.workdir.clone();
-                                let handle = runner.spawn(&task.workdir, command, args, env)?;
+                                let handle = match runner.spawn(&task.workdir, command, args, env) {
+                                    Ok(h) => h,
+                                    Err(e) => {
+                                        state.mark_failed(&id, e.to_string());
+                                        state.save()?;
+                                        continue;
+                                    }
+                                };
 
                                 // Fire OnStart hooks
                                 let ctx = crate::monitoring::HookContext {
@@ -305,12 +333,16 @@ impl Workflow {
                                     }
                                 }
 
-                                handles.insert(id.clone(), (handle, Instant::now(), monitors, task.collect, task.workdir.clone()));
+                                handles.insert(id.clone(), InFlightTask {
+                                    handle,
+                                    started_at: Instant::now(),
+                                    monitors,
+                                    collect: task.collect,
+                                    workdir: task.workdir,
+                                });
                             }
                             ExecutionMode::Queued { .. } => {
                                 unreachable!("Queued tasks rejected by upfront validation");
```

### `workflow_utils/src/executor.rs`
```diff
 use std::collections::HashMap;
-use std::path::PathBuf;
+use std::path::{Path, PathBuf};
+use std::process::{Child, Command, Stdio};
+use std::time::Instant;
 
 pub use workflow_core::WorkflowError;
@@ -98,11 +103,6 @@ impl ExecutionHandle {
     }
 }
-
-
-use std::path::Path;
-use std::process::{Child, Command, Stdio};
-use std::time::Instant;
-
 pub use workflow_core::{ProcessRunner, ProcessHandle, ProcessResult};
```

### `workflow_core/src/state.rs`
```diff
 impl StateStore for JsonStateStore {
     }
 }
-
-impl JsonStateStore {
-    /// Returns all task statuses.
-    pub fn all_task_statuses(&self) -> HashMap<String, TaskStatus> {
-        self.tasks.clone()
-    }
-}
-
 fn now_iso8601() -> String {
@@ -334,7 +327,7 @@ mod tests {
         let mut s = JsonStateStore::new("test", PathBuf::from("/tmp"));
         s.mark_completed("a");
         s.mark_running("b");
-        assert_eq!(s.all_task_statuses().len(), 2);
+        assert_eq!(s.all_tasks().len(), 2);
     }
```

**git diff HEAD~1 -- workflow_core/src/workflow.rs workflow_utils/src/executor.rs workflow_core/src/state.rs**
