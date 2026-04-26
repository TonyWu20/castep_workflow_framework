# Plan: Rewrite example binaries for multi-parameter sweep & SCF+DOS chain

## Context

The merged `hubbard_u_sweep_slurm` binary has three bugs:
1. String-parsed "second values" with no CLI format hints — users don't know what convention to use
2. Default sweep mode "single" only generates SCF tasks — product/pairwise modes silently require a flag
3. Product/pairwise modes always build SCF→DOS chains, causing `duplicate task id: dos_kpt8x8x8` because DOS task IDs don't include the U value

The fix is to replace this binary with two purpose-built binaries that correctly test the intended workflows.

---

## Plan

### Binary 1: `multi_param_sweep` — independent SCF parameter sweep

**Purpose**: Sweep Hubbard U, k-point MP grid, and cutoff energy in product or pairwise mode. All tasks are independent SCF calculations (no children).

**File structure**:
```
examples/multi_param_sweep/
├── Cargo.toml          (deps: anyhow, clap, castep-cell-fmt, castep-cell-io, itertools, workflow_core, workflow_utils)
├── src/
│   ├── main.rs         (entry point, task builders, sweep logic, workflow runner)
│   ├── config.rs       (clap CLI config)
│   └── job_script.rs   (SLURM script generation, ported from old binary)
└── seeds/
    ├── ZnO.cell
    └── ZnO.param
```

**CLI flags**:

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--u-values` | `String` | `"0.0,1.0,2.0,3.0,4.0,5.0"` | Comma-separated Hubbard U values (eV) |
| `--kpoints` | `Option<String>` | `None` | Comma-separated MP grids, e.g. `"8x8x8,6x6x6"`. If omitted, no KPOINTS block is written (use seed defaults) |
| `--cutoffs` | `Option<String>` | `None` | Comma-separated cutoff energies (eV), e.g. `"300,500,800"`. If omitted, use seed defaults |
| `--sweep-mode` | `String` | `"product"` | `"product"` (cartesian product) or `"pairwise"` (zip — all three lists must be same length) |
| `--element` | `String` | `"Zn"` | Element for Hubbard U |
| `--orbital` | `char` | `"d"` | Orbital: `'d'` or `'f'` |
| `--seed-name` | `String` | `"ZnO"` | CASTEP input file prefix |
| `--max-parallel` | `usize` | `4` | Max concurrent tasks |
| `--local` | `bool` | `false` | Direct process execution (no SLURM) |
| `--dry-run` | `bool` | `false` | Print topological order and exit |
| `--castep-command` | `String` | `"castep"` | CASTEP binary (local mode only) |
| `--workdir` | `String` | `"."` | Root directory for runs/logs |
| Slurm flags | | | `--partition`, `--ntasks`, `--nix-flake`, `--mpi-if` (same as old binary) |

**Sweep logic** (`main.rs`):

- `product` mode: generate all combinations via `itertools::iproduct!(u_values, kpoints, cutoffs)`
- `pairwise` mode: zip all three lists (must be same length; error if not)
- Each combination → one `Task` with `ExecutionMode::direct(...)` (local) or `Queued` (Slurm)
- Task ID: `scf_U{u}_k{k}_c{c}` (e.g., `scf_U3.0_k8x8x8_c500`) — unique, no collisions
- Workdir: `runs/U{u}_k{k}_c{c}/`

**Setup closure** — three parameter injections:

1. **Hubbard U**: Parse seed cell → `CellDocument`, inject `hubbard_u` field using builder pattern
2. **K-points**: Parse compact string `"8x8x8"` → `[u32; 3]`, create `KpointsMpGrid(...)`, call `.to_cell()` to get a `Cell` block, append to `cell_doc.to_cell_file()` output before formatting. Note: `KpointsMpGrid` is *not* a field on `CellDocument` in v0.4.0 — issue to be filed at `TonyWu20/castep-cell-io`. Workaround: serialize `KpointsMpGrid` to a `Cell` block via its `ToCell` impl and merge into the output block list.
3. **Cutoff energy**: Parse seed param → `ParamDocument`, set `basis_set.cutoff_energy = Some(CutOffEnergy { value, unit: None })` — unit defaults to eV in CASTEP, leaving `None` keeps serialization cleaner (no unit suffix)
4. Serialize both documents to file via `to_string_many_spaced()`
5. If Slurm: write `job.sh` via ported `generate_job_script()`

**API usage** (from `castep-cell-io` v0.4.0):
- `CellDocument` — `parse()`, field mutation, `.to_cell_file()`
- `KpointsMpGrid([u32; 3])` — `.to_cell()` → `Cell` block appended to output
- `ParamDocument` — `parse()`, `.basis_set.cutoff_energy` field mutation, `.to_cell_file()`
- `HubbardU::builder()`, `AtomHubbardU::builder()`, `OrbitalU::D(f64)` / `OrbitalU::F(f64)`
- `CutOffEnergy { value: f64, unit: None }` — None defaults to eV in CASTEP, cleaner serialization

---

### Binary 2: `scf_dos_chain` — SCF+DOS task chain

**Purpose**: Test chained task automation — SCF followed by DOS (BandStructure) calculation that depends on SCF checkpoint output.

**File structure**:
```
examples/scf_dos_chain/
├── Cargo.toml          (deps: same as multi_param_sweep)
├── src/
│   ├── main.rs         (entry point, SCF task, DOS task, chain wiring)
│   ├── config.rs       (simpler CLI — single U, no sweep, no kpoints/cutoff)
│   └── job_script.rs   (SLURM script, same as Binary 1)
└── seeds/
    ├── ZnO.cell
    └── ZnO.param
