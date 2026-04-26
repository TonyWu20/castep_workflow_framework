# Branch Status: `phase-6` — 2026-04-26

## Last Fix Round

- **Fix document**: `notes/pr-reviews/phase-6/fix-plan.toml`
- **Applied**: 2026-04-26 01:00
- **Tasks**: 5 total — 5 passed, 0 failed, 0 blocked

## Files Modified This Round

- `workflow-cli/src/main.rs` — Remove dead `task_ids.is_empty()` branch in `read_task_ids`
- `examples/hubbard_u_sweep_slurm/src/main.rs` — Change `second` parameter to `Option<&str>`; handle single-mode task IDs
- `examples/hubbard_u_sweep_slurm/Cargo.toml` — No change (already had trailing newline)
- `workflow_core/tests/collect_failure_policy.rs` — No change (already had trailing newline)
- `workflow_core/src/prelude.rs` — No change (already had trailing newline)

## Outstanding Issues

None — all tasks passed.

## Build Status

- **cargo check**: Passed
- **cargo clippy**: Passed (zero warnings)
- **cargo test**: Passed (102 tests across all crates)

## Branch Summary

Phase-6 fix round applied: removed dead code in `read_task_ids` and converted the `second` parameter in hubbard_u_sweep_slurm from `&str` to `Option<&str>` to properly handle single-mode sweep tasks. The branch builds cleanly with all tests passing.

## Diff Snapshot

### `workflow-cli/src/main.rs`

```diff
--- a/workflow-cli/src/main.rs
+++ b/workflow-cli/src/main.rs
@@ -29,7 +29,7 @@ enum Commands {
 /// - `["-"]` or empty + piped input → read stdin (one ID per line)
 /// - Empty + TTY → usage error
 fn read_task_ids(task_ids: &[String]) -> anyhow::Result<Vec<String>> {
-    if task_ids.first().map(|s| s.as_str()) == Some("-") || task_ids.is_empty() {
+    if task_ids.first().map(|s| s.as_str()) == Some("-") {
         let mut input = String::new();
         if io::stdin().is_terminal() {
```

### `examples/hubbard_u_sweep_slurm/src/main.rs`

```diff
--- a/examples/hubbard_u_sweep_slurm/src/main.rs
+++ b/examples/hubbard_u_sweep_slurm/src/main.rs
@@ -16,12 +16,18 @@ use job_script::generate_job_script;
 fn build_one_task(
     config: &SweepConfig,
     u: f64,
-    second: &str,
+    second: Option<&str>,
     seed_cell: &str,
     seed_param: &str,
 ) -> Result<Task, WorkflowError> {
-    let task_id = format!("scf_U{u:.1}_{second}");
-    let workdir = std::path::PathBuf::from(format!("runs/U{u:.1}/{second}"));
+    let task_id = match second {
+        Some(s) => format!("scf_U{u:.1}_{s}"),
+        None => format!("scf_U{u:.1}"),
+    };
+    let workdir = match second {
+        Some(s) => std::path::PathBuf::from(format!("runs/U{u:.1}/{s}")),
+        None => std::path::PathBuf::from(format!("runs/U{u:.1}")),
+    };
     let seed_cell = seed_cell.to_owned();
     let seed_param = seed_param.to_owned();
@@ -110,14 +116,20 @@ fn build_one_task(
 fn build_chain(
     config: &SweepConfig,
     u: f64,
-    second: &str,
+    second: Option<&str>,
     seed_cell: &str,
     seed_param: &str,
 ) -> Result<Vec<Task>, WorkflowError> {
     let scf = build_one_task(config, u, second, seed_cell, seed_param)?;
     // DOS task depends on SCF completing successfully
-    let dos_id = format!("dos_{second}");
-    let dos_workdir = std::path::PathBuf::from(format!("runs/U{u:.1}/{second}/dos"));
+    let dos_id = match second {
+        Some(s) => format!("dos_{s}"),
+        None => "dos".to_string(),
+    };
+    let dos_workdir = match second {
+        Some(s) => std::path::PathBuf::from(format!("runs/U{u:.1}/{s}/dos")),
+        None => std::path::PathBuf::from(format!("runs/U{u:.1}/dos")),
+    };
     let seed_name = config.seed_name.clone();
@@ -155,7 +167,7 @@ fn build_sweep_tasks(config: &SweepConfig) -> Result<Vec<Task>, anyhow::Error> {
             let mut tasks = Vec::new();
             for (u, second) in itertools::iproduct!(u_values, second_values) {
-                tasks.extend(build_chain(config, u, &second, seed_cell, seed_param)?);
+                tasks.extend(build_chain(config, u, Some(&second), seed_cell, seed_param)?);
             }
             Ok(tasks)
@@ -167,7 +179,7 @@ fn build_sweep_tasks(config: &SweepConfig) -> Result<Vec<Task>, anyhow::Error> {
             let mut tasks = Vec::new();
             for (u, second) in u_values.iter().zip(second_values.iter()) {
-                tasks.extend(build_chain(config, *u, second, seed_cell, seed_param)?);
+                tasks.extend(build_chain(config, *u, Some(second), seed_cell, seed_param)?);
             }
             Ok(tasks)
@@ -177,7 +189,7 @@ fn build_sweep_tasks(config: &SweepConfig) -> Result<Vec<Task>, anyhow::Error> {
             u_values
                 .into_iter()
-                .map(|u| build_one_task(config, u, "default", seed_cell, seed_param).map_err(Into::into))
+                .map(|u| build_one_task(config, u, None, seed_cell, seed_param).map_err(Into::into))
                 .collect()
```
