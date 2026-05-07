# Draft Architectural Elaboration -- Phase 6 Fix

> Generated from the plan at `plans/phase-6-fix/PHASE_6_FIX_PLAN.md`.
> Inputs: `deferred-and-patterns.md`, `codebase-state.md`, workspace-map.json,
> and source inspection of `hubbard_u_sweep`, `hubbard_u_sweep_slurm`.

---

## 1. Goal Decomposition and Design Decisions

The plan has five high-level goals. Below is the architectural reasoning for each.

### 1.1 Goal: Create `multi_param_sweep` binary

**Decision: Parse-time validation at CLI boundary, not in setup closures.**

The plan defines `parse_kpoints(s: &str) -> anyhow::Result<Vec<[u32; 3]>>` and `parse_cutoffs(s: &str) -> anyhow::Result<Vec<f64>>`. These must run *before* task construction, not inside setup closures. Rationale: if a user provides `"8xx8"` as a k-point, they should get an error immediately at CLI parse time, not three hours later when the 12th SCF task is about to start. This follows Pattern 3 (parse-time validation gates) and mitigates F.8 (string-format coupling without documentation).

**Decision: `parse_kpoints` uses `[u32; 3]` not a custom struct.**

The internal representation `[u32; 3]` is directly convertible to `KpointsMpGrid([kx, ky, kz])` with zero overhead. Introducing a `KpointSpec` newtype adds indirection for no benefit — the validation lives in parsing, and the type guarantee ("exactly 3 u32s") is enforced by the array type itself.

**Decision: `parse_u_values` is imported, not duplicated.**

The old `hubbard_u_sweep_slurm/src/config.rs` has a well-tested `parse_u_values` function. Since both new binaries and the updated `hubbard_u_sweep` need it, there are three options:
1. Duplicate it in both new binaries (plan appears to assume this)
2. Extract to a shared utility
3. Import from the old binary (impossible after deletion)

**Chosen: Duplicate.** The function is ~15 lines and has stable semantics. Extracting it to a shared library would require a new crate or adding it to `workflow_utils`, both of which are overkill for a comma-separated f64 parser. The slight duplication is acceptable; the tests port along with each copy. If a third binary ever needs it, extraction becomes justified.

**Decision: Setup closure orchestrates three parameter injections, not one.**

Each task's setup closure must:
1. Parse seed cell → inject `hubbard_u` → serialize
2. If kpoints provided: mutate `cell_doc.kpoints_mp_grid` → re-serialize
3. Parse seed param → inject `cutoff_energy` → serialize
4. If Slurm: write `job.sh`

The closures are constructed with `move` captures for all parameter values. This is heavyweight but correct — each task gets immutable, pre-computed values in its closure. The pattern follows the existing `hubbard_u_sweep` binary's approach.

**Decision: Sweep product logic uses `itertools::iproduct!` for clarity.**

Rather than nested loops, `itertools::iproduct!` yields a flat iterator of all combinations. This makes the Cartesian product explicit and the pairwise zip path equally readable. The `itertools` crate is already a workspace dependency.

### 1.2 Goal: Create `scf_dos_chain` binary

**Decision: Task name collision resolved via qualified path on `Task::BandStructure` only.**

The collision is between `workflow_core::task::Task` (used ~20 times per binary) and `castep_cell_io::param::general::Task` (used exactly once: setting `Task::BandStructure` in the DOS param setup). Two resolution strategies:

| Strategy | Pro | Con |
|----------|-----|-----|
| Alias: `use ... as CastepTask` | Clear at use-site | Still clutters import namespace with two "Task" things |
| Qualified path: `castep_cell_io::param::general::Task::BandStructure` at the single use-site | No alias, no import shadow risk, minimal scope | Verbose at one line |

**Chosen: Fully-qualified path at the single use-site.** The `scf_dos_chain` binary imports `workflow_utils::prelude::*` (which re-exports `workflow_core::task::Task`). Since `castep_cell_io::param::general::Task` is needed exactly once (to set `general.task = Some(Task::BandStructure)`), the fully-qualified path keeps the import surface clean and avoids any risk of shadow confusion. The DOS setup closure contains:

