# Architecture Status

## Current Implementation

**Architecture:** Utilities-based (no traits, no adapters)

**Status:** Phase 1 Complete (as of 2026-04-08)

### Implemented Components

#### Phase 1.1: workflow_utils (Layer 2) ✅

- `TaskExecutor`: Generic process execution utility
- `files` module: Generic file I/O utilities (re-exported flat at crate root — use `workflow_utils::{create_dir, write_file, ...}`)
- `MonitoringHook`: External monitoring integration
- **No traits, no adapters** - pure utilities

#### Phase 1.2: workflow_core (Layer 1) ✅

- `Workflow`: DAG container with `bon` builder — `Workflow::builder().name(...).build()?`
- `max_parallel`: Configurable via builder (defaults to `available_parallelism`)
- `Task`: Execution unit with `Arc<dyn Fn() -> Result<()>>`
- `DAG`: Dependency resolution with petgraph
- `WorkflowState`: JSON-based state persistence

#### Phase 1.3: Integration & Examples ✅

- Resume bug fixed: `Running` tasks reset to `Pending` on `WorkflowState::load`
- `examples/hubbard_u_sweep`: Layer 3 reference implementation (workspace member)
- Integration tests: sweep pattern, resume semantics, DAG ordering/failure propagation

### Architecture Documents

**Current (Authoritative):**

- `ARCHITECTURE.md` - Utilities-based three-layer architecture (v2.1)
- `PHASE1_IMPLEMENTATION_PLAN.md` - Implementation plan for Phase 1
- `plans/PHASE1.3_IMPLEMENTATION_PLAN.md` - Phase 1.3 integration & examples plan

**Outdated (Do Not Use):**

- `RUST_API_DESIGN_PLAN.md.OUTDATED` - Describes trait-based adapter pattern that was NOT implemented

## Three-Layer Architecture (Current)

```
Layer 3: Project Crates (User Code)
  ↓ uses
Layer 2: workflow_utils (Generic Utilities)
  - TaskExecutor, create_dir, write_file, ... (flat re-exports)
  - NO traits, NO adapters
  ↓ uses
Layer 1: workflow_core (Foundation)
  - Workflow (bon builder), Task, DAG, State
  ↓ uses
Parser Libraries: castep-cell-io, etc.
```

## Next Steps

**Phase 2: Examples and Documentation**

- Full HubbardU sweep with castep-cell-io builders (pending castep-cell-io integration)
- Convergence test example
- Comprehensive documentation

## Key Design Decisions

1. **No Adapters**: Software-specific logic belongs in parser libraries (castep-cell-io) or user code (Layer 3)
2. **Utilities Only**: Layer 2 provides generic utilities, not software-specific abstractions
3. **Closure-Based**: Tasks contain execution closures with full control
4. **Rust-First**: Users write Rust code, not TOML configuration

## Migration Notes

If you see references to:

- `TaskAdapter trait` → This was NOT implemented
- `CastepAdapter` → This is now in `castep_adapter/` as an example, not a core pattern
- TOML workflow definitions → Not used; users write Rust code directly

Refer to `ARCHITECTURE.md` for the authoritative architecture.
