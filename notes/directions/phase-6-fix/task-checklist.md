# Phase 6 Fix Directions Task Checklist

**Review date**: 2026-05-05
**Source files reviewed**:
- `/Users/tony/programming/castep_workflow_framework/notes/directions/phase-6-fix/draft-directions.json`
- `/Users/tony/programming/castep_workflow_framework/notes/directions/phase-6-fix/codebase-state.md`
- `/Users/tony/programming/castep_workflow_framework/Cargo.toml` (workspace root)
- `/Users/tony/programming/castep_workflow_framework/examples/hubbard_u_sweep_slurm/src/config.rs`
- `/Users/tony/programming/castep_workflow_framework/examples/hubbard_u_sweep_slurm/src/job_script.rs`
- `/Users/tony/programming/castep_workflow_framework/examples/hubbard_u_sweep_slurm/Cargo.toml`

**External dependency verified**:
- `/Users/tony/programming/castep-cell-io/castep_cell_io/Cargo.toml` exists, version = "0.5.0"
- `KpointsMpGrid(pub [u32; 3])` confirmed (tuple struct, `./cell/bz_sampling_kpoints/kpoints_mp_grid.rs`)
- `CutOffEnergy { value: f64, unit: None }` confirmed (construction pattern at `./param/basis_set_params.rs` lines 81, 95)
- `HubbardUUnit` enum confirmed in `./cell/species/hubbard_u/mod.rs` (NOT `HubburdUUnit` — see issue note below)

---

## Group 1: multi_param_sweep binary

### Task G1-1: Create multi_param_sweep crate skeleton and add to workspace

| Criterion | Assessment |
|---|---|
| Goal clear? | CLEAR — create directory, Cargo.toml, stub files, seed files, workspace member |
| Files in scope correct? | YES — matches the old slurm binary structure (Cargo.toml, 3 src files, 2 seed files) |
| Implementation detail sufficient? | YES — dependency list complete, stub content specified, workspace member insertion point specified |
| Acceptance commands sufficient? | YES — `cargo check --workspace` plus `ls` for file existence |
| Wiring checklist correct? | YES — add mod declarations and workspace member |
| Depends_on/enables explicit? | YES — no depends, enables G1-2, G1-3 |

**Verdict: CLEAR**

---

### Task G1-2: Implement parse functions and SweepConfig in config.rs with TDD

| Criterion | Assessment |
|---|---|
| Goal clear? | CLEAR — implement SweepConfig struct and three parse functions |
| Files in scope correct? | YES — config.rs only |
| Implementation detail sufficient? | YES — full SweepConfig field list with defaults, parse function semantics, error message expectations |
| Acceptance commands sufficient? | YES — cargo check + cargo test with count (20 tests) |
| Depends_on/enables explicit? | YES — depends G1-1, enables G1-4, G1-5 |
| tdd_interface.test_code specific/falsifiable? | PASS — 20 tests total (7 parse_u_values + 7 parse_kpoints + 6 parse_cutoffs), all assert concrete values or error message patterns |
| tdd_interface.signature matches test code? | YES — `parse_u_values`, `parse_kpoints`, `parse_cutoffs` all called by exact name |
| tdd_interface.signature covers all tested functions? | YES — all three functions tested |

**Verdict: CLEAR** — Tests are comprehensive and falsifiable.

---

### Task G1-3: Implement generate_job_script with D.2 fix using TDD

| Criterion | Assessment |
|---|---|
| Goal clear? | CLEAR — implement generate_job_script with D.2 formatting fix |
| Files in scope correct? | YES — job_script.rs |
| Implementation detail sufficient? | YES — heredoc template guidance, D.2 fix requirements, import pattern (SweepConfig from crate::config) |
| Acceptance commands sufficient? | YES — cargo check + cargo test with specific test (no_literal_tabs) |
| Depends_on/enables explicit? | SEE ISSUE below |
| tdd_interface.test_code specific/falsifiable? | PASS — 6 tests, all assert concrete strings or tab absence |
| tdd_interface.signature matches test code? | YES — `generate_job_script` called by exact name |
| tdd_interface.signature covers all tested functions? | YES — only one function under test |

