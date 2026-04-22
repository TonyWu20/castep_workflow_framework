## Deferred Improvements: `phase-4` — 2026-04-22

### Whitespace artifact in `workflow-cli/src/main.rs`
**Source:** Round 6 review
**Rationale:** Two blank lines remain where `downstream_tasks` was removed (around line 71). Cosmetic only — no semantic or compilation impact — but inconsistent with the file's single-blank-line convention between items.
**Candidate for:** Phase 5 plan (or any edit to main.rs)
**Precondition:** Any future edit to `workflow-cli/src/main.rs`

### Design newtypes with full encapsulation on introduction
**Source:** Round 6 review (cross-round pattern: v4 introduced `TaskSuccessors` with `inner()`, v5 removed it)
**Rationale:** Introducing a newtype with a public raw-accessor and sealing it one round later caused churn across two fix plans. Future newtypes should ship with method delegation from the start, never exposing the inner collection directly.
**Candidate for:** Phase 5 onward (implementation guideline)
**Precondition:** Any new newtype introduction

### Place domain logic in `workflow_core` from initial implementation
**Source:** Round 6 review (cross-round pattern: BFS migrated CLI → workflow_core across v2/v4/v5)
**Rationale:** The `downstream_tasks` BFS function was written in the CLI binary, tested there, then migrated to `workflow_core` over three fix rounds. Domain-layer logic operating on core types (like `TaskSuccessors`) belongs in `workflow_core` at authoring time, not as a post-review migration.
**Candidate for:** Phase 5 onward (implementation guideline)
**Precondition:** Any new domain logic that operates on `workflow_core` types

### `downstream_of` signature: accept `&[&str]` instead of `&[String]`
**Source:** Round 6 review
**Rationale:** `downstream_of(&self, start: &[String])` requires callers to pass owned `String` slices. Changing to `start: &[impl AsRef<str>]` or `start: &[&str]` would reduce caller allocations without changing correctness. The current implementation is correct — the `to_owned()` inside the loop is necessary because the BFS queue must own its values. This is purely an ergonomic improvement.
**Candidate for:** Phase 5 API audit
**Precondition:** If `downstream_of` gains a second caller or the public API surface is reviewed
