# Refined Software Requirements Specification

## castep_workflow_framework

A workflow framework for automatic `CASTEP` job pipelines, written in rust.

## Requirements Specification

### Terms

- `seed_name`: The stem part of the filename without file extensions. This is a user-provided label that may not be globally unique (e.g., multiple jobs in a parameter sweep might share a base `seed_name`).
- `JobId`: A UUID-based identifier that serves as the true unique identifier for jobs within the workflow DAG. Generated automatically by the framework.
- `seed files`: The files required for `CASTEP` to run a calculation job. Files for a specific job share the same `<seed_name>`, but with different extensions. This is required by `CASTEP`.
  - A `<seed_name>` may have suffixes like `_DOS`, `_BandStr`, etc. These are generally calculations of specific properties, and they're child jobs of the base `<seed_name>` job. They usually should inherit the `<seed_name>.check` generated from the run of the base job to calculate properties from the calculated/optimized electron density solution.

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
5. Provide interfaces for consumer crates to define workflows in a declarative manner.
6. Support extensions by calling external scripts/programs for data pre/post-processing.
7. Implement graceful shutdown: When terminated, the framework cancels all running jobs through appropriate mechanisms (e.g., `SIGTERM` for local jobs, `scancel` for SLURM jobs).

### Prerequisites

1. Users should prepare all necessary seed files (`.cell`, `.param`, etc.) for `CASTEP` on their own.
2. The framework handles:
   - Copying of `.check` files for dependency resolution
   - Creation of working directory hierarchy for parent/child jobs
   - Job monitoring and execution sequencing
3. Consumers are responsible for:
   - Providing the base working directory for each job
   - Ensuring seed files exist in the appropriate working directories
   - Defining the job dependency relationships

### Working Directory Structure

The framework follows a hierarchical directory structure for job organization:

- Parent jobs define a base working directory (e.g., `./jobs/my_calculation`)
- Child jobs are placed in subdirectories of their parent's working directory (e.g., `./jobs/my_calculation/DOS`)
- The framework automatically creates these subdirectories when setting up dependent jobs
- Only the `.check` file is copied between jobs for continuation purposes
- If a directory already exists, it will be reused (not cleared or failed)

### Job Identification Model

To resolve the conflict between user-friendly naming and unique identification:

- Each job has a `JobId` (UUID) that serves as its true unique identifier within the DAG
- Each job has a `seed_name` that serves as a user-provided label (may not be globally unique)
- Dependency relationships are defined using `JobId` references via the `continuation_from` field
- This enables parameter sweeps where multiple jobs share a similar base `seed_name` but have different parameters

### Async Strategy

The framework employs a precise async/sync boundary:

1. **Job Submission**: Synchronous execution

   - Blocking call to `castep` or job scheduler submission commands
   - Short-lived operation (seconds), no async required
   - Returns a process/job handle for monitoring

2. **Job Monitoring**: Async polling

   - Non-blocking status checks at regular intervals (default: 5 seconds)
   - Implemented with `tokio::time::interval` for efficient resource usage
   - Cancellable futures for graceful shutdown

3. **Graceful Shutdown**
   - Signal handler captures termination signals (SIGINT, SIGTERM)
   - Cancels all running jobs through appropriate mechanisms:
     - Local jobs: `SIGTERM` → `SIGKILL` after 5 seconds timeout
     - SLURM jobs: `scancel` command
   - Ensures no orphaned processes remain after termination

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

### Hook System

The framework supports external command hooks for pre/post-processing:

- **Pre-run hooks**: Execute before job starts
  - Failure fails the entire workflow
- **Post-run hooks**: Execute after successful job completion
  - Failure logs error and continues workflow
- Hooks run synchronously to maintain clear execution order
- Hooks execute immediately after job completion (before next job starts)

### Job Scheduler Integration

For HPC environments, the framework supports job schedulers:

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
   - **Use `petgraph::from_elements` for fully functional DAG construction**

2. **Type safety**:

   - Use newtype pattern to finely control and validate input parameters
   - Implement `Deref` where appropriate to improve newtypes' ergonomics
   - Introduce `JobId` as a UUID-based newtype for unambiguous job identification

3. **Builder pattern**:

   - Use the `bon` crate to implement builder pattern for complex structs
   - Improve code readability and self-explanatory ability
   - Enable partial construction with validation at build time

4. **Modular architecture**:

   - Separate responsibilities into focused modules:
     - `core`: Job definitions, DAG construction
     - `runners`: Execution backends (local, SLURM, Torque)
     - `hooks`: Pre/post-processing hooks
     - `monitor`: Job status monitoring
   - Reduce overall building time through careful module separation

5. **Hexagonal architecture**:

   - Implement dependency injection through traits
   - Define `JobRunner` trait for execution backends
   - Define `JobMonitor` trait for status monitoring
   - Enable comprehensive unit testing with mock implementations

6. **Async considerations**:

   - Confine async operations to job monitoring only
   - Keep submission and file operations synchronous
   - Implement cancellable monitoring futures
   - Use `tokio` as the async runtime

7. **Resource management**:

   - Delegate computing resource management to job schedulers on HPC clusters
   - Implement simple resource management for local execution (lower priority)

8. **Hook system**:
   - Implement pre/post-processing through external command execution only
   - No support for Rust function hooks (avoid FFI complexity)
   - Synchronous execution of hooks to maintain clear execution order
