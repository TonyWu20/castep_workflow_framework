# Branch Status: `phase-5b` — 2026-04-24

## Last Fix Round

- **Fix document**: `notes/pr-reviews/phase-5b/fix-plan.toml`
- **Applied**: 2026-04-24 00:01
- **Tasks**: 2 total — 2 passed, 0 failed, 0 blocked

## Files Modified This Round

- `ARCHITECTURE.md` — Fix code blocks to match actual API (pub fields, renamed `execution_mode` → `mode`, `workdir` non-optional, updated TaskClosure type)
- `examples/hubbard_u_sweep_slurm/src/config.rs` — Add 2 missing parse_u_values test cases (empty string, negative values)

## Outstanding Issues

None — all tasks passed.

## Build Status

- **cargo check**: Passed
- **cargo clippy**: Passed (0 warnings)
- **cargo test**: Passed (96 passed, 0 failed)

## Branch Summary

Round 2 (v3) fix plan applied successfully. ARCHITECTURE.md code examples updated to match actual API signatures, and 2 missing test cases added for hubbard_u_sweep_slurm config parser.

## Diff Snapshot

### `ARCHITECTURE.md`

```diff
diff --git a/ARCHITECTURE.md b/ARCHITECTURE.md
index 6a9c0f8..b977328 100644
--- a/ARCHITECTURE.md
+++ b/ARCHITECTURE.md
@@ -137,17 +137,17 @@ impl Workflow {

 /// Task: execution unit with setup/collect closures
 pub struct Task {
-    id: String,
-    dependencies: Vec<String>,
-    execution_mode: ExecutionMode,
-    workdir: Option<PathBuf>,
-    setup: Option<TaskClosure>,
-    collect: Option<TaskClosure>,
-    monitors: Vec<MonitoringHook>,
+    pub id: String,
+    pub dependencies: Vec<String>,
+    pub mode: ExecutionMode,
+    pub workdir: PathBuf,
+    pub setup: Option<TaskClosure>,
+    pub collect: Option<TaskClosure>,
+    pub monitors: Vec<MonitoringHook>,
 }

 /// Closure type alias to avoid type_complexity lint
-pub type TaskClosure = Box<dyn Fn(&Path) -> Result<(), WorkflowError> + Send + Sync>;
+pub type TaskClosure = Box<dyn Fn(&Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static>;

 impl Task {
     /// Create task with execution mode
@@ -209,28 +209,41 @@ pub struct WorkflowSummary {
 }

 /// State storage trait (I/O boundary abstraction)
-pub trait StateStore {
-    fn load(&mut self) -> Result<(), WorkflowError>;       // crash-recovery (resets Failed/Running → Pending)
-    fn load_raw(&self) -> Result<WorkflowState, WorkflowError>; // read-only, no resets
-    fn save(&mut self) -> Result<(), WorkflowError>;
+pub trait StateStore: Send + Sync {
     fn get_status(&self, id: &str) -> Option<TaskStatus>;
     fn set_status(&mut self, id: &str, status: TaskStatus);
+    fn all_tasks(&self) -> Vec<(String, TaskStatus)>;
+    fn save(&self) -> Result<(), WorkflowError>;
 }

+/// Extension trait providing convenience wrappers (blanket-implemented over StateStore)
 pub trait StateStoreExt: StateStore {
-    /// BFS over task_successors graph from given start nodes (Phase 5B: generic S)
-    fn downstream_of<S: AsRef<str>>(&self, start: &[S]) -> Vec<String>;
+    fn mark_running(&mut self, id: &str);
+    fn mark_completed(&mut self, id: &str);
+    fn mark_failed(&mut self, id: &str, error: String);
+    // ... other convenience methods
 }

-/// JSON-backed state store with atomic writes
-pub struct JsonStateStore {
-    name: String,
-    path: PathBuf,
-    state: Option<WorkflowState>,
-}
+/// JSON-backed state store with atomic writes (write-to-temp + rename)
+pub struct JsonStateStore { /* ... */ }

 impl JsonStateStore {
     pub fn new(name: impl Into<String>, path: PathBuf) -> Self;
+
+    // crash-recovery: resets Failed/Running/SkippedDueToDependencyFailure → Pending
+    pub fn load(&mut self) -> Result<(), WorkflowError>;
+
+    // read-only inspection without crash-recovery resets (used by CLI status/inspect)
+    pub fn load_raw(&self) -> Result<WorkflowState, WorkflowError>;
+}
+
+/// Persisted successor graph for graph-aware retry (Phase 4+)
+pub struct TaskSuccessors { /* ... */ }
+
+impl TaskSuccessors {
+    /// BFS from given start IDs; returns all transitively reachable downstream IDs.
+    /// Starting IDs are NOT included. Accepts &[&str] or &[String] (Phase 5B ergonomics).
+    pub fn downstream_of<S: AsRef<str>>(&self, start: &[S]) -> HashSet<String>;
 }

 /// Error type
```

### `examples/hubbard_u_sweep_slurm/src/config.rs`

```diff
diff --git a/examples/hubbard_u_sweep_slurm/src/config.rs b/examples/hubbard_u_sweep_slurm/src/config.rs
index d50ff38..1c8f784 100644
--- a/examples/hubbard_u_sweep_slurm/src/config.rs
+++ b/examples/hubbard_u_sweep_slurm/src/config.rs
@@ -107,4 +107,17 @@ mod tests {
         let err = parse_u_values("1.0,,2.0").unwrap_err();
         assert!(err.contains("invalid"), "error should report parse failure: {err}");
     }
+
+    #[test]
+    fn parse_empty_string() {
+        // The whole input is empty (distinct from an empty token in the middle)
+        let err = parse_u_values("").unwrap_err();
+        assert!(!err.is_empty());
+    }
+
+    #[test]
+    fn parse_negative_values() {
+        let vals = parse_u_values("-1.0,2.0").unwrap();
+        assert_eq!(vals, vec![-1.0, 2.0]);
+    }
 }
```
