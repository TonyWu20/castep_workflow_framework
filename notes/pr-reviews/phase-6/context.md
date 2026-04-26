## Memory
No project memory available.

## Phase Plan
plans/phase-6/phase6_implementation.toml + plans/phase-6/PHASE_PLAN.md
6 tasks, dependencies: TASK-2 depends on TASK-1, TASK-3 depends on TASK-1+2, TASK-6 depends on TASK-5.
Task 1: CollectFailurePolicy enum + field wiring
Task 2: root_dir / --workdir support
Task 3: CollectFailurePolicy integration + process_finished reordering
Task 4: retry stdin support in workflow-cli
Task 5: Multi-parameter sweep (itertools, product/pairwise modes)
Task 6: Documentation accuracy sweep + clippy

## Snapshot
Branch `phase-6` — last fix round (5 tasks) applied 2026-04-26.
All tasks passed, build and test clean.
Changes: removed dead `task_ids.is_empty()` branch in `read_task_ids`; converted `second` param of `build_one_task`/`build_chain` from `&str` to `Option<&str>`.
Outstanding issues: none.
