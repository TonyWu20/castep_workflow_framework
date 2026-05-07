# Draft Elaboration -- Phase 6 Fix Plan

Generated: 2026-04-26
Source plan: `plans/phase-6-fix/PHASE_6_FIX_PLAN.md`
Codebase state: `notes/plan-enrichment/phase-6-fix/codebase-state.md`
Deferred items: `notes/plan-enrichment/phase-6-fix/deferred-and-patterns.md`

---

## Binary 1: `multi_param_sweep`

### Item 1.1: Config struct (`config.rs`) -- Clap derive definition

The plan's flag table maps to a clap derive struct. The struct must hold all parsed values before they are passed to task-building logic.

**Proposed type signature:**

```rust
#[derive(Parser)]
#[command(name = "multi_param_sweep", about = "Independent SCF parameter sweep")]
pub struct SweepConfig {
    #[arg(long, default_value = "0.0,1.0,2.0,3.0,4.0,5.0")]
    pub u_values: String,

    #[arg(long)]
    pub kpoints: Option<String>,

    #[arg(long)]
    pub cutoffs: Option<String>,

    #[arg(long, default_value = "product")]
    pub sweep_mode: String,

    #[arg(long, default_value = "Zn")]
    pub element: String,

    #[arg(long, default_value = "d")]
    pub orbital: char,

    #[arg(long, default_value = "ZnO")]
    pub seed_name: String,

    #[arg(long, default_value = "4")]
    pub max_parallel: usize,

    #[arg(long, default_value = "false")]
    pub local: bool,

    #[arg(long, default_value = "false")]
    pub dry_run: bool,

    #[arg(long, default_value = "castep")]
    pub castep_command: String,

    #[arg(long, default_value = ".")]
    pub workdir: String,

    // SLURM flags -- ported from old binary
    #[arg(long, default_value = "defq")]
    pub partition: String,

    #[arg(long, default_value = "8")]
    pub ntasks: usize,

    #[arg(long)]
    pub nix_flake: Option<String>,

    #[arg(long)]
    pub mpi_if: Option<String>,
}
```

**Module placement:** `examples/multi_param_sweep/src/config.rs`

**Error handling strategy:** None at struct level. Parsing of comma-separated strings happens in utility functions (see Item 1.2), not in clap value-parsers. This is consistent with the old binary pattern where `parse_u_values` is called after clap parsing, not as a `value_parser`.

**Ownership/lifetime notes:** All `String` fields (owned), no borrowed data. The config struct is short-lived (consumed in `main()` to drive task construction).

**Trait coherence notes:** None. This is a standalone binary crate.

**Uncertainty:** The old binary also had `--account`, `--walltime`, `--modules`, `--castep-command` as plan-specified portable fields (see D.1). The current plan table omits them. Confirming: the plan intentionally keeps the NixOS-specific fields (`--nix-flake`, `--mpi-if`) and does NOT introduce portable fields in this fix. Reason: D.1 precondition ("second user or non-NixOS cluster") is not met.

---

### Item 1.2: Parsing utility functions -- signatures and error types

The plan names three parsing functions. The old binary's `parse_u_values` returns `Result<Vec<f64>, String>` -- but the new binary depends on `anyhow`, so all three should be unified to `anyhow::Result`.

**Proposed type signatures:**

```rust
/// Parse comma-separated f64 values (ported from old binary, error type upgraded).
/// Example: "0.0,1.0,2.0" -> Ok(vec![0.0, 1.0, 2.0])
pub fn parse_u_values(s: &str) -> anyhow::Result<Vec<f64>>;

/// Parse comma-separated k-point MP grids.
/// Example: "8x8x8,6x6x6" -> Ok(vec![[8,8,8], [6,6,6]])
/// Errors: wrong number of axes ("8x8"), non-numeric ("abc"), empty segment
pub fn parse_kpoints(s: &str) -> anyhow::Result<Vec<[u32; 3]>>;

/// Parse comma-separated cutoff energies.
/// Example: "300,500,800" -> Ok(vec![300.0, 500.0, 800.0])
pub fn parse_cutoffs(s: &str) -> anyhow::Result<Vec<f64>>;
```