```rust
doc.general.task = Some(castep_cell_io::param::general::task::Task::BandStructure);
```

(Confirm the exact module path at implementation time — it depends on how `castep-cell-io` v0.5.0 re-exports the `Task` enum.)

**Decision: Shared workdir pattern — both SCF and DOS tasks in `runs/scf_dos/`.**

This is specified in the plan (Pattern 6). The SCF task populates the workdir with `ZnO.check`; the DOS task's setup copies it to `ZnO_DOS.check` within the same directory. No cross-workdir path resolution needed. The DOS task executes `castep ZnO_DOS` (different seed name, same directory).

**Decision: DOS collect step mirrors SCF collect — verify `ZnO_DOS.castep` exists and contains "Total time".**

Consistency with the SCF collect pattern. This is a lightweight correctness check, not a full output parser.

### 1.3 Goal: Delete `hubbard_u_sweep_slurm` and all its files

**Decision: Port tests before deleting, not after.**

The plan says "Port existing tests from `hubbard_u_sweep_slurm` before deleting it." The port target is:
- `config.rs` tests (7 `parse_u_values` tests) → `multi_param_sweep/src/config.rs`
- `job_script.rs` tests (6 tests) → both `multi_param_sweep/src/job_script.rs` and `scf_dos_chain/src/job_script.rs`

The `no_literal_tabs` test must be ported and must pass with the cleaned heredoc template (D.2).

**Decision: Delete is atomic — one commit removes the entire directory.**

After all tests are ported and verified passing, the entire `examples/hubbard_u_sweep_slurm/` directory is removed in a single operation. The `.validation-complete` marker file is deleted along with it.

### 1.4 Goal: Update workspace `Cargo.toml`

**Decision: Workspace member list changes are atomic and last.**

The workspace `Cargo.toml` members list must match the filesystem. The sequence:
1. Create `examples/multi_param_sweep/` (with its `Cargo.toml`)
2. Create `examples/scf_dos_chain/` (with its `Cargo.toml`)
3. Update workspace members: remove `hubbard_u_sweep_slurm`, add both new entries
4. Delete `examples/hubbard_u_sweep_slurm/`

Steps 3-4 can happen in either order, but doing 3 first means `cargo build --workspace` won't try to resolve the deleted crate. Actually, after deletion, `cargo` won't find the member and will error. So the order is: update workspace members, then delete.

**Decision: Local path dep for `castep-cell-io` in all affected Cargo.tomls.**

All affected Cargo.tomls (`multi_param_sweep`, `scf_dos_chain`, `hubbard_u_sweep`) use:
```toml
castep-cell-io = { path = "../castep-cell-io/castep_cell_io" }
```
This resolves to a sibling workspace on the local filesystem. The `castep-cell-fmt` crate is already version `0.1.0` from crates.io and does not need a path dep (it is unchanged between v0.4.0 and v0.5.0 of `castep-cell-io`).

### 1.5 Goal: Update `hubbard_u_sweep` for v0.5.0 API

**Decision: Minimal change — only Cargo.toml and ensure compilation.**

The existing `hubbard_u_sweep/src/main.rs` already uses `castep_cell_fmt::parse::<CellDocument>(&input)` — the v0.5.0 parse API. The code does not use any v0.4.0-specific builders or types. The only required change is the `Cargo.toml` dependency:
```diff
- castep-cell-io = "0.4.0"
+ castep-cell-io = { path = "../castep-cell-io/castep_cell_io" }
```

If compilation reveals API breakage (e.g., `Species::Symbol` now takes a different type, `OrbitalU::D(f64)` changed signature), fix those at implementation time. The plan's verification step (`cargo build --workspace`) will catch these.

---

## 2. Crate Boundary Decisions

### 2.1 What goes where

