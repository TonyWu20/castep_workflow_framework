# castep_workflow_framework — Development Plan

## Project Goal

A generic HPC workflow framework for running computational chemistry/physics job
pipelines (CASTEP, VASP, CP2K, LAMMPS, etc.), replacing fragile bash scripts with
a maintainable, declarative Rust system.

## Workspace Structure

```
castep_workflow_framework/
  Cargo.toml          ← workspace root
  workflow_core/      ← generic core: traits, DAG, scheduler, state, TOML schema
  castep_adapter/     ← CASTEP executor factory
  lammps_adapter/     ← LAMMPS executor factory
  workflow_cli/       ← binary entry point
```

## Architecture Decisions

| Concern | Decision |
|---|---|
| Workflow definition | TOML (user-facing, no Rust required) |
| DAG library | `petgraph` (wrapped, not exposed in public API) |
| Async runtime | `tokio` |
| State persistence | SQLite via `rusqlite` (`.workflow_state.db`) |
| Sweep expansion | Parse-time, not runtime |
| Failure policy | Independent branches continue; skip only when all upstream paths blocked |
| Adapter coupling | `code = "..."` string → `ExecutorRegistry` |
| Resume | Auto on rerun: completed tasks skipped, running tasks re-queried from backend |

## TOML Schema

### Executor backends

```toml
[executors.local]
type = "local"
parallelism = 4

[executors.slurm]
type = "slurm"
partition = "compute"
ntasks = 32
walltime = "04:00:00"
```

### Task declaration

```toml
[[tasks]]
id = "scf"
code = "castep"
executor = "slurm"
workdir = "runs/scf"
depends_on = []

[tasks.inputs]
CUT_OFF_ENERGY = 700
```

### Sweep modes

**Zip** (default) — params advance together, equal-length lists:

```toml
[tasks.sweep]
mode = "zip"
params = [
    { name = "u_val",  values = [2.0, 3.0, 4.0] },
    { name = "cutoff", values = [500, 600, 700]  },
]
```

**Product** — cartesian product of all param values:

```toml
[tasks.sweep]
mode = "product"
params = [
    { name = "u_val",  values = [2.0, 3.0, 4.0] },
    { name = "cutoff", values = [500, 700, 900]  },
]
```

Template variables in `id`, `workdir`, and `depends_on` use `{param_name}` syntax.

## Core Modules (`workflow_core`)

| Module | Responsibility |
|---|---|
| `schema.rs` | TOML types + parse-time sweep expansion → `Vec<ConcreteTask>` |
| `executor.rs` | `Executor`, `ExecutorFactory` traits + `ExecutorRegistry` |
| `pipeline.rs` | `Pipeline`: DAG construction, topological sort, successor/predecessor queries |
| `state.rs` | `TaskState` enum + `StateDb` (SQLite persistence) |
| `scheduler.rs` | `Scheduler::run()` — main tokio poll loop (TODO) |

## Task State Machine

```
Pending → Ready → Submitted → Running → Completed
                                      ↘ Failed → (mark transitive dependents Skipped)
```

A dependent task is only `Skipped` when **all** its upstream paths are `Failed` or
`Skipped` (handles merge nodes correctly).

## Resume Behaviour

On every invocation:
1. Parse TOML → expand sweeps → build DAG
2. Load `.workflow_state.db` — tasks already `Completed` are skipped
3. Tasks found as `Running` are re-queried against the executor backend
4. Execution continues from current state

## Implementation Roadmap

- [x] Workspace scaffold
- [x] `workflow_core` skeleton (all modules compile)
- [x] `SweepDef` with `zip` and `product` modes
- [ ] `Scheduler::run()` — tokio poll loop with partial-failure handling
- [ ] Local executor (spawn subprocess, poll PID)
- [ ] SLURM executor (`sbatch` / `squeue` / `scancel`)
- [ ] `CastepFactory::build()` — dependency resolver (`.check` file copy)
- [ ] `LammpsFactory::build()`
- [ ] Graceful shutdown (SIGTERM → cancel all running jobs)
- [ ] Resume: re-query `Running` tasks on startup
- [ ] End-to-end integration test with mock executor
