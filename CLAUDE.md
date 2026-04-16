## Rules

- Always create a backup copy when you start modifying existing files in
  `bypassPermissions` mode.
- Before making any code changes, leverage LSP tools for understanding and validation:
  - **Before editing**: Use LSP to understand existing code structure, find symbol definitions, check types
  - **During implementation**: Use LSP diagnostics to catch errors early, before running cargo check
  - **For refactoring**: Use LSP rename operations instead of manual find-replace
  - **For validation**: Check LSP diagnostics first, then run acceptance commands

  **Key LSP workflows**:
  **These are not bash commands**
  1. **Understanding code**: `LSP hover`, `LSP definition`, `LSP references` to understand what exists
  2. **Finding symbols**: `LSP documentSymbol` to locate functions/structs/modules before editing
  3. **Validation**: `LSP diagnostics` to check for errors immediately after edits
  4. **Refactoring**: `LSP rename` for symbol renames, `LSP references` to find all usages

- Always pass corresponding `subagent_type` to `Agent` tool when prompted to call subagents.
- Always use space instead of tab in writing and editing code files.
