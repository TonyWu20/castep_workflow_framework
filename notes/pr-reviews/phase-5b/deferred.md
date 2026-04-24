## Deferred Improvements: `phase-5b` — 2026-04-24

### Add trailing newline to `workflow_utils/src/prelude.rs`
**Source:** Round 2 review
**Rationale:** `workflow_utils/src/prelude.rs` is missing a trailing newline, causing `git diff` to show `\ No newline at end of file` on its last line. `workflow_core/src/prelude.rs` had its trailing newline fixed in the fix round. This is a standard code hygiene convention — POSIX text file specification and most editors expect files to end with a newline.
**Candidate for:** Next maintenance pass
**Precondition:** Any future edit to `workflow_utils/src/prelude.rs` — fix the trailing newline at the same time to avoid extra churn.

## Deferred Improvements: `phase-5b` — 2026-04-24 (Round 3)

### ARCHITECTURE.md: `setup`/`collect` builder signature mismatch
**Source:** Round 3 review
**Rationale:** ARCHITECTURE.md documents `setup<F>` with a single type parameter returning `Result<(), WorkflowError>`, but the actual implementation is `setup<F, E>` accepting any `E: std::error::Error + Send + Sync + 'static`. The two-param form is more ergonomic and is what callers actually see — the doc creates a misleading expectation. This pattern also repeats: v2 TASK-2 was supposed to do a full ARCHITECTURE.md update, but code-block accuracy issues recurred in v3.
**Candidate for:** Phase 6 plan
**Precondition:** Any future ARCHITECTURE.md edit — audit all code examples against `git grep` of actual signatures before committing.

### ARCHITECTURE.md: `JsonStateStore::new` signature shows `impl Into<String>` but actual is `&str`
**Source:** Round 3 review
**Rationale:** The doc API surface shows `pub fn new(name: impl Into<String>, path: PathBuf) -> Self` but the actual function takes `name: &str`. Users reading the docs would expect more flexibility than the implementation provides. Minor but misleading.
**Candidate for:** Phase 6 plan
**Precondition:** When `JsonStateStore::new` is next touched (e.g., accepting `impl Into<String>` for real, or when docs are regenerated with `cargo doc`).

### ARCHITECTURE.md: `load`/`load_raw` shown as instance methods but are static constructors
**Source:** Round 3 review
**Rationale:** ARCHITECTURE.md shows `pub fn load(&mut self)` and `pub fn load_raw(&self)` as instance methods on `JsonStateStore`. The actual implementations take `path: impl AsRef<Path>` and return `Result<Self, WorkflowError>` — they are static factory methods. This misrepresents the API design (especially the crash-recovery semantics of `load`).
**Candidate for:** Phase 6 plan
**Precondition:** Same as above — next ARCHITECTURE.md audit pass.

### ARCHITECTURE_STATUS.md Phase 3/4 entries: stale `TaskClosure` and `downstream_of` descriptions
**Source:** Round 3 review
**Rationale:** Historical Phase 3/4 entries in ARCHITECTURE_STATUS.md still describe the old `TaskClosure = Box<dyn Fn(&Path) -> Result<(), WorkflowError> + Send + Sync>` type and `downstream_of` as a `StateStoreExt` method returning `Vec<String>`. These are factually incorrect — Phase 5B changed both. Readers comparing Phase 3 to Phase 5B entries will see contradictory API descriptions.
**Candidate for:** Phase 6 plan
**Precondition:** Next ARCHITECTURE_STATUS.md edit — update all historical entries to use "as of Phase N, later superseded" language, or add footnotes.

### `parse_empty_string` test: strengthen assertion to match module style
**Source:** Round 3 review
**Rationale:** `parse_empty_string` asserts `!err.is_empty()` — only that some error string exists. Other tests in the same module (`parse_invalid_token`, `parse_empty_token`) assert `err.contains("...")` — verifying the error actually names the problematic input. The weak assertion provides minimal regression protection: any non-empty error string would pass, even if the code path changed.
**Candidate for:** Next maintenance pass
**Precondition:** When `parse_u_values` error messages are next touched — upgrade to `assert!(err.contains("invalid"))` for consistency.
