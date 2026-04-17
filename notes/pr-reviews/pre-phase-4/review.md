## PR Review: `pre-phase-4` → `main`

**Rating:** Request Changes

**Summary:** Solid preparatory work: `poll_finished` extraction cleans up the run loop, new integration tests cover important behaviors (timeout, resume, hooks, domain workflow), and `StateStore` documentation is accurate. Two issues need attention before merge: the hook ordering test makes fragile assumptions about HashMap iteration order, and `interrupt_handle()` leaks `Arc<AtomicBool>` as public API when a narrower `set_interrupted()` method would suffice.

**Axis Scores:**

- Plan & Spec: Pass — Covers the planned pre-phase-4 scope: refactoring, test infrastructure, documentation, and `interrupt_handle` accessor.
- Architecture: Partial — `interrupt_handle()` exposes `Arc<AtomicBool>` (an implementation detail) in the public API; a narrower method would preserve encapsulation.
- Rust Style: Pass — `poll_finished` extraction is clean, docs are accurate, no dead code or unused imports.
- Test Coverage: Partial — Hook ordering test is fragile due to HashMap iteration nondeterminism; `hubbard_u_sweep` is a good domain smoke test but depends on a shell script in `tests/bin/`.

---

## Fix Document for Author

### TASK-1: Fragile hook ordering assertion in `hooks_fire_on_start_complete_failure`

**File:** `workflow_core/tests/hook_recording.rs`
**Severity:** Blocking
**Problem:** The test asserts `calls.len() == 4` and checks per-task call ordering by index (`success_calls[0]`, `success_calls[1]`). Tasks "success" and "failure" are independent (no dependency edge), so their dispatch order depends on `HashMap` iteration order, which is nondeterministic. If "failure" dispatches first, the per-task filtered vectors will still work, but the `calls.len() == 4` assertion could fail if timing causes the poll loop to see one task finish before the other dispatches (unlikely but possible). More critically, `success_calls[0].0 == "onstart"` relies on the filter preserving insertion order from the global vec, which only holds if both tasks' hooks interleave correctly.
**Fix:** Instead of asserting exact count and positional order, use set-based assertions:
```rust
let success_hooks: HashSet<_> = calls.iter()
    .filter(|(_, id)| id == "success")
    .map(|(name, _)| name.as_str())
    .collect();
assert!(success_hooks.contains("onstart"));
assert!(success_hooks.contains("oncomplete"));
assert!(!success_hooks.contains("onfailure"));

let failure_hooks: HashSet<_> = calls.iter()
    .filter(|(_, id)| id == "failure")
    .map(|(name, _)| name.as_str())
    .collect();
assert!(failure_hooks.contains("onstart"));
assert!(failure_hooks.contains("onfailure"));
assert!(!failure_hooks.contains("oncomplete"));
```

### TASK-2: `interrupt_handle()` leaks implementation detail as public API

**File:** `workflow_core/src/workflow.rs`
**Severity:** Major
**Problem:** Returning `Arc<AtomicBool>` couples callers to the specific interrupt mechanism. If you later change to a `tokio::sync::Notify`, an event fd, or a multi-signal enum, this is a breaking API change. The plan says "no new public API surface except `interrupt_handle()`" but the surface it adds is wider than necessary.
**Fix:** Replace with a method that hides the mechanism:
```rust
/// Requests workflow interruption (for testing signal injection).
pub fn set_interrupted(&self) {
    self.interrupt.store(true, Ordering::SeqCst);
}
```
If tests need to share the flag across threads (e.g., set it from a setup closure), keep `interrupt_handle()` but gate it behind `#[cfg(test)]` or a `#[doc(hidden)]` attribute so it is not part of the stable public API.

### TASK-3: `hubbard_u_sweep.rs` depends on shell script without `+x` guarantee

**File:** `workflow_core/tests/bin/mock_castep`
**Severity:** Minor
**Problem:** The `mock_castep` shell script must be executable. On a fresh clone (or on Windows), the execute bit may not be set, causing the test to fail with a confusing "permission denied" error. Git does track the execute bit on Unix, but CI runners or contributor setups may strip it.
**Fix:** Either (a) add a `chmod +x` call in the test setup, or (b) invoke via `sh mock_castep` instead of relying on the shebang:
```rust
command: "sh".into(),
args: vec![bin_dir.join("mock_castep").display().to_string(), "ZnO".into()],
```

### TASK-4: `state.rs` doc comment references `load`/`load_raw` but they are not trait methods

**File:** `workflow_core/src/state.rs`
**Severity:** Minor
**Problem:** The `StateStore` trait doc (lines 48-51) mentions `load` and `load_raw` and then clarifies they are `JsonStateStore`-specific. This is slightly confusing on a trait doc -- a reader scanning the trait expects the doc to describe the trait contract, not implementation-specific methods.
**Fix:** Move the "Persistence Semantics" paragraph to the `JsonStateStore` struct doc instead, and keep the trait doc focused on the trait contract (get/set/save).