| Code | Location | Rationale |
|------|----------|-----------|
| `parse_u_values` | Duplicated in `multi_param_sweep` and `scf_dos_chain` `config.rs` | Stable 15-line function; extraction overhead not justified for 2 consumers |
| `parse_kpoints` | `multi_param_sweep/src/main.rs` (or `config.rs` — see 2.2) | Only used by Binary 1; Binary 2 has no k-point sweeping |
| `parse_cutoffs` | Same as `parse_kpoints` | Same reasoning |
| `generate_job_script` | Duplicated in both binaries' `job_script.rs` | Stable template function; D.2 formatting fix applies to both copies |
| `SweepConfig` / `ChainConfig` | Each binary's `config.rs` | Structurally different CLI surfaces; no shared base type |
| Sweep logic (`build_sweep_tasks`) | `multi_param_sweep/src/main.rs` | Binary-specific combinatorics |
| Chain logic (`build_chain`) | `scf_dos_chain/src/main.rs` | Binary-specific task wiring |
| Seed files | Each binary's `seeds/` directory | Self-contained binaries; identical content is acceptable |

### 2.2 `config.rs` vs `main.rs` boundary

The plan shows CLI config in `config.rs` and sweep/chain logic in `main.rs`. This is a convention, not a strict boundary. The parsing utility functions could live in either file. Recommended split:

- `config.rs`: `#[derive(Parser)]` struct, `parse_u_values`, `parse_kpoints`, `parse_cutoffs`, and their tests.
- `main.rs`: `build_one_task` (or equivalent), sweep/chain builders, `main()` entry point.

This keeps CLI surface and validation separate from workflow construction. It matches the existing `hubbard_u_sweep_slurm` structure.

### 2.3 No new library crates

No new library crates are created. All new code lives in binary (example) crates. This is correct: the new code is workflow _usage_ (orchestration, not abstractions). The library crates (`workflow_core`, `workflow_utils`) are unchanged.

If a third binary later needs `parse_kpoints` or `generate_job_script`, extraction to a shared utility crate becomes appropriate. The threshold is 3+ consumers.

---

## 3. Pattern Requirements

### 3.1 Patterns to follow (from `deferred-and-patterns.md`)

All seven patterns from the deferred document apply. Here is the concrete implementation guidance for each:

**Pattern 1: Purpose-specific binaries over mode-flags.**
Already satisfied by the plan's two-binary design. Implementation check: neither binary should have a `--sweep-mode` flag or any flag that toggles between fundamentally different workflows.

**Pattern 2: Parameter encoding in task IDs.**
Implementation check: `multi_param_sweep` task IDs must be `scf_U{u}_k{k}_c{c}` (all three params encoded). `scf_dos_chain` uses `scf` and `dos` (single parameter set, no collision risk). Use `format!` with the exact pattern string from the plan.

**Pattern 3: Parse-time validation gates.**
Implementation check: Every parsing function returns `anyhow::Result<T>` with a message containing the expected format and what went wrong. Examples:
- `parse_kpoints("8x8")` → `Err(anyhow!("invalid k-point '8x8': expected format 'NxNxN' with exactly 3 axes, got 2"))`
- `parse_kpoints("")` → `Err(anyhow!("kpoints list is empty"))`
- `parse_cutoffs("300,abc,800")` → `Err(anyhow!("invalid cutoff 'abc': expected a number"))`

**Pattern 4: Explicit defaults.**
Implementation check: `--sweep-mode` defaults to `"product"` (most complete sweep). `--kpoints` and `--cutoffs` are `Option<String>` defaulting to `None` (seed defaults used). Pairwise mode errors if either is `None`.

**Pattern 5: In-memory document mutation.**
Implementation check: All CASTEP file modifications go through `parse → mutate typed fields → to_cell_file() → to_string_many_spaced()`. Never use `format!` on raw strings to build .cell/.param content.

**Pattern 6: Shared workdir for chained tasks.**
Implementation check: `scf_dos_chain` places both tasks in `runs/scf_dos/`. The DOS setup copies `ZnO.check → ZnO_DOS.check` in the same directory.

**Pattern 7: Clean heredoc template.**
Implementation check: The `no_literal_tabs` test must pass. Use consistent space-only indentation. SBATCH directives use consistent quoting (double-quotes around values that might contain spaces).

### 3.2 Patterns NOT to follow (anti-patterns to avoid)

