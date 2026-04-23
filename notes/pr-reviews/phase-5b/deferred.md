## Deferred Improvements: `phase-5b` — 2026-04-24

### Add trailing newlines to prelude modules
**Source:** Round 1 review
**Rationale:** Both `workflow_core/src/prelude.rs` and `workflow_utils/src/prelude.rs` are missing trailing newlines, causing `git diff` to show `\ No newline at end of file` on the last line of each file. This is a standard code hygiene convention — most editors, diff tools, and POSIX text file specifications expect files to end with a newline. While clippy's `missing_trailing_newline` lint is not enabled in this project, it is a minor hygiene issue that should be fixed to avoid noisy diffs and ensure consistency with the rest of the codebase.
**Candidate for:** Next maintenance pass
**Precondition:** Any future edit to these files — fix the trailing newline at the same time to avoid extra churn.
