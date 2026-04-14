# `pare-cargo` tool usage

pare-cargo - check (MCP)

- Use for workspace-wide compilation checks before/after edits.
- Always use `workspace: true, allTargets: true, keepGoing: true` to catch all errors across all crates.

Example:

- path: "/Users/tony/programming/castep_workflow_framework",
  keepGoing: true, allTargets: true, workspace: true, compact: true — all other boolean flags: false

pare-cargo - test (MCP)

- Always specify `package` when running integration tests (files under `<crate>/tests/`);
  without it, `filter` only matches unit test names and returns 0 results.
- `filter` matches `#[test]` function names, not the binary/file name.
- Integration test binary name == the filename (e.g. `timeout_integration.rs` → binary `timeout_integration`).
- `cargo test --test <binary>` has NO direct MCP equivalent (`--test` is a cargo-level flag, not exposed).
  Workaround: use `package` + `filter` with the function name, or omit `filter` to run all tests in the package.
- For workspace-wide tests (`cargo test --workspace`): omit `package`, set no filter.

Example (integration test by function name):

- package: "workflow_core", filter: "timeout_task_fails_and_dependent_skips",
  path: "/Users/tony/programming/castep_workflow_framework",
  compact: true — all other boolean flags: false

pare-cargo - clippy (MCP)

- Use `package` to lint a single crate (maps to `cargo clippy -p <pkg>`).
- Use `allTargets: true` to include tests and examples.
- `fix: true` auto-applies suggestions (implies `--allow-dirty`).

Example:

- package: "workflow_core", allTargets: true, noDeps: true,
  path: "/Users/tony/programming/castep_workflow_framework",
  compact: true — all other boolean flags: false
