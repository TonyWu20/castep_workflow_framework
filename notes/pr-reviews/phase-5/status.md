# Branch Status: `phase-5` — 2026-04-23

## Last Fix Round

- **Fix document**: `notes/pr-reviews/phase-5/fix-plan.toml`
- **Applied**: 2026-04-23
- **Tasks**: 4 total — 4 passed, 0 failed, 0 blocked

## Files Modified This Round

- `workflow_utils/src/queued.rs` — Extract hardcoded 'job.sh' literal into a pub const JOB_SCRIPT_NAME
- `workflow_utils/src/lib.rs` — Re-export JOB_SCRIPT_NAME from workflow_utils top-level lib.rs
- `examples/hubbard_u_sweep_slurm/src/config.rs` — Make parse_u_values return Result<Vec<f64>, String> instead of silently dropping unparseable values
- `examples/hubbard_u_sweep_slurm/src/main.rs` — Update parse_u_values call site in main.rs to propagate the error

## Outstanding Issues

None — all tasks passed.

## Build Status

- **cargo check**: Passed
- **cargo clippy**: Passed (0 warnings)
- **cargo test**: Skipped

## Branch Summary

Phase 5 fix round complete. All 4 tasks applied successfully: extracted the hardcoded "job.sh" string into a constant, re-exported it from workflow_utils, and made parse_u_values return Result with proper error propagation.

## Diff Snapshot

### `workflow_utils/src/queued.rs`

```diff
@@ -5,6 +5,9 @@ use std::time::{Duration, Instant};
 use workflow_core::error::WorkflowError;
 use workflow_core::process::{OutputLocation, ProcessHandle, ProcessResult};
 
+/// Default job script filename used by [`QueuedRunner::submit`].
+pub const JOB_SCRIPT_NAME: &str = "job.sh";
+
 /// The type of HPC job scheduler to target.
 #[derive(Debug, Clone, Copy)]
 pub enum SchedulerKind {
@@ -78,7 +81,7 @@ impl workflow_core::process::QueuedSubmitter for QueuedRunner {
             SchedulerKind::Pbs => Command::new("qsub"),
         }
         .args(["-o", &stdout_path.to_string_lossy(), -e", &stderr_path.to_string_lossy()])
-        .arg("job.sh")
+        .arg(JOB_SCRIPT_NAME)
         .current_dir(workdir)
         .output()
         .map_err(|e| WorkflowError::QueueSubmitFailed(e.to_string()))?;

```

### `workflow_utils/src/lib.rs`

```diff
@@ -7,5 +7,5 @@ pub use executor::{ExecutionHandle, ExecutionResult, OutputLocation, TaskExecuto
 pub use files::{copy_file, create_dir, exists, read_file, remove_dir, write_file};
 // Re-export hook types from workflow_core for backward compatibility
 pub use monitoring::ShellHookExecutor;
-pub use queued::{QueuedRunner, SchedulerKind};
+pub use queued::{QueuedRunner, SchedulerKind, JOB_SCRIPT_NAME};
 pub use workflow_core::{HookContext, HookResult, HookTrigger, MonitoringHook};

```

### `examples/hubbard_u_sweep_slurm/src/config.rs`

```diff
@@ -49,10 +49,14 @@ pub struct SweepConfig {
 }
 
 impl SweepConfig {
-    pub fn parse_u_values(&self) -> Vec<f64> {
+    pub fn parse_u_values(&self) -> Result<Vec<f64>, String> {
         self.u_values
             .split(',')
-            .filter_map(|s| s.trim().parse::<f64>().ok())
-            .collect()
+            .map(|s| {
+                s.trim()
+                    .parse::<f64>()
+                    .map_err(|e| format!("invalid U value '{}': {}", s.trim(), e))
+            })
+            .collect::<Result<Vec<_>, _>>()
     }
 }

```

### `examples/hubbard_u_sweep_slurm/src/main.rs`

```diff
@@ -22,7 +22,7 @@ use job_script::generate_job_script;
 fn main() -> Result<()> {
     workflow_core::init_default_logging().ok();
     let config = SweepConfig::parse();
-    let u_values = config.parse_u_values();
+    let u_values = config.parse_u_values().map_err(|e| anyhow::anyhow!(e))?;
 
     let seed_cell = include_str!("../seeds/ZnO.cell");
     let seed_param = include_str!("../seeds/ZnO.param");

```
