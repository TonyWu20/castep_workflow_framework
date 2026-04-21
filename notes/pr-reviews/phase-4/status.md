# Branch Status: `phase-4` — 2026-04-22

## Last Fix Round

- **Fix document**: `notes/pr-reviews/phase-4/fix-plan.toml` (v5, 4 tasks)
- **Applied**: 2026-04-22
- **Tasks**: 4 total — 4 passed, 0 failed, 0 blocked

## Files Modified This Round

- `workflow_core/src/state.rs` — Removed `inner()` dead API; added `downstream_of()` BFS method with unit tests
- `workflow_core/src/lib.rs` — Added `TaskSuccessors` to root re-exports
- `workflow-cli/src/main.rs` — Updated to use `successors.downstream_of()` instead of local function; removed unused import
- `workflow_utils/src/queued.rs` — Added exit-code semantics doc comment; fixed stale comment in `is_running()`

## Outstanding Issues

None — all tasks passed.

## Build Status

- **cargo check**: Passed
- **cargo clippy**: 1 warning (unused `TaskSuccessors` import in CLI, fixed)
- **cargo test**: Passed

## Branch Summary

Phase 4 fix plan v5 completed. All 4 tasks passed. BFS logic moved into `workflow_core`, dead API removed, re-exports updated, and `QueuedProcessHandle::wait()` semantics documented.

## Diff: `workflow-cli/src/main.rs`

```diff
diff --git a/workflow-cli/src/main.rs b/workflow-cli/src/main.rs
index f744893..41f1fa5 100644
--- a/workflow-cli/src/main.rs
+++ b/workflow-cli/src/main.rs
@@ -1,5 +1,5 @@
 use clap::{Parser, Subcommand};
-use workflow_core::state::{JsonStateStore, StateStore, StateStoreExt, TaskStatus, TaskSuccessors};
+use workflow_core::state::{JsonStateStore, StateStore, StateStoreExt, TaskStatus};
 
 #[derive(Parser)]
 #[command(name = "workflow-cli", about = "Workflow state inspection tool")]
@@ -69,23 +69,7 @@ fn cmd_inspect(state: &dyn StateStore, task_id: Option<&str>) -> anyhow::Result<
     }
 }
 
-fn downstream_tasks(
-    start: &[String],
-    successors: &TaskSuccessors,
-) -> std::collections::HashSet<String> {
-    let mut visited = std::collections::HashSet::new();
-    let mut queue: std::collections::VecDeque<String> = start.iter().cloned().collect();
-    while let Some(id) = queue.pop_front() {
-        if let Some(deps) = successors.get(&id) {
-            for dep in deps {
-                if visited.insert(dep.clone()) {
-                    queue.push_back(dep.clone());
-                }
-            }
-        }
-    }
-    visited
-}
+
 
 fn cmd_retry(state: &mut JsonStateStore, task_ids: &[String]) -> anyhow::Result<()> {
     for id in task_ids {
@@ -110,7 +94,7 @@ fn cmd_retry(state: &mut JsonStateStore, task_ids: &[String]) -> anyhow::Result<
             }
         }
         Some(successors) => {
-            let downstream = downstream_tasks(task_ids, successors);
+            let downstream = successors.downstream_of(task_ids);
             let to_reset: Vec<String> = state
                 .all_tasks()
                 .into_iter()
@@ -212,74 +196,4 @@ mod tests {
         assert!(cmd_inspect(&s, Some("nonexistent")).is_err());
     }
 
-    #[test]
-    fn downstream_linear_chain() {
-        // a -> b -> c: start [a] returns {b, c}
-        let mut map = std::collections::HashMap::new();
-        map.insert("a".into(), vec!["b".into()]);
-        map.insert("b".into(), vec!["c".into()]);
-        map.insert("c".into(), vec![]);
-        let succ = TaskSuccessors::new(map);
-        let result = downstream_tasks(&["a".into()], &succ);
-        assert_eq!(result.len(), 2);
-        assert!(result.contains("b"));
-        assert!(result.contains("c"));
-    }
-
-    #[test]
-    fn downstream_diamond() {
-        // a -> b, a -> c, b -> d, c -> d: start [a] returns {b, c, d}
-        let mut map = std::collections::HashMap::new();
-        map.insert("a".into(), vec!["b".into(), "c".into()]);
-        map.insert("b".into(), vec!["d".into()]);
-        map.insert("c".into(), vec!["d".into()]);
-        map.insert("d".into(), vec![]);
-        let succ = TaskSuccessors::new(map);
-        let result = downstream_tasks(&["a".into()], &succ);
-        assert_eq!(result.len(), 3);
-        assert!(result.contains("b"));
-        assert!(result.contains("c"));
-        assert!(result.contains("d"));
-    }
-
-    #[test]
-    fn downstream_start_not_in_map() {
-        let succ = TaskSuccessors::new(std::collections::HashMap::new());
-        let result = downstream_tasks(&["x".into()], &succ);
-        assert!(result.is_empty());
-    }
-
-    #[test]
-    fn downstream_empty_start() {
-        let mut map = std::collections::HashMap::new();
-        map.insert("a".into(), vec!["b".into()]);
-        let succ = TaskSuccessors::new(map);
-        let result = downstream_tasks(&[], &succ);
-        assert!(result.is_empty());
-    }
-
-    #[test]
-    fn downstream_multiple_starts() {
-        // a -> c, b -> c: start [a, b] returns {c}
-        let mut map = std::collections::HashMap::new();
-        map.insert("a".into(), vec!["c".into()]);
-        map.insert("b".into(), vec!["c".into()]);
-        map.insert("c".into(), vec![]);
-        let succ = TaskSuccessors::new(map);
-        let result = downstream_tasks(&["a".into(), "b".into()], &succ);
-        assert_eq!(result.len(), 1);
-        assert!(result.contains("c"));
-    }
-
-    #[test]
-    fn downstream_cycle_terminates() {
-        // a -> b -> a (cycle): BFS must terminate; visited set prevents re-enqueue
-        let mut map = std::collections::HashMap::new();
-        map.insert("a".into(), vec!["b".into()]);
-        map.insert("b".into(), vec!["a".into()]);
-        let succ = TaskSuccessors::new(map);
-        let result = downstream_tasks(&["a".into()], &succ);
-        assert!(result.contains("b"));
-        assert!(result.contains("a"));
-    }
 }
```

