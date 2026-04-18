# Branch Status: `pre-phase-4` — 2026-04-18

## Last Fix Round
- **Fix document**: `notes/pr-reviews/pre-phase-4/fix-plan.md`
- **Applied**: 2026-04-18 03:47
- **Tasks**: 4 total — 4 passed, 0 failed, 0 blocked

## Files Modified This Round
- `workflow_core/tests/hubbard_u_sweep.rs` — replaced fragile hook ordering assertions with set-based checks
- `workflow_core/src/workflow.rs` — gated interrupt_handle() behind #[cfg(test)]
- `workflow_core/tests/hubbard_u_sweep.rs` — invoke mock_castep via sh to avoid execute-bit fragility
- `workflow_core/src/state.rs` — moved impl-specific doc paragraphs from StateStore trait to JsonStateStore struct

## Outstanding Issues
None — all tasks passed.

## Build Status
- **cargo check**: Passed
- **cargo clippy**: Passed (clean)
- **cargo test**: Passed (35+ tests)

## Branch Summary
Pre-phase-4 fixes applied: 4 independent changes to improve test robustness (set-based assertions, test gating, sh invocation) and documentation organization. All changes verified with clean builds and passing tests.

## Diff Snapshot

### workflow_core/tests/hubbard_u_sweep.rs
```diff
- ExecutionMode::Direct {
-     command: "mock_castep".into(),
-     args: vec!["ZnO".into()],
-     env,
-     timeout: None,
-   },
+ ExecutionMode::Direct {
+     command: "sh".into(),
+     args: vec![
+         bin_dir.join("mock_castep").to_string_lossy().into_owned(),
+         "ZnO".into(),
+       ],
+     env,
+     timeout: None,
+   },
```

### workflow_core/src/workflow.rs
```diff
-     pub fn interrupt_handle(&self) -> Arc<AtomicBool> {
+     #[cfg(test)]
+     pub fn interrupt_handle(&self) -> Arc<AtomicBool> {
```

### workflow_core/src/state.rs
```diff
- /// JSON-based state store implementation.
+ /// JSON-based state store implementation.
+ ///
+ /// # Crash Recovery and Resume
+ ///
+ /// When loading via [`JsonStateStore::load`], any tasks marked as `Running`, `Failed`, or
+ /// `SkippedDueToDependencyFailure` are automatically reset to `Pending`. This ensures
+ /// that incomplete or failed runs can be safely resumed without stale state blocking
+ /// progress. Note that `Skipped` and `SkippedDueToDependencyFailure` (when not in
+ /// a failed context) are preserved as-is.
+ ///
+ /// # Read-Only Inspection
+ ///
+ /// For read-only status inspection (e.g., CLI display, `workflow inspect` commands),
+ /// use [`JsonStateStore::load_raw`]. Unlike `load`, this method does not apply crash
+ /// recovery resets and returns the state exactly as persisted to disk.
+ ///
+ /// # Persistence Semantics
+ ///
+ /// State is persisted to disk via atomic writes (temp file + rename). See [`JsonStateStore::load`]
+ /// and [`JsonStateStore::load_raw`] for details on crash recovery behavior.
```
