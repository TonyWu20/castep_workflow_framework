# Branch Status: `phase-5b-impl` — 2026-04-23

## Last Fix Round

- **Fix document**: `notes/pr-reviews/phase-5/fix-plan.toml`
- **Applied**: 2026-04-23
- **Tasks**: 10 total — 7 passed, 3 failed, 0 blocked

## Files Modified This Round

- `examples/hubbard_u_sweep_slurm/src/main.rs` — Use `JOB_SCRIPT_NAME` constant instead of hardcoded `'job.sh'`
- `workflow_utils/tests/queued_integration.rs` — Use `JOB_SCRIPT_NAME` constant in queued integration tests

## Outstanding Issues

- **TASK-1** (`downstream_of` generics): Type inference fails when `S: AsRef<str>` is not specified; tests need explicit type annotations or the method signature needs adjustment
- **TASK-2** (`ExecutionMode` changes): Test compilation fails due to conflicting `AsRef<str>` implementations from `tracing` crate; derive `Debug` may need guard against existing derives
- **TASK-10** (clippy warnings): `uninlined_format_args` lint check fails across `workflow_core`, `hubbard_u_sweep_slurm`, and `hubbard_u_sweep`; broader test suite also affected by TASK-1/2 issues

## Build Status

- **cargo check**: Passed (workspace)
- **cargo clippy**: Failed (TASK-10 only)
- **cargo test**: Failed (TASK-2, TASK-10 only)

## Branch Summary

This round replaced hardcoded `'job.sh'` with the `JOB_SCRIPT_NAME` constant in both the SLURM consumer and integration tests (TASK-1, TASK-2). The changes are trivial and compile cleanly, but three tasks failed:

1. **TASK-1** attempted to generalize `TaskSuccessors::downstream_of` to accept generic `AsRef<str>` types, but Rust's type inference struggles with multiple `AsRef<str>` impls (including from `tracing`).

2. **TASK-2** added `ExecutionMode::direct()` and `Debug`, but test compilation fails for similar type inference reasons.

3. **TASK-10** failed because the base compilation errors (TASK-1, TASK-2) prevent clippy from running cleanly.

The `JOB_SCRIPT_NAME` refactoring (TASK-1, TASK-2) is the only completed work this round.

## Diff Snapshot

### `examples/hubbard_u_sweep_slurm/src/main.rs`

```diff
diff --git a/examples/hubbard_u_sweep_slurm/src/main.rs b/examples/hubbard_u_sweep_slurm/src/main.rs
index 5ba1a99..f913d64 100644
--- a/examples/hubbard_u_sweep_slurm/src/main.rs
+++ b/examples/hubbard_u_sweep_slurm/src/main.rs
@@ -13,7 +13,7 @@ use workflow_core::workflow::Workflow;
 use workflow_core::{HookExecutor, ProcessRunner, WorkflowError};
 use workflow_utils::{
     create_dir, read_file, write_file, QueuedRunner, SchedulerKind, ShellHookExecutor,
-    SystemProcessRunner,
+    SystemProcessRunner, JOB_SCRIPT_NAME,
 };
 
 use config::SweepConfig;
@@ -83,7 +83,7 @@ fn main() -> Result<()> {
                     workdir.join(format!("{}.param", seed_name_setup)),
                     &seed_param,
                 )?;
-                write_file(workdir.join("job.sh"), &job_script)?;
+                write_file(workdir.join(JOB_SCRIPT_NAME), &job_script)?;
                 Ok(())
             })
             .collect(move |workdir| -> Result<(), WorkflowError> {
```

### `workflow_utils/tests/queued_integration.rs`

```diff
diff --git a/workflow_utils/tests/queued_integration.rs b/workflow_utils/tests/queued_integration.rs
index f671e34..a85fb6b 100644
--- a/workflow_utils/tests/queued_integration.rs
+++ b/workflow_utils/tests/queued_integration.rs
@@ -6,7 +6,7 @@
 
 use serial_test::serial;
 use workflow_core::process::QueuedSubmitter;
-use workflow_utils::{QueuedRunner, SchedulerKind};
+use workflow_utils::{QueuedRunner, SchedulerKind, JOB_SCRIPT_NAME};
 
 /// Compile-time verification that `QueuedRunner` implements `QueuedSubmitter`.
 #[test]
@@ -36,7 +36,7 @@ fn submit_returns_err_when_sbatch_unavailable() {
     std::fs::create_dir_all(&log_dir).unwrap();
     let workdir = dir.path().join("work");
     std::fs::create_dir_all(&workdir).unwrap();
-    std::fs::write(workdir.join("job.sh"), "#!/bin/sh\necho hello\n").unwrap();
+    std::fs::write(workdir.join(JOB_SCRIPT_NAME), "#!/bin/sh\necho hello\n").unwrap();
 
     // Restrict PATH to an empty directory so `sbatch` cannot be found.
     let empty_bin = dir.path().join("empty_bin");
@@ -77,7 +77,7 @@ fn submit_with_mock_sbatch_returns_on_disk_handle() {
     std::fs::create_dir_all(&log_dir).unwrap();
     let workdir = dir.path().join("work");
     std::fs::create_dir_all(&workdir).unwrap();
-    std::fs::write(workdir.join("job.sh"), "#!/bin/sh\necho hello\n").unwrap();
+    std::fs::write(workdir.join(JOB_SCRIPT_NAME), "#!/bin/sh\necho hello\n").unwrap();
 
     // Mock `sbatch` that prints a SLURM-style submission line and exits 0.
     let mock_dir = dir.path().join("mock_bin");
```
