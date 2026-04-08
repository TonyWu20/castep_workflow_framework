# Fix Plan for Phase 1.3

## Overview

This fix plan addresses test code quality issues in `workflow_core/tests/hubbard_u_sweep.rs`. All issues are independent and can be fixed in parallel, though sequential execution (1→2→3) is recommended to minimize merge conflicts.

---

## Step 1: Replace global state mutation with TaskExecutor::env()

**File:** `/Users/tony/programming/castep_workflow_framework/workflow_core/tests/hubbard_u_sweep.rs:28-49`
**Depends on:** None
**Severity:** Major

**Problem:** Uses `std::env::set_var("PATH", &path_clone)` which mutates global process state and can cause test interference when tests run in parallel.

**Before:**
```rust
        let task = Task::new(&task_id, move || {
            // Set PATH to include mock_castep
            std::env::set_var("PATH", &path_clone);

            // Create workflow files
            let cell_content = format!(
                "%BLOCK LATTICE_CART\n  3.25 0.0 0.0\n  0.0 3.25 0.0\n  0.0 0.0 5.21\n%ENDBLOCK LATTICE_CART\n"
            );
            let param_content = "task : SinglePoint\n";

            let _ = std::fs::create_dir_all(&abs_workdir);
            let _ = std::fs::write(format!("{}/ZnO.cell", abs_workdir.display()), &cell_content);
            let _ = std::fs::write(
                format!("{}/ZnO.param", abs_workdir.display()),
                param_content,
            );

            let result = TaskExecutor::new(&abs_workdir)
                .command("mock_castep")
                .arg("ZnO")
                .execute()
                .unwrap();
```

**After:**
```rust
        let task = Task::new(&task_id, move || {
            // Create workflow files
            let cell_content = format!(
                "%BLOCK LATTICE_CART\n  3.25 0.0 0.0\n  0.0 3.25 0.0\n  0.0 0.0 5.21\n%ENDBLOCK LATTICE_CART\n"
            );
            let param_content = "task : SinglePoint\n";

            let _ = std::fs::create_dir_all(&abs_workdir);
            let _ = std::fs::write(format!("{}/ZnO.cell", abs_workdir.display()), &cell_content);
            let _ = std::fs::write(
                format!("{}/ZnO.param", abs_workdir.display()),
                param_content,
            );

            let result = TaskExecutor::new(&abs_workdir)
                .env("PATH", &path_clone)
                .command("mock_castep")
                .arg("ZnO")
                .execute()?;
```

**Changes:**
1. Remove the line: `std::env::set_var("PATH", &path_clone);` and its comment
2. Add `.env("PATH", &path_clone)` in the TaskExecutor builder chain before `.command("mock_castep")`
3. Change `.execute().unwrap()` to `.execute()?`

**Verify:** `cargo test -p workflow_core --test hubbard_u_sweep`

---

## Step 2: Replace std::fs with workflow_utils functions

**File:** `/Users/tony/programming/castep_workflow_framework/workflow_core/tests/hubbard_u_sweep.rs:38-42`
**Depends on:** None (can run in parallel with Step 1 and 3)
**Severity:** Minor

**Problem:** Uses `std::fs` directly instead of `workflow_utils` functions for consistency with example code.

**Before:**
```rust
            let _ = std::fs::create_dir_all(&abs_workdir);
            let _ = std::fs::write(format!("{}/ZnO.cell", abs_workdir.display()), &cell_content);
            let _ = std::fs::write(
                format!("{}/ZnO.param", abs_workdir.display()),
                param_content,
            );
```

**After:**
```rust
            workflow_utils::create_dir(&abs_workdir)?;
            workflow_utils::write_file(&abs_workdir.join("ZnO.cell"), &cell_content)?;
            workflow_utils::write_file(&abs_workdir.join("ZnO.param"), param_content)?;
```

**Changes:**
1. Replace `std::fs::create_dir_all` with `workflow_utils::create_dir`
2. Replace `std::fs::write` with `workflow_utils::write_file`
3. Change `let _ =` to proper error propagation with `?`
4. Use `.join()` for path construction instead of string formatting

**Verify:** `cargo test -p workflow_core --test hubbard_u_sweep`

---

## Step 3: Replace panic! with anyhow::bail!

**File:** `/Users/tony/programming/castep_workflow_framework/workflow_core/tests/hubbard_u_sweep.rs:51-53`
**Depends on:** None (can run in parallel with Step 1 and 2)
**Severity:** Minor

**Problem:** Uses `panic!()` instead of returning an error. The closure returns `Result`, so should use `anyhow::bail!` for consistency.

**Before:**
```rust
            if !result.success() {
                panic!("castep failed: {:?}\n{}", result.exit_code, result.stderr);
            }
```

**After:**
```rust
            if !result.success() {
                anyhow::bail!("castep failed: {:?}\n{}", result.exit_code, result.stderr);
            }
```

**Changes:**
1. Replace `panic!` with `anyhow::bail!`

**Verify:** `cargo test -p workflow_core --test hubbard_u_sweep`

---

## Execution Strategy

**Parallel execution:** All three steps are independent and can be applied simultaneously.

**Sequential execution (recommended):** Apply in order 1→2→3 to minimize merge conflicts since they modify the same code block.

**Final verification:** 
```bash
cargo test -p workflow_core --test hubbard_u_sweep
cargo clippy -p workflow_core --tests
```

---

## Verification Status

✅ **strict-code-reviewer verification complete:** All "Before" code matches actual file, API names corrected (create_dir not create_dir_all), line numbers accurate.

✅ **fix-plan-reader verification complete:** All steps are clear and executable. Step 1 ambiguity resolved by removing line number references from Changes section.
