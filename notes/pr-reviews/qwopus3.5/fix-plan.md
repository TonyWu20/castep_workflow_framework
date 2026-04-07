## v2 (2026-04-08)

# Fix Plan: qwopus3.5 → main

## Issue 1: `bon` and `serde_json` bypass workspace dependency management

**File:** `workflow_core/Cargo.toml:10-11`
**Severity:** Minor

**Before:**
```toml
bon = "3.9.1"
serde_json = "1"
```

**After:**
```toml
bon = { workspace = true }
serde_json = { workspace = true }
```

**Verification:** `cargo build -p workflow_core` passes.

---

## Issue 2: Repeated lock comment in `run()`

**File:** `workflow_core/src/workflow.rs:103,144,179,200,220` — `run()` method
**Severity:** Minor

The comment `// task threads don't hold the lock when they panic — poisoning is not expected` appears 5 times (lines 103, 144, 179, 200, 220), once before every `lock().unwrap()` call. Keep one instance at the top of the loop body; remove the other 4.

**Verification:** `cargo clippy -p workflow_core` clean; `cargo test -p workflow_core` passes.

---

## Issue 3: `valid_dependency_add` test allocates unnecessary tempdir

**File:** `workflow_core/src/workflow.rs:361-375` — `tests::valid_dependency_add`
**Severity:** Minor

The test only calls `add_task` and never touches the filesystem. The `tempdir` is dead setup.

**Before:**
```rust
fn valid_dependency_add() -> anyhow::Result<()> {
    let dir = tempdir().unwrap();
    let mut wf = Workflow::builder()
        .name("wf_dep".to_string())
        .state_dir(dir.path().to_path_buf())
        .build()
        .unwrap();
```

**After:**
```rust
fn valid_dependency_add() -> anyhow::Result<()> {
    let mut wf = Workflow::builder()
        .name("wf_dep".to_string())
        .build()
        .unwrap();
```

(`state_dir` defaults to `"."` via `#[builder(default)]`.)

**Verification:** `cargo test -p workflow_core -- valid_dependency_add` passes.

---

## Resolved from v1 (no longer applicable)

Issues 1–5 from the prior fix plan (underscore-prefixed variables, misleading `resume()` comment, missing `pub use` re-exports, workspace dep pinning, missing `workdir()` method) are all resolved on the branch. The recurring-issue notes (R1, R2) are retained below for reference.

---

## Recurring Patterns to Avoid

### R1: Underscore-prefixed names on live variables

Only use `_foo` for genuinely unused bindings. If a variable is read anywhere, name it without the prefix. Catch with `cargo clippy`.

### R2: Removing `pub use` re-exports from `lib.rs` without replacement

`lib.rs` re-exports are the public API contract. Any removal is a breaking change. Run `cargo build --workspace` after any `lib.rs` edit to surface broken downstream imports immediately.