```

**CLI flags** (simpler — single parameter set for testing the chain):

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--u-value` | `f64` | `3.0` | Single Hubbard U value (eV) |
| `--element` | `String` | `"Zn"` | Element for Hubbard U |
| `--orbital` | `char` | `"d"` | Orbital type |
| `--seed-name` | `String` | `"ZnO"` | CASTEP input prefix |
| `--max-parallel` | `usize` | `1` | Max concurrent tasks |
| `--local` | `bool` | `false` | Direct execution |
| `--dry-run` | `bool` | `false` | Print topological order |
| `--castep-command` | `String` | `"castep"` | CASTEP binary |
| `--workdir` | `String` | `"."` | Root directory |
| Slurm flags | | | Same as Binary 1 |

**Task chain**: Two tasks sharing the same workdir.

**Task 1 — `scf`**:
- Task ID: `scf`
- Workdir: `runs/scf_dos/`
- Setup: parse seed cell, inject HubbardU (same pattern as Binary 1), write `ZnO.cell` + `ZnO.param`
- Execution: `ExecutionMode::direct("castep", &["ZnO"])` or `Queued`
- Collect: verify `ZnO.castep` exists and has "Total time" marker

**Task 2 — `dos`** (depends on `scf`):
- Task ID: `dos`
- Workdir: `runs/scf_dos/` (same workdir as SCF)
- Depends on: `scf`
- Setup:
  1. Parse in-memory seed cell into `CellDocument`, write to `<workdir>/ZnO_DOS.cell`
  2. Parse in-memory seed param into `ParamDocument`, set `general.task = Some(Task::BandStructure)`, write to `<workdir>/ZnO_DOS.param`
  3. Copy `<workdir>/ZnO.check` → `<workdir>/ZnO_DOS.check` (SCF checkpoint produced by Task 1)
- Execution: `ExecutionMode::direct("castep", &["ZnO_DOS"])` or `Queued` — note seed name is `ZnO_DOS`

**API usage** (from `castep-cell-io` v0.4.0):
- `ParamDocument.general.task = Some(Task::BandStructure)` — direct field mutation
- `Task::BandStructure` imported from `castep_cell_io::param::general::task::Task`
- `read_file`/`copy_file` from `workflow_utils::files` for .check file handling

---

### Workspace changes

**`Cargo.toml`** (root):
- Remove `"examples/hubbard_u_sweep_slurm"` from workspace members
- Add `"examples/multi_param_sweep"` and `"examples/scf_dos_chain"`

**Delete**:
- `examples/hubbard_u_sweep_slurm/` (entire directory)

**Keep**:
- `examples/hubbard_u_sweep/` (unchanged)

---

## Verification

1. **Binary 1 — dry run**: `cargo run --bin multi_param_sweep -- --dry-run` → prints topological order with all SCF task combos, no duplicate IDs, no DOS tasks
2. **Binary 1 — dry run pairwise**: `cargo run --bin multi_param_sweep -- --dry-run --sweep-mode pairwise --u-values 0,1,2 --kpoints 8x8x8,6x6x6,4x4x4 --cutoffs 300,500,800` → zip mode, 3 tasks, no errors
3. **Binary 1 — local run**: `cargo run --bin multi_param_sweep -- --local --u-values 0,1 --kpoints 8x8x8,6x6x6` (if CASTEP available) → generates correct .cell/.param files with HubbardU block, KPOINTS_MP_GRID block, and correct cutoff energy
4. **Binary 2 — dry run**: `cargo run --bin scf_dos_chain -- --dry-run` → prints `scf` then `dos` (chain dependency)
5. **Binary 2 — SCF phase only**: `cargo run --bin scf_dos_chain -- --local --dry-run` → verify scf→dos topological order
6. **Build**: `cargo build --workspace` → all crates compile, no warnings
7. **Tests**: `cargo test --workspace` → all existing tests pass (old slurm tests removed, new binary tests run)
