# Phase 2.1 Implementation Plan: Wire castep-cell-io into hubbard_u_sweep

## Context

Phase 1 is complete: `workflow_core` (DAG orchestration) and `workflow_utils` (generic utilities) are fully implemented and tested. The `examples/hubbard_u_sweep` binary currently uses a hardcoded cell file string (line 19-21 in main.rs has a TODO comment). This phase replaces it with proper `castep-cell-io` domain types, establishing the canonical Layer 3 pattern: seed files as immutable templates, programmatically mutated for parameter sweeps.

**Architectural Goal**: Demonstrate clean integration between workflow orchestration (`workflow_core`), domain-specific types (`castep-cell-io`), and utility functions (`workflow_utils`) without violating layer boundaries.

**Critical Constraint**: `workflow_core/tests/hubbard_u_sweep.rs` must NOT be modified — it tests orchestration, not cell content.

---

## Implementation Strategy

### Ownership & Closure Capture

**Decision**: Parse inside each task closure (not once before the loop). Rationale:
- `seed_cell` is `&'static str` from `include_str!` — trivially copyable, no clone needed
- Parsing happens at runtime inside each task, allowing independent mutation
- Each closure captures `seed_cell` by copy and `u` by move
- This avoids lifetime complexity and keeps tasks independent

### Type Flow

```rust
&'static str (seed_cell from include_str!)
  → parse() → Result<CellDocument, castep_cell_fmt::Error>
  → mutate field → CellDocument
  → .to_cell_file() → Vec<Cell<'_>>  // temporary, borrowed from CellDocument
  → to_string_many_spaced(&Vec<Cell<'_>>) → String
  → write_file(path, &String)  // Deref coercion: &String → &str
```

**Lifetime Safety**: `to_string_many_spaced(&cell_doc.to_cell_file())` creates a temporary `Vec<Cell<'_>>` that borrows from `cell_doc`. This is safe because the borrow ends before `cell_doc` is dropped. Keep this call inline — do NOT store the `Vec<Cell<'_>>` in a variable.

---

## Task Breakdown

### TASK-1: Add castep-cell-fmt dependency
**File**: `examples/hubbard_u_sweep/Cargo.toml`

**Action**: Add `castep-cell-fmt = "0.1.0"` under `[dependencies]`

**Acceptance Criteria**:
- `castep-cell-fmt = "0.1.0"` appears in manifest
- `castep-cell-io = "0.4.0"` remains unchanged
- `cargo check -p hubbard_u_sweep` resolves the dependency without errors

**Notes**: The Cargo.toml already exists as a workspace member. This is a simple dependency addition. Do not modify workspace-level Cargo.toml.

---

### TASK-2: Create seed files
**Files**: `examples/hubbard_u_sweep/seeds/ZnO.cell` and `examples/hubbard_u_sweep/seeds/ZnO.param`

**Action**: Create the `seeds/` directory and write the exact seed files below.

**ZnO.cell** (wurtzite structure, a=3.25 Å, c=5.21 Å):
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

**ZnO.param**:
```
task : SinglePoint
```

**Acceptance Criteria**:
- Directory `examples/hubbard_u_sweep/seeds/` exists
- Both files exist with exact contents above
- Files are valid CASTEP input (parseable by `castep-cell-fmt`)
- `include_str!("../seeds/ZnO.cell")` resolves correctly from `src/main.rs`

**Notes**: The `.cell` file has no HUBBARD_U block — that's added programmatically. The `.param` file is not parsed in this phase (ParamDocument deferred to later phase).

---

### TASK-3: Update imports in main.rs
**File**: `examples/hubbard_u_sweep/src/main.rs`

**Action**: Add the following imports at the top of the file:
```rust
use anyhow::{Context, Result};
use castep_cell_fmt::{parse, ToCellFile, format::to_string_many_spaced};
use castep_cell_io::CellDocument;
use castep_cell_io::cell::species::{AtomHubbardU, HubbardU, HubbardUUnit, OrbitalU, Species};
use workflow_core::{Task, Workflow};
use workflow_utils::{TaskExecutor, create_dir, write_file};
```

**Acceptance Criteria**:
- All imports present and correctly formatted
- `cargo check -p hubbard_u_sweep` compiles without import errors
- Verify imports resolve: `CellDocument`, `HubbardU`, and format functions are accessible from published crates