**Module placement:** `examples/multi_param_sweep/src/main.rs`

The plan says these go in `main.rs`. The old binary put `parse_u_values` in `config.rs` -- but for the new binary, placing all three parsing functions together in `main.rs` keeps them near their single call site (the sweep generation logic). `config.rs` is reserved for the clap struct only.

**Uncertainty (low):** Should `parse_u_values` stay in `config.rs` to match the old binary pattern? The plan says "imported or duplicated" into main.rs. Going with main.rs since plan says main.rs has "task builders, sweep logic, workflow runner" and the parsing functions are part of sweep logic input preparation.

**Error handling strategy:** All three use `anyhow::Context` for clear error messages:
- `parse_kpoints`: `.with_context(|| format!("invalid k-point grid: {segment}"))` for each segment
- `parse_cutoffs`: `.with_context(|| format!("invalid cutoff energy: {segment}"))`
- `parse_u_values`: same as old binary but with `anyhow::Error` instead of `String` via `.map_err(|e| anyhow::anyhow!("{}", e))`

**Ownership/lifetime notes:** All take `&str` and return owned `Vec`. No lifetime coupling.

**Trait coherence notes:** None.

---

### Item 1.3: Sweep generation function

The plan describes product and pairwise sweep modes. This logic should be extracted into a dedicated function.

**Proposed type signature:**

```rust
/// Generate the list of (u_value, kpoint, cutoff) triples based on sweep mode.
///
/// - `product`: cartesian product via itertools::iproduct!
/// - `pairwise`: zip all three lists (must be equal length)
///
/// `kpoints` and `cutoffs` may be None in product mode (seeds provide defaults).
/// In pairwise mode, both must be Some or this returns an error.
pub fn generate_sweep_combinations(
    u_values: &[f64],
    kpoints: Option<&[[u32; 3]]>,
    cutoffs: Option<&[f64]>,
    sweep_mode: &str,
) -> anyhow::Result<Vec<(f64, Option<[u32; 3]>, Option<f64>)>>;
```

**Module placement:** `examples/multi_param_sweep/src/main.rs`

**Error handling strategy:** Returns `anyhow::Error` for pairwise-mode validation failures:
- `"pairwise mode requires --kpoints and --cutoffs to be specified"` if either is None
- `"pairwise mode requires equal-length lists: u_values({n_u}), kpoints({n_k}), cutoffs({n_c})"` if lengths differ

**Ownership/lifetime notes:** Inputs are borrowed slices; output is owned `Vec` of tuples. The `Option<[u32; 3]>` and `Option<f64>` in the output allow downstream task-building to distinguish "explicit kpoints/cutoff" from "use seed defaults".

**Trait coherence notes:** None.

---

### Item 1.4: Task builder function -- parameter injection setup closure

The plan describes a setup closure that injects HubbardU, KPOINTS_MP_GRID, and cutoff energy. The closure type depends on `TaskClosure`, which is referenced in the plan but whose exact definition is not visible in codebase-state.md.

**Proposed type signature:**

```rust
/// Build a single SCF sweep task for one (U, kpoint, cutoff) combination.
///
/// Returns a fully configured Task ready to add to a Workflow.
pub fn build_sweep_task(
    u_value: f64,
    kpoint: Option<[u32; 3]>,
    cutoff: Option<f64>,
    element: &str,
    orbital: char,
    seed_name: &str,
    seed_cell_path: &Path,
    seed_param_path: &Path,
    workdir_root: &Path,
    castep_command: &str,
    is_local: bool,
    config: &SweepConfig,       // for SLURM script generation
) -> anyhow::Result<Task>;
```

**Module placement:** `examples/multi_param_sweep/src/main.rs`

