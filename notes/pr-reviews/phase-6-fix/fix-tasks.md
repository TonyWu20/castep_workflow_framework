# Phase 6 Fix: Fix directions for defects found in review
**Source branch:** phase-6-fix

## Architecture Notes

These fix directions address defects found during the make-judgement review of phase-6-fix.
Two P1 runtime bugs in `scf_dos_chain`, one missing `HubbardUUnit` import in
`multi_param_sweep`, one unused dependency, one architecture note discrepancy, and G3-1
workspace cleanup.

The `TaskClosure` type alias and manual `Debug` impl added to `workflow_core/src/task.rs`
are accepted as necessary enablers for the binary crates. They do not alter the library's
public API contract meaningfully.

## Known Pitfalls

Fix order: Apply P1 fixes first (functional correctness), then P2/P3 fixes (cleanup).
G3-1 should run last since it modifies workspace-wide state.

## Task Group: fixes
**Reason:** Fix 6 defects found during make-judgement review of phase-6-fix.
**Depends on groups:** none

### FIX-1: Fix SCF task job script using wrong task_id
**Files:** `examples/scf_dos_chain/src/main.rs`
**Depends on:** none
**Kind:** direct

**Changes:**
- **modify** `examples/scf_dos_chain/src/main.rs`:
  Change the SCF setup closure's `generate_job_script` call from
  `generate_job_script(config, &seed_name, &seed_name)` to
  `generate_job_script(config, "scf", &seed_name)`.
  The first parameter is the task_id for the SBATCH --job-name directive,
  which should be `"scf"` to match the task ID.

**Acceptance:**
- `cargo check -p scf_dos_chain`
- `cargo test -p scf_dos_chain -- job_script::tests::contains_sbatch_directives`
  (verify #SBATCH --job-name="scf" is produced)

### FIX-2: Fix DOS collect closure ignoring workdir path
**Files:** `examples/scf_dos_chain/src/main.rs`
**Depends on:** none
**Kind:** direct

**Changes:**
- **modify** `examples/scf_dos_chain/src/main.rs`:
  Replace `|_: &Path|` with `|path: &Path|` in the DOS collect closure.
  Replace `PathBuf::from("ZnO_DOS.castep")` with `path.join("ZnO_DOS.castep")`.
  This matches the SCF collect pattern which correctly uses the workdir path argument.

**Acceptance:**
- `cargo check -p scf_dos_chain`

### FIX-3: Add HubbardUUnit import and .unit() call
**Files:** `examples/multi_param_sweep/src/main.rs`
**Depends on:** none
**Kind:** direct

**Changes:**
- **modify** `examples/multi_param_sweep/src/main.rs`:
  Add `HubbardUUnit` to the import from `castep_cell_io::cell::species`:
  change the existing import to include `HubbardUUnit` alongside
  `AtomHubbardU`, `HubbardU`, `OrbitalU`, `Species`.
  Add `.unit(HubbardUUnit::ElectronVolt)` to the `HubbardU::builder()` chain
  before `.atom_u_values()`. The correct chain is:
  `HubbardU::builder().unit(HubbardUUnit::ElectronVolt).atom_u_values(vec![atom_u]).build()`

**Acceptance:**
- `cargo check -p multi_param_sweep`
- `cargo test -p multi_param_sweep`

### FIX-4: Remove unused itertools dependency
**Files:** `examples/scf_dos_chain/Cargo.toml`
**Depends on:** none
**Kind:** direct

**Changes:**
- **modify** `examples/scf_dos_chain/Cargo.toml`:
  Remove the line `itertools = { workspace = true }` from `[dependencies]`.
  `itertools` is not imported or used in any scf_dos_chain source file.

**Acceptance:**
- `cargo check -p scf_dos_chain` (must compile without itertools)
- `cargo test -p scf_dos_chain` (all tests must pass)

### FIX-5: Accept TaskClosure changes in workflow_core/src/task.rs
**Files:** `workflow_core/src/task.rs`
**Depends on:** none
**Kind:** direct

**Changes:**
The `TaskClosure` type alias and manual `Debug` impl added to
`workflow_core/src/task.rs` are accepted as necessary enablers for the binary
crates. No reversion needed — the Architecture Notes section above documents
this as an intentional deviation.

- (no source changes needed)

**Acceptance:**
- `cargo check --workspace`

### FIX-6: Document castep-cell-fmt path dep deviation in hubbard_u_sweep
**Files:** `examples/hubbard_u_sweep/Cargo.toml`
**Depends on:** none
**Kind:** direct

**Changes:**
- **modify** `examples/hubbard_u_sweep/Cargo.toml`:
  No code change needed — the path dep `{ version = "0.1.0", path = "../castep-cell-io/castep_cell_fmt" }`
  was likely necessary for Cargo resolution with the local castep-cell-io dependency.
  Verify it compiles, then keep as an intentional deviation.

**Acceptance:**
- `cargo check -p hubbard_u_sweep` (must compile)

## Task Group: cleanup
**Reason:** Execute G3-1 workspace cleanup that was not yet performed.
**Depends on groups:** fixes

### FIX-7: Execute G3-1 workspace cleanup
**Files:** `Cargo.toml` (workspace root), `examples/hubbard_u_sweep_slurm/`
**Depends on:** FIX-1, FIX-2, FIX-3, FIX-4, FIX-5, FIX-6
**Kind:** direct

**Changes:**
- **modify** `Cargo.toml` (workspace root):
  Remove `"examples/hubbard_u_sweep_slurm"` from the `[workspace]` members array.
  The members list should contain exactly:
  `'workflow_core'`, `'workflow_utils'`, `'examples/hubbard_u_sweep'`,
  `'examples/multi_param_sweep'`, `'examples/scf_dos_chain'`, `'workflow-cli'`.
  Do NOT change any other configuration.

- **delete** `examples/hubbard_u_sweep_slurm/`:
  Delete the entire directory and all contents.

**Acceptance:**
- `cargo build --workspace` (all crates compile with zero errors, zero warnings)
- `cargo test --workspace` (all tests pass)
- `cargo clippy --workspace --all-targets -- -W dead-code` (no dead-code warnings)
- `cargo run --bin multi_param_sweep -- --dry-run` (produces correct topological order with 6 default tasks)
- `cargo run --bin scf_dos_chain -- --dry-run` (produces 'scf' then 'dos')
- `ls examples/hubbard_u_sweep_slurm/ 2>&1 | grep -q 'No such file' || false`
