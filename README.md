# Refined Software Requirements Specification

## castep_workflow_framework

A workflow framework for automatic `CASTEP` job pipelines, written in rust.

## Core Architecture Principle

**The DAG is the central orchestrator** for all workflow operations, including job execution, dependency management, file transformations, and status monitoring.

## Requirements Specification

### Terms

- `seed_name`: The stem part of the filename without file extensions. This is a user-provided label that may not be globally unique (e.g., multiple jobs in a parameter sweep might share a base `seed_name`).
- `JobId`: A UUID-based identifier that serves as the true unique identifier for jobs within the workflow DAG. Generated automatically by the framework.
- `seed files`: The files required for `CASTEP` to run a calculation job. This includes:
  - Core CASTEP files: `.cell`, `.param`
  - Pseudopotential files: `.usp`
  - Other required files like `.recip` (if any)
  - Note: `.check` file is handled separately as continuation file

### Expected outcome

In general, our crate has the following features:

1. Allow users to define their own calculation pipelines including dependency relationships among jobs, and automatically execute jobs in the order respecting the dependency requirements as defined.
2. Automatic dependency handling as declared: In `CASTEP`, some types of calculations continue from a `.check` file produced by either `SinglePoint` or `GeometryOptimization`. For example, if a user declares `<seed_name>_DOS` as dependent on `<seed_name>`, the framework will:
   - Copy only the `<parent_seed_name>.check` file from the parent job's working directory
   - Place it in the child job's working directory as `<child_seed_name>.check`
   - Execute the `CASTEP` call for the child job
3. Monitor job status through:
   - Tracking process IDs in system processes (for local execution)
   - Tracking job IDs in job schedulers (e.g., SLURM, Torque, OpenPBS)
   - Checking keywords in the `CASTEP` output file `<seed_name>.castep`
4. Build a directional acyclic graph (DAG) to represent the calculation pipeline, using topological sorting to determine execution order.
5. Provide simple interfaces for consumer crates to define workflows.
6. Support extensions by calling external scripts/programs for data pre/post-processing.
7. Implement graceful shutdown: When terminated, the framework cancels all running jobs through appropriate mechanisms (e.g., `SIGTERM` for local jobs, `scancel` for SLURM jobs).

### Prerequisites

1. Users should prepare all necessary seed files (`.cell`, `.param`, `.usp`, etc.) for `CASTEP` on their own.
2. The framework handles:
   - Copying of `.check` files for dependency resolution
   - Creation of working directory hierarchy for parent/child jobs
   - Job monitoring and execution sequencing
   - File transformations between parent and child jobs
3. Consumers are responsible for:
   - Providing the base working directory for each job
   - Ensuring seed files exist in the appropriate working directories
   - Defining the job dependency relationships
   - Implementing user-defined file transformations

### Working Directory Structure

The framework follows a hierarchical directory structure for job organization:

- Parent jobs define a base working directory (e.g., `./jobs/my_calculation`)
- Child jobs are placed in subdirectories of their parent's working directory (e.g., `./jobs/my_calculation/DOS`)
- The framework automatically creates these subdirectories when setting up dependent jobs
- If a directory already exists, it will be reused (not cleared or failed)

### Job Identification Model

To resolve the conflict between user-friendly naming and unique identification:

- Each job has a `JobId` (UUID) that serves as its true unique identifier within the DAG
- Each job has a `seed_name` that serves as a user-provided label (may not be globally unique)
- Dependency relationships are defined using `JobId` references via the `continuation_from` field
- This enables parameter sweeps where multiple jobs share a similar base `seed_name` but have different parameters

### DAG-Centric Architecture

The DAG structure is the central component for workflow orchestration:

1. **Centralized Execution**: The DAG orchestrates the complete workflow execution
2. **Automatic Dependency Management**: Jobs execute only when dependencies complete successfully
3. **Built-in File Transformations**: DAG applies transformations between parent-child relationships
4. **Real-time Status Monitoring**: Async status updates through watch channels
5. **Error Propagation**: Failed jobs prevent dependent jobs from starting

### User-Defined File Transformations

The framework supports user-defined operations on seed files:

1. **Transformation Interface**: Users implement transformations that receive:

   - Parent job's seed directory path and seed name
   - Child job's working directory
   - Return modified file contents as `HashMap<PathBuf, Vec<u8>>`

2. **Lazy Evaluation**: Transformations are applied only when child jobs are ready to execute

3. **Default Behavior**: If no user transformation is defined, the framework provides sensible defaults:

   - Copy `.check` file from parent to child (if continuation is required)
   - Rename `.check` file to match child's seed name

4. **File Handling Process**:
   - Framework copies all required seed files from parent to child directory
   - User transformation overwrites specific files with modified content
   - Framework writes the final files to child's working directory

### Modular Execution Architecture

The framework uses a modular backend system with tightly-coupled runner-monitor pairs:

1. **Runner-Monitor Pairs**: Each execution environment has an integrated runner and monitor

   - Local execution: Direct process execution + PID monitoring
   - SLURM: `sbatch` submission + `squeue` monitoring
   - PBS/Torque: `qsub` submission + `qstat` monitoring
   - Others as needed

