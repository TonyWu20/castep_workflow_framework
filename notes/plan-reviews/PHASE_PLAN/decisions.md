## Plan Review Decisions — PHASE_PLAN (Phase 6) — 2026-04-25

### Design Assessment

The plan is architecturally sound. All five goals are well-scoped, correctly sequenced, and respect the established crate boundaries. The `CollectFailurePolicy` design correctly places the mechanism in the framework and the criteria in Layer 3, maintaining software-agnosticism. The `root_dir` approach is the right level of abstraction. The retry stdin design follows Unix philosophy. The multi-parameter sweep approach (Layer 3, no new framework types) is appropriately conservative given the lack of cluster validation. One mechanical necessity in Goal 1 (passing `collect_failure_policy` through `InFlightTask`) and one architectural clarification in Goal 3 (resolution at dispatch time, not by mutating stored tasks) need to be made explicit in the plan.

### Deferred Item Decisions

#### Phase 4: Whitespace artifact in `workflow-cli/src/main.rs`
**Decision:** Absorb into Goal 4
**Rationale:** Goal 4 modifies `workflow-cli/src/main.rs` for stdin support. Zero marginal cost to fix the whitespace in the same edit.
**Action:** Add to Goal 4's critical files: "While editing `main.rs`, fix the two-blank-line whitespace artifact around line 71."

#### Phase 4: Design newtypes with full encapsulation on introduction
**Decision:** Close
**Rationale:** Already codified as an implementation guideline in ARCHITECTURE.md. Process rule, not a code change. Nothing to implement.
**Action:** None.

#### Phase 4: Place domain logic in `workflow_core` from initial implementation
**Decision:** Close
**Rationale:** Already codified in ARCHITECTURE.md. Process rule, not a code change.
**Action:** None.

#### Phase 4: `downstream_of` signature: accept `&[&str]` instead of `&[String]`
**Decision:** Close
**Rationale:** Already fixed in Phase 5B. Actual signature is `pub fn downstream_of<S: AsRef<str>>(&self, start: &[S])`. Stale deferred item.
**Action:** None.

#### D.1: Restore plan-specified portable config fields
**Decision:** Defer again
**Rationale:** No second user or non-NixOS cluster exists yet. Speculative generalization.
**Updated precondition:** When a second user attempts to adopt the example, or Tony moves to a non-NixOS cluster.

#### D.2: `generate_job_script` formatting inconsistencies
**Decision:** Defer again
**Rationale:** Goal 2 extends task generation, not job scripts. `job_script.rs` may not be touched.
**Updated precondition:** Next functional edit to `job_script.rs`.

#### D.3: Unit tests for `parse_u_values` and `generate_job_script`
**Decision:** Close (partially done)
**Rationale:** `parse_u_values` tests are comprehensive (basic, whitespace, single, invalid, empty token, empty string, negative). `generate_job_script` tests are brittle given NixOS-specific output — defer until D.1 (portable template) is addressed.
**Action:** None for this phase. Reopen `generate_job_script` test question when D.1 is addressed.

#### D.4: `submit()` log-path absolutization
**Decision:** Close (subsumed)
**Rationale:** Correctly absorbed into Goal 3. `root_dir` resolution in `Workflow::run()` produces absolute log paths before `submit()` is called.
**Action:** None beyond Goal 3 implementation.

#### D.5: Pedantic clippy findings (`uninlined_format_args`, `doc_markdown`)
**Decision:** Absorb into Goal 5
**Rationale:** Goal 5 touches files that have these warnings (config.rs, main.rs). Trivial marginal cost.
**Action:** Add Goal 5 item 7: run `cargo clippy --workspace -- -W clippy::uninlined_format_args` and fix instances in files touched by this phase.

#### D.6: `--workdir` flag
**Decision:** Close (subsumed)
**Rationale:** Correctly absorbed into Goal 3.

#### D.7: `squeue` empty-output false-positive
**Decision:** Close (subsumed)
**Rationale:** Correctly absorbed into Goal 1. `CollectFailurePolicy::FailTask` default ensures collect closure failure marks task `Failed` even when squeue reports exit 0.