**Anti-pattern: Multi-year functions.** The old `build_sweep_tasks` function is ~50 lines with nested match arms and mode-specific logic. The new binary should have smaller, single-purpose functions:
- `build_one_scf_task(config, u, kpoint, cutoff, seed_cell, seed_param) -> Result<Task>`
- `build_all_scf_tasks(config, seed_cell, seed_param) -> Result<Vec<Task>>` (dispatches to product or pairwise)
- `main()` orchestrates but does not contain sweep logic

**Anti-pattern: Mode flag for workflow type.** Neither new binary should have a flag that toggles between "independent tasks" and "chained tasks".

---

## 4. Type Signatures for Key New Types

### 4.1 `multi_param_sweep` config

```rust
#[derive(Parser, Debug)]
#[command(name = "multi_param_sweep")]
pub struct SweepConfig {
    /// Comma-separated Hubbard U values (eV), e.g. "0.0,1.0,2.0,3.0,4.0,5.0"
    #[arg(long, default_value = "0.0,1.0,2.0,3.0,4.0,5.0")]
    pub u_values: String,

    /// Comma-separated MP grids, e.g. "8x8x8,6x6x6". Omit to use seed defaults.
    #[arg(long)]
    pub kpoints: Option<String>,

    /// Comma-separated cutoff energies (eV), e.g. "300,500,800". Omit to use seed defaults.
    #[arg(long)]
    pub cutoffs: Option<String>,

    /// Sweep mode: "product" (Cartesian product) or "pairwise" (zip — all three lists must be same length)
    #[arg(long, default_value = "product")]
    pub sweep_mode: String,

    /// Element for Hubbard U
    #[arg(long, default_value = "Zn")]
    pub element: String,

    /// Orbital: 'd' or 'f'
    #[arg(long, default_value = "d")]
    pub orbital: char,

    /// CASTEP input file prefix
    #[arg(long, default_value = "ZnO")]
    pub seed_name: String,

    /// Max concurrent tasks
    #[arg(long, default_value_t = 4)]
    pub max_parallel: usize,

    /// Direct process execution (no SLURM)
    #[arg(long)]
    pub local: bool,

    /// Print topological order and exit
    #[arg(long)]
    pub dry_run: bool,

    /// CASTEP binary name or path (local mode)
    #[arg(long, default_value = "castep")]
    pub castep_command: String,

    /// Root directory for runs/logs
    #[arg(long, default_value = ".")]
    pub workdir: String,

    // --- SLURM fields (same as old binary) ---
    #[arg(long, env = "CASTEP_SLURM_PARTITION", default_value = "debug")]
    pub partition: String,

    #[arg(long, default_value_t = 16)]
    pub ntasks: u32,

    #[arg(long, env = "CASTEP_NIX_FLAKE",
          default_value = "git+ssh://git@github.com/TonyWu20/CASTEP-25.12-nixos#castep_25_mkl")]
    pub nix_flake: String,

    #[arg(long, env = "CASTEP_MPI_IF", default_value = "enp6s0")]
    pub mpi_if: String,
}
```

### 4.2 `scf_dos_chain` config

```rust
#[derive(Parser, Debug)]
#[command(name = "scf_dos_chain")]
pub struct ChainConfig {
    /// Single Hubbard U value (eV)
    #[arg(long, default_value_t = 3.0)]
    pub u_value: f64,

    /// Element for Hubbard U
    #[arg(long, default_value = "Zn")]
    pub element: String,

    /// Orbital: 'd' or 'f'
    #[arg(long, default_value = "d")]
    pub orbital: char,

    /// CASTEP input file prefix
    #[arg(long, default_value = "ZnO")]
    pub seed_name: String,

    /// Max concurrent tasks
    #[arg(long, default_value_t = 1)]
    pub max_parallel: usize,

    /// Direct process execution (no SLURM)
    #[arg(long)]
    pub local: bool,

    /// Print topological order and exit
    #[arg(long)]
    pub dry_run: bool,

    /// CASTEP binary name or path (local mode)
    #[arg(long, default_value = "castep")]
    pub castep_command: String,

    /// Root directory for runs/logs
    #[arg(long, default_value = ".")]
    pub workdir: String,

    // --- SLURM fields (same as SweepConfig) ---
    #[arg(long, env = "CASTEP_SLURM_PARTITION", default_value = "debug")]
    pub partition: String,

    #[arg(long, default_value_t = 16)]
    pub ntasks: u32,

    #[arg(long, env = "CASTEP_NIX_FLAKE",
          default_value = "git+ssh://git@github.com/TonyWu20/CASTEP-25.12-nixos#castep_25_mkl")]
    pub nix_flake: String,

    #[arg(long, env = "CASTEP_MPI_IF", default_value = "enp6s0")]
    pub mpi_if: String,
}
```