2. **Pluggable Backends**: Consumers choose execution backends independently
3. **Unified Interface**: All backends implement common traits for seamless integration
4. **Execution Context**: Runner and monitor share execution context and job handles

### Consumer Interface

Consumers interact primarily with the DAG through a simple interface:

```rust
// Define jobs and dependencies with transformations
let mut workflow = Dag::new();
workflow.add_job(parent_job);
workflow.add_job(child_job);

// Configure execution backend
let backend = LocalBackend::new(); // or SlurmBackend::new(), etc.

// Execute entire workflow with monitoring
let job_handles = workflow.execute_with_monitoring(&backend).await?;

// Monitor job statuses
let results = Dag::monitor_jobs(job_handles).await?;
```

### Async Strategy

The framework employs a precise async/sync boundary:

1. **Job Execution**: Async execution with background tasks
2. **Job Monitoring**: Real-time status updates via async channels
3. **File Operations**: Async file operations for better I/O performance
4. **Graceful Shutdown**: Signal handler with async cancellation

### Configuration

The framework provides configurable parameters:

- **CASTEP command**: Configurable command path/name to support different CASTEP binaries
- **Polling interval**: Global setting for job status monitoring (default: 5 seconds)
- **Shutdown timeout**: Time to wait before force-killing local processes (fixed: 5 seconds)

### Error Handling Strategy

When a job fails:

- Dependent jobs are automatically cancelled
- Other independent branches of the DAG continue execution
- No automatic retry mechanism for failed jobs
- Comprehensive error reporting through DAG

### Hook System

The framework supports external command hooks for pre/post-processing:

- **Pre-run hooks**: Execute before job starts
  - Failure fails the entire workflow
- **Post-run hooks**: Execute after successful job completion
  - Failure logs error and continues workflow
- Hooks run synchronously within each job's execution context
- Hooks execute immediately after job completion

### Job Scheduler Integration

For HPC environments, the framework supports job schedulers through integrated runner-monitor pairs:

- Users provide complete scheduler submission scripts with all parameters
- Framework executes submission commands:
  - SLURM: `sbatch`
  - Torque/OpenPBS: `qsub`
  - Others as needed
- Framework monitors jobs using scheduler commands:
  - SLURM: `squeue`
  - Torque/OpenPBS: `qstat`
- All scheduler-specific parameters are handled in user-provided scripts

### Example usage:

1. A user wants to test convergence with different cut-off energy settings. He provides a range of cut-off energies to test. Our crate takes the instructions, creates appropriately named jobs with UUID identifiers, and orchestrates the runs in the defined order, automatically copying `.check` files between dependent jobs.

2. A user wants to run `CASTEP` jobs on the same system but with different settings of specific parameters. For example, he wants to change the input of Hubbard U value in a defined sequence. The framework handles this through the UUID-based job identification, allowing multiple jobs with similar `seed_name` patterns.

3. A user wants to automatically call an external script or program to analyze results after each successfully completed run. Using our crate, he provides the path and necessary arguments to execute the program/script, and our crate will synchronously call it once the job successfully completes.

## Design style

1. **Functional programming style**:

   - Use iterators and functional composition as much as possible
   - Minimize usage of `mut Vec` and `push` operations
   - Compose functions to achieve purpose
   - Use unit structs, marker traits and generic bounds to implement finite state machines
   - **Use `petgraph::GraphMap` for efficient DAG representation**

2. **Type safety**:

   - Use newtype pattern to finely control and validate input parameters
   - Implement `Deref` where appropriate to improve newtypes' ergonomics
   - Introduce `JobId` as a UUID-based newtype for unambiguous job identification

3. **Builder pattern**:

   - Use the `bon` crate to implement builder pattern for complex structs
   - Improve code readability and self-explanatory ability
   - Enable partial construction with validation at build time

4. **Modular architecture**:

   - Multi-crate approach with clear boundaries:
     - `castep-workflow-core`: Core types and DAG structure
     - `castep-workflow-execution`: Runner-monitor backend pairs
     - `castep-workflow`: High-level orchestration
   - Runner-monitor pairs as atomic units of functionality
   - Clear separation between orchestration and execution

5. **Async-first design**:

   - Use `tokio` as the async runtime
   - Implement async execution with background tasks
   - Provide real-time status monitoring via async channels

6. **Resource management**:

   - Delegate computing resource management to job schedulers on HPC clusters
   - Implement simple resource management for local execution

7. **Immutability benefits**:

   - Immutable DAG design prevents cycles automatically
   - Eliminates need for expensive cycle detection during job addition
   - Clear data flow and predictable behavior

8. **Extensible transformation system**:

   - Declarative file transformations between jobs
   - Support for custom external scripts
   - Easy to extend with new transformation types
   - User-defined transformations with functional programming principles

9. **Backend extensibility**:
   - Runner-monitor pairs as pluggable backends
   - Common traits for unified interface
   - Easy to add new execution environments
   - Consumer choice of execution backends
