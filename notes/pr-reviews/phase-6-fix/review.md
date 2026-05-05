# Review: Phase 6 Fix — Rewrite example binaries for multi-parameter sweep & SCF+DOS chain

**Index**: notes/directions/phase-6-fix/directions-index.json
**Reviewed**: 2026-05-05

## Summary

**PASS WITH ISSUES** — 6 issues found (2 P1, 3 P2, 1 P3). The core-1 and core-2 groups are substantially complete with correct architecture. G3-1 (workspace cleanup) has not been executed. Two runtime bugs exist in `scf_dos_chain/src/main.rs`.

## Per-Task Results

### Group core-1: multi_param_sweep crate

| Task | Status | Detail |
|------|--------|--------|
| G1-1 | ✓ Passed | Crate skeleton, Cargo.toml, stubs, seed files, workspace member all created |
| G1-2 | ✓ Passed | SweepConfig with 16 fields, parse_u_values/parse_kpoints/parse_cutoffs with all 20 tests |
| G1-3 | ✓ Passed | generate_job_script with D.2 fix, all 6 ported tests passing |
| G1-4 | ✓ Passed | build_one_scf_task, build_all_scf_tasks, 12 sweep combinatorics tests — minor: missing HubbardUUnit import |
| G1-5 | ✓ Passed | main() with dry-run and execution modes, all wiring correct |

### Group core-2: scf_dos_chain crate + hubbard_u_sweep

| Task | Status | Detail |
|------|--------|--------|
| G2-1 | ✓ Passed | Crate skeleton, Cargo.toml, stubs, seed files, workspace member all created |
| G2-2 | ✓ Passed | ChainConfig, parse_u_values, generate_job_script with D.2 fix, all 13 tests |
| G2-3 | ⚠ Minor Issues | build_scf_task and build_dos_task implemented — 2 bugs found (P1 each) |
| G2-4 | ✓ Passed | main() with dry-run and execution modes |
| G2-5 | ✓ Passed | hubbard_u_sweep path dep update compiles — minor castep-cell-fmt path dep deviation |

### Group workspace: Cleanup

| Task | Status | Detail |
|------|--------|--------|
| G3-1 | ✗ Not Executed | hubbard_u_sweep_slurm not removed from workspace members; directory still exists |

## Issues Found

### P1-1: SCF job script uses wrong task_id

**File**: `examples/scf_dos_chain/src/main.rs:41`
**Severity**: P1 (runtime functional bug)

The SCF task calls `generate_job_script(config, &seed_name, &seed_name)` passing the seed name "ZnO" as the task_id for the `#SBATCH --job-name` directive. It should pass `"scf"` — change to `generate_job_script(config, "scf", &seed_name)`. The DOS task on line 129 correctly uses `"dos"`.

### P1-2: DOS collect closure ignores workdir path

**File**: `examples/scf_dos_chain/src/main.rs:162-163`
**Severity**: P1 (runtime functional bug)

The DOS collect closure uses `|_: &Path|` discarding the workdir path argument, then reads from `PathBuf::from("ZnO_DOS.castep")` — a bare relative path. This will fail at runtime if CWD ≠ workdir. Fix: use `|path: &Path|` and `path.join("ZnO_DOS.castep")`, matching the SCF collect pattern at lines 93-94.

### P2-1: Missing HubbardUUnit import and .unit() call

**File**: `examples/multi_param_sweep/src/main.rs:649,785-787`
**Severity**: P2 (potential correctness issue)

The `HubbardU` builder is called without `.unit(HubbardUUnit::ElectronVolt)`, and `HubbardUUnit` is not imported. If the builder's default unit differs from ElectronVolt, the generated `.cell` output will be incorrect. Add `HubbardUUnit` to the import and add `.unit(HubbardUUnit::ElectronVolt)` to the builder chain.

### P2-2: Unused itertools dependency in scf_dos_chain

**File**: `examples/scf_dos_chain/Cargo.toml`
**Severity**: P2 (unnecessary dependency)

`itertools = { workspace = true }` is listed but never used in any scf_dos_chain source file. Remove from Cargo.toml.

### P2-3: workflow_core/task.rs modified despite architecture notes

**File**: `workflow_core/src/task.rs`
**Severity**: P2 (architecture note contradiction)

Two additions were made: `TaskClosure` type alias (line 8) and manual `Debug` impl for `Task` (lines 68-81). The architecture notes state library crates should be unchanged. The `TaskClosure` alias is used by both new binaries. Accept the addition and update architecture notes, or revert and inline the closure type.

### P3-1: castep-cell-fmt path dep deviation in hubbard_u_sweep

**File**: `examples/hubbard_u_sweep/Cargo.toml:12`
**Severity**: P3 (minor — likely necessary)

`castep-cell-fmt` was given a path dep `{ version = "0.1.0", path = "../castep-cell-io/castep_cell_fmt" }` beyond the original direction. Likely necessary for Cargo resolution with local path deps. Document the rationale.

## Unresolved Work

### G3-1: Workspace cleanup — not executed

The following actions remain:
1. Remove `"examples/hubbard_u_sweep_slurm"` from `[workspace] members` in root `Cargo.toml`
2. Delete the entire `examples/hubbard_u_sweep_slurm/` directory
3. Run full verification: `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -W dead-code`, dry-run both binaries

## Deferred Items

See `deferred.md` for items flagged for future phases.

## Strategic Assessment

The architecture is sound — purpose-specific binaries, in-memory document mutation, parse-time validation, and duplication over extraction all follow the directions correctly. The two runtime bugs (P1-1, P1-2) are straightforward to fix. The unused dep (P2-2) and missing unit (P2-1) are minor. G3-1 remains to be executed as the final wiring step.
