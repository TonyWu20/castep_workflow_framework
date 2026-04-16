# Branch Status: `phase-3` — 2026-04-16

## Last Fix Round
- **Fix document**: `notes/pr-reviews/phase-3/fix-plan.md` (v6)
- **Applied**: 2026-04-16
- **Tasks**: 6 total — 6 passed, 0 failed, 0 blocked

## Files Modified This Round
- `workflow_core/Cargo.toml` — added `time = { version = "0.3", features = ["formatting"] }` dependency
- `workflow_core/src/state.rs` — replaced hand-rolled calendar arithmetic with `time` crate
- `workflow_core/src/task.rs` — added doc comment to `ExecutionMode::Queued`
- `workflow_core/src/workflow.rs` — replaced anonymous tuple with `FailedTask` struct
- `workflow_core/tests/integration.rs` — updated test to use `.id` field instead of tuple destructuring
- `workflow_core/tests/timeout_integration.rs` — updated test to use `.id` and `.error` fields
- `workflow_core/src/lib.rs` — re-exported `FailedTask`, added comment to `init_default_logging`
- `workflow_utils/src/executor.rs` — added re-export comment

## Outstanding Issues
None — all tasks passed.

## Build Status
- **cargo check**: Passed
- **cargo clippy**: Passed (0 warnings)
- **cargo test**: Passed (48 tests)

## Branch Summary
Phase 3 v6 fix plan completed. Replaced hand-rolled calendar arithmetic with the `time` crate (eliminating correctness risks), added documentation to incomplete features (`ExecutionMode::Queued`), and improved API ergonomics by replacing anonymous tuples with the `FailedTask` struct. All changes compile cleanly and pass the full test suite.

## Diff Snapshot

### `workflow_core/Cargo.toml`
```diff
  thiserror = "1"
+time = { version = "0.3", features = ["formatting"] }
```

### `workflow_core/src/state.rs`
```diff
 fn now_iso8601() -> String {
-    use std::time::{SystemTime, UNIX_EPOCH};
-    let secs = SystemTime::now()
-        .duration_since(UNIX_EPOCH)
-        .unwrap_or_default()
-        .as_secs();
-    let s = secs % 60;
-    let m = (secs / 60) % 60;
-    let h = (secs / 3600) % 24;
-    let days = secs / 86400;
-    let z = days + 719468;
-    let era = z / 146097;
-    let doe = z - era * 146097;
-    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
-    let y = yoe + era * 400;
-    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
-    let mp = (5 * doy + 2) / 153;
-    let d = doy - (153 * mp + 2) / 5 + 1;
-    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
-    let y = if mo <= 2 { y + 1 } else { y };
-    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
+    use time::format_description::well_known::Rfc3339;
+    time::OffsetDateTime::now_utc()
+        .format(&Rfc3339)
+        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
 }
```

### `workflow_core/src/task.rs`
```diff
     },
+    /// Not yet implemented. Constructing a task with this mode will cause
+    /// `Workflow::run()` to return `Err(WorkflowError::InvalidConfig)`.
+    /// Reserved for future HPC queue integration (SLURM/PBS).
     Queued {
```

### `workflow_core/src/workflow.rs`
```diff
+/// A task that failed during workflow execution.
+#[derive(Debug, Clone)]
+pub struct FailedTask {
+    pub id: String,
+    pub error: String,
+}
+
 /// Summary of workflow execution results.
 #[derive(Debug, Clone)]
 pub struct WorkflowSummary {
     pub succeeded: Vec<String>,
-    pub failed: Vec<(String, String)>, // (task_id, error_message)
+    pub failed: Vec<FailedTask>,
     pub skipped: Vec<String>,
```

```diff
                 TaskStatus::Completed => succeeded.push(id),
-                TaskStatus::Failed { error } => failed.push((id, error)),
+                TaskStatus::Failed { error } => failed.push(FailedTask { id, error }),
```

### `workflow_core/tests/integration.rs`
```diff
     let summary1 = wf1.run(&mut state1, runner(), executor()).unwrap();
-    assert!(summary1.failed.iter().any(|(id, _)| id == "b"));
+    assert!(summary1.failed.iter().any(|f| f.id == "b"));
```

### `workflow_core/tests/timeout_integration.rs`
```diff
     assert!(wall_start.elapsed() < Duration::from_secs(1));
-    let (_, err) = summary.failed.iter().find(|(id, _)| id == "sleeper").expect("sleeper should fail");
-    assert!(err.contains("timed out"), "error was: {}", err);
+    let f = summary.failed.iter().find(|f| f.id == "sleeper").expect("sleeper should fail");
+    assert!(f.error.contains("timed out"), "error was: {}", f.error);
```

### `workflow_core/src/lib.rs`
```diff
 pub use workflow::{FailedTask, Workflow, WorkflowSummary};
 
+// Returns Box<dyn Error> rather than WorkflowError because tracing_subscriber's
+// SetGlobalDefaultError is not convertible to any WorkflowError variant without
+// introducing a logging-specific variant that doesn't belong in the domain error type.
 /// Initialize default tracing subscriber with env-based filtering.
```

### `workflow_utils/src/executor.rs`
```diff
+// Re-exported so consumers that only depend on `workflow_utils` can access
+// the core process/error types without a direct `workflow_core` dependency.
 pub use workflow_core::WorkflowError;
pub use workflow_core::{ProcessHandle, ProcessResult, ProcessRunner};
```

---
**Commit**: [`f31673a`](https://github.com/tony/programming/castep_workflow_framework/commit/f31673a)
**Status**: Ready for merge
```