## Plan Review Decisions — PHASE5B_API_ERGONOMICS — 2026-04-23

### Design Assessment

The plan is architecturally sound. All changes are additive ergonomic improvements that do not alter any fundamental invariants, introduce crate boundary violations, or create hard-to-reverse API decisions. The `downstream_of` generic signature adds no lifetime complexity since the BFS interior already clones everything into the queue. `run_default` respects the existing layering (lives in `workflow_utils`, depends only on `workflow_core` types). Prelude modules are re-exports only — no new traits, no coherence risk. No new error variants are introduced. The `anyhow::Error::msg` fix (D.9) is a strict improvement. Every addition is purely additive (new constructors, new modules, new functions). The plan is well-sequenced with independent library changes first, example changes second, and docs last.

### Deferred Item Decisions

#### Whitespace artifact in `workflow-cli/src/main.rs` (phase-4)
**Decision:** Close
**Rationale:** The artifact no longer exists. There is only a single blank line between `cmd_inspect` and `cmd_retry`, consistent with the file's convention.
**Action:** Closed — no further tracking needed.

#### Design newtypes with full encapsulation on introduction (phase-4)
**Decision:** Absorb
**Rationale:** Already absorbed into the plan as B.4.3 (implementation guidelines in ARCHITECTURE.md).
**Action:** Covered by Step 11 of the plan.

#### Place domain logic in `workflow_core` from initial implementation (phase-4)
**Decision:** Absorb
**Rationale:** Already absorbed into the plan as B.4.3. Confirmed handled.
**Action:** Covered by Step 11 of the plan.

#### `downstream_of` signature: accept `&[&str]` instead of `&[String]` (phase-4 B.4.2)
**Decision:** Absorb
**Rationale:** Already absorbed as Step 1 (B.4.2) of the plan.
**Action:** Covered by Step 1 of the plan.

#### D.1: Portable SLURM config fields (phase-5)
**Decision:** Defer again
**Rationale:** The NixOS-specific job script is a valid production artifact. Parameterizing it requires a templating strategy design decision; there is no second user yet.
**Action:** Updated precondition: a second user attempts to run the example on a module-based (non-NixOS) cluster.

#### D.2: `generate_job_script` formatting inconsistencies (phase-5)
**Decision:** Absorb
**Rationale:** Already absorbed as Step 5 of the plan.
**Action:** Covered by Step 5 of the plan.

#### D.3: Unit tests for `parse_u_values` and `generate_job_script` (phase-5)
**Decision:** Absorb
**Rationale:** Already absorbed as Steps 4 and 6 of the plan.
**Action:** Covered by Steps 4 and 6 of the plan.

#### D.4: `std::path::absolute` for log paths (phase-5)
**Decision:** Defer again
**Rationale:** `cwd.join(log_dir)` is correct for all realistic inputs. The `..`/symlink edge case has not manifested. Low-value change in an ergonomics pass.
**Action:** Updated precondition: a bug report involving symlinked or `..`-containing log directories.

#### D.5: Pedantic clippy findings (phase-5)
**Decision:** Absorb
**Rationale:** Already absorbed as Step 10 of the plan.
**Action:** Covered by Step 10 of the plan.

#### D.6: `--workdir` flag (phase-5)
**Decision:** Defer again
**Rationale:** Requires `Workflow` root_dir support in `workflow_core` — a feature, not an ergonomic fix. Does not belong in a cleanup phase.
**Action:** Updated precondition: Phase 6 planning, or when a user submits from a non-project-root directory and hits a path error.

#### D.7: `squeue` false-positive as job success (phase-5)
**Decision:** Defer again
**Rationale:** Requires a `CollectFailurePolicy` design in `workflow_core` — structural work, not ergonomics. The existing collect closure guard mitigates immediate risk.
**Action:** Updated precondition: a second false-success incident, or Phase 6 when `CollectFailurePolicy` is designed.

#### D.8: Double `s.trim()` call in `parse_u_values` (phase-5)
**Decision:** Absorb
**Rationale:** Already absorbed as Step 3 of the plan.
**Action:** Covered by Step 3 of the plan.

#### D.9: `anyhow::anyhow!(e)` vs `anyhow::Error::msg` (phase-5)
**Decision:** Absorb
**Rationale:** Already absorbed as part of Step 7 of the plan.
**Action:** Covered by Step 7 of the plan.

#### D.10: `fn main()` 135-line monolith (phase-5)
**Decision:** Absorb
**Rationale:** Already absorbed as Step 7 (`build_sweep_tasks` extraction) of the plan.
**Action:** Covered by Step 7 of the plan.

#### D.11: Direct for-loop in parameter sweep (phase-5)
**Decision:** Absorb
**Rationale:** Already absorbed as Step 7 (iterator-based sweep) of the plan.
**Action:** Covered by Step 7 of the plan.

### Plan Amendments

Six amendments applied to `plans/phase-5/PHASE5B_API_ERGONOMICS.md`:

1. **Return type correction (B.4.2):** Corrected `downstream_of` return type from `Vec<String>` to `HashSet<String>` in the plan text.
2. **New imports note (B.3):** Added a note to the `run_default` section that `workflow_utils/src/lib.rs` requires new imports (`Arc`, `Workflow`, `WorkflowSummary`, `WorkflowError`, `StateStore`, `ProcessRunner`, `HookExecutor` from `workflow_core`).
3. **Prelude additive note (B.2):** Added a clarifying note that the explicit `use crate::{...}` list in `workflow_utils/src/prelude.rs` is additive to `workflow_core::prelude::*`; types like `HookExecutor` are already covered by the glob.
4. **`--local` + `--dry-run` interaction (B.6):** Added a one-line clarification that `--local` has no effect in `--dry-run` mode (dry-run exits before execution).
5. **Closure lifetime note (D.10):** Added a note to `build_one_task` that config fields moved into `TaskClosure` closures (`'static`) must be cloned before `move` capture — mirrors the existing for-loop body pattern.
6. **`ExecutionMode::direct` args choice (B.1):** No code change. Noted that `args: &[&str]` was considered vs `impl IntoIterator<Item = impl AsRef<str>>` and rejected for simplicity; the current form is correct.
