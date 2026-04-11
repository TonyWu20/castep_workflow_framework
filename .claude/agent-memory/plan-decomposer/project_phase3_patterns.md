---
name: Phase 3 task archetype patterns
description: Common task sequences and dependency patterns discovered during Phase 3 Production Trust plan decomposition
type: project
---

Phase 3 "Production Trust" decomposes into 25 tasks following a consistent pattern for this project.

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

Critical path bottleneck: Workflow::run() rewrite is always the convergence point -- all trait definitions, state store, task model, and process runner must be done before the execution engine can be rewritten.

Test update task should always be a separate task from implementation -- tests touch many files and have high breakage risk.

The crate has ~27 tests across 7 files as of Phase 2 completion. TDD is mandatory: test tasks always precede implementation tasks.

Key revision lesson (2026-04-10): `#[ignore]` tests are the correct TDD red-phase strategy when the API being tested doesn't exist yet. Tests written against future API signatures with `#[ignore]` + explanatory comment avoid the "tests compile but fail against non-existent API" contradiction. The pattern is: write the test as if the new API exists, mark `#[ignore]`, document which task will make it pass.

Reviewer feedback pattern: task descriptions must include full type signatures (all fields, all method signatures) and concrete implementation guidance. "Define trait X" without showing the exact `trait X { ... }` block is always flagged as UNCLEAR. Every struct needs its fields, every trait needs its method signatures, every enum needs its variants with payloads.