### 4.3 Parsing utility signatures

```rust
/// Parse a comma-separated list of u32 triplets.
///
/// Each segment must be exactly "NxNxN" format.
/// Returns clear anyhow errors for wrong number of axes or non-numeric input.
///
/// # Errors
/// - `anyhow!("kpoints list is empty")` for empty or whitespace-only input
/// - `anyhow!("invalid k-point '8x8': expected 3 axes, got 2")` for wrong format
/// - `anyhow!("invalid k-point 'abc': ...")` for non-numeric axes
pub fn parse_kpoints(s: &str) -> anyhow::Result<Vec<[u32; 3]>>;

/// Parse a comma-separated list of f64 values.
///
/// Each segment is trimmed before parsing.
///
/// # Errors
/// - `anyhow!("invalid cutoff '{}': {parse_error}")` for non-numeric tokens
/// - `anyhow!("cutoffs list is empty")` for empty input
pub fn parse_cutoffs(s: &str) -> anyhow::Result<Vec<f64>>;

/// Parse a comma-separated list of f64 values (ported from old binary).
/// Same semantics as parse_cutoffs but with Hubbard-U-specific error context.
pub fn parse_u_values(s: &str) -> anyhow::Result<Vec<f64>>;
```

### 4.4 `generate_job_script` signature

```rust
/// Generate a SLURM job script for the given task.
///
/// The script uses a clean heredoc template with consistent space-only
/// indentation and double-quoted SBATCH directive values.
///
/// Passes the `no_literal_tabs` test: output must not contain literal '\t'.
pub fn generate_job_script(config: &SweepConfig, task_id: &str, seed_name: &str) -> String;
```

For `scf_dos_chain`, the function takes `&ChainConfig` instead of `&SweepConfig`. Since the SLURM fields are identical, the function body is the same. The config parameter type is the only difference.

**Open question**: Should `generate_job_script` accept a trait (e.g., `SlurmConfig`) to deduplicate between the two binaries? Decision: No. The two copies are ~30 lines each and have independent lifetimes. A trait for two impls adds an abstraction layer with no immediate benefit. If a third SLURM config variant appears, introduce the trait then.

### 4.5 Task builder signatures (in main.rs)

```rust
/// Build a single SCF task for one (U, kpoint, cutoff) combination.
///
/// The setup closure:
/// 1. Parses seed cell, injects HubbardU block
/// 2. If kpoint provided, sets `cell_doc.kpoints_mp_grid`
/// 3. Parses seed param, injects cutoff_energy
/// 4. Serializes both documents to the workdir
/// 5. If running SLURM, writes job.sh
///
/// The collect closure verifies `<seed>.castep` exists and contains "Total time".
fn build_one_scf_task(
    config: &SweepConfig,
    u: f64,
    kpoint: Option<[u32; 3]>,
    cutoff: Option<f64>,
    seed_cell: &str,
    seed_param: &str,
) -> Result<Task, WorkflowError>;
```

For `scf_dos_chain`:

```rust
/// Build the SCF task for the chain (no k-point/cutoff variation).
fn build_scf_task(
    config: &ChainConfig,
    seed_cell: &str,
    seed_param: &str,
) -> Result<Task, WorkflowError>;

/// Build the DOS task that depends on scf_task.
/// Copies <workdir>/ZnO.check → <workdir>/ZnO_DOS.check in setup.
/// Sets general.task = Task::BandStructure in the param.
/// Execute castep on ZnO_DOS seed.
fn build_dos_task(
    config: &ChainConfig,
    scf_task_id: &str,
    seed_cell: &str,
    seed_param: &str,
) -> Result<Task, WorkflowError>;
```

