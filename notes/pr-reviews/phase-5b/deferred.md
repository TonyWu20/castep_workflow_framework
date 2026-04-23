## Deferred Improvements: `phase-5b` — 2026-04-24

### Add trailing newline to `workflow_utils/src/prelude.rs`
**Source:** Round 2 review
**Rationale:** `workflow_utils/src/prelude.rs` is missing a trailing newline, causing `git diff` to show `\ No newline at end of file` on its last line. `workflow_core/src/prelude.rs` had its trailing newline fixed in the fix round. This is a standard code hygiene convention — POSIX text file specification and most editors expect files to end with a newline.
**Candidate for:** Next maintenance pass
**Precondition:** Any future edit to `workflow_utils/src/prelude.rs` — fix the trailing newline at the same time to avoid extra churn.