**Error handling strategy:** Setup closure operations (reading seed files, parsing, writing modified files) can fail with I/O or parse errors. The `TaskClosure` type likely wraps `Box<dyn FnOnce() -> Result<(), WorkflowError> + Send>`, but setup operations that fail in the closure will be reported via the workflow system's error propagation.

**Uncertainty (HIGH):** The exact definition of `TaskClosure` is not visible in codebase-state.md. It may be:
- `Box<dyn FnOnce() -> Result<(), WorkflowError> + Send>`
- `Box<dyn FnOnce() -> Result<(), Box<dyn Error>> + Send>`
- Something else entirely.

This must be resolved before implementation by reading `workflow_core/src/task.rs` to check the type alias. If `TaskClosure` is not a `WorkflowError`-returning closure, the setup function may need to map errors (e.g., `.map_err(|e| WorkflowError::InvalidConfig(e.to_string()))`).

**Ownership/lifetime notes:** The setup closure must capture owned copies of `u_value`, `kpoint`, `cutoff`, `element.to_string()`, `seed_name.to_string()`, `castep_command.to_string()`, `workdir_root.to_path_buf()`, etc. All captures are `move` to avoid borrowing issues with the closure's `'static` bound.

**Trait coherence notes:** None.

---

### Item 1.5: KPOINTS_MP_GRID injection workaround

The plan documents that `KpointsMpGrid` is NOT a field on `CellDocument` in `castep-cell-io` v0.4.0. The workaround is: create a `KpointsMpGrid`, call `.to_cell()` to get a `Cell` block, merge it into the `Vec<Cell>` from `CellDocument.to_cell_file()`, then serialize with `to_string_many_spaced()`.

**Proposed type signature:**

```rust
/// Serialize a modified cell file with an injected KPOINTS_MP_GRID block.
///
/// Workaround for castep-cell-io v0.4.0 where KpointsMpGrid is not a field
/// on CellDocument. Converts KpointsMpGrid to a Cell block, merges it into
/// the output of CellDocument.to_cell_file(), then serializes.
pub fn cell_with_kpoints(
    cell_doc: &CellDocument,
    kpoint_grid: [u32; 3],
) -> anyhow::Result<String>;
```

**Module placement:** `examples/multi_param_sweep/src/main.rs`

**Error handling strategy:** The only fallible operation is `KpointsMpGrid::new()` if the grid is invalid, and `cell_doc.to_cell_file()` (which is infallible per existing pattern). Returns `anyhow::Result` for consistency. If all operations are infallible in practice, this could be a plain `String` return -- but the `Result` wrapper is defensive.

**Ownership/lifetime notes:** Takes `&CellDocument` (borrowed). Returns owned `String`. The `KpointsMpGrid` is stack-allocated and consumed by `.to_cell()`.

**Uncertainty (medium):** The exact API for `KpointsMpGrid::to_cell()` is not confirmed. The plan states it returns a `Cell` -- but whether it takes `&self` or `self`, and whether `to_string_many_spaced` takes `&[Cell]` or `Vec<Cell>`, needs verification against the actual `castep-cell-io` v0.4.0 API. If `to_string_many_spaced` takes `&[Cell]`, the sequence is:

```rust
let cells: Vec<Cell> = cell_doc.to_cell_file();        // Vec<Cell>
let kp_cell: Cell = KpointsMpGrid(grid).to_cell();     // Cell
let all_cells: Vec<Cell> = cells.into_iter()
    .chain(std::iter::once(kp_cell))
    .collect();