---

## 5. Known Pitfalls and Constraints

### 5.1 The `Task` naming collision (P0)

As documented in codebase-state.md Observation B: `use workflow_utils::prelude::*` brings `workflow_core::task::Task` into scope. Using `use castep_cell_io::param::general::Task` in the same file will shadow it.

**Resolution in `scf_dos_chain/src/main.rs`**: Do not import `castep_cell_io::param::general::Task`. Use the fully qualified path at the single use-site in the DOS setup closure:

```rust
doc.general.task = Some(castep_cell_io::param::general::task::Task::BandStructure);
```

The exact module path depends on how `castep-cell-io` v0.5.0 re-exports the `Task` enum. Verify during implementation with LSP hover on the type.

### 5.2 Local path dep must exist on the filesystem

The path `../castep-cell-io/castep_cell_io` must resolve to a sibling directory that contains a `Cargo.toml` with the `castep-cell-io` crate. If the user's filesystem layout differs, this path dep will fail. The plan assumes the default layout where `castep-cell-io` is a sibling workspace.

**Pre-implementation check**: Verify `../castep-cell-io/castep_cell_io/Cargo.toml` exists. If it doesn't, the path needs adjustment.

### 5.3 `castep-cell-fmt` version compatibility

`castep-cell-fmt = "0.1.0"` is used from crates.io while `castep-cell-io` uses a local path dep at v0.5.0. If `castep-cell-io` v0.5.0 depends on a newer `castep-cell-fmt`, Cargo will resolve the semver-compatible version automatically. No conflict expected (both are parse-time libraries, not runtime).

### 5.4 Pairwise mode validation ordering

The plan says pairwise mode "requires both --kpoints and --cutoffs to be explicitly provided." The validation logic is:

```rust
match config.sweep_mode.as_str() {
    "pairwise" => {
        let kpoints = config.kpoints.as_ref()
            .ok_or_else(|| anyhow!("pairwise mode requires --kpoints to be specified"))?;
        let cutoffs = config.cutoffs.as_ref()
            .ok_or_else(|| anyhow!("pairwise mode requires --cutoffs to be specified"))?;
        let kpts = parse_kpoints(kpoints)?;
        let cuts = parse_cutoffs(cutoffs)?;
        let u_vals = parse_u_values(&config.u_values)?;
        if kpts.len() != u_vals.len() || cuts.len() != u_vals.len() {
            anyhow::bail!(
                "pairwise mode requires all three lists to have the same length \
                 (U values: {}, kpoints: {}, cutoffs: {})",
                u_vals.len(), kpts.len(), cuts.len()
            );
        }
        // zip and build tasks
    }
    "product" => { /* Cartesian product — optional kpoints/cutoffs OK */ }
    _ => anyhow::bail!("unknown sweep mode '{}', expected 'product' or 'pairwise'", config.sweep_mode),
}
```

### 5.5 `hubbard_u_sweep` compatibility with v0.5.0 path dep

The old binary uses:
```rust
castep_cell_fmt::{format::to_string_many_spaced, parse, ToCellFile}
castep_cell_io::cell::species::{AtomHubbardU, HubbardU, HubbardUUnit, OrbitalU, Species}
castep_cell_io::CellDocument
```

All of these should work unchanged with v0.5.0. The `parse` free function and `to_cell_file()` pattern already match the v0.5.0 API. However, v0.5.0 may have changed:
- `Species::Symbol(String)` signature (could be `Species::Symbol(&str)` or a newtype)
- Builder method names on `AtomHubbardU` / `HubbardU`
- `HubbardUUnit` enum variants

If compilation fails after the path-dep change, fix these call sites. The build error messages will point to the exact location.

### 5.6 `cargo test --workspace` with new path dep