**Notes**: The `anyhow::Context` import enables `.context()` chaining on parse errors. HubbardU types are re-exported at `castep_cell_io::cell::species` (the `hubbard_u` submodule is private in the published crate).

---

### TASK-4: Add seed file loading
**File**: `examples/hubbard_u_sweep/src/main.rs`

**Action**: Add these two lines immediately after `fn main() -> Result<()> {`:
```rust
let seed_cell = include_str!("../seeds/ZnO.cell");
let seed_param = include_str!("../seeds/ZnO.param");
```

**Acceptance Criteria**:
- Variables defined before the workflow builder
- `include_str!` paths are correct relative to `src/main.rs`
- `cargo check -p hubbard_u_sweep` compiles (unused variable warnings acceptable at this stage)

**Notes**: These are compile-time inclusions (`&'static str`). The variables will be captured by task closures in later tasks.

---

### TASK-5: Replace hardcoded cell with parsing
**File**: `examples/hubbard_u_sweep/src/main.rs` (inside task closure, line 19-21)

**Action**: Replace lines 19-21 (the TODO comment and `let cell_content = format!(...)`) with:
```rust
let mut cell_doc: CellDocument =
    parse(seed_cell).context("failed to parse seed ZnO.cell")?;
```

**Acceptance Criteria**:
- `parse(seed_cell)` call added with `.context()` error wrapping
- `cell_doc` is mutable (`let mut`)
- Type annotation `: CellDocument` is explicit
- Old `let cell_content = format!(...)` line is removed
- `cargo check -p hubbard_u_sweep` compiles
- Parse errors propagate correctly via `?` operator (integrates with existing workflow error handling)

**Notes**: Parsing happens inside each task closure (see "Ownership & Closure Capture" section). The `parse` function returns `Result<CellDocument, castep_cell_fmt::Error>`. The error type implements `std::error::Error`, so `.context()` works directly with `anyhow`.

---

### TASK-6: Add HubbardU mutation logic
**File**: `examples/hubbard_u_sweep/src/main.rs` (inside task closure, after parsing)

**Action**: Immediately after the parsing code from TASK-5, add:
```rust
let atom_u = AtomHubbardU::builder()
    .species(Species::Symbol("Zn".to_string()))
    .orbitals(vec![OrbitalU::D(u)])
    .build();
let hubbard_u = HubbardU::builder()
    .unit(HubbardUUnit::ElectronVolt)
    .atom_u_values(vec![atom_u])
    .build();
cell_doc.hubbard_u = Some(hubbard_u);
```

**Acceptance Criteria**:
- Builder code added exactly as shown
- `cell_doc.hubbard_u = Some(hubbard_u);` assigns the constructed value
- `cargo check -p hubbard_u_sweep` compiles
- Manual verification: After running the binary, inspect a generated `.cell` file (e.g., `runs/U1.0/ZnO.cell`) to confirm it contains `%BLOCK HUBBARD_U` with correct species and U value

**Notes**: The `u` variable is captured from the loop (`for u in [0.0_f64, 1.0, ...]`). `OrbitalU::D(f64)` is the correct variant for d-orbital U values. The `bon` builder automatically wraps `.unit()` in an Option.

---

### TASK-7: Replace file writing with serialization
**File**: `examples/hubbard_u_sweep/src/main.rs` (inside task closure, line 22-23)

**Action**: Replace lines 22-23 (the two `write_file` calls) with:
```rust
let output = to_string_many_spaced(&cell_doc.to_cell_file());
write_file(format!("{workdir}/ZnO.cell"), &output)?;
write_file(format!("{workdir}/ZnO.param"), seed_param)?;
```

**Acceptance Criteria**:
- `to_string_many_spaced(&cell_doc.to_cell_file())` produces the serialized .cell content
- `write_file` calls use the new `output` variable and `seed_param` (not hardcoded strings)
- Old hardcoded `write_file` calls are removed
- `cargo build -p hubbard_u_sweep` compiles without errors or warnings
- Output `.cell` files contain `%BLOCK HUBBARD_U` with correct U values

**Notes**: `to_cell_file()` returns `Vec<Cell<'_>>` (infallible). `to_string_many_spaced` returns `String` (infallible). `write_file` takes `&str` — pass `&output` (String coerces via Deref). The param file is written verbatim from `seed_param`. Keep the serialization call inline as shown — do NOT store `cell_doc.to_cell_file()` in a variable (see "Type Flow" section).