to_string_many_spaced(&all_cells)                       // String
```

**Trait coherence notes:** None. This is a free function in binary code, not touching any trait impls.

---

### Item 1.6: `job_script.rs` -- ported SLURM generation with formatting fix (absorbing D.2)

The plan says to port `generate_job_script` from the old binary and apply a formatting fix: no literal tab characters mixed with spaces, consistent quoting around SBATCH directives.

**Proposed type signature:**

```rust
/// Generate a SLURM job script for a single task.
///
/// Uses a clean heredoc-style template string (no literal tabs mixed with spaces).
/// SBATCH directives use consistent quoting: all `--*=` values are double-quoted.
pub fn generate_job_script(
    config: &SweepConfig,
    task_id: &str,
    seed_name: &str,
) -> String;
```

**Module placement:** `examples/multi_param_sweep/src/job_script.rs`

**Error handling strategy:** Returns `String` (infallible). This matches the old binary pattern where `generate_job_script` is pure string formatting with no I/O.

**Formatting fix details (D.2 absorbed):**
- Replace any residual `\t` literals with spaces.
- Quote all SBATCH directive values consistently: `#SBATCH --partition="defq"`, not `#SBATCH --partition=defq`.
- The existing `no_literal_tabs` test from the old binary should be ported and updated to verify the fix.

**Ownership/lifetime notes:** Takes `&SweepConfig` (borrowed). Returns owned `String`. No allocations besides the result.

**Trait coherence notes:** None.

**Uncertainty (low):** The old binary's `generate_job_script` signature takes a config struct, task_id, and seed_name. The new binary's config struct (`SweepConfig`) has the same relevant fields (`partition`, `ntasks`, `nix_flake`, `mpi_if`), so the port is straightforward.

---

### Item 1.7: Task ID format helper

The plan specifies task ID format `scf_U{u}_k{k}_c{c}`. This format should be encapsulated in a function to ensure consistency and avoid duplication.

**Proposed type signature:**

```rust
/// Format a task ID for a sweep combination.
/// Example: format_sweep_task_id(3.0, Some([8,8,8]), Some(500.0)) -> "scf_U3.0_k8x8x8_c500"
///          format_sweep_task_id(3.0, None, None)               -> "scf_U3.0"
pub fn format_sweep_task_id(
    u_value: f64,
    kpoint: Option<[u32; 3]>,
    cutoff: Option<f64>,
) -> String;
```

**Module placement:** `examples/multi_param_sweep/src/main.rs`

**Error handling strategy:** Infallible (pure formatting).

**Ownership/lifetime notes:** None. All inputs are `Copy`.

**Uncertainty (medium):** When kpoints or cutoffs are None (product mode with seed defaults), the plan says the workdir is `runs/U{u}_k{k}_c{c}/`. Should the task ID also include the k/c values when present? The plan example shows `scf_U3.0_k8x8x8_c500`. For None values, I propose omitting that segment: `scf_U3.0`. This avoids ambiguity and keeps IDs clean. The workdir path should also follow this naming convention.

---

### Item 1.8: Main function flow

The plan describes the overall flow but does not decompose it into function calls.

**Proposed structure** (all in `main.rs`):

```rust
fn main() -> anyhow::Result<()> {
    let config = SweepConfig::parse();

    // 1. Parse all input lists
    let u_values = parse_u_values(&config.u_values)?;
    let kpoints: Option<Vec<[u32; 3]>> = config.kpoints
        .as_ref()
        .map(|s| parse_kpoints(s))
        .transpose()?;
    let cutoffs: Option<Vec<f64>> = config.cutoffs
        .as_ref()
        .map(|s| parse_cutoffs(s))
        .transpose()?;

    // 2. Generate sweep combinations
    let combos = generate_sweep_combinations(
        &u_values,
        kpoints.as_deref(),
        cutoffs.as_deref(),
        &config.sweep_mode,
    )?;

    // 3. Build tasks
    let root = PathBuf::from(&config.workdir);
    let seed_cell = root.join("seeds").join(format!("{}.cell", config.seed_name));
    let seed_param = root.join("seeds").join(format!("{}.param", config.seed_name));
    let mut workflow = Workflow::new("multi_param_sweep")
        .with_max_parallel(config.max_parallel)?
        .with_root_dir(root.clone());
    for (u, k, c) in combos {
        let task = build_sweep_task(
            u, k, c,
            &config.element, config.orbital,
            &config.seed_name,
            &seed_cell, &seed_param,
            &root,
            &config.castep_command,
            config.local,
            &config,
        )?;
        workflow.add_task(task)?;
    }

    // 4. Execute or dry-run
    if config.dry_run {
        let order = workflow.dry_run()?;
        for id in order {
            println!("{id}");
        }
        return Ok(());
    }

    let state = JsonStateStore::new("multi_param_sweep", root.join("state.json"));
    if config.local {
        let runner = SystemProcessRunner::new().with_log_dir(root.join("logs"));
        run_default(workflow, state)?;
    } else {
        let qr = QueuedRunner::new(SchedulerKind::Slurm);
        let workflow = workflow.with_queued_submitter(qr);
        run_default(workflow, state)?;
    }
    Ok(())
}
```

