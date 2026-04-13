The impl-plan-reviewer agent has analyzed the plan and found **17 unclear instructions** that would confuse a junior developer. Here's a summary of the critical gaps:

## Critical Gaps Requiring Clarification

### 1. **Fix 3 — `mock_castep` context is missing**
- No command to verify the binary exists (`test -x workflow_core/tests/bin/mock_castep`)
- No explanation of what the script should do when invoked with `ZnO`
- No mention that this creates `ZnO.castep` in the working directory

### 2. **Fix 2c — Use-after-move confusion**
- No explanation of *why* `JsonStateStore::new(PathBuf)` consumes the path (likely because it stores the path internally)
- Junior devs won't understand why cloning is necessary

### 3. **Fix 2d — Self-reference confusion**
- Unclear why `workflow_core::` prefix was used in the first place if these are "own types"
- Should clarify that `workflow_core::process::SystemProcessRunner` and `workflow_core::monitoring::ShellHookExecutor` are the fully-qualified paths

### 4. **Fix 5 — Command arg-splitting explanation incomplete**
- Should explain that `Command::new("echo a")` treats `"echo a"` as the program name (file literal), not as `echo` with argument `a`

## Ambiguous Steps

- **Fix 1**: States `ShellHookExecutor` is the "local" version but doesn't explicitly say what to write instead of `pub use workflow_core::ShellHookExecutor`
- **Fix 2a**: Unclear why `crate::` imports are needed if the types are "already in scope"
- **Fix 4**: No command to verify the file is unreferenced (`grep -r "periodic_hooks" workflow_core`)

## Verification Commands Missing

| Fix | Missing Verification |
|-----|---------------------|
| 3 | `rustc --version` check (needs Rust 1.74+) |
| 3 | `test -x workflow_core/tests/bin/mock_castep` |
| 4 | `grep -r "periodic_hooks" workflow_core/tests/` |
| 5 | No verification for `StateStoreExt` trait usage |

**Recommendation**: The plan should be updated with concrete commands and explicit trait scoping explanations before a junior developer attempts to execute these fixes.