#### D.8: Double `s.trim()` call in `parse_u_values`
**Decision:** Close (already fixed)
**Rationale:** Current config.rs extracts `let trimmed = segment.trim()` and uses it in both parse and error message. Fixed.

#### D.9: `anyhow::anyhow!(e)` vs `anyhow::Error::msg(e)`
**Decision:** Close (already fixed)
**Rationale:** Current main.rs uses `.map_err(anyhow::Error::msg)`. Already idiomatic.

#### D.10: `fn main()` 135 lines
**Decision:** Absorb into Goal 2
**Rationale:** Goal 2 restructures the example for multi-parameter support. Current main.rs already extracted `build_one_task()` and `build_sweep_tasks()`, reducing main() to ~47 lines. Goal 2 will further refactor for multi-param. Mark addressed by Goal 2's restructuring.
**Action:** Goal 2 inherits the constraint: keep `main()` short via appropriate helper extraction.

#### D.11: Direct `for loop` in parameter sweeping
**Decision:** Absorb into Goal 2
**Rationale:** Goal 2 explicitly replaces for-loop pattern with iterator-based `iproduct!` and `zip`. Directly addressed.
**Action:** None beyond Goal 2's existing scope.

#### Phase 5B: Trailing newline in `workflow_utils/src/prelude.rs`
**Decision:** Absorb into Goal 5
**Rationale:** Already listed in Goal 5 item 6.

#### Phase 5B: ARCHITECTURE.md `setup`/`collect` builder signature mismatch
**Decision:** Absorb into Goal 5
**Rationale:** Already listed in Goal 5 item 1. Confirmed: actual is `setup<F, E>` vs doc `setup<F>`.

#### Phase 5B: ARCHITECTURE.md `JsonStateStore::new` signature
**Decision:** Absorb into Goal 5
**Rationale:** Already listed in Goal 5 item 2. Recommendation: update impl to accept `impl Into<String>` (backward-compatible, more ergonomic) rather than just fixing the doc.

#### Phase 5B: ARCHITECTURE.md `load`/`load_raw` as instance methods
**Decision:** Absorb into Goal 5
**Rationale:** Already listed in Goal 5 item 3. Confirmed: both are static constructors returning `Result<Self, WorkflowError>`.

#### Phase 5B: ARCHITECTURE_STATUS.md stale entries
**Decision:** Absorb into Goal 5
**Rationale:** Already listed in Goal 5 item 4.

#### Phase 5B: `parse_empty_string` test weak assertion
**Decision:** Absorb into Goal 5
**Rationale:** Already listed in Goal 5 item 5.

### Plan Amendments

The following amendments were recommended by the architect and approved for inclusion:

1. **Goal 1 — Make `InFlightTask` changes explicit**: Add to Critical files: "`workflow_core/src/workflow.rs` `InFlightTask` struct — add `collect_failure_policy: CollectFailurePolicy` field; populate from `task.collect_failure_policy` at dispatch (around lines 273-280)."

2. **Goal 3 — Correct file path**: Replace `workflow_utils/src/runner.rs` with `workflow_core/src/workflow.rs` (resolving `log_dir` against `root_dir` before passing to `qs.submit()`). Note that `workflow_utils/src/queued.rs` likely needs no changes; the existing `cwd.join()` fallback becomes redundant but can stay for defense in depth.

3. **Goal 3 — Clarify resolution semantics**: Resolution happens at dispatch time in `run()`, not by mutating `Task::workdir`. `dry_run()` does not apply `root_dir` resolution (path resolution is a runtime concern of `run()`).

4. **Goal 4 — clap argument change**: `task_ids` must change from `#[arg(required = true)]` to optional. When empty and stdin is not a TTY (or `-` is present), read from stdin. When empty and stdin is a TTY, print a usage error.

5. **Goal 4 — Absorb whitespace artifact**: While editing `workflow-cli/src/main.rs`, fix the two-blank-line whitespace artifact around line 71.

6. **Goal 5 — Add pedantic clippy item**: Add item 7: run `cargo clippy --workspace -- -W clippy::uninlined_format_args` and fix instances in files touched by this phase.