**Error handling strategy:** `main` returns `anyhow::Result<()>`, consistent with the `anyhow` dependency. All `?` operators propagate to main, where anyhow prints the error chain.

**Uncertainty (medium):** The exact API for `Workflow::with_max_parallel(n)`, `with_root_dir(path)`, and `with_queued_submitter(qs)` is confirmed in codebase-state.md. However, the state store path (`state.json`) and log directory conventions need verification against what the old binary uses.

**Uncertainty (medium):** `JsonStateStore::new` takes `(name, path)` per codebase-state.md. The plan uses a root workdir; the state store should live at `<workdir>/state.json` to not conflict with per-task workdirs. If the state store's `path` argument is a directory (not a file path), this needs adjustment -- check `JsonStateStore::new` signature.

---

## Binary 2: `scf_dos_chain`

### Item 2.1: Config struct (`config.rs`) -- Clap derive definition

**Proposed type signature:**

```rust
#[derive(Parser)]
#[command(name = "scf_dos_chain", about = "SCF + DOS task chain")]
pub struct ChainConfig {
    #[arg(long, default_value = "3.0")]
    pub u_value: f64,

    #[arg(long, default_value = "Zn")]
    pub element: String,

    #[arg(long, default_value = "d")]
    pub orbital: char,

    #[arg(long, default_value = "ZnO")]
    pub seed_name: String,

    #[arg(long, default_value = "1")]
    pub max_parallel: usize,

    #[arg(long, default_value = "false")]
    pub local: bool,

    #[arg(long, default_value = "false")]
    pub dry_run: bool,

    #[arg(long, default_value = "castep")]
    pub castep_command: String,

    #[arg(long, default_value = ".")]
    pub workdir: String,

    // SLURM flags (same as Binary 1)
    #[arg(long, default_value = "defq")]
    pub partition: String,

    #[arg(long, default_value = "8")]
    pub ntasks: usize,

    #[arg(long)]
    pub nix_flake: Option<String>,

    #[arg(long)]
    pub mpi_if: Option<String>,
}
```

**Module placement:** `examples/scf_dos_chain/src/config.rs`

**Error handling strategy:** None at struct level. Single `--u-value` is parsed directly by clap as `f64` -- no comma-separated parsing needed.

**Uncertainty (low):** The plan specifies `--orbital` default `'d'`. Clap's `char` type parser accepts a single character, which matches.

---

### Item 2.2: SCF task builder function

**Proposed type signature:**

```rust
/// Build the SCF task (Task 1 in the chain).
/// Injects Hubbard U into the seed cell file, writes modified files to workdir.
pub fn build_scf_task(
    u_value: f64,
    element: &str,
    orbital: char,
    seed_name: &str,
    seed_cell_path: &Path,
    seed_param_path: &Path,
    workdir: &Path,
    castep_command: &str,
    is_local: bool,
    config: &ChainConfig,
) -> anyhow::Result<Task>;
```

**Module placement:** `examples/scf_dos_chain/src/main.rs`

