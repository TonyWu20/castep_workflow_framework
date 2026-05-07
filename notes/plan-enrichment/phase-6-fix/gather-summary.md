## Gather Summary: phase-6-fix

**Tasks created:** 5
**Dependency chain:** TASK-5 depends on TASK-1..TASK-4. TASK-1 through TASK-4 are all parallel.
**Deferred items absorbed:** 2 (D.2 fully absorbed; D.3 partially absorbed — port `no_literal_tabs` test)

**Gather completeness:**
- [x] deferred-and-patterns.md — saved
- [x] codebase-state.md — saved — Files documented: 19
- [x] draft-elaboration.md — saved
- [x] draft-plan.toml — saved
- [x] task-checklist.md — saved

**Before-block verification:** 1/1 confirmed from Step 4
**Unverified tasks:** none (from Step 4)
**Wiring issues flagged:** 1

**Confidence notes:**
1 HIGH uncertainty: exact `TaskClosure` type alias in `workflow_core` — must read `workflow_core/src/task.rs` before implementation.
4 MEDIUM uncertainties: `KpointsMpGrid` API surface, `parse_u_values` module placement vs old pattern, `WorkflowError`-anyhow compatibility in setup closures, task ID naming when kpoints/cutoffs are None.
The HIGH uncertainty must be resolved before any setup/collect closure code can be written.

**Questions for user:**
None
