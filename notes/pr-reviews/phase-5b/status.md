# Branch Status: `phase-5b` — 2026-04-23

## Last Fix Round

- **Fix document**: `notes/pr-reviews/phase-5b/fix-plan.toml`
- **Applied**: 2026-04-23
- **Tasks**: 7 plan tasks + 1 hotfix — 8 passed, 0 failed, 0 blocked

## Files Modified This Round

- `examples/hubbard_u_sweep_slurm/src/config.rs` — changed `2.71828` → `42.0` (approx_constant lint); inlined format args in assert messages
- `workflow_core/src/prelude.rs` — wrapped `workflow_core` in backticks in doc comment (doc_markdown lint); ensured trailing newline
- `workflow_core/src/lib.rs` — inlined format arg in `init_default_logging` (`{e}` instead of `{}`, e`)
- `examples/hubbard_u_sweep/src/main.rs` — replaced individual imports with `use workflow_utils::prelude::*`
- `examples/hubbard_u_sweep_slurm/src/main.rs` — replaced individual imports with `use workflow_utils::prelude::*`; local mode now uses `run_default()`, SLURM mode keeps manual Arc wiring
- `workflow_utils/src/lib.rs` — declared `pub mod prelude` (hotfix: module was unreachable without this declaration)
- `ARCHITECTURE.md` — marked Phases 5A/5B complete; updated Layer 3 example to use prelude and `run_default()`
- `ARCHITECTURE_STATUS.md` — marked Phases 5A/5B complete; added Phase 5B feature list; updated Next Steps

## Outstanding Issues

None — all tasks passed.

## Build Status

- **cargo check**: Passed
- **cargo clippy**: Passed (0 warnings, -D warnings)
- **cargo test**: Passed (109 passed, 0 failed, 1 ignored)

## Branch Summary

Phase 5B API ergonomics fixes are fully applied: both prelude modules are reachable, `run_default()` is wired in both examples, all clippy lints resolved, and architecture documentation reflects the completed state of Phases 1–5. The branch is ready for merge review.

## Diff Snapshot

### `examples/hubbard_u_sweep_slurm/src/config.rs`

```diff
diff --git a/examples/hubbard_u_sweep_slurm/src/config.rs b/examples/hubbard_u_sweep_slurm/src/config.rs
index 6c4b84d..d50ff38 100644
--- a/examples/hubbard_u_sweep_slurm/src/config.rs
+++ b/examples/hubbard_u_sweep_slurm/src/config.rs
@@ -92,19 +92,19 @@ mod tests {
 
     #[test]
     fn parse_single_value() {
-        let vals = parse_u_values("2.71828").unwrap();
-        assert_eq!(vals, vec![2.71828]);
+        let vals = parse_u_values("42.0").unwrap();
+        assert_eq!(vals, vec![42.0]);
     }
 
     #[test]
     fn parse_invalid_token() {
         let err = parse_u_values("1.0,abc,2.0").unwrap_err();
-        assert!(err.contains("abc"), "error should mention the invalid token: {}", err);
+        assert!(err.contains("abc"), "error should mention the invalid token: {err}");
     }
 
     #[test]
     fn parse_empty_token() {
         let err = parse_u_values("1.0,,2.0").unwrap_err();
-        assert!(err.contains("invalid"), "error should report parse failure: {}", err);
+        assert!(err.contains("invalid"), "error should report parse failure: {err}");
     }
 }
```

### `workflow_core/src/prelude.rs`

```diff
diff --git a/workflow_core/src/prelude.rs b/workflow_core/src/prelude.rs
index 6010ec9..031c860 100644
--- a/workflow_core/src/prelude.rs
+++ b/workflow_core/src/prelude.rs
@@ -1,4 +1,4 @@
-//! Convenience re-exports for common workflow_core types.
+//! Convenience re-exports for common `workflow_core` types.
 //!
 //! ```
 //! use workflow_core::prelude::*;
```

### `workflow_core/src/lib.rs`

```diff
diff --git a/workflow_core/src/lib.rs b/workflow_core/src/lib.rs
index 65aa186..933b94c 100644
--- a/workflow_core/src/lib.rs
+++ b/workflow_core/src/lib.rs
@@ -28,5 +28,5 @@ pub fn init_default_logging() -> Result<(), Box<dyn std::error::Error>> {
                 .add_directive(tracing::Level::INFO.into()),
         )
         .try_init()
-        .map_err(|e| format!("Failed to initialize logging: {}", e).into())
+        .map_err(|e| format!("Failed to initialize logging: {e}").into())
 }