**Error handling strategy:** Same as Binary 1's `build_sweep_task`: setup closure operations can fail; errors propagate via the closure's error type.

**Setup closure details:**
1. Read seed cell file via `read_file(seed_cell_path)` -> parse into `CellDocument`
2. Build `HubbardU` via builder pattern, inject into `cell_doc.hubbard_u`
3. Serialize cell: `cell_doc.to_cell_file()` -> `to_string_many_spaced()` -> `write_file(workdir.join(format!("{seed_name}.cell")), content)`
4. Copy seed param (no modifications needed for SCF): `copy_file(seed_param_path, workdir.join(format!("{seed_name}.param")))`
5. Write job script if queued mode

**Task configuration:**
- `id`: `"scf"`
- `workdir`: `"runs/scf_dos/"`
- `mode`: `ExecutionMode::direct("castep", &["ZnO"])` or `Queued`
- No dependencies

---

### Item 2.3: DOS task builder function

**Proposed type signature:**

```rust
/// Build the DOS task (Task 2 in the chain, depends on Task 1).
/// Modifies seed param to set general.task = BandStructure, copies SCF checkpoint.
pub fn build_dos_task(
    seed_name: &str,
    seed_cell_path: &Path,
    seed_param_path: &Path,
    workdir: &Path,
    castep_command: &str,
    is_local: bool,
    config: &ChainConfig,
) -> anyhow::Result<Task>;
```

**Module placement:** `examples/scf_dos_chain/src/main.rs`

**Error handling strategy:** Same as SCF task builder.

**Setup closure details:**
1. Parse seed cell into `CellDocument`, write to `workdir/ZnO_DOS.cell`
2. Parse seed param into `ParamDocument`, set `general.task = Some(Task::BandStructure)`, write to `workdir/ZnO_DOS.param`
3. Copy `workdir/ZnO.check` -> `workdir/ZnO_DOS.check` via `copy_file`

**Task configuration:**
- `id`: `"dos"`
- `workdir`: `"runs/scf_dos/"` (same as SCF)
- `mode`: `ExecutionMode::direct("castep", &["ZnO_DOS"])` or `Queued`
- Dependency: `.depends_on("scf")`

**Uncertainty (medium):** The plan says to "copy" `ZnO.check` -> `ZnO_DOS.check`. However, `copy_file` in `workflow_utils::files` returns `Result<(), WorkflowError>`, not `anyhow::Result`. If the setup closure's error type is `WorkflowError`, `.map_err()` is needed. If it's `anyhow::Error`, `anyhow::Error` can be created from `WorkflowError` via `From` impl (since `WorkflowError` likely implements `std::error::Error`). This needs verification.

**Uncertainty (medium):** The `.check` file is produced by CASTEP after successful SCF completion. The DOS task depends on the SCF task, so the `.check` file should exist by the time DOS runs. However, the plan does not specify whether the setup closure should verify the file exists or just attempt the copy. If the copy fails because the SCF task failed, the workflow's dependency mechanism should have already skipped the DOS task. Confirming: the setup closure should attempt the copy and let the error propagate if it fails.

---

### Item 2.4: Collect closures for completion verification

The plan specifies that both SCF and DOS tasks should verify `ZnO.castep` (or `ZnO_DOS.castep`) exists and contains a "Total time" completion marker. This is identical logic for both tasks.

**Proposed type signature:**

```rust
/// Build a collect closure that verifies the CASTEP output file exists
/// and contains a "Total time" completion marker.
///
/// Returns a closure suitable for `Task::collect()`.
pub fn verify_castep_output(
    workdir: PathBuf,
    output_file: String,
) -> TaskClosure;
```

**Module placement:** `examples/scf_dos_chain/src/main.rs`

**Error handling strategy:** The collect closure returns a `TaskClosure`-compatible error if the file is missing or lacks the marker. Exact error type depends on `TaskClosure` definition (see HIGH uncertainty in Item 1.4).

