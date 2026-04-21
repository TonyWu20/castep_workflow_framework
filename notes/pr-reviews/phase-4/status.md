# Branch Status: `phase-4` — 2026-04-21

## Last Fix Round

- **Fix document**: notes/pr-reviews/phase-4/fix-plan.toml (v2, 7 tasks)
- **Applied**: 2026-04-21 11:10
- **Tasks**: 7 total — 7 passed, 0 failed

## Files Modified This Round

- `workflow_core/src/process.rs` — Added doc comments to ProcessHandle trait (is_running, terminate, wait)
- `workflow_core/src/workflow.rs` — Default log_dir to task workdir instead of "."
- `workflow_utils/Cargo.toml` — Added serial_test dev-dependency for test isolation
- `workflow_utils/src/queued.rs` — Eliminated shell injection via sh -c, added parse_job_id unit tests, documented SchedulerKind and QueuedRunner public API
- `workflow_utils/tests/queued_integration.rs` — Added #[serial] to PATH-mutating integration tests

## Outstanding Issues

None — all 7 tasks in the v2 fix plan have been completed.

## Build Status

- **cargo check**: Passed
- **cargo clippy**: Passed
- **cargo test**: Passed (parse_job_id tests added)

## Branch Summary

Phase-4 fix round v2 completed. Addressed 7 issues: shell injection vulnerability, missing documentation for public API (ProcessHandle, SchedulerKind, QueuedRunner), log_dir default behavior, PBS test coverage, and PATH race condition in integration tests. All fixes compile and pass validation.

## Diff Snapshot

### workflow_core/src/process.rs

```diff
+/// A handle to a running (or finished) process, used to poll, wait, or terminate it.
+///
+/// Implementations must be `Send` so handles can be stored across thread boundaries.
 pub trait ProcessHandle: Send {
+    /// Returns `true` if the process is still running.
+    ///
+    /// Implementations may cache the result and only re-poll periodically.
     fn is_running(&mut self) -> bool;
+  
+    /// Requests termination of the process.
+    ///
+    /// Best-effort: the process may already have exited.
     fn terminate(&mut self) -> Result<(), WorkflowError>;
+  
+    /// Returns the process result once the process has finished.
+    ///
+    /// For queued (HPC) handles this may return immediately with `OnDisk` output
+    /// paths rather than captured output. Callers should ensure `is_running()`
+    /// has returned `false` before calling `wait()`, as behaviour when called
+    /// on a still-running process is implementation-defined.
     fn wait(&mut self) -> Result<ProcessResult, WorkflowError>;
 }
```

### workflow_core/src/workflow.rs

```diff
-                                    .unwrap_or_else(|| std::path::Path::new("."));
+                                    .unwrap_or(task.workdir.as_path());
```

### workflow_utils/Cargo.toml

```diff
 [dev-dependencies]
+serial_test = "3"
 tempfile = "3"
```

### workflow_utils/src/queued.rs

```diff
+/// The type of HPC job scheduler to target.
 #[derive(Debug, Clone, Copy)]
 pub enum SchedulerKind {
+    /// SLURM Workload Manager (`sbatch` / `squeue` / `scancel`).
     Slurm,
+    /// Portable Batch System (`qsub` / `qstat` / `qdel`).
     Pbs,
 }

+/// Submits and manages jobs via an HPC batch scheduler.
+///
+/// Implements [`QueuedSubmitter`](workflow_core::process::QueuedSubmitter) to
+/// integrate with the workflow engine's `Queued` execution mode.
 pub struct QueuedRunner {
+    /// Which scheduler dialect to use for command construction.
     pub scheduler: SchedulerKind,
 }
```

Shell injection elimination:
```diff
-        let submit_cmd = self.build_submit_cmd(...);
-        let output = Command::new("sh").args(["-c", &submit_cmd])...
+
+        let output = match self.scheduler {
+            SchedulerKind::Slurm => Command::new("sbatch"),
+            SchedulerKind::Pbs => Command::new("qsub"),
+        }
+        .args(["-o", &stdout_path.to_string_lossy(), "-e", &stderr_path.to_string_lossy()])
```

Added parse_job_id tests:
```diff
+#[cfg(test)]
+mod tests {
+    #[test]
+    fn parse_slurm_job_id_from_submit_output() { /* ... */ }
+    #[test]
+    fn parse_slurm_job_id_single_word() { /* ... */ }
+    #[test]
+    fn parse_slurm_job_id_empty_fails() { /* ... */ }
+    #[test]
+    fn parse_pbs_job_id_typical() { /* ... */ }
+    #[test]
+    fn parse_pbs_job_id_empty_fails() { /* ... */ }
+    #[test]
+    fn parse_pbs_job_id_whitespace_only_fails() { /* ... */ }
+}
```

### workflow_utils/tests/queued_integration.rs

```diff
+use serial_test::serial;
 use workflow_core::process::QueuedSubmitter;
 use workflow_utils::{QueuedRunner, SchedulerKind};
```

```diff
 #[test]
+#[serial]
 fn submit_returns_err_when_sbatch_unavailable() { /* ... */ }
```

```diff
 #[test]
+#[serial]
 fn submit_with_mock_sbatch_returns_on_disk_handle() {
     use workflow_core::process::OutputLocation;
```

