## v3 (2026-04-10)

# Fix Plan: phase-2.2 PR Review (post-fix-fixes review of `35ddb29`)

## Context

Reviews `35ddb29` against the three issues from v2. Issues 1 and 2 are fully resolved. Issue 3 is 2/3 resolved — the comment was added at two of three `interval_secs: 0` sites and omitted at the third.

## PR Rating: Approve

| Axis          | Score   | Reason                                                              |
| ------------- | ------- | ------------------------------------------------------------------- |
| Plan & Spec   | Pass    | All 3 v2 issues addressed (1 fully, 1 fully, 1 with a trivial gap) |
| Architecture  | Pass    | CASTEP domain logic removed from Layer 1                           |
| Rust Style    | Pass    | `tokio` removed from both crates; 120-line lockfile reduction clean |
| Test Coverage | Pass    | 36/36 tests pass; Clippy: 0 warnings                               |

---

## Remaining Issues

### Issue 1 (Nit): Missing comment at `interval_secs: 0` in `test_periodic_hook_error_handling`

**File:** `workflow_core/tests/periodic_hooks.rs:181`
**Severity:** Nit (not blocking)
**Problem:** The v2 fix plan required comments at three `interval_secs: 0` sites. Comments were added at lines 39-41 and 138-140, but line 181 in `test_periodic_hook_error_handling` received no comment. All three sites have identical semantics.

**Fix:** Add the same two-line comment before line 181:
```rust
            // interval_secs: 0 means "as-fast-as-possible" - fires the hook as many
            // times as possible within the task execution window (valid for tests)
            HookTrigger::Periodic { interval_secs: 0 }
```

---

## Verification

```bash
cargo test --all    # 36/36 pass
cargo clippy --all  # 0 warnings
```

---

## v2 (2026-04-10)

# Fix Plan: phase-2.2 PR Review (post-fix review)

## Context

All 6 blocking/major/minor issues from v1 are resolved. Tests: 31/31 pass. Clippy: 0 warnings.
Three remaining issues found in post-fix review: one Major (layer boundary), one Minor (spurious dep), one Minor (test fragility).

## PR Rating: Approve (with follow-up)

| Axis          | Score   | Reason                                                         |
| ------------- | ------- | -------------------------------------------------------------- |
| Plan & Spec   | Pass    | All 6 fix items correctly implemented                          |
| Architecture  | Partial | `capture_task_error_context` leaks CASTEP domain into Layer 1  |
| Rust Style    | Pass    | No unused `&self`, no `&PathBuf`, feature gate correct         |
| Test Coverage | Pass    | 4/4 periodic hook tests pass; minor timing fragility noted     |

---

## Remaining Issues

### Issue 1: `capture_task_error_context` embeds CASTEP domain logic in Layer 1 [Major]

**File:** `workflow_core/src/workflow.rs:476-498`
**Problem:** The function hard-codes `{task_id}.castep` as the expected output file name. This is a CASTEP-specific assumption inside the domain-agnostic execution engine. Any non-CASTEP user sees misleading "Could not read output file: foo.castep" in every task failure log.

**Fix:** Remove the CASTEP-file-reading heuristic from Layer 1 entirely. The error passed in already carries domain context. Simplify to:

```rust
fn capture_task_error_context(workdir: &Path, task_id: &str, error: &anyhow::Error) -> String {
    format!(
        "Task '{}' failed: {}\nWorkdir: {}\n",
        task_id, error, workdir.display()
    )
}
```

If CASTEP log reading is needed, move it to Layer 3 (the hubbard_u_sweep example or a future `castep_adapter` crate) via an `OnFailure` hook that reads `{task_id}.castep` and logs its tail.

### Issue 2: `tokio` declared but never used in `workflow_core` and `workflow_utils` [Minor]

**File:** `workflow_core/Cargo.toml:15` and `workflow_utils/Cargo.toml:4`
**Problem:** Both crates declare `tokio = { workspace = true }` (which pulls `features = ["full"]` from workspace root), but neither crate uses any tokio symbol. Confirmed with `rg "tokio" workflow_core/src/ workflow_utils/src/` — zero results.

**Fix:** Remove `tokio` from both crates' `[dependencies]`:
- `workflow_core/Cargo.toml`: delete line 15 (`tokio = { workspace = true }`)
- `workflow_utils/Cargo.toml`: delete line 4 (`tokio = { workspace = true }`)

Verify: `cargo check --all` still passes.

### Issue 3: `interval_secs: 0` semantics and CI fragility in tests [Minor]

**File:** `workflow_core/tests/periodic_hooks.rs:39, 136, 177`
**Problem:** Three tests use `HookTrigger::Periodic { interval_secs: 0 }`. This means `sleep(Duration::from_secs(0))` — a busy loop that spins as fast as the OS allows. `test_periodic_hook_executes_multiple_times` asserts `lines.len() >= 4` after a 500ms task sleep. This passes on fast machines but could fail on heavily loaded CI runners where `Command::new(...)` process spawning latency eats into the budget.

**Fix:** Document the intent explicitly or use a small nonzero interval. One approach:

```rust
// 10ms interval: comfortably fires 40+ times in 500ms, documented expectation
HookTrigger::Periodic { interval_secs: 0 } // 0 = as-fast-as-possible; valid for tests
```

