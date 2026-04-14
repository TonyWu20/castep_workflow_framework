## Rules

- Always create a backup copy when you start modifying existing files in
  `bypassPermissions` mode.
- Before making any code changes, leverage LSP tools for understanding and validation:
  - **Before editing**: Use LSP to understand existing code structure, find symbol definitions, check types
  - **During implementation**: Use LSP diagnostics to catch errors early, before running cargo check
  - **For refactoring**: Use LSP rename operations instead of manual find-replace
  - **For validation**: Check LSP diagnostics first, then run acceptance commands

  **Key LSP workflows**:
  1. **Understanding code**: `LSP hover`, `LSP definition`, `LSP references` to understand what exists
  2. **Finding symbols**: `LSP documentSymbol` to locate functions/structs/modules before editing
  3. **Validation**: `LSP diagnostics` to check for errors immediately after edits
  4. **Refactoring**: `LSP rename` for symbol renames, `LSP references` to find all usages

## MCP tool usage

pare-cargo - check (MCP)
- Use for workspace-wide compilation checks before/after edits.
- Always use `workspace: true, allTargets: true, keepGoing: true` to catch all errors across all crates.

Example:
  path: "/Users/tony/programming/castep_workflow_framework",
  keepGoing: true, allTargets: true, workspace: true, compact: true — all other boolean flags: false

pare-cargo - test (MCP)

- Always specify `package` when running integration tests (files under `<crate>/tests/`);
  without it, `filter` only matches unit test names and returns 0 results.
- The `filter` param matches `#[test]` function names, not the binary/file name.
- Integration test binary name == the filename (e.g. `timeout_integration.rs` → binary `timeout_integration`).

Example (workflow_core integration test):

- package: "workflow_core", filter: "timeout_task_fails_and_dependent_skips",
  path: "/Users/tony/programming/castep_workflow_framework",
  compact: true — all other boolean flags: false