**ISSUE: Dependency contradiction (CRITICAL)** — G1-3 declares `depends_on: ["G1-1"]` and `can_run_in_parallel_with: ["G1-2"]`. However, G1-3's function signature takes `config: &SweepConfig` and its tests import `SweepConfig` from `crate::config`. The `SweepConfig` type is defined in G1-2. If G1-3 runs before G1-2 (as the parallel declaration allows), `SweepConfig` will not exist yet in `config.rs`, causing a compilation failure. G1-3 should either:
  - Add G1-2 as a dependency (removing parallel execution), OR
  - Provide explicit guidance for handling the missing SweepConfig (e.g., "If SweepConfig has not been implemented yet by G1-2, define a minimal placeholder type")

**Verdict: BLOCKED without fix** — Dependency on G1-2's SweepConfig is real, but `can_run_in_parallel_with: ["G1-2"]` says the opposite.

---

### Task G1-4: Implement sweep logic and task builders with TDD

| Criterion | Assessment |
|---|---|
| Goal clear? | CLEAR — implement build_one_scf_task and build_all_scf_tasks |
| Files in scope correct? | YES — main.rs (tests in-module) |
| Implementation detail sufficient? | YES — very detailed: task ID format, workdir path, setup closure steps, collect closure, sweep mode dispatch |
| Acceptance commands sufficient? | YES — cargo check + cargo test; also lists expected behavioral outcomes per test |
| Depends_on/enables explicit? | YES — depends G1-2, G1-3; enables G1-5 |
| tdd_interface.test_code specific/falsifiable? | PASS — 12 tests, all assert concrete task counts, IDs, dependency structure, error messages |
| tdd_interface.signature matches test code? | YES — `build_all_scf_tasks` called directly; `build_one_scf_task` listed but tested indirectly |
| tdd_interface.signature covers all tested functions? | MINOR — `build_one_scf_task` is in the signature array but no test calls it directly. The tests only exercise it via `build_all_scf_tasks`. A subagent could inline the logic into `build_all_scf_tasks` and skip the function entirely while still passing all tests. However, `expected_behavior` does describe both functions, so a diligent subagent should implement both. |

**Verdict: CLEAR** — Detailed and implementable.

---

### Task G1-5: Wire main() entry point for multi_param_sweep

| Criterion | Assessment |
|---|---|
| Goal clear? | CLEAR — implement main() orchestrating config, task building, workflow, dry-run, execution |
| Files in scope correct? | YES — main.rs |
| Implementation detail sufficient? | YES — full main() structure with local/queued/dry-run paths, workflow construction, state store, summary printing |
| Acceptance commands sufficient? | YES — cargo check + 3 cargo run invocations (dry-run normal, dry-run pairwise, pairwise missing args errors) |
| Depends_on/enables explicit? | YES — depends G1-4, enables G3-1 |
| Wiring checklist correct? | N/A (no checklist items — wiring is in main() logic itself) |

**Verdict: CLEAR**

---

## Group 2: scf_dos_chain binary + hubbard_u_sweep update

### Task G2-1: Create scf_dos_chain crate skeleton and add to workspace

| Criterion | Assessment |
|---|---|
| Goal clear? | CLEAR — symmetric to G1-1 |
| Files in scope correct? | YES — same structure as G1-1 |
| Implementation detail sufficient? | YES — same pattern as G1-1 with correct paths |
| Acceptance commands sufficient? | YES — cargo check + ls |
| Depends_on/enables explicit? | YES — depends G1-1 (file collision on root Cargo.toml), enables G2-2 |

**Verdict: CLEAR** — Intentional serialization with G1-1 is documented.

---

### Task G2-2: Implement config.rs and job_script.rs for scf_dos_chain with TDD

| Criterion | Assessment |
|---|---|
| Goal clear? | CLEAR — implement ChainConfig, parse_u_values, generate_job_script |
| Files in scope correct? | YES — config.rs and job_script.rs |
| Implementation detail sufficient? | YES — ChainConfig fields specified (simpler than SweepConfig: single u_value, no sweep fields), parse_u_values duplicated, job_script template identical |
| Acceptance commands sufficient? | YES — cargo check + cargo test for both modules (13 total tests) |
| Depends_on/enables explicit? | YES — depends G2-1, enables G2-3 |
| tdd_interface.test_code specific/falsifiable? | PASS — 7 + 6 = 13 tests, same assertions as G1-2/G1-3 counterparts, adapted for ChainConfig |
| tdd_interface.signature matches test code? | YES — `parse_u_values` and `generate_job_script` called by exact name |
| tdd_interface.test_file format | ISSUE — `test_file` is a descriptive string ("parse_u_values tests in examples/scf_dos_chain/src/config.rs; job_script tests in examples/scf_dos_chain/src/job_script.rs") instead of a clean path. The explore-implement pipeline typically expects a single file path. However, `test_module` says "tests (in each file)" which clarifies the intent. The subagent must split the `test_code` block between two files manually. |

