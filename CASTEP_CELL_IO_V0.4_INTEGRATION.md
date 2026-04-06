# castep-cell-io v0.4.0 Integration Guide

## Overview

castep-cell-io v0.4.0 (released 2026-04-06) provides all the builder patterns required for the workflow framework. This document describes how to integrate with the new API.

## What's Available in v0.4.0

### ✅ Cell File Builders (All Required Blocks)

1. **HubbardU and AtomHubbardU** - Hubbard U parameter specification
2. **PositionsFrac and PositionFracEntry** - Fractional atomic positions with SPIN/MIXTURE support
3. **SpeciesPot and SpeciesPotEntry** - Pseudopotential file mapping
4. **KpointsList** - Brillouin zone k-point sampling

### ✅ Param File Builders (18 Parameter Groups)

ParamDocument has been refactored into 18 nested sub-structs:
- `GeneralParams`, `ElectronicParams`, `BasisSetParams`, `ExchangeCorrelationParams`
- `ElectronicMinimisationParams`, `GeometryOptimizationParams`, `PhononParams`
- `BandStructureParams`, `MolecularDynamicsParams`, `ElectricFieldParams`
- `PseudopotentialParams`, `DensityMixingParams`, `PopulationAnalysisParams`
- `OpticsParams`, `NmrParams`, `SolvationParams`, `ElectronicExcitationsParams`
- `TransitionStateParams`

## Builder API Examples

### HubbardU Construction

```rust
use castep_cell_io::cell::species::hubbard_u::*;
use castep_cell_io::cell::species::Species;

// Build AtomHubbardU for a single species
let atom_u = AtomHubbardU::builder()
    .species(Species::Symbol("Zn".to_string()))
    .orbitals(vec![OrbitalU::D(5.0)])
    .build();

// Build HubbardU block
let hubbard_u = HubbardU::builder()
    .unit(HubbardUUnit::ElectronVolt)
    .atom_u_values(vec![atom_u])
    .build();

// Apply to CellDocument
cell_doc.hubbard_u = Some(hubbard_u);
```

### PositionsFrac Construction

```rust
use castep_cell_io::cell::positions::positions_frac::*;
use castep_cell_io::cell::species::Species;

// Build individual position entries
let entry1 = PositionFracEntry::builder()
    .species(Species::Symbol("Fe".to_string()))
    .coord([0.0, 0.0, 0.0])
    .spin(2.0)  // Optional
    .build();

let entry2 = PositionFracEntry::builder()
    .species(Species::Symbol("O".to_string()))
    .coord([0.5, 0.5, 0.5])
    .build();

// Build PositionsFrac block
let positions = PositionsFrac::builder()
    .positions(vec![entry1, entry2])
    .build();

// Apply to CellDocument
cell_doc.positions_frac = Some(positions);
```

### SpeciesPot Construction

```rust
use castep_cell_io::cell::species::species_pot::*;
use castep_cell_io::cell::species::Species;

// Build individual entries
let entry1 = SpeciesPotEntry::builder()
    .species(Species::Symbol("Fe".to_string()))
    .filename("Fe_00PBE.usp".to_string())
    .build();

let entry2 = SpeciesPotEntry::builder()
    .species(Species::Symbol("O".to_string()))
    .filename("O_00PBE.usp".to_string())
    .build();

// Build SpeciesPot block
let species_pot = SpeciesPot::builder()
    .potentials(vec![entry1, entry2])
    .build();

// Apply to CellDocument
cell_doc.species_pot = Some(species_pot);
```

### KpointsList Construction

```rust
use castep_cell_io::cell::bz_sampling_kpoints::kpoints_list::*;
use castep_cell_io::cell::bz_sampling_kpoints::kpoint::Kpoint;

// Build k-points
let kpt1 = Kpoint { coord: [0.0, 0.0, 0.0], weight: 0.5 };
let kpt2 = Kpoint { coord: [0.5, 0.5, 0.5], weight: 0.5 };

// Build KpointsList block
let kpoints = KpointsList::builder()
    .kpts(vec![kpt1, kpt2])
    .build();

// Apply to CellDocument
cell_doc.kpoints_list = Some(kpoints);
```

### ParamDocument Construction (v0.4.0 Nested Structure)

```rust
use castep_cell_io::{ParamDocument, param::*};

let param_doc = ParamDocument::builder()
    .general(general::GeneralParams::builder()
        .task(general::Task::SinglePoint)
        .build())
    .exchange_correlation(exchange_correlation::ExchangeCorrelationParams::builder()
        .xc_functional(exchange_correlation::XcFunctional::Pbe)
        .build())
    .basis_set(basis_set::BasisSetParams::builder()
        .cutoff_energy(basis_set::CutOffEnergy { 
            value: 500.0, 
            unit: Some(crate::units::energy_units::EnergyUnit::Ev) 
        })
        .build())
    .build();
```

## Migration from Old API (Hypothetical)

### Before (Hypothetical Old API)
```rust
// This was NEVER implemented, but shows the conceptual difference
let hubbard_u = HubbardU::builder()
    .unit(HubbardUUnit::ElectronVolt)
    .add_atom_u(AtomHubbardU::builder()
        .species("Zn")
        .add_orbital(OrbitalU::D(5.0))
        .build()?)
    .build()?;
```

### After (v0.4.0 Actual API)
```rust
// v0.4.0 uses Vec-based construction
let atom_u = AtomHubbardU::builder()
    .species(Species::Symbol("Zn".to_string()))
    .orbitals(vec![OrbitalU::D(5.0)])
    .build();

let hubbard_u = HubbardU::builder()
    .unit(HubbardUUnit::ElectronVolt)
    .atom_u_values(vec![atom_u])
    .build();
```

## Key Differences

1. **Species Type**: Use `Species::Symbol(String)` instead of `&str`
2. **Vec-based Construction**: Use `vec![]` for collections instead of `.add_*()` methods
3. **No Result Returns**: Builders return values directly, not `Result<T>`
4. **Nested Param Groups**: ParamDocument fields are now grouped (e.g., `doc.general.task`)

## Workflow Framework Integration

### Task Cell Modifier Example

```rust
use castep_cell_io::{CellDocument, cell::species::hubbard_u::*, cell::species::Species};

// In workflow task definition
task.set_cell_modifier(Box::new(move |mut cell_doc: CellDocument| {
    // Modify HubbardU using v0.4.0 builders
    let atom_u = AtomHubbardU::builder()
        .species(Species::Symbol("Zn".to_string()))
        .orbitals(vec![OrbitalU::D(u_value)])
        .build();

    let hubbard_u = HubbardU::builder()
        .unit(HubbardUUnit::ElectronVolt)
        .atom_u_values(vec![atom_u])
        .build();

    cell_doc.hubbard_u = Some(hubbard_u);
    Ok(cell_doc)
}));
```

## References

- **Changelog**: `/Users/tony/programming/castep-cell-io/CHANGELOG.md`
- **Migration Guide**: `/Users/tony/programming/castep-cell-io/MIGRATION_0.4.md`
- **Source Code**: `/Users/tony/programming/castep-cell-io/castep_cell_io/src/`

## Status

✅ All prerequisite builders for workflow framework are complete and tested in castep-cell-io v0.4.0.

The workflow framework can now proceed with Phase 1 implementation.
