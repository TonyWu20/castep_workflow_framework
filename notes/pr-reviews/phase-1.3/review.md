# PR Review: Phase 1.3 → main

**Rating:** Request Changes

**Summary:** Phase 1.3 successfully implements all planned features - the resume bug fix, three integration tests, and the hubbard_u_sweep example. The core functionality is correct and all tests pass. However, the hubbard_u_sweep integration test violates the plan's explicit guidance by using `std::env::set_var` instead of `TaskExecutor::env()`, which can cause test interference through global state mutation.

**Axis Scores:**
- Plan & Spec: Pass — All deliverables present, minor implementation deviations
- Architecture: Pass — DAG-centric design preserved, proper crate boundaries
- Rust Style: Partial — Test code has quality issues (global state, unwrap/panic)
- Test Coverage: Pass — Adequate coverage of all scenarios, tests pass

---

## Fix Document for Author

### Issue 1: Global state mutation via std::env::set_var

**File:** `workflow_core/tests/hubbard_u_sweep.rs:30`
**Severity:** Major
**Problem:** The test uses `std::env::set_var("PATH", &path_clone)` inside the task closure, which mutates global process state. This can cause test interference when tests run in parallel. The plan explicitly states: "pass via TaskExecutor::env("PATH", &path) inside each closure" to avoid this issue.

**Fix:** Replace `std::env::set_var` with `TaskExecutor::env()`. The TaskExecutor builder pattern allows setting environment variables per-execution without global side effects.

Change from:
```rust
let task = Task::new(&task_id, move || {
    // Set PATH to include mock_castep
    std::env::set_var("PATH", &path_clone);
    
    // ... file creation code ...
    
    let result = TaskExecutor::new(&abs_workdir)
        .command("mock_castep")
        .arg("ZnO")
        .execute()
        .unwrap();
```

To:
```rust
let task = Task::new(&task_id, move || {
    // ... file creation code ...
    
    let result = TaskExecutor::new(&abs_workdir)
        .command("mock_castep")
        .arg("ZnO")
        .env("PATH", &path_clone)
        .execute()?;
```

### Issue 2: Inconsistent use of std::fs instead of workflow_utils

**File:** `workflow_core/tests/hubbard_u_sweep.rs:38-42`
**Severity:** Minor
**Problem:** The test uses `std::fs::create_dir_all` and `std::fs::write` directly instead of the workflow_utils functions (`create_dir`, `write_file`). While functionally equivalent, using workflow_utils maintains consistency with the example code and tests the actual utilities users will use.

**Fix:** Replace std::fs calls with workflow_utils functions.

Change from:
```rust
let _ = std::fs::create_dir_all(&abs_workdir);
let _ = std::fs::write(format!("{}/ZnO.cell", abs_workdir.display()), &cell_content);
let _ = std::fs::write(
    format!("{}/ZnO.param", abs_workdir.display()),
    param_content,
);
```

To:
```rust
workflow_utils::create_dir(&abs_workdir)?;
workflow_utils::write_file(format!("{}/ZnO.cell", abs_workdir.display()), &cell_content)?;
workflow_utils::write_file(
    format!("{}/ZnO.param", abs_workdir.display()),
    param_content,
)?;
```

### Issue 3: unwrap() instead of ? operator

**File:** `workflow_core/tests/hubbard_u_sweep.rs:49`
**Severity:** Minor
**Problem:** Uses `.execute().unwrap()` instead of the `?` operator. The closure returns `Result<(), anyhow::Error>`, so errors should propagate naturally.

**Fix:** Already addressed in Issue 1 fix above (change `.execute().unwrap()` to `.execute()?`).

### Issue 4: panic! instead of error propagation

**File:** `workflow_core/tests/hubbard_u_sweep.rs:52`
**Severity:** Minor
**Problem:** Uses `panic!()` to handle CASTEP failure instead of returning an error. The closure returns Result, so should use `anyhow::bail!` for consistency.

**Fix:** Replace panic with anyhow::bail.

Change from:
```rust
if !result.success() {
    panic!("castep failed: {:?}\n{}", result.exit_code, result.stderr);
}
```

To:
```rust
if !result.success() {
    anyhow::bail!("castep failed: {:?}\n{}", result.exit_code, result.stderr);
}
```