```

### `examples/hubbard_u_sweep/src/main.rs`

```diff
diff --git a/examples/hubbard_u_sweep/src/main.rs b/examples/hubbard_u_sweep/src/main.rs
index 8636830..41d59cb 100644
--- a/examples/hubbard_u_sweep/src/main.rs
+++ b/examples/hubbard_u_sweep/src/main.rs
@@ -2,11 +2,7 @@ use anyhow::Result;
 use castep_cell_fmt::{format::to_string_many_spaced, parse, ToCellFile};
 use castep_cell_io::cell::species::{AtomHubbardU, HubbardU, HubbardUUnit, OrbitalU, Species};
 use castep_cell_io::CellDocument;
-use workflow_core::state::JsonStateStore;
-use workflow_core::task::{ExecutionMode, Task};
-use workflow_core::workflow::Workflow;
-use workflow_core::WorkflowError;
-use workflow_utils::{create_dir, write_file};
+use workflow_utils::prelude::*;
 
 fn main() -> Result<()> {
```

### `examples/hubbard_u_sweep_slurm/src/main.rs`

```diff
diff --git a/examples/hubbard_u_sweep_slurm/src/main.rs b/examples/hubbard_u_sweep_slurm/src/main.rs
index 1d01dd1..39baabb 100644
--- a/examples/hubbard_u_sweep_slurm/src/main.rs
+++ b/examples/hubbard_u_sweep_slurm/src/main.rs
@@ -7,14 +7,7 @@ use castep_cell_io::cell::species::{AtomHubbardU, HubbardU, HubbardUUnit, Orbit
 use castep_cell_io::CellDocument;
 use clap::Parser;
 use std::sync::Arc;
-use workflow_core::state::JsonStateStore;
-use workflow_core::task::{ExecutionMode, Task};
-use workflow_core::workflow::Workflow;
-use workflow_core::{HookExecutor, ProcessRunner, WorkflowError};
-use workflow_utils::{
-    create_dir, read_file, write_file, QueuedRunner, SchedulerKind, ShellHookExecutor,
-    SystemProcessRunner, JOB_SCRIPT_NAME,
-};
+use workflow_utils::prelude::*;
 
 use config::{parse_u_values, SweepConfig};
 use job_script::generate_job_script;
@@ -154,10 +147,15 @@ fn main() -> Result<()> {
 
     let state_path = std::path::PathBuf::from(".hubbard_u_sweep_slurm.workflow.json");
     let mut state = JsonStateStore::new("hubbard_u_sweep_slurm", state_path);
-    let runner: Arc<dyn ProcessRunner> = Arc::new(SystemProcessRunner::new());
-    let executor: Arc<dyn HookExecutor> = Arc::new(ShellHookExecutor);
 
-    let summary = workflow.run(&mut state, runner, executor)?;
+    let summary = if config.local {
+        run_default(&mut workflow, &mut state)?
+    } else {
+        let runner: Arc<dyn ProcessRunner> = Arc::new(SystemProcessRunner::new());
+        let executor: Arc<dyn HookExecutor> = Arc::new(ShellHookExecutor);
+        workflow.run(&mut state, runner, executor)?
+    };
+
     println!(
         "Workflow complete: {} succeeded, {} failed, {} skipped ({:.1}s)",
         summary.succeeded.len(),
```

### `workflow_utils/src/lib.rs`

```diff
diff --git a/workflow_utils/src/lib.rs b/workflow_utils/src/lib.rs
index 398478f..48240f1 100644
--- a/workflow_utils/src/lib.rs
+++ b/workflow_utils/src/lib.rs
@@ -1,6 +1,7 @@
 mod executor;
 mod files;
 mod monitoring;
+pub mod prelude;
 mod queued;
```
