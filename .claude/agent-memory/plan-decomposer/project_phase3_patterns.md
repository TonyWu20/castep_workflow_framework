---
name: Phase 3 task archetype patterns
description: Common task sequences and dependency patterns discovered during Phase 3 Production Trust plan decomposition
type: project
---

Phase 3 "Production Trust" decomposes into tasks following consistent patterns for this project.

**Why:** Understanding these patterns helps future decompositions be consistent and faster.

**How to apply:**

Common task archetype sequence for adding a new abstraction (trait + impl):
1. Define error types first (everything touches errors)
2. Write tests for the trait contract (TDD red phase) as `#[ignore]` tests
3. Implement the trait and types (TDD green phase, un-ignore tests)
4. Write concrete implementation in workflow_utils
5. Update all consumers

Dependency flip pattern (moving types from workflow_utils to workflow_core):
- Create trait in workflow_core
- Move data types to workflow_core
- Remove impl method that caused the original dependency
- Add concrete impl struct in workflow_utils
- workflow_utils depends on workflow_core (not vice versa)
- Re-export from workflow_utils for backward compatibility

Critical path bottleneck: Workflow::run() is always the convergence point — all trait definitions, state store, task model, and process runner must be done before the execution engine can be modified.

Test update task should always be a separate task from implementation — tests touch many files and have high breakage risk.

The crate has ~27 tests across 7 files as of Phase 2 completion. TDD is mandatory: test tasks always precede implementation tasks.

Key revision lesson (2026-04-10): `#[ignore]` tests are the correct TDD red-phase strategy when the API being tested doesn't exist yet. Tests written against future API signatures with `#[ignore]` + explanatory comment avoid the "tests compile but fail against non-existent API" contradiction. The pattern is: write the test as if the new API exists, mark `#[ignore]`, document which task will make it pass.

Reviewer feedback pattern: task descriptions must include full type signatures (all fields, all method signatures) and concrete implementation guidance. "Define trait X" without showing the exact `trait X { ... }` block is always flagged as UNCLEAR. Every struct needs its fields, every trait needs its method signatures, every enum needs its variants with payloads.

## Phase 3.1 patterns (2026-04-13)

File conflict risk: when two parallel tasks both modify `workflow_core/src/workflow.rs` (e.g., signal handling + timeout enforcement), the orchestrator must serialize those edits or assign them to the same subagent. Flag this explicitly in the decomposition.

Ordering hazard pattern: when a "delete crate" task and a "clean workspace deps" task are both present, the delete task must remove the crate from `[workspace.members]` before the dep-cleanup task removes deps that the dead crate still references. Encode this as a dependency edge (dep-cleanup → delete, or merge into one task).

`Workflow::run` API stability: the existing `&mut self` signature must not be changed to consuming `self` without explicit user instruction — it is a breaking change affecting all test call sites. Note this as a risk flag whenever the plan mentions making `run()` consume `self`.

Resume semantics invariant: `TaskStatus::Skipped` (user-explicit) must NEVER be reset on load. Only `Running`, `Failed { .. }`, and `SkippedDueToDependencyFailure` reset to `Pending`. This distinction must be called out in every task that touches `JsonStateStore::load`.

Side-effect counter pattern for "task did not re-run" assertions: use `Arc<AtomicUsize>` captured by the `.setup()` closure (closures must be `'static`; local variables cannot be captured by reference across runs).

## Fix plan decomposition patterns (2026-04-13)

Trait return-type change cascade: when changing `fn get_status() -> Option<T>` to `-> Option<&T>`, all callers using `==` or `assert_eq!` must switch to `matches!()` or `.cloned()`. This is a separate task from the trait change itself. Group: (1) trait signature, (2) impl signature, (3) caller/test fixups.

Concrete-impl removal order: (1) remove struct+impl from `workflow_core`, (2) remove from `pub use` re-export in `lib.rs`, (3) update test imports to use `workflow_utils::` versions. Steps 1+2 are in the same file cluster and parallel-safe with other file removals. Step 3 depends on both.

`StateStore` trait object coercion: `Box<JsonStateStore>.as_mut()` auto-coerces to `&mut dyn StateStore` after the signature change. No caller changes needed for boxed usage. Direct stack references (`&mut state`) also coerce. This is safe to assume -- do not add unnecessary `.as_mut()` calls.

Hook wiring complexity: `Workflow::run()` removes tasks from `self.tasks` at dispatch time. Any data needed later (monitors, workdir) must be captured before insertion into the handles map. This is a consistent footgun -- always flag it when modifying the run loop.
