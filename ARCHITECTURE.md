# Architecture & Implementation Status

## Crate Layout

```
castep_workflow_framework/
├── workflow_core/          # Core engine (scheduler, pipeline, executor traits, state DB)
│   └── src/
│       ├── schema.rs       # TOML deserialization, sweep expansion → Vec<ConcreteTask>
│       ├── pipeline.rs     # DAG (petgraph DiGraph), cycle detection, topological order
│       ├── executor.rs     # Executor / ExecutorFactory traits, ExecutorRegistry
│       ├── executors/
│       │   ├── local.rs    # LocalExecutor — PID-based, sentinel-file exit tracking
│       │   └── slurm.rs    # SlurmExecutor — sbatch submit, squeue poll, scancel
│       ├── scheduler.rs    # Scheduler — poll loop, parallelism cap, resume, cancellation
│       └── state.rs        # StateDb — SQLite via tokio-rusqlite, TaskRecord persistence
├── castep_adapter/         # CASTEP-specific executor factory
│   └── src/lib.rs          # CastepFactory → CastepExecutor (defers .param write to submit)
├── lammps_adapter/         # LAMMPS stub (not yet implemented)
└── workflow_cli/           # Binary entry point (reads TOML, wires registry, runs scheduler)
```

## Key Design Decisions

### Two kinds of state
- **Persistent** (SQLite): task ID, `TaskState`, job handle string. Survives restarts.
- **Ephemeral** (in-memory): `live_executors: HashMap<String, Box<dyn Executor>>`. Rebuilt on resume from DB records with `Submitted`/`Running` state.

### Executor trait is stateless after submit
`Executor::poll` and `cancel` receive the `JobHandle` and must not rely on owned process state. The handle is the durable identity.

### LocalExecutor sentinel files
`submit` spawns the process, captures PID, hands `Child` to a background `tokio::spawn` task that calls `wait()` and writes `<workdir>/.exit.<pid>` with the exit code. `poll` reads the sentinel first; if absent, uses `kill(pid, 0)` to check liveness. This survives PID reuse and scheduler restarts.

### Parallelism cap
`submitted_count` is seeded with already-active (`Submitted`/`Running`) tasks before the submission loop each cycle, so the cap is respected across poll cycles, not just within a single cycle.

### Failure propagation
Tasks whose dependency is `Failed` are transitively marked `Skipped`. Independent branches continue running.

## Implementation Status

| Component | Status | Notes |
|---|---|---|
| `schema.rs` — TOML parsing + sweep expansion | ✅ Done | product/zip/no-sweep; template substitution in id/workdir |
| `pipeline.rs` — DAG + topo sort | ✅ Done | cycle detection, unknown-dep error, `tasks()` iterator |
| `executor.rs` — traits + registry | ✅ Done | `Executor: Send + Sync`; registry dispatches on `task.code` |
| `executors/local.rs` — LocalExecutor | ✅ Done | PID-based, sentinel file, background reaper, resume-safe |
| `executors/slurm.rs` — SlurmExecutor | ✅ Done | sbatch/squeue/scancel; async fs write; mock-runner tested |
| `state.rs` — SQLite state DB | ✅ Done | upsert, load, `Send` bound verified |
| `scheduler.rs` — poll loop | ✅ Done | resume, parallelism cap, failure propagation, cancellation token |
| `castep_adapter` — CastepFactory | ✅ Done | deferred I/O in CastepExecutor; Local + Slurm backends |
| `lammps_adapter` | 🔲 Stub | factory registered, executor not implemented |
| `workflow_cli` — binary | 🔲 Partial | reads TOML, wires CastepFactory; error handling minimal |
| `.check` file copy on dependency | 🔲 Not started | README feature #2 — copy parent `.check` to child workdir |
| Output parsing (`.castep` file) | 🔲 Not started | README feature #3 — extract convergence data to CSV |

## Test Coverage

19 tests, all passing (`cargo test --workspace`).

| Test | What it covers |
|---|---|
| `schema` (4 tests) | sweep expansion, template substitution |
| `pipeline` (4 tests) | topo order, unknown dep, cycle, successors/predecessors |
| `executors::local` (4 tests) | echo run, cancel, PID-only resume, sentinel double-poll |
| `executors::slurm` (2 tests) | submit parses job ID, poll maps squeue states |
| `state` (1 test) | StateDb is Send |
| `scheduler` (4 tests) | chain completes, cancellation, parallelism cap, resume re-hydration |
| `castep_adapter` (1 test) | build() defers filesystem I/O |

## Known Gaps

- `lammps_adapter`: executor not implemented.
- `.check` file propagation: parent→child `.check` copy not yet wired into `CastepExecutor::submit`.
- Output parsing: no `.castep` file reader or CSV export.
- `workflow_cli`: minimal error handling; no progress display.