Tests in `workflow_core` (including `tests/hubbard_u_sweep.rs`) may not resolve the `castep-cell-io` path dep because `workflow_core` does not depend on `castep-cell-io`. The integration test `test_hubbard_u_sweep_with_mock_castep` uses mock executables and never touches `castep-cell-io`. This should continue to pass.

However, if any test transitively exercises `hubbard_u_sweep`'s task construction (which does use `castep-cell-io`), the path dep must be resolvable from that test's Cargo context. Since the test is in `workflow_core`, not in the example binary, this is unlikely to be an issue.

### 5.7 Dead-code detection after refactoring (mitigating F.3)

After deleting `hubbard_u_sweep_slurm`, run:
```
cargo clippy --workspace --all-targets -- -W dead-code
```
to detect any stale imports or dead functions left over from the refactoring. The old binary's crate will be gone, but any shared constants or utility code that was only used by it should be found.

### 5.8 Carrier of `no_literal_tabs` test and D.2 formatting fix

The `no_literal_tabs` test from the old `job_script.rs` must be ported to both new binaries. The D.2 fix (clean heredoc template) must be applied to the ported `generate_job_script` function. The test verifies the fix.

Implementation checklist for D.2:
- [ ] `generate_job_script` uses all-space indentation (no literal `\t`)
- [ ] SBATCH directives use double-quotes consistently: `#SBATCH --job-name="value"`
- [ ] Shell continuation lines use `\\` not literal line breaks mixed with indentation
- [ ] `no_literal_tabs` test passes in both binaries

### 5.9 Deferred items: what NOT to do

- **D.1 (portable SLURM config)**: Do NOT implement now. Precondition not met (still on NixOS cluster). Keep NixOS-specific fields.
- **D.3 (unit tests for generate_job_script)**: Do NOT expand beyond the ported tests. The existing tests are NixOS-specific; broadening them requires D.1 first.
- **`read_task_ids` edge case**: Do NOT fix now. Precondition not met (not touching `read_task_ids`).

---

## 6. Suggested Task Grouping

The plan's five goals decompose into three implementation groups, designed for sequential `/explore-implement` passes.

### Group 1: `multi_param_sweep` binary (standalone)

**Goal**: Create the complete `examples/multi_param_sweep/` crate.

**Files to create**:
- `examples/multi_param_sweep/Cargo.toml`
- `examples/multi_param_sweep/src/main.rs`
- `examples/multi_param_sweep/src/config.rs`
- `examples/multi_param_sweep/src/job_script.rs`
- `examples/multi_param_sweep/seeds/ZnO.cell` (copy from existing)
- `examples/multi_param_sweep/seeds/ZnO.param` (copy from existing)

**Implementation approach**: `lib-tdd`
- `config.rs` tests (ported `parse_u_values` + new `parse_kpoints` + new `parse_cutoffs`) — write tests first
- `job_script.rs` tests (ported: sbatch directives, seed name, shebang, no_literal_tabs, nix, mpi) — write tests first
- Sweep logic combinatorics (product mode, pairwise mode, error cases) — write tests first
- Task builder (setup closure writes correct .cell/.param content) — integration-style

**Dependencies**: None (self-contained binary). Can build independently once created, but workspace membership needed for `cargo run --bin multi_param_sweep`.

### Group 2: `scf_dos_chain` binary + `hubbard_u_sweep` update (standalone)

**Goal**: Create Binary 2 and update the existing `hubbard_u_sweep` for v0.5.0.

**Files to create**:
- `examples/scf_dos_chain/Cargo.toml`
- `examples/scf_dos_chain/src/main.rs`
- `examples/scf_dos_chain/src/config.rs`
- `examples/scf_dos_chain/src/job_script.rs`
- `examples/scf_dos_chain/seeds/ZnO.cell`
- `examples/scf_dos_chain/seeds/ZnO.param`

**Files to update**:
- `examples/hubbard_u_sweep/Cargo.toml` (path dep for castep-cell-io)
- `examples/hubbard_u_sweep/src/main.rs` (if v0.5.0 API requires code changes)

