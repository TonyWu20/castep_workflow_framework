# Feature Request: Parameter Merge Strategy for CASTEP Adapter

## Problem Statement

The `castep_adapter` needs to merge user-specified parameter overrides (`task.inputs`) with a base `.param` file from the seed directory. However, there's a type mismatch:

- **User input**: `HashMap<String, toml::Value>` with generic key-value pairs like `{"HUBBARD_U": "2.0", "CUT_OFF_ENERGY": "400"}`
- **castep-cell-io API**: Strongly-typed `ParamDocument` with 100+ typed fields like `pub cutoff_energy: Option<CutOffEnergy>`

The adapter cannot generically map string keys to typed struct fields without reflection (which Rust doesn't have).

## Current Workflow

```toml
[[tasks]]
id = "run_U2"
seed_dir = "/data/seeds/GDY_111_Fe_U"
seed_name = "GDY_111_Fe_U"

[tasks.inputs]
param.HUBBARD_U = "2.0"
param.CUT_OFF_ENERGY = "400"
cell.KPOINTS_MP_GRID = "4 4 4"
```

The adapter needs to:
1. Read `<seed_dir>/<seed_name>.param` (base parameters)
2. Apply `task.inputs.param.*` overrides on top
3. Write merged result to `<workdir>/<seed_name>.param`
4. Same for `.cell` file

## Proposed Solutions

### Option 1: Low-level key-value merge (Recommended)

Add a utility function to `castep-cell-io` that operates at the `Cell<'_>` IR level:

```rust
pub fn merge_param_keyvalues(
    base_text: &str,
    overrides: &HashMap<String, String>
) -> Result<String, Error> {
    let mut cells = parse_cell_file(base_text)?;
    
    for (key, value) in overrides {
        // Find existing Cell::KeyValue with this key and replace its value
        // Or append a new Cell::KeyValue if not found
    }
    
    Ok(to_string_many(&cells))
}
```

**Pros:**
- Works with any CASTEP keyword, even ones not yet typed in `ParamDocument`
- Simple string-based API that adapters can use directly
- No need to maintain a key → field mapping

**Cons:**
- Bypasses type safety of `ParamDocument`
- No validation of parameter values

### Option 2: Reflection-like field mapping

Add a `set_field_by_name` method to `ParamDocument`:

```rust
impl ParamDocument {
    pub fn set_field_by_name(&mut self, key: &str, value: &str) -> Result<(), Error> {
        match key.to_uppercase().as_str() {
            "HUBBARD_U" => { /* parse value and set self.hubbard_u */ }
            "CUT_OFF_ENERGY" => { /* parse value and set self.cutoff_energy */ }
            // ... 100+ more cases
            _ => return Err(Error::UnknownKeyword(key.to_string()))
        }
    }
}
```

**Pros:**
- Type-safe, validated parameters
- Leverages existing typed structs

**Cons:**
- Requires maintaining a giant match statement for all keywords
- Brittle — breaks when new keywords are added to CASTEP
- High maintenance burden

### Option 3: Hybrid approach

Provide both:
- Low-level `merge_param_keyvalues` for generic workflows (Option 1)
- High-level typed API for programmatic construction (existing `ParamDocumentBuilder`)

Users choose based on their needs. The workflow framework uses the low-level API.

## Recommendation

**Option 1** (low-level merge) is the right choice for the workflow framework because:

1. The framework is a **data pipeline**, not a parameter validator
2. Users already validated their seed `.param` files in Materials Studio
3. The framework just needs to apply sweep overrides, not understand CASTEP semantics
4. Future-proof — works with any CASTEP version without code changes

## Implementation in `castep-cell-io`

Add to `castep_cell_fmt/src/lib.rs`:

```rust
/// Merge key-value overrides into a CASTEP .param or .cell file.
///
/// Existing keys are replaced; new keys are appended.
pub fn merge_keyvalues(
    base_text: &str,
    overrides: &HashMap<String, String>
) -> CResult<String> {
    let mut cells = parse_cell_file(base_text)?;
    let mut remaining: HashSet<&str> = overrides.keys().map(|s| s.as_str()).collect();
    
    // Pass 1: replace existing keys
    for cell in &mut cells {
        if let Cell::KeyValue(key, value) = cell {
            if let Some(new_val) = overrides.get(*key) {
                *value = CellValue::Str(new_val);
                remaining.remove(key);
            }
        }
    }
    
    // Pass 2: append new keys
    for key in remaining {
        let value = overrides.get(key).unwrap();
        cells.push(Cell::KeyValue(key, CellValue::Str(value)));
    }
    
    Ok(to_string_many(&cells))
}
```

## Usage in `castep_adapter`

```rust
use castep_cell_fmt::merge_keyvalues;

let base_param = std::fs::read_to_string(
    format!("{}/{}.param", task.seed_dir, task.seed_name)
)?;

let param_overrides: HashMap<String, String> = task.inputs
    .get("param")
    .and_then(|v| v.as_table())
    .map(|t| t.iter().map(|(k, v)| (k.clone(), v.to_string())).collect())
    .unwrap_or_default();

let merged_param = merge_keyvalues(&base_param, &param_overrides)?;

std::fs::write(
    format!("{}/{}.param", task.workdir, task.seed_name),
    merged_param
)?;
```

## Open Questions

1. Should `merge_keyvalues` preserve comments and formatting from the base file?
2. Should it validate that keys are known CASTEP keywords, or pass through anything?
3. How should it handle block-level parameters (e.g. `%BLOCK SPECIES_POT`)?

## Next Steps

1. Discuss this design with the `castep-cell-io` maintainer (you)
2. Implement `merge_keyvalues` in `castep-cell-io` v0.3.1 or v0.4.0
3. Update `castep_adapter` to use the new API
4. Test with real CASTEP seed folders
