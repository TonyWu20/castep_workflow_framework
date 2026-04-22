## PR Review: `phase-4` → `main`

**Rating:** Approve

**Summary:** Phase 4 is complete and merge-ready. All four deliverables (per-task log persistence, HookTrigger::Periodic, ExecutionMode::Queued, graph-aware CLI retry) are implemented. The v5 fix round closed the final four hygiene items from the prior round: `inner()` removed from `TaskSuccessors`, re-exports added, BFS logic moved into `workflow_core` as `downstream_of()`, and `QueuedProcessHandle::wait()` semantics documented. No defect or correctness issues remain.

**Cross-Round Patterns:**

- [Contradictory] v4 introduced `TaskSuccessors` with `inner()` accessor; v5 immediately removed it. Resolved — a two-step refinement, not a regression. Signals that newtypes should ship with full encapsulation from introduction rather than being sealed a round later.
- [Recurring] BFS logic placement: v2 added graph-aware retry in CLI binary, v4 added BFS unit tests still in CLI context, v5 moved function and tests to `workflow_core`. Three rounds touched the same concern; final placement is correct. Signals domain logic should start in `workflow_core`, not be migrated post-review.

**Deferred Improvements:** 4 items → `notes/pr-reviews/phase-4/deferred.md`

**Axis Scores:**

- Plan & Spec: Pass — All 4 Phase 4 parts and 3 pre-phase-4 deferred items implemented
- Architecture: Pass — BFS in workflow_core, set_task_graph on JsonStateStore not trait, TaskSuccessors encapsulated, shell injection fixed, no tokio
- Rust Style: Pass — downstream_of is correct (start IDs excluded from result, to_owned() necessary); two cosmetic blank lines in main.rs only
- Test Coverage: Pass — 6 BFS unit tests cover linear chain, diamond, missing/empty start, multiple starts, and cycle termination

---

## Fix Document for Author

No defect or correctness issues found. Fix Document is empty.
