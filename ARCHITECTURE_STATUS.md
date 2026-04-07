# Architecture Status

## Current Implementation

**Architecture:** Utilities-based (no traits, no adapters)

**Status:** Phase 1 Complete (as of 2026-04-07)

### Implemented Components

#### Phase 1.1: workflow_utils (Layer 2) ✅
- `TaskExecutor`: Generic process execution utility
- `files` module: Generic file I/O utilities  
- `MonitoringHook`: External monitoring integration
- **No traits, no adapters** - pure utilities

#### Phase 1.2: workflow_core (Layer 1) ✅
- `Workflow`: DAG container with closure-based tasks
- `Task`: Execution unit with `Arc<dyn Fn() -> Result<()>>`
- `DAG`: Dependency resolution with petgraph
- `WorkflowState`: JSON-based state persistence

### Architecture Documents

**Current (Authoritative):**
- `FINAL_ARCHITECTURE_DESIGN.md` - Utilities-based three-layer architecture
- `PHASE1_IMPLEMENTATION_PLAN.md` - Implementation plan for Phase 1
- `LAYER2_RETHINK.md` - First-principles analysis eliminating adapters

**Outdated (Do Not Use):**
- `RUST_API_DESIGN_PLAN.md.OUTDATED` - Describes trait-based adapter pattern that was NOT implemented

## Three-Layer Architecture (Current)

```
Layer 3: Project Crates (User Code)
  ↓ uses
Layer 2: workflow_utils (Generic Utilities)
  - TaskExecutor, files, MonitoringHook
  - NO traits, NO adapters
  ↓ uses
Layer 1: workflow_core (Foundation)
  - Workflow, Task, DAG, State
  ↓ uses
Parser Libraries: castep-cell-io, etc.
```

## Next Steps

**Phase 1.2.1: Builder Pattern Integration** (Current)
- Add `bon` crate for Workflow builder
- Expose `max_parallel` configuration
- See: `plans/PHASE1.2_BUILDER_PATTERN.md`

**Phase 2: Examples and Documentation**
- HubbardU sweep example
- Convergence test example
- Integration tests

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

Refer to `FINAL_ARCHITECTURE_DESIGN.md` for the authoritative architecture.
