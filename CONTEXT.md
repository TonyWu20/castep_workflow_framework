# castep_workflow_framework Context

A Rust workflow framework for defining, executing, and monitoring directed-acyclic computation pipelines as a set of tasks with dependency resolution. Designed to be software-agnostic (DFT-code-agnostic) in the core engine, with CASTEP DFT+U workflows as the initial and primary use case.

## Language

### Core Concepts

**Workflow**:
A named DAG of Tasks with orchestration state. The top-level container.
_Avoid_: Pipeline, Job Chain, run

**Task**:
The atomic unit of work in a Workflow. Has an id, dependencies, workdir, ExecutionMode, and three lifecycle closures (setup, collect, monitors).
_Avoid_: Job, Step, Node, Process

**Direct**:
Local process execution mode: spawns a process via `std::process::Command` using `SystemProcessRunner`.
_Avoid_: Local, Interactive

**Queued**:
HPC batch scheduler execution mode: submits via `sbatch` (SLURM) or `qsub` (PBS) using `QueuedRunner`.
_Avoid_: Remote, Batch

### Task Lifecycle

**Setup**:
A closure run before task execution. Prepares working directory and input files. Receives the task's `workdir` as argument.

**Collect**:
A closure run after successful task execution. Extracts results from outputs. Failure behavior configurable via `CollectFailurePolicy` (FailTask or WarnOnly).

**Hook**:
A configurable lifecycle event handler attached to a Task. Has a name, command, and HookTrigger. Executed by a HookExecutor.
_Avoid_: Callback, EventHandler, Trigger, Script

**HookTrigger**:
The event that fires a Hook. Variants: OnStart, OnComplete, OnFailure, Periodic { interval_secs }.
_Avoid_: Event type, Trigger kind

### Execution State

**State**:
The persistence layer for workflow execution. Trait `StateStore` with concrete implementation `JsonStateStore`. Concern: durability, load/save, crash recovery.
_Avoid_: Status, Store, Database

**Status**:
The per-Task lifecycle value. Enum `TaskStatus`: Pending, Running, Completed, Failed, Skipped, SkippedDueToDependencyFailure. Concern: task progression rules.

### Architecture Layers

**Layer 1 (Foundation)**: `workflow_core` crate. DAG engine, Task/Workflow types, trait definitions (ProcessRunner, HookExecutor, StateStore), WorkflowError, signal handling. Dependency-light, no OS-specific code in tests.

**Layer 2 (Utilities)**: `workflow_utils` crate. Concrete implementations (SystemProcessRunner, ShellHookExecutor, QueuedRunner), generic file I/O, `run_default()` convenience. No software-specific code. No adapter traits.

**Layer 3 (Project)**: Workspace member binaries (hubbard_u_sweep, multi_param_sweep, scf_dos_chain, workflow-cli). Domain logic using parser libraries (castep-cell-io, etc.).

## Relationships

- A **Workflow** contains one or more **Tasks**
- A **Task** depends on zero or more other **Tasks** (forming a DAG)
- A **Task** has exactly one **ExecutionMode** (Direct or Queued)
- A **Task** may have a **Setup** closure and/or a **Collect** closure
- A **Task** may have zero or more **Hooks** attached via **HookTrigger**s
- A **Task** has exactly one **Status** at any point in time
- A **Workflow** has exactly one **State** at rest (persisted to JSON)

## Example dialogue

> **Dev:** If a **Task** with a failed dependency is **SkippedDueToDependencyFailure**, and I fix the upstream **Task** and call `cmd_retry`, does the downstream get reset to **Pending**?
> **Domain expert:** Yes. The **State**'s successor graph tracks downstream dependencies. Retry resets the failed task AND all transitively downstream **SkippedDueToDependencyFailure** tasks to **Pending**, so the full chain re-runs on the next workflow execution.

## Flagged ambiguities

- **TaskPhase vs TaskStatus**: `TaskPhase` (Running, Completed, Failed) is a simplified 3-state view used in `HookContext` for hook triggering. `TaskStatus` (Pending, Running, Completed, Failed, Skipped, SkippedDueToDependencyFailure) is the full 6-variant state machine for workflow orchestration. They overlap on Running/Completed/Failed but serve different audiences. Documented but not yet resolved -- the separation is intentional and functional.

## Coding Patterns

### Mandatory conventions

**SRP (Single Responsibility Principle)**:
Every module, struct, and function has exactly one reason to change. Modular architecture strictly enforced.

**Builder pattern via `bon` crate**:
Use `#[derive(bon::Builder)]` for ALL non-trivial structs. `bon` provides ergonomic builders with method chaining and automatic collection helpers.
_Avoid_: Manual builder impls, massive constructors

**Functional programming style**:
- Prefer iterators (`iter()`, `map`, `filter`, `filter_map`, `flat_map`, `fold`, `collect`) over imperative `for` loops
- Minimize mutable state; favor immutable transformations and method chaining
- Use higher-order functions and combinators where they improve clarity
_Avoid_: Imperative for-loops, nested mutable state, index-based iteration

**Semantic newtypes**:
Use semantic newtype wrappers to distinguish common types (e.g., `f64`) in different usage contexts. Nested structures use wrappers to encode relationships (e.g., `Vec<Vec<Vec<f64>>>` -> `Kpoint<Spin<Eigenvalue<f64>>>`).
_Avoid_: Raw primitive types in domain-critical positions

**Error handling**:
`thiserror` with `#[non_exhaustive]` enums in library crates. `anyhow` in binaries only.

**Trait objects**:
`Send + Sync` bounds on all trait objects (`ProcessRunner`, `ProcessHandle`, `HookExecutor`, `QueuedSubmitter`, `StateStore`).

**Documentation**:
Rustdoc on all public API items. Doc examples encouraged. Best-effort (no CI enforcement currently).

**Import style**:
Grouped: std first, then external crates, then local (`self`, `super`, `crate`).

**Unit tests**:
Inline in source files (`#[cfg(test)] mod tests`). Use `tempfile::tempdir()` for filesystem-backed tests.

**Integration tests**:
Required for CLI and plugin entry points. Go in `tests/` directories.

**State persistence**:
Atomic writes via temp file + rename pattern.

**`#[must_use]`**:
On all accessor methods that return values and could be silently ignored.