**Verdict: CLEAR** — Functional logic is clear. Test file split requires subagent care but is documented.

---

### Task G2-3: Implement chain task builders (SCF + DOS) with TDD

| Criterion | Assessment |
|---|---|
| Goal clear? | CLEAR — implement build_scf_task and build_dos_task with correct dependency chain |
| Files in scope correct? | YES — main.rs |
| Implementation detail sufficient? | YES — detailed guidance on setup closures, checkpoint copy (Pattern 6), Task::BandStructure, shared workdir |
| Acceptance commands sufficient? | YES — cargo check + cargo test + behavioral criteria list |
| Depends_on/enables explicit? | YES — depends G2-2, enables G2-4 |
| tdd_interface.test_code specific/falsifiable? | PASS — 7 tests, all assert concrete IDs, dependency counts, workdir paths |
| tdd_interface.signature matches test code? | YES — both `build_scf_task` and `build_dos_task` called by exact name |
| Typo in guidance | ISSUE — The import list at line ~473 contains `HubburdUUnit` (typo: extra 'r'). The actual v0.5.0 API enum is `HubbardUUnit` (confirmed via MCP search on the castep-cell-io source). A subagent following this literally will get a compilation error. |

**Verdict: CLEAR** — Content is clear; the `HubburdUUnit` typo will be caught at compile time.

---

### Task G2-4: Wire main() entry point for scf_dos_chain

| Criterion | Assessment |
|---|---|
| Goal clear? | CLEAR — symmetric to G1-5, simpler (2 tasks, no sweep logic) |
| Files in scope correct? | YES — main.rs |
| Implementation detail sufficient? | YES — follows same pattern as G1-5 with explicit chain construction order |
| Acceptance commands sufficient? | YES — cargo check + 2 cargo run invocations |
| Depends_on/enables explicit? | YES — depends G2-3, enables G3-1 |

**Verdict: CLEAR**

---

### Task G2-5: Update hubbard_u_sweep for castep-cell-io v0.5.0 path dep

| Criterion | Assessment |
|---|---|
| Goal clear? | CLEAR — change Cargo.toml dep, fix compilation errors |
| Files in scope correct? | YES — Cargo.toml and main.rs (latter only if compiler reports errors) |
| Implementation detail sufficient? | ADEQUATE — "fix what the compiler reports" is the right strategy for an API migration where exact v0.5.0 signatures are unknown at plan time. Known breakage points are listed (Species::Symbol, builder methods, enum variants). |
| Acceptance commands sufficient? | YES — cargo check -p hubbard_u_sweep (zero errors, zero warnings) |
| Depends_on/enables explicit? | YES — no depends, enables G3-1, can run in parallel with everything |

**Verdict: CLEAR** — Minimal task, correct strategy.

---

## Group 3: Workspace integration, cleanup, verification

### Task G3-1: Remove old crate, finalize workspace, run full verification

| Criterion | Assessment |
|---|---|
| Goal clear? | CLEAR — remove old binary, update workspace, full verification |
| Files in scope correct? | YES — root Cargo.toml (members list) + delete directory |
| Implementation detail sufficient? | YES — exact members list provided, deletion instruction clear |
| Acceptance commands sufficient? | YES — 7 command acceptance criteria: build, test, clippy dead-code, 2 multi_param_sweep dry-run variants, 2 scf_dos_chain dry-run variants, old-directory-gone check |
| Depends_on/enables explicit? | YES — depends G1-5, G2-4, G2-5 (all new binaries + old binary fix complete) |

**Minor note**: The expected members list after removal excludes `'examples/hubbard_u_sweep_slurm'`, keeping 6 members. The guidance says "Do NOT change any other configuration (resolver, workspace.dependencies, etc.)" which is explicit.

**Verdict: CLEAR** — Most thorough verification of all tasks.

---

## Overall Assessment

