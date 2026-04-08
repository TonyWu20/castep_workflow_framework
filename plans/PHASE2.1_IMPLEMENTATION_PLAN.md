# Phase 2.1: Wire castep-cell-io into hubbard_u_sweep

## Context

Phase 1 is complete: `workflow_core` (DAG orchestration) and `workflow_utils` (generic utilities) are fully implemented and tested. The `examples/hubbard_u_sweep` binary compiles and runs the workflow logic, but uses a hardcoded raw string for the `.cell` file instead of `castep-cell-io` builders — a `// TODO` comment marks the exact location.

`castep-cell-io = "0.4.0"` is now published on crates.io and already declared in `examples/hubbard_u_sweep/Cargo.toml`. This phase replaces the placeholder with real domain types, establishing the canonical Layer 3 pattern: seed files as templates, programmatic mutation for sweep parameters.

## Architectural Constraints

- `workflow_core/tests/hubbard_u_sweep.rs` must **not** be modified — it tests orchestration, not cell content. Adding `castep-cell-fmt` to `workflow_core` test deps would violate the layer boundary.
- All cell-building logic stays inline in `main.rs` — no new helpers, no new modules, no new types in `workflow_utils`.
- Write param file verbatim from seed — do not introduce `ParamDocument` builder (defer to later phase).
- Use direct field assignment `cell_doc.hubbard_u = Some(hubbard_u)` — do not rebuild `CellDocument`.

## Confirmed API (verified against published crate)

**Serialization** (`castep-cell-fmt = "0.1.0"` — separate crate, must be added):
```rust
use castep_cell_fmt::{parse, ToCellFile, format::to_string_many_spaced};
// parse returns Result<CellDocument, castep_cell_fmt::Error>
// Error implements std::error::Error — .context() works directly
// to_string_many_spaced returns String (infallible)
// to_cell_file() returns Vec<Cell<'_>>
```

**HubbardU builders** — types re-exported at `castep_cell_io::cell::species` (the `hubbard_u` submodule is private in the published crate):
```rust
use castep_cell_io::cell::species::{AtomHubbardU, HubbardU, HubbardUUnit, OrbitalU, Species};

let atom_u = AtomHubbardU::builder()
    .species(Species::Symbol("Zn".to_string()))
    .orbitals(vec![OrbitalU::D(u)])  // OrbitalU::D(f64) confirmed
    .build();

let hubbard_u = HubbardU::builder()
    .unit(HubbardUUnit::ElectronVolt)  // bon wraps Option automatically
    .atom_u_values(vec![atom_u])
    .build();
```

**write_file** takes `content: &str` — `String` coerces via deref, pass `&output`.

## Tasks

### TASK-1: Add castep-cell-fmt dependency
**File**: `examples/hubbard_u_sweep/Cargo.toml`
**Change**: Add `castep-cell-fmt = "0.1.0"` under `[dependencies]`. (`castep-cell-io = "0.4.0"` is already present.)
**Acceptance**: Both crates appear in the manifest. No other manifests modified.

### TASK-2: Create seed files
**Files to create** (`examples/hubbard_u_sweep/seeds/` does not yet exist):

`examples/hubbard_u_sweep/seeds/ZnO.cell`:
```
%BLOCK LATTICE_CART
  3.25 0.0 0.0
  0.0 3.25 0.0
  0.0 0.0 5.21
%ENDBLOCK LATTICE_CART

%BLOCK POSITIONS_FRAC
Zn  0.333333  0.666667  0.0
Zn  0.666667  0.333333  0.5
O   0.333333  0.666667  0.375
O   0.666667  0.333333  0.875
%ENDBLOCK POSITIONS_FRAC
```
(Standard wurtzite ZnO: a=3.25 Å, c=5.21 Å, u≈0.375 — physically correct Wyckoff positions.)

`examples/hubbard_u_sweep/seeds/ZnO.param`:
```
task : SinglePoint
```

**Acceptance**: Both files exist. `include_str!("../seeds/ZnO.cell")` resolves relative to `src/main.rs`.

### TASK-3: Rewrite main.rs
**File**: `examples/hubbard_u_sweep/src/main.rs`

```rust
use anyhow::{Context, Result};
use castep_cell_fmt::{ToCellFile, format::to_string_many_spaced, parse};
use castep_cell_io::CellDocument;
use castep_cell_io::cell::species::{AtomHubbardU, HubbardU, HubbardUUnit, OrbitalU, Species};
use workflow_core::{Task, Workflow};
use workflow_utils::{TaskExecutor, create_dir, write_file};

fn main() -> Result<()> {
    let seed_cell = include_str!("../seeds/ZnO.cell");
    let seed_param = include_str!("../seeds/ZnO.param");

    let mut workflow = Workflow::builder()
        .name("hubbard_u_sweep".to_string())
        .state_dir("./".into())
        .build()?;

    for u in [0.0_f64, 1.0, 2.0, 3.0, 4.0, 5.0] {
        let task_id = format!("scf_U{:.1}", u);
        let workdir = format!("runs/U{:.1}", u);

        let task = Task::new(&task_id, move || {
            create_dir(&workdir)?;

            let mut cell_doc: CellDocument =
                parse(seed_cell).context("failed to parse seed ZnO.cell")?;

            let atom_u = AtomHubbardU::builder()
                .species(Species::Symbol("Zn".to_string()))
                .orbitals(vec![OrbitalU::D(u)])
                .build();
            let hubbard_u = HubbardU::builder()
                .unit(HubbardUUnit::ElectronVolt)
                .atom_u_values(vec![atom_u])
                .build();
            cell_doc.hubbard_u = Some(hubbard_u);

            let output = to_string_many_spaced(&cell_doc.to_cell_file());
            write_file(format!("{workdir}/ZnO.cell"), &output)?;
            write_file(format!("{workdir}/ZnO.param"), seed_param)?;

            let result = TaskExecutor::new(&workdir)
                .command("castep")
                .arg("ZnO")
                .execute()?;
            if !result.success() {
                anyhow::bail!("castep failed: {:?}\n{}", result.exit_code, result.stderr);
            }
            Ok(())
        });

        workflow.add_task(task)?;
    }

    workflow.run()
}
```

**Acceptance**:
- `cargo build -p hubbard_u_sweep` compiles without errors or warnings.
- `cargo test -p workflow_core` still passes (test file unmodified).
- Output `.cell` files contain `%BLOCK HUBBARD_U` with the correct U value per iteration.

## Execution Order

| Phase | Tasks | Notes |
|-------|-------|-------|
| 1 (parallel) | TASK-1, TASK-2 | Independent |
| 2 (sequential) | TASK-3 | Requires TASK-1 and TASK-2 |

## Verification

```bash
cargo check -p hubbard_u_sweep
cargo build -p hubbard_u_sweep
cargo test -p workflow_core
```
