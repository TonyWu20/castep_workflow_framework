# Architecture Status

## Current Implementation

**Architecture:** Utilities-based (no traits, no adapters)

**Status:** Phases 1–5 Complete (as of 2026-04-23)

### Implemented Components

#### Phase 1.1: workflow_utils (Layer 2) ✅

- `TaskExecutor`: Generic process execution utility
- `files` module: Generic file I/O utilities (re-exported flat at crate root — use `workflow_utils::{create_dir, write_file, ...}`)
- `MonitoringHook`: External monitoring integration
- **No traits, no adapters** - pure utilities
- `tokio` removed — pure std-thread

#### Phase 1.2: workflow_core (Layer 1) ✅

- `Workflow`: DAG container — `Workflow::new(name).with_max_parallel(n)?`
- `max_parallel`: Configurable (defaults to `available_parallelism`)
- `Task`: Execution unit with `ExecutionMode`, `setup`/`collect` closures, `TaskClosure` type alias
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

#### Phase 3: Production Framework ✅ (2026-04-15)

- `StateStore` / `StateStoreExt` trait + `JsonStateStore` with atomic writes (write-to-temp then rename)
- `load()`: crash-recovery path — resets Failed/Running/SkippedDueToDependencyFailure → Pending
- `load_raw()`: read-only inspection without crash-recovery resets (used by CLI status/inspect)
- `WorkflowError` `#[non_exhaustive]` enum with `thiserror`: `DuplicateTaskId`, `CycleDetected`, `UnknownDependency`, `StateCorrupted`, `TaskTimeout`, `InvalidConfig`, `Io`, `Interrupted`
- `run()` returns `Result<WorkflowSummary>` (succeeded/failed/skipped task IDs + duration)
- `ExecutionMode::Direct` with per-task `Option<Duration>` timeout
- OS signal handling: SIGTERM/SIGINT via `signal-hook`; graceful shutdown; re-registers on each `run()`
- `workflow-cli` binary: `status`, `inspect`, `retry` subcommands
- `Task` gains `setup`/`collect` closure fields; `TaskClosure = Box<dyn Fn(&Path) -> Result<(), WorkflowError> + Send + Sync>` type alias
- `anyhow` removed from `workflow_core`; `TaskStatus` re-exported from crate root
- End-to-end resume and timeout integration tests

#### Phase 4: Queued Execution & Graph-Aware Retry ✅ (2026-04-20)

- Log persistence for task stdout/stderr via `with_log_dir()`
- `HookTrigger::Periodic` background thread management
- `ExecutionMode::Queued` fully implemented: submits `job.sh` via `sbatch`/`qsub`, polls `squeue`/`qstat` for completion
- `QueuedRunner { kind: SchedulerKind }` in `workflow_utils` (`SchedulerKind::Slurm` | `SchedulerKind::Pbs`)
- `TaskSuccessors`: successor graph persisted in `JsonStateStore` (`task_successors: HashMap<String, Vec<String>>` with `#[serde(default)]`)
- `set_task_graph()` on `StateStore` trait (default no-op implementation)
- Workflow calls `set_task_graph()` after `build_dag()` in `run()`
- `downstream_of(&self, start: &[String]) -> Vec<String>`: BFS over successor graph in `StateStoreExt`
- CLI `retry` skips already-successful downstream tasks; `downstream_tasks(id)` helper
- `SystemProcessRunner::default()` via derive

#### Phase 5A: Production SLURM Sweep ✅ (2026-04-22)

- New workspace member: `examples/hubbard_u_sweep_slurm/`
- `clap` CLI binary with env-var support:
  - `CASTEP_SLURM_ACCOUNT`, `CASTEP_SLURM_PARTITION` (prefixed to avoid collision with SLURM env vars inside jobs)
  - `CASTEP_MODULES`, `CASTEP_COMMAND`
- First end-to-end consumer of `ExecutionMode::Queued` + `QueuedRunner::new(SchedulerKind::Slurm)`
- Job script generation (`job.sh`) per task; `#SBATCH` directives from config
- `collect` closure validates CASTEP output (`*.castep` exists, contains "Total time")
- `--dry-run` flag: prints topological order and exits without submitting
- Implementation plan: `plans/phase-5/phase5a_implementation.toml` (7 tasks)

#### Phase 5B: API Ergonomics ✅ (2026-04-23)

