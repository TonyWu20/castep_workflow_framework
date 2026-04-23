## PR Review: `phase-5b` → `main`

**Rating:** Request Changes

**Summary:** Phase 5B is structurally solid — all tests pass (109), clippy is clean (0 warnings), prelude modules are correctly wired, and the core ergonomics features work. However, two plan tasks are incompletely fulfilled: Task 11 updated the ARCHITECTURE.md narrative but left inaccurate code blocks that Task 11 explicitly required fixing, and Task 4 delivers 5 of the 6 spec-listed `parse_u_values` test cases (missing empty-string and negative-value cases). Both are Minor defects with no functional impact.

**Cross-Round Patterns:** None

**Deferred Improvements:** 1 item → `notes/pr-reviews/phase-5b/deferred.md`

**Axis Scores:**

- Plan & Spec: Partial — 9 of 11 tasks fully implemented; Task 11 (ARCHITECTURE.md code blocks not fixed) and Task 4 (2 of 6 spec test cases missing) are incomplete
- Architecture: Pass — Prelude modules follow utilities-first pattern; `run_default()` correctly placed at I/O boundary; DAG-centric design preserved; crate boundaries respected; no `anyhow` in `workflow_core`
- Rust Style: Pass — clippy clean (0 warnings), no dead code, error handling uses `?` appropriately, format args inlined
- Test Coverage: Partial — `parse_u_values` (5 tests, 2 missing from spec); `generate_job_script` (6 tests); `ExecutionMode::direct()` has doc test; `run_default()` has no unit tests (acceptable for a 3-line wrapper)

---

## Fix Document for Author

### Issue 1: ARCHITECTURE.md code blocks do not match actual API

**Classification:** Defect
**File:** `ARCHITECTURE.md`
**Severity:** Minor
**Problem:** Task 11 / B.5 explicitly required "Fix all code examples to match actual API." The Phase 5B fix round updated the narrative sections (Implementation Status, phase lists) and the Layer 3 example code, but left several pseudocode blocks in the "Core Types" section with inaccurate signatures:

1. **`StateStore` trait (around line 212):** Shows `load`, `load_raw` as trait methods with `&mut self`. Actual: `load` and `load_raw` are inherent methods on `JsonStateStore`, not on the `StateStore` trait. The actual trait has `get_status`, `set_status`, `all_tasks`, `save(&self)` — none of which appear in the ARCHITECTURE.md code block.
2. **`downstream_of` (around line 221):** Shown as `fn downstream_of<S: AsRef<str>>` on `StateStoreExt`, returning `Vec<String>`. Actual: it is an inherent method on `TaskSuccessors`, returning `HashSet<String>`.
3. **`TaskClosure` (around line 150):** Error type shown as `Result<(), WorkflowError>`. Actual: `Result<(), Box<dyn std::error::Error + Send + Sync>>`.
4. **`Task` struct (around line 139):** Shows private fields `execution_mode: ExecutionMode` and `workdir: Option<PathBuf>`. Actual fields: `pub mode: ExecutionMode` and `pub workdir: PathBuf`.

**Fix:** Update the four pseudocode blocks in the "Core Types / workflow_core" section to match the actual signatures. These are illustrative pseudocode blocks — they don't need to be copy-paste compilable, but the field names, return types, and trait membership must be accurate. Specifically:
- Rewrite the `StateStore` trait block to show `get_status`, `set_status`, `all_tasks`, `save(&self)`
- Remove `load`/`load_raw` from the trait block; document them as inherent methods on `JsonStateStore`
- Move `downstream_of` to a `TaskSuccessors` block, fix return type to `HashSet<String>`
- Fix `TaskClosure` error type to `Box<dyn std::error::Error + Send + Sync>`
- Fix `Task` struct to show `pub mode: ExecutionMode` and `pub workdir: PathBuf`

### Issue 2: `parse_u_values` missing 2 of 6 spec-required test cases

**Classification:** Defect
**File:** `examples/hubbard_u_sweep_slurm/src/config.rs`
**Severity:** Minor
**Problem:** Task 4 / D.3a specified exactly 6 test cases for `parse_u_values`. The current test module has 5:
- `parse_basic_values` — covers normal input `"0.0,1.0,2.0"`
- `parse_with_whitespace` — covers whitespace-padded input
- `parse_single_value` — covers single token
- `parse_invalid_token` — covers non-numeric token
- `parse_empty_token` — covers empty token in middle (`"1.0,,2.0"`)

Missing from the plan spec:
- Empty string: `""` → `Err(...)` (different from empty-token-in-middle; the whole input is empty)
- Negative values: `"-1.0,2.0"` → `[-1.0, 2.0]` (verifies that negative f64 parsing is accepted)

**Fix:** Add two tests to the `#[cfg(test)] mod tests` block in `config.rs`:
```rust
#[test]
fn parse_empty_string() {
    let err = parse_u_values("").unwrap_err();
    assert!(!err.is_empty());
}

#[test]
fn parse_negative_values() {
    let vals = parse_u_values("-1.0,2.0").unwrap();
    assert_eq!(vals, vec![-1.0, 2.0]);
}
```