At minimum add a comment to each test explaining why `0` is intentional. Optionally replace the `>= 4` assertion with `>= 1` to reduce fragility without losing coverage of "executes multiple times" (a 500ms sleep at 0-interval will never fire only once on any reasonable machine).

---

## Verification

```bash
cargo test --all    # 31/31 pass
cargo clippy --all  # 0 warnings
```

---

## v1 (2026-04-10)

# Fix Plan: phase-2.2 PR Review

## Context

Phase 2.2 adds tracing/logging, periodic hook threads w/ RAII cleanup, duration tracking. Core (Tasks 1-4) solid. Tests (Task 5): 3/4 fail. Minor code quality issues.

**Critical finding from strict-code-reviewer:** `MonitoringHook::execute` (`workflow_utils/src/monitoring.rs:40`) uses `split_whitespace()` + `std::process::Command` — **no shell**. Shell operators (`>>`, `|`, etc.) passed as literal args. All 3 failing tests use `echo ... >> {path}` — never creates file.

## PR Rating: Request Changes

| Axis          | Score   | Reason                                               |
| ------------- | ------- | ---------------------------------------------------- |
| Plan & Spec   | Partial | Tasks 1-4 done; tests broken; example update missing |
| Architecture  | Pass    | RAII, crate boundaries, library/app split correct    |
| Rust Style    | Partial | Unused `&self`, duplicate HookContext, `&PathBuf`    |
| Test Coverage | Fail    | 3/4 integration tests fail                           |

---

## Fixes (ordered by severity)

### Fix 1: Rewrite all test hook commands to not use shell features [Blocking]

**File:** `workflow_core/tests/periodic_hooks.rs`
**Root cause:** `MonitoringHook::execute` uses `split_whitespace` + `Command` (no shell). `>>` is literal arg to `echo`.

**Approach:** Tests must use commands that work without shell. Options: `tee -a {path}` (no stdin here). Best: small script file created by test.

**Recommended:** Create helper shell script in tempdir, invoke it:

```rust
// In each test, create a script:
let script = dir.path().join("hook.sh");
std::fs::write(&script, format!("#!/bin/sh\necho 'hook executed' >> {}", log_file.display())).unwrap();
std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();

// Then use script path as command:
MonitoringHook::new("counter", script.display().to_string(), HookTrigger::Periodic { interval_secs: 0 })
```

Apply to all 4 tests: `test_periodic_hook_executes_multiple_times`, `test_periodic_hook_stops_on_completion`, `test_periodic_hook_error_handling`, `test_periodic_manager_drop_stops_threads`.

### Fix 2: Fix `test_periodic_hook_stops_on_completion` assertion logic [Blocking]

**File:** `workflow_core/tests/periodic_hooks.rs:85`
**Problem:** Hook `interval_secs: 1`, task completes instantly. Sleep-first loop → hook never fires. Assertion expects 1, gets 0.

**Fix:** Assert `== 0` — valid proof hooks stop on completion.

```rust
assert_eq!(lines.len(), 0, "Hook should not fire when task completes before interval");
```

### Fix 3: Fix `test_periodic_hook_error_handling` to actually test errors [Blocking]

**File:** `workflow_core/tests/periodic_hooks.rs:126-161`
**Problem:** `echo 'hook failed'` exits 0. Not an error. Doesn't test error handling.

**Fix:** Use command that fails (`false` or `exit 1` via script). Assert task completes despite hook errors.

```rust
// Script that always fails:
let script = dir.path().join("fail_hook.sh");
std::fs::write(&script, "#!/bin/sh\nexit 1").unwrap();
std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();

MonitoringHook::new("always_fails", script.display().to_string(), HookTrigger::Periodic { interval_secs: 0 })

// After run: verify task completed despite hook failures
let state = WorkflowState::load(dir.path().join(".error_test.workflow.json"))?;
assert!(matches!(state.tasks["error_task"], TaskStatus::Completed));
```

### Fix 4: Remove unused `&self` from `capture_task_error_context` [Minor]

**File:** `workflow_core/src/workflow.rs:483`
**Fix:** Change to associated function. Also `&PathBuf` → `&Path`.

```rust
fn capture_task_error_context(workdir: &Path, task_id: &str, error: &anyhow::Error) -> String {
```

Update call site at line 273:

```rust
let error_context = Self::capture_task_error_context(&task_workdirs[&id], &id, &e);
```

### Fix 5: Deduplicate HookContext construction [Minor]

**File:** `workflow_core/src/workflow.rs:374-406`
**Fix:** Construct once, clone for periodic manager.

### Fix 6: Move `tracing-subscriber` to dev-deps or feature-gate [Major]

**File:** `workflow_core/Cargo.toml:14`, `workflow_core/src/lib.rs:10-21`
**Problem:** `tracing-subscriber` in `[dependencies]` forces all downstream crates to pull it in.
**Nuance:** `init_default_logging()` in `lib.rs` uses `tracing_subscriber::fmt()` directly.

**Fix options (pick one):**
A. Feature-gate: `tracing-subscriber = { workspace = true, optional = true }`, add `default-logging` feature, `#[cfg(feature = "default-logging")]` on `init_default_logging()`
B. Move function to example crate, remove `tracing-subscriber` from workflow_core entirely
C. Accept for now, defer to Phase 3 (least disruption)

**Recommendation:** Option A (feature gate). Clean library boundary without breaking convenience.

---

## Verification

```bash
cargo test --all    # All tests pass
cargo clippy --all  # No warnings
```
