# Architecture Status

## Current Implementation

**Architecture:** Utilities-based (no traits, no adapters)

**Status:** Phase 2.2 Complete (as of 2026-04-10)

### Implemented Components

#### Phase 1.1: workflow_utils (Layer 2) ✅

- `TaskExecutor`: Generic process execution utility
- `files` module: Generic file I/O utilities (re-exported flat at crate root — use `workflow_utils::{create_dir, write_file, ...}`)
- `MonitoringHook`: External monitoring integration
- **No traits, no adapters** - pure utilities
- `tokio` removed — pure std-thread

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

#### Phase 2.1: castep-cell-io Integration ✅

- castep-cell-io wired into `hubbard_u_sweep` example
- Execution reports tracked in `execution_reports/`

#### Phase 2.2: Production Readiness — Logging & Periodic Monitoring ✅ (2026-04-10)

- `tracing` integrated into `workflow_core`: structured `debug`/`info`/`error` events for workflow start/finish, task start/complete/fail
- `init_default_logging()` convenience fn (behind `default-logging` feature, uses `tracing-subscriber` + `RUST_LOG`)
- `PeriodicHookManager`: background-thread manager for `HookTrigger::Periodic` hooks; spawns on task start, stops cleanly on task completion or failure
- `Task::monitors()` builder method for attaching `MonitoringHook` lists to a task
- Per-task duration logging; summary (succeeded/failed counts + total duration) at workflow end
- `capture_task_error_context` is domain-agnostic (no CASTEP filenames in Layer 1)
- 36/36 tests pass; Clippy: 0 warnings

### Architecture Documents

**Current (Authoritative):**

- `ARCHITECTURE.md` - Utilities-based three-layer architecture (v2.2)
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

**Phase 3: Examples and Documentation**

- Convergence test example
- Comprehensive documentation

## Key Design Decisions

1. **No Adapters**: Software-specific logic belongs in parser libraries (castep-cell-io) or user code (Layer 3)
2. **Utilities Only**: Layer 2 provides generic utilities, not software-specific abstractions
3. **Closure-Based**: Tasks contain execution closures with full control
4. **Rust-First**: Users write Rust code, not TOML configuration
5. **No `anyhow` in lib crates**: `workflow_core` and `workflow_utils` use `WorkflowError` directly; `anyhow` is permitted only in binary/example crates (Layer 3)

## Migration Notes

If you see references to:

- `TaskAdapter trait` → This was NOT implemented
- `CastepAdapter` → This is now in `castep_adapter/` as an example, not a core pattern
- TOML workflow definitions → Not used; users write Rust code directly

Refer to `ARCHITECTURE.md` for the authoritative architecture.