- `ExecutionMode::direct(cmd, &[args])` convenience constructor (eliminates struct literal boilerplate)
- `workflow_core::prelude` module: re-exports all commonly used types from `workflow_core`
- `workflow_utils::prelude` module: re-exports all commonly used types from both crates; Layer 3 binaries now use `use workflow_utils::prelude::*`
- `run_default(&mut workflow, &mut state)` in `workflow_utils`: eliminates repeated Arc wiring (`SystemProcessRunner` + `ShellHookExecutor`) in binaries
- `downstream_of<S: AsRef<str>>` generic signature — callers pass `&[&str]` without allocating
- `hubbard_u_sweep_slurm`: local mode now uses `run_default()`; SLURM mode keeps manual Arc wiring
- Inlined format args throughout (`{e}` instead of `{}`, e`)
- `init_default_logging()` exposed in `workflow_core` crate root
- `doc_markdown` fix on `workflow_core::prelude` doc comment

### Architecture Documents

**Current (Authoritative):**

- `ARCHITECTURE.md` - Utilities-based three-layer architecture (v5.0)
- `plans/phase-5/PHASE5A_PRODUCTION_SWEEP.md` - Phase 5A plan document
- `plans/phase-5/phase5a_implementation.toml` - Phase 5A implementation TOML
- `plans/phase-5/PHASE5B_API_ERGONOMICS.md` - Phase 5B plan document

**Outdated (Do Not Use):**

- `RUST_API_DESIGN_PLAN.md.OUTDATED` - Describes trait-based adapter pattern that was NOT implemented
- `PHASE1_IMPLEMENTATION_PLAN.md` - Phase 1 (superseded)
- `plans/PHASE1.3_IMPLEMENTATION_PLAN.md` - Phase 1.3 (superseded)

## Three-Layer Architecture (Current)

```
Layer 3: Project Crates (User Code)
  ↓ uses
Layer 2: workflow_utils (Generic Utilities)
  - TaskExecutor, create_dir, write_file, ... (flat re-exports)
  - SystemProcessRunner, ShellHookExecutor
  - QueuedRunner (SLURM/PBS)
  - run_default() (Phase 5B)
  - prelude (Phase 5B)
  - NO traits, NO adapters
  ↓ uses
Layer 1: workflow_core (Foundation)
  - Workflow::new(), Task::new(id, mode), ExecutionMode
  - StateStore trait, JsonStateStore, WorkflowError, WorkflowSummary
  - Signal handling, tracing logging
  - workflow-cli binary
  - prelude (Phase 5B)
  ↓ uses
Parser Libraries: castep-cell-io, castep-cell-fmt, etc.
```

#### Phase 6: Reliability, Multi-Parameter Patterns, and Ergonomics 🚧 (planned 2026-04-25)

- **CollectFailurePolicy** — fix correctness bug: `mark_completed` currently runs *before* collect; collect failures only warn. New: run collect before marking completed; `FailTask` (default) marks task Failed, `WarnOnly` preserves old behavior. Software-agnostic: Layer 3 defines what "success" means, framework defines the policy.
- **Multi-parameter sweep** — build and validate on HPC cluster: product (`iproduct!`) and pairwise (`zip`) modes, dependent task chains (SCF → DOS per parameter combo). No new framework API — Layer 3 patterns with `itertools`.
- **`--workdir` / root_dir** — `Workflow::with_root_dir()` resolves relative task workdirs against a configurable root; enables binary invocation from any directory.
- **`retry` stdin support** — accept task IDs from stdin (`workflow-cli retry state.json -`) for Unix pipeline composition; avoids reimplementing grep with `--match` glob.
- **Documentation accuracy sweep** — fix 6 deferred doc/test mismatches from Phase 5B.

## Next Steps

**Phases 1–5 are complete.** Phase 6 is planned. The framework is production-ready for single-parameter CASTEP sweeps on SLURM. Phase 6 extends reliability and validates multi-parameter sweep patterns on real hardware.

Future work beyond Phase 6:
- Typed result collection / convergence patterns (Phase 7)
- Additional scheduler backends (PBS via `SchedulerKind::Pbs`)
- Tier 2 interactive CLI (guided prompts for non-Rust-savvy researchers)
- TUI/interactive monitoring interface (Tier 3)

## Key Design Decisions

1. **No Adapters**: Software-specific logic belongs in parser libraries (castep-cell-io, castep-cell-fmt) or user code (Layer 3)
2. **Utilities Only**: Layer 2 provides generic utilities, not software-specific abstractions
3. **Closure-Based**: Tasks contain `setup`/`collect` closures with full control over `&Path` workdir
4. **Rust-First**: Users write Rust code, not TOML configuration
5. **No `anyhow` in lib crates**: `workflow_core` and `workflow_utils` use `WorkflowError` directly; `anyhow` is permitted only in binary/example crates (Layer 3)
6. **One justified trait**: `StateStore` — I/O boundary enabling future SQLite swap
7. **Prefixed env vars**: `CASTEP_SLURM_*` prefix avoids collision with SLURM's own env vars set inside running jobs

## Implementation Guidelines

**Newtype Encapsulation:** Design newtypes with full encapsulation on introduction. Never expose raw inner type via public accessor — introducing then removing `inner()` causes churn across fix plans.

**Domain Logic Placement:** Place domain logic operating on `workflow_core` types in `workflow_core` from the initial implementation. Logic written in the CLI binary and later migrated causes churn (BFS `downstream_tasks` pattern, v2/v4/v5).