## Diff: `workflow_core/src/state.rs`

```diff
diff --git a/workflow_core/src/state.rs b/workflow_core/src/state.rs
index 2cf9bda..233d635 100644
--- a/workflow_core/src/state.rs
+++ b/workflow_core/src/state.rs
@@ -144,9 +144,24 @@ impl TaskSuccessors {
         self.0.is_empty()
     }
 
-    /// Returns a reference to the inner map.
-    pub fn inner(&self) -> &HashMap<String, Vec<String>> {
-        &self.0
+    /// Returns the set of all task IDs transitively reachable downstream
+    /// from the given starting task IDs via BFS over the successor graph.
+    ///
+    /// The starting task IDs themselves are NOT included in the result.
+    /// Cycle-safe: the visited set prevents re-enqueuing.
+    pub fn downstream_of(&self, start: &[String]) -> std::collections::HashSet<String> {
+        let mut visited = std::collections::HashSet::new();
+        let mut queue: std::collections::VecDeque<String> = start.iter().cloned().collect();
+        while let Some(id) = queue.pop_front() {
+            if let Some(deps) = self.get(&id) {
+                for dep in deps {
+                    if visited.insert(dep.to_owned()) {
+                        queue.push_back(dep.to_owned());
+                    }
+                }
+            }
+        }
+        visited
     }
 }
 
@@ -449,6 +464,77 @@ mod tests {
         assert_eq!(succ.get("b").unwrap(), &["d".to_string()]);
     }
 
+    #[test]
+    fn downstream_of_linear_chain() {
+        // a -> b -> c: start [a] returns {b, c}
+        let mut map = HashMap::new();
+        map.insert("a".into(), vec!["b".into()]);
+        map.insert("b".into(), vec!["c".into()]);
+        map.insert("c".into(), vec![]);
+        let succ = TaskSuccessors::new(map);
+        let result = succ.downstream_of(&["a".into()]);
+        assert_eq!(result.len(), 2);
+        assert!(result.contains("b"));
+        assert!(result.contains("c"));
+    }
+
+    #[test]
+    fn downstream_of_diamond() {
+        // a -> b, a -> c, b -> d, c -> d: start [a] returns {b, c, d}
+        let mut map = HashMap::new();
+        map.insert("a".into(), vec!["b".into(), "c".into()]);
+        map.insert("b".into(), vec!["d".into()]);
+        map.insert("c".into(), vec!["d".into()]);
+        map.insert("d".into(), vec![]);
+        let succ = TaskSuccessors::new(map);
+        let result = succ.downstream_of(&["a".into()]);
+        assert_eq!(result.len(), 3);
+        assert!(result.contains("b"));
+        assert!(result.contains("c"));
+        assert!(result.contains("d"));
+    }
+
+    #[test]
+    fn downstream_of_start_not_in_map() {
+        let succ = TaskSuccessors::new(HashMap::new());
+        let result = succ.downstream_of(&["x".into()]);
+        assert!(result.is_empty());
+    }
+
+    #[test]
+    fn downstream_of_empty_start() {
+        let mut map = HashMap::new();
+        map.insert("a".into(), vec!["b".into()]);
+        let succ = TaskSuccessors::new(map);
+        let result = succ.downstream_of(&[]);
+        assert!(result.is_empty());
+    }
+
+    #[test]
+    fn downstream_of_multiple_starts() {
+        // a -> c, b -> c: start [a, b] returns {c}
+        let mut map = HashMap::new();
+        map.insert("a".into(), vec!["c".into()]);
+        map.insert("b".into(), vec!["c".into()]);
+        map.insert("c".into(), vec![]);
+        let succ = TaskSuccessors::new(map);
+        let result = succ.downstream_of(&["a".into(), "b".into()]);
+        assert_eq!(result.len(), 1);
+        assert!(result.contains("c"));
+    }
+
+    #[test]
+    fn downstream_of_cycle_terminates() {
+        // a -> b -> a (cycle): BFS must terminate; visited set prevents re-enqueue
+        let mut map = HashMap::new();
+        map.insert("a".into(), vec!["b".into()]);
+        map.insert("b".into(), vec!["a".into()]);
+        let succ = TaskSuccessors::new(map);
+        let result = succ.downstream_of(&["a".into()]);
+        assert!(result.contains("b"));
+        assert!(result.contains("a"));
+    }
+
     #[test]
     fn old_state_file_deserializes_without_task_successors() {
         let dir = tempdir().unwrap();
```