**Implementation approach**: Mixed
- `config.rs`: `direct` — simple single-value config, no new parsing functions (reuse `parse_u_values`)
- `job_script.rs`: `direct` — identical to Group 1's version, copy + adapt config type parameter
- `main.rs` chain logic: `lib-tdd` — test task dependency structure (SCF → DOS), test DOS setup closure (checkpoint copy, task type set to BandStructure)
- `hubbard_u_sweep` update: `direct` — dependency change + compile check

**Dependencies**: None on Group 1 (independent). The two binaries share no code.

### Group 3: Workspace integration, cleanup, verification

**Goal**: Wire everything together, delete old crate, verify full build.

**Files to update**:
- `/Users/tony/programming/castep_workflow_framework/Cargo.toml` (workspace members)

**Files to delete**:
- `examples/hubbard_u_sweep_slurm/` (entire directory)

**Implementation approach**: `direct`
- Update workspace members list: remove `hubbard_u_sweep_slurm`, add `multi_param_sweep` and `scf_dos_chain`
- Delete old crate directory
- Run `cargo build --workspace` to verify all crates compile
- Run `cargo test --workspace` to verify all tests pass (old slurm tests removed, new binary tests run)
- Run `cargo clippy --workspace --all-targets -- -W dead-code` for F.3 mitigation

**Verification checklist** (from plan):
1. [x] Dry run Binary 1: `cargo run --bin multi_param_sweep -- --dry-run`
2. [x] Dry run Binary 1 pairwise: `cargo run --bin multi_param_sweep -- --dry-run --sweep-mode pairwise --u-values 0,1,2 --kpoints 8x8x8,6x6x6,4x4x4 --cutoffs 300,500,800`
3. [x] Local run Binary 1 (if CASTEP available)
4. [x] Dry run Binary 2: `cargo run --bin scf_dos_chain -- --dry-run`
5. [x] Dry run Binary 2 chain: `cargo run --bin scf_dos_chain -- --local --dry-run`
6. [x] Full build: `cargo build --workspace` (no warnings)
7. [x] Full test: `cargo test --workspace` (all pass)

### Dependency graph between groups

```
Group 1 ──┐
           ├── Group 3
Group 2 ──┘
```

Groups 1 and 2 are independent and can be implemented in parallel. Group 3 depends on both being complete.

### Rationale for this grouping

1. **Groups 1 and 2 are self-contained binaries.** Each can be built and tested in isolation before workspace integration. This reduces coupling during implementation — a bug in Binary 1 doesn't block Binary 2.

2. **Group 3 is the integration step.** It wires the new crates into the workspace, removes the old one, and validates end-to-end. Keeping this separate means Group 1 and 2 implementers don't need to think about workspace-level concerns.

3. **`hubbard_u_sweep` update is grouped with Binary 2** because it's a small, high-confidence change (primarily a Cargo.toml edit). It doesn't warrant its own group.

4. **Test porting is embedded in each group** rather than a separate step. This ensures tests migrate with the code they test, not as an afterthought.

---

## 7. Open Questions for Implementation

| # | Question | Default answer | When to resolve |
|---|----------|---------------|-----------------|
| Q1 | Exact module path for `Task::BandStructure` in `castep-cell-io` v0.5.0? | `castep_cell_io::param::general::task::Task::BandStructure` | During Group 2 implementation — use LSP hover |
| Q2 | Does `Species::Symbol` in v0.5.0 take `String` or `&str`? | Assume `String` (same as v0.4.0) | During Group 1 implementation — compiler will catch |
| Q3 | Does `castep-cell-io` v0.5.0 re-export `KpointsMpGrid` from the crate root or a submodule? | `castep_cell_io::KpointsMpGrid` (plan says it's a direct field on `CellDocument`) | During Group 1 implementation |
| Q4 | Should `parse_u_values` return `anyhow::Result` or `Result<_, String>`? | `anyhow::Result` — consistent with other parsing functions (Pattern 3) | During Group 1 implementation |
| Q5 | Are the existing `hubbard_u_sweep_slurm` tests in `config.rs` and `job_script.rs` using `#[cfg(test)]` modules within the same file? | Yes (confirmed from source inspection) | Port these as-is to the new binaries |
