# Branch Status: `pre-phase-4` — 2026-04-18

## Last Fix Round
- **Fix document**: `notes/pr-reviews/pre-phase-4/fix-plan.md`
- **Applied**: 2026-04-18 13:20
- **Tasks**: 4 total — 4 passed, 0 failed, 0 blocked

## Files Modified This Round
- `examples/hubbard_u_sweep/src/main.rs` — Added return type annotation to setup closure
- `workflow_core/tests/common/mod.rs` — Added dead_code suppressions for test helpers
- `workflow_core/tests/hook_recording.rs` — Removed duplicate helper, fixed clippy warnings
- `workflow_core/src/workflow.rs` — Guarded unnecessary disk writes in propagate_skips()

## Outstanding Issues
None — all tasks passed.

## Build Status
- **cargo check**: Passed
- **cargo clippy**: Passed (after manual fix of deprecated Error::new pattern)
- **cargo test**: Passed

## Branch Summary
This branch contains fixes for pre-phase-4 review issues: compilation error in hubbard_u_sweep example, dead_code warnings in test helpers, duplicate helper function removal, and unnecessary disk I/O optimization.

## Diff Snapshot

### `examples/hubbard_u_sweep/src/main.rs`
```diff
 .setup(move |workdir| {
+        .setup(move |workdir| -> Result<(), WorkflowError> {```

### `workflow_core/src/workflow.rs`
```diff
 ) -> Result<(), WorkflowError> {
+    let mut any_skipped = false;
     let mut changed = true;
     while changed {
         changed = false;
@@ -380,12 +381,15 @@ fn propagate_skips(
             .collect();
         if !to_skip.is_empty() {
             changed = true;
+            any_skipped = true;
             for id in to_skip.iter() {
                 state.mark_skipped_due_to_dep_failure(id);
             }
         }
     }
-    state.save()?;
+    if any_skipped {
+        state.save()?;
+    }
```

### `workflow_core/tests/common/mod.rs`
```diff
+#[allow(dead_code)]
 /// A test executor that records all hook invocations.
@@ -23,6 +24,7 @@ impl Clone for RecordingExecutor {
 
 impl RecordingExecutor {
     /// Creates a new `RecordingExecutor` with an empty call log.
+    #[allow(dead_code)]
     pub fn new() -> Self {
@@ -32,6 +34,7 @@ impl RecordingExecutor {
     /// Returns a reference to the recorded calls.
+    #[allow(dead_code)]
     pub fn calls(&self) -> Vec<(String, String)> {
@@ -58,6 +61,7 @@ impl HookExecutor for RecordingExecutor {
     }
 }
+#[allow(dead_code)]
 /// Creates an `ExecutionMode::Direct` executor for test tasks.
```

### `workflow_core/tests/hook_recording.rs`
```diff
-use std::collections::HashMap;
 use std::sync::Arc;
 
-use workflow_core::{HookExecutor, process::ProcessRunner, state::{JsonStateStore, StateStore, TaskStatus}, task::ExecutionMode, Workflow, Task};
+use workflow_core::{HookExecutor, process::ProcessRunner, state::{JsonStateStore, StateStore, TaskStatus}, Workflow, Task};
 use workflow_utils::{ShellHookExecutor, SystemProcessRunner};
 
 mod common;
-use common::RecordingExecutor;
+use common::{RecordingExecutor, direct};
 
 fn runner() -> Arc<dyn ProcessRunner> { Arc::new(SystemProcessRunner) }
-fn direct(cmd: &str) -> ExecutionMode {
-    ExecutionMode::Direct { command: cmd.into(), args: vec![], env: HashMap::new(), timeout: None }
-}
 
 #[test]
 fn setup_failure_skips_dependent() {
@@ -22,7 +18,7 @@ fn setup_failure_skips_dependent() {
     wf.add_task(
         Task::new("a", direct("true"))
-            .setup(|_| -> Result<(), std::io::Error> { Err(std::io::Error::new(std::io::ErrorKind::Other, "setup failed")) })
+            .setup(|_| -> Result<(), std::io::Error> { Err(std::io::Error::other("setup failed")) })
     ).unwrap();
 
     wf.add_task(Task::new("b", direct("true")).depends_on("a")).unwrap();
@@ -54,7 +50,7 @@ fn collect_failure_does_not_fail_task() {
 
     wf.add_task(
         Task::new("a", direct("true"))
-            .collect(|_| -> Result<(), std::io::Error> { Err(std::io::Error::new(std::io::ErrorKind::Other, "collect failed")) })
+            .collect(|_| -> Result<(), std::io::Error> { Err(std::io::Error::other("collect failed")) })
     ).unwrap();
```