**Uncertainty (HIGH):** Same `TaskClosure` uncertainty as Item 1.4. Must be resolved first.

---

### Item 2.5: Main function flow

**Proposed structure** (all in `main.rs`):

```rust
fn main() -> anyhow::Result<()> {
    let config = ChainConfig::parse();

    let root = PathBuf::from(&config.workdir);
    let seed_cell = root.join("seeds").join(format!("{}.cell", config.seed_name));
    let seed_param = root.join("seeds").join(format!("{}.param", config.seed_name));
    let workdir = root.join("runs/scf_dos/");

    // Build SCF task
    let scf_task = build_scf_task(
        config.u_value, &config.element, config.orbital,
        &config.seed_name, &seed_cell, &seed_param,
        &workdir, &config.castep_command, config.local, &config,
    )?;

    // Build DOS task (depends on SCF)
    let dos_task = build_dos_task(
        &config.seed_name, &seed_cell, &seed_param,
        &workdir, &config.castep_command, config.local, &config,
    )?.depends_on("scf");

    let mut workflow = Workflow::new("scf_dos_chain")
        .with_max_parallel(config.max_parallel)?
        .with_root_dir(root.clone());
    workflow.add_task(scf_task)?;
    workflow.add_task(dos_task)?;

    if config.dry_run {
        for id in workflow.dry_run()? {
            println!("{id}");
        }
        return Ok(());
    }

    // Run -- same local/queued pattern as Binary 1
    let state = JsonStateStore::new("scf_dos_chain", root.join("state.json"));
    // ... (same runner logic as Binary 1)
    Ok(())
}
```

---

## Workspace Changes

### Item 3.1: Root `Cargo.toml` -- workspace members

**Proposed change** (line 3-8 of root `Cargo.toml`):

Remove:
```toml
"examples/hubbard_u_sweep_slurm",
```

Add:
```toml
"examples/multi_param_sweep",
"examples/scf_dos_chain",
```

**Module placement:** `/Users/tony/programming/castep_workflow_framework/Cargo.toml`

**Error handling strategy:** N/A (configuration change). Validate with `cargo build --workspace` after all files are created.

---

### Item 3.2: Seed file copying

The plan says to create seed files for both new binaries. The old binary's seed files should be copied verbatim.

**Files to copy:**
- `examples/hubbard_u_sweep_slurm/seeds/ZnO.cell` -> `examples/multi_param_sweep/seeds/ZnO.cell`
- `examples/hubbard_u_sweep_slurm/seeds/ZnO.param` -> `examples/multi_param_sweep/seeds/ZnO.param`
- `examples/hubbard_u_sweep_slurm/seeds/ZnO.cell` -> `examples/scf_dos_chain/seeds/ZnO.cell`
- `examples/hubbard_u_sweep_slurm/seeds/ZnO.param` -> `examples/scf_dos_chain/seeds/ZnO.param`

**Module placement:** N/A (plain files, not modules)

**Error handling strategy:** N/A (file copy, no error handling needed beyond OS-level copy)

---

## Deferred Items Assessment

For each item from `deferred-and-patterns.md`, assess whether it is directly relevant to the current plan and should be absorbed.

- **D.1 (portable SLURM config fields):** **Skip -- not applicable yet.**
  The plan intentionally retains NixOS-specific config fields (`--nix-flake`, `--mpi-if`) without adding portable fields (`--account`, `--walltime`, `--modules`, `--castep_command`). The deferred item's precondition ("second user or non-NixOS cluster required") is not met. The deferred item remains valid for a future phase but is out of scope for this fix.

- **D.2 (`generate_job_script` formatting fix):** **Absorb -- directly addressed.**
  The plan explicitly references D.2 in the setup closure description: "use a clean heredoc template -- no literal tab characters mixed with spaces, consistent quoting around SBATCH directives." This is absorbed as part of Item 1.6 (ported `job_script.rs`). The formatting fix applies to both binaries since both share the same `job_script.rs` content.

