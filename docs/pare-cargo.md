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

pare-cargo - add (MCP)

- Use to add dependencies to a crate (`cargo add` equivalent).
- Specify `package` to target a specific crate in a workspace.
- Pass `features` as an array to enable specific features.
- Use `dev: true` for dev dependencies, `build: true` for build dependencies.
- `dryRun: true` previews changes without modifying `Cargo.toml`.

Example (add serde with derive feature to a workspace crate):

- packages: ["serde"], features: ["derive"], package: "workflow_core",
  path: "/Users/tony/programming/castep_workflow_framework",
  compact: true — all other boolean flags: false

Example (add tokio as dev dependency):

- packages: ["tokio"], features: ["full"], dev: true, package: "workflow_core",
  path: "/Users/tony/programming/castep_workflow_framework",
  compact: true — all other boolean flags: false

## Tool loading note

`pare-cargo add` (and other tools like `remove`, `fmt`, `doc`, `update`, `tree`, `audit`) are
**not loaded by default**. You must explicitly load them before use:

1. Put `"name":"mcp__pare-cargo__discover-tools"` in the `<tool_call>` schema, with `load: ["add"]` to load the tool.
2. Then call `ToolSearch` with `query: "select:mcp__pare-cargo__add"` to fetch its schema.
3. Only after both steps is `mcp__pare-cargo__add` callable.

Example (add tempfile to workflow-cli):

Step 1 — discover and load:

```
{"name":"mcp__pare-cargo__discover-tools", "input": { "load": ["add"] }
```

Step 2 — fetch schema:

```
ToolSearch { query: "select:mcp__pare-cargo__add", max_results: 1 }
```

Step 3 — call the tool:

- packages: ["tempfile"], package: "workflow-cli",
  path: "/Users/tony/programming/castep_workflow_framework",
  compact: true — all other boolean flags: false

Expected response: `{ "success": true, "packages": ["tempfile"], "dependencyType": "normal" }`

pare-cargo - clippy (MCP)

- Use `package` to lint a single crate (maps to `cargo clippy -p <pkg>`).
- Use `allTargets: true` to include tests and examples.
- `fix: true` auto-applies suggestions (implies `--allow-dirty`).

Example:

- package: "workflow_core", allTargets: true, noDeps: true,
  path: "/Users/tony/programming/castep_workflow_framework",
  compact: true — all other boolean flags: false
