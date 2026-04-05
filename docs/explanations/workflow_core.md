# workflow_core — Explained Simply

## What it does

Manages running hundreds of simulations automatically: reads a recipe, figures out order, launches jobs, watches them, saves progress for resuming, and cleans up on Ctrl-C.

---

## Five pieces (restaurant kitchen analogy)

| File | Role |
|---|---|
| `schema.rs` | The menu — what jobs exist, what params they use |
| `pipeline.rs` | The order ticket board — what depends on what |
| `state.rs` | The whiteboard — tracking each job's status |
| `executor.rs` | The cooks — things that actually run jobs |
| `scheduler.rs` | The head chef — orchestrates everything |

---

## Piece 1: The Recipe (`schema.rs`)

You write TOML like:
```toml
[[tasks]]
id = "scf_U{u}"
depends_on = ["preprocess"]
sweep.params = [{name = "u", values = [2, 3, 4]}]
```

The `{u}` is a placeholder. "Sweep" stamps out multiple copies with real values:
- **zip**: pairs values like a zipper — `(u=2, cutoff=500)`, `(u=3, cutoff=600)`
- **product**: every combination — `(u=2, cutoff=500)`, `(u=2, cutoff=700)`, ...

Result: a flat list of `ConcreteTask`s — fully filled in, no more placeholders.

---

## Piece 2: The Dependency Map (`pipeline.rs`)

Arranges tasks into a flowchart where arrows mean "must finish before":

```
preprocess
    ↓
scf_U2    scf_U3    scf_U4
    ↓         ↓         ↓
dos_U2    dos_U3    dos_U4
```

Checks at build time: no circular dependencies, no references to nonexistent tasks.

---

## Piece 3: The Progress Whiteboard (`state.rs`)

Every task moves through:
```
Pending → Ready → Submitted → Running → Completed
                                      ↘ Failed → (children) Skipped
```

State is saved to a SQLite file (`.workflow_state.db`) after every poll cycle. Crash and restart? It picks up exactly where it left off — completed tasks are never re-run.

---

## Piece 4: The Cooks (`executor.rs`)

An `Executor` has three jobs: `submit()`, `poll(handle)`, `cancel(handle)`.

**LocalExecutor** — runs a process directly on your machine. Handle is the PID. A background task writes a sentinel file (`.exit.<pid>`) with the exit code when the process finishes — so even after the OS recycles the PID, a later `poll()` can still find out what happened.

**SlurmExecutor** — talks to a supercomputer cluster via `sbatch` (submit), `squeue` (check), `scancel` (kill). Handle is the SLURM job ID.

The `ExecutorRegistry` is a plugin system — register factories by name (e.g. `"castep"`), and the scheduler looks up the right one per task.

---

## Piece 5: The Head Chef (`scheduler.rs`)

The main loop, every 100ms:

1. **Promote** — tasks whose dependencies are all done become `Ready`
2. **Submit** — submit `Ready` tasks (respecting a parallelism cap)
3. **Poll** — ask every running task "are you done yet?"
4. **Skip** — cascade `Failed` → `Skipped` to children
5. **Persist** — write state to SQLite
6. **Check** — if everything is in a terminal state, stop
7. **Wait** — sleep 100ms, or stop immediately on cancellation signal

---

## The async model

Runs on **Tokio** — like a single efficient waiter juggling many tables instead of one thread per job.

Key design: the scheduler loop is sequential. Complexity lives only at the edges:
- `submit()` launches a process and returns immediately (process runs in the OS)
- `poll()` just checks a file or runs a quick command — never blocks
- The only true concurrency is `LocalExecutor::submit()`, which spawns a background task to wait for the child process and write the sentinel file

**Cancellation** uses a `CancellationToken` — a shared flag. The sleep step uses `tokio::select!`: "wake me after 100ms, OR immediately if the cancel flag is set." On cancel, it calls `cancel()` on every running job before exiting.

---

## End-to-end flow

```
TOML file
   ↓ schema::expand_sweeps()
Vec<ConcreteTask>
   ↓ Pipeline::from_tasks()
Pipeline (DAG)
   ↓ Scheduler::run()
   ├── reads/writes StateDb (SQLite)
   ├── builds Executors via ExecutorRegistry
   └── polls every 100ms until all tasks settle
```

Each piece has one job and doesn't know about the others. The `Scheduler` is the only piece that holds them all together.