- **D.3 (unit tests for `generate_job_script`):** **Absorb partially.**
  The plan says to "port the `no_literal_tabs` test" from the old binary. The remaining deferred concern (portable-template tests) is preconditioned on D.1, which is not in scope. So: port `no_literal_tabs` test (absorbed), defer portable-template tests (skipped).

- **`read_task_ids` empty-string edge case:** **Skip -- not applicable.**
  This is a bug in `workflow-cli` code that is not touched by the binary rewrite. The CLI binary is not being modified in this plan. Out of scope.

- **Known failure modes from prior phases:**
  - **Missing `pub use` re-exports:** **Skip.** Library code is not being modified.
  - **Incomplete consumer updates:** **Caution.** The old binary used `"job.sh"` literally; the new binary should use the `JOB_SCRIPT_NAME` constant from `workflow_utils` to avoid repeating this failure mode. Noted as a best practice for implementation.
  - **Stale imports after refactoring:** **Caution.** When deleting `examples/hubbard_u_sweep_slurm/`, ensure no other crate depends on it (check `cargo tree` or workspace member list).
  - **Dead / unenforced API surface:** **Skip.** No new library API is being added.
  - **Stale documentation:** **Skip.** `ARCHITECTURE.md` is not being updated in this plan.

---

## Summary of Uncertainties (prioritized)

| Priority | Item | Uncertainty |
|----------|------|-------------|
| **HIGH** | 1.4, 2.4 | Exact definition of `TaskClosure` type alias in `workflow_core` -- must read `workflow_core/src/task.rs` before implementation |
| **MEDIUM** | 1.5 | Exact API for `KpointsMpGrid::to_cell()` and `to_string_many_spaced()` -- verify against `castep-cell-io` v0.4.0 |
| **MEDIUM** | 1.2 | Module placement of `parse_u_values`: old binary has it in `config.rs`, plan says main.rs -- confirm placement |
| **MEDIUM** | 2.3 | Error type compatibility between `workflow_utils::files::copy_file` (returns `WorkflowError`) and setup closure's error type |
| **MEDIUM** | 1.7 | Task ID and workdir naming when kpoints/cutoffs are None in product mode |
| **LOW** | 1.1 | Whether portable SLURM fields should be included now (plan says no, deferred D.1 confirms) |
| **LOW** | 1.6 | Whether old binary's `generate_job_script` uses NixOS-specific fields that affect the template |

---

## Patterns Used (from codebase-state.md)

| Pattern | Source | Applied To |
|---------|--------|------------|
| Clap `#[derive(Parser)]` with `#[arg(long)]` | Old binary `config.rs` | Both new `config.rs` files |
| `Task::new(id, mode)` builder with `.workdir()`, `.setup()`, `.collect()`, `.depends_on()` | `workflow_core` API | All task construction |
| `ExecutionMode::direct(cmd, &[args])` convenience constructor | `workflow_core` API | Local execution mode |
| `Workflow::new(name)` with `.with_max_parallel()`, `.add_task()`, `.dry_run()` | `workflow_core` API | Both binary main functions |
| `run_default(workflow, state)` from `workflow_utils` | `workflow_utils` API | Workflow execution |
| `SystemProcessRunner::new()` / `QueuedRunner::new(SchedulerKind::Slurm)` | `workflow_utils` API | Runner selection |
| `read_file` / `write_file` / `copy_file` from `workflow_utils::files` | `workflow_utils` API | File I/O in setup closures |
| `CellDocument::parse()` / `ParamDocument::parse()` from `castep-cell-fmt` | External crate API | Seed file parsing |
| `HubbardU::builder()` / `AtomHubbardU::builder()` pattern | External crate API | Hubbard U injection |
| `format!()` -based template for SLURM script (not `indoc!`) | Old binary pattern | Ported `job_script.rs` |