### Strengths
- Architecture decisions are documented in detail and consistently applied across all tasks.
- Parse-time validation (Pattern 3), in-memory document mutation (Pattern 5), and shared workdir (Pattern 6) are well-motivated and consistently referenced.
- TDD test code is consistently falsifiable — every test asserts concrete values, counts, or error message strings. No trivial assertions found.
- The crate boundary map is precise about which functions are duplicated vs. shared.
- Pitfalls from prior phases (task naming collision, path dep existence, dead-code detection) are explicitly addressed.
- Task IDs (`scf_U{u}_k{k}_c{c}`) are specified with exact formatting rationale.

### Issues Requiring Fixes

1. **CRITICAL — G1-3 dependency contradiction**: G1-3 (`generate_job_script`) depends on `SweepConfig` (defined in G1-2) but declares `depends_on: ["G1-1"]` and `can_run_in_parallel_with: ["G1-2"]`. This is impossible since the function signature and tests both import `SweepConfig`. **Fix**: Change `depends_on` to `["G1-1", "G1-2"]` and remove from `can_run_in_parallel_with`.

2. **MINOR — G2-2 `test_file` format**: The field uses a human-readable description ("parse_u_values tests in ...") instead of a clean file path. The explore-implement pipeline expects a single-path or array-of-paths format. **Fix**: Use `"test_file": ["examples/scf_dos_chain/src/config.rs", "examples/scf_dos_chain/src/job_script.rs"]`.

3. **MINOR — G2-3 typo in import list**: `HubburdUUnit` (typo) should be `HubbardUUnit`. **Fix**: Correct the spelling.

4. **MINOR — G1-4 signature includes untested function**: `build_one_scf_task` is listed in `signature` but no test calls it. An inlining subagent could skip it. The `expected_behavior` partially mitigates this but the test code itself does not force its existence. **Fix**: Either add a test that calls `build_one_scf_task` directly, or remove it from the `signature` array (and rely on `expected_behavior` alone).

### Non-Issues (noted but acceptable)
- The `castep_cell_io::param::general::task::Task` module path for `Task::BandStructure` in G2-3 is flagged as needing LSP verification. This is explicitly documented as a verification step, not a gap.
- The castep-cell-io v0.5.0 API details (Species::Symbol signature, builder method names) are not precisely known at plan time. The "fix compiler errors" strategy in G2-5 is the correct approach.
- Function duplication (parse_u_values, generate_job_script) between G1 and G2 is intentional per the architecture decision `duplication_choice`.

### Verification: TDD Tasks

| Task | tdd_interface.kind | test_code meaningful? | Concrete assertions? | signature matches? | Verdict |
|---|---|---|---|---|---|
| G1-2 | lib-tdd | YES — 20 tests | YES — all assert values or error patterns | YES | PASS |
| G1-3 | lib-tdd | YES — 6 tests | YES — all assert concrete strings | YES | PASS |
| G1-4 | lib-tdd | YES — 12 tests | YES — task counts, IDs, dependencies, errors | YES (minor: build_one_scf_task not directly called) | PASS |
| G2-2 | lib-tdd | YES — 13 tests | YES — same assertions as G1-2/G1-3 | YES | PASS |
| G2-3 | lib-tdd | YES — 7 tests | YES — IDs, dependency count, workdir path | YES | PASS |

All `lib-tdd` tasks have meaningful, falsifiable test code. No `assert!(true)` or other trivial assertions found. The `signature` functions match the functions called in `test_code`.

---

## Final Verdict

**Needs More Detail** — Specifically the G1-3 dependency contradiction (issue #1) must be resolved before implementation can proceed. The remaining issues (typo, test_file format, minor signature gap) are non-blocking but should also be fixed for clarity. Once the dependency issue is resolved, the directions are precise enough to implement all 11 tasks without ambiguity.

### Serialization path after fix

With G1-3 corrected to depend on G1-2:

```
G1-1 ──> G1-2 ──> G1-3 ──> G1-4 ──> G1-5 ──┐
  │                                             │
  └──> G2-1 ──> G2-2 ──> G2-3 ──> G2-4 ──────┤
                                               │
                 G2-5 ─────────────────────────┤
                                               │
                                               v
                                              G3-1
```

G1-2 and G2-2 (and their children) can still run interleaved since G2-2 does not depend on G1-2 at the Rust type level (ChainConfig is independent of SweepConfig).