## Diff: `workflow_utils/src/queued.rs`

```diff
diff --git a/workflow_utils/src/queued.rs b/workflow_utils/src/queued.rs
index 98ca90d..abe325c 100644
--- a/workflow_utils/src/queued.rs
+++ b/workflow_utils/src/queued.rs
@@ -138,7 +138,9 @@ impl ProcessHandle for QueuedProcessHandle {
                     && !output.stdout.is_empty();
                 self.cached_running = running;
                 if !running {
-                    self.finished_exit_code = Some(0); // default; accounting query in wait() may refine
+                    // Job no longer appears in the queue; assume success (exit code 0).
+                    // The scheduler does not provide the actual exit code at poll time.
+                    self.finished_exit_code = Some(0);
                 }
             }
             Err(_) => {
@@ -168,6 +170,17 @@ impl ProcessHandle for QueuedProcessHandle {
         Ok(())
     }
 
+    /// Returns the process result after `is_running()` has returned `false`.
+    ///
+    /// # Exit code semantics (approximate)
+    ///
+    /// - `Some(0)` — job left the scheduler queue normally (assumed success).
+    /// - `Some(-1)` — the scheduler status query command itself failed (I/O error);
+    ///   this conflates "cannot reach scheduler" with an actual -1 exit code.
+    /// - `None` — `is_running()` was never called or never transitioned to finished;
+    ///   callers should treat `None` as an unknown outcome.
+    ///
+    /// The caller in `workflow.rs` handles all three cases defensively via `unwrap_or(-1)`.
     fn wait(&mut self) -> Result<ProcessResult, WorkflowError> {
         Ok(ProcessResult {
             exit_code: self.finished_exit_code,
```