---

### TASK-8: Run tests and verify
**Action**: Run the following commands:
```bash
cargo check -p hubbard_u_sweep
cargo build -p hubbard_u_sweep
cargo test -p workflow_core
```

**Acceptance Criteria**:
- `cargo check` succeeds with no errors
- `cargo build` produces a binary
- `cargo test -p workflow_core` passes (verifies orchestration layer is unaffected)
- No new warnings are introduced

**Notes**: This validates that the integration is complete and no regressions were introduced. The `workflow_core` tests use a mock task and should not be affected by changes in the example binary. If they fail, the layer boundary was violated.

---

### TASK-9: Run clippy checks
**Action**: Run the following commands:
```bash
cargo clippy -p hubbard_u_sweep
```

If clippy suggests improvements, review them and apply with:
```bash
cargo clippy --fix --allow-dirty -p hubbard_u_sweep
```

**Acceptance Criteria**:
- `cargo clippy -p hubbard_u_sweep` runs without errors
- Any clippy warnings are reviewed and either fixed or explicitly allowed
- If `--fix` is used, verify changes with `git diff` before committing

**Notes**: The `--allow-dirty` flag is required if there are uncommitted changes. Review clippy's suggestions carefully — some may conflict with the architectural constraints (e.g., don't add abstractions that violate the "inline in main.rs" constraint). User confirmation is required before running `--fix`.

---

## Dependency Graph

```
TASK-1 (Add dependency) ──┐
                          ├──> TASK-3 (Update imports) ──> TASK-4 (Add include_str!) ──┐
TASK-2 (Create seeds) ────┘                                                             │
                                                                                         ├──> TASK-5 (Parse) ──> TASK-6 (Mutate) ──> TASK-7 (Serialize) ──> TASK-8 (Test) ──> TASK-9 (Clippy)
```

**Critical Path**: TASK-1 → TASK-3 → TASK-4 → TASK-5 → TASK-6 → TASK-7 → TASK-8 → TASK-9

**Parallel Opportunities**: TASK-1 and TASK-2 can run concurrently.

---

## Verification

After all tasks complete, the expected output structure is:
```
runs/
  U0.0/
    ZnO.cell  # Contains %BLOCK HUBBARD_U with U=0.0
    ZnO.param
  U1.0/
    ZnO.cell  # Contains %BLOCK HUBBARD_U with U=1.0
    ZnO.param
  ...
```

Manual verification: Inspect `runs/U1.0/ZnO.cell` to confirm it contains:
```
%BLOCK HUBBARD_U
eV
Zn 1: d: 1.0
%ENDBLOCK HUBBARD_U
```

---

## Critical Files

- `examples/hubbard_u_sweep/Cargo.toml` — Add dependency
- `examples/hubbard_u_sweep/src/main.rs` — All code changes
- `examples/hubbard_u_sweep/seeds/ZnO.cell` — New seed file
- `examples/hubbard_u_sweep/seeds/ZnO.param` — New seed file

**Do NOT modify**:
- `workflow_core/Cargo.toml` — Must not add `castep-cell-fmt` dependency
- `workflow_core/tests/hubbard_u_sweep.rs` — Tests orchestration, not cell content
- Any files in `workflow_utils/` — No new abstractions in this phase

---

## Risks & Mitigations

**API Mismatches**: If published crate APIs differ from plan assumptions, TASK-3 will fail at compile time. Mitigation: Plan documents confirmed public API paths.

**Lifetime Fragility**: The inline call `to_string_many_spaced(&cell_doc.to_cell_file())` is safe but fragile. If future refactoring stores the `Vec<Cell<'_>>` in a variable, the borrow checker will require explicit lifetime annotations. Mitigation: Keep the call inline as specified.

**Layer Boundary Violation**: If `castep-cell-fmt` leaks into `workflow_core` dependencies, TASK-8 will catch it. Mitigation: Only modify files in `examples/hubbard_u_sweep/`.

---

## Out of Scope (Deferred)

- Integration tests for generated `.cell` files (manual inspection sufficient for this phase)
- ParamDocument builder (param file written verbatim from seed)
- Error handling improvements beyond basic `.context()` wrapping
- Documentation/doc comments (focus on functionality)
