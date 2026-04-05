# castep_workflow_framework

A workflow framework for automatic `CASTEP` job pipelines, written in rust.

## Motive of this project

The bash scripts in `./castep_auto_hubbard/` were a previous project in our
research group. My professor wants to run a series of `CASTEP` calculations with
a range of `Hubbard U`/`Hubbard Alpha` values to experiment how to improve the
`DFT+U` method's accuracy.

The scripts were designed to handle:

1. User inputs to start the defined calculation jobs.
2. Modifications of `.cell` and `.param` files accordingly, and file system
   operations to generate new job folders grouped by the changed parameter
   `Hubbard U`/`Hubbard Alpha`.
3. Collect the target data value from `CASTEP`'s output, `.castep` file, and
   save them into a csv table.
4. Commands to run and kill the jobs on the computing node. It offers parallel
   and serial mode.

Writing and maintaining Bash scripts is a nightmare. I don't want to repeat the
process in the future when we're doing some high throughput DFT calculation
projects. So I want to abstract the project into a workflow framework that can
be reused and maintained systematically.

## Expected outcome

In general, our crate has the following target features:

1. Allow users to define their own calculation pipelines including dependency relationships among jobs, and automatically execute jobs in the order respecting the dependency requirements as defined.
   - example: The user would like to call `castep <seed_name>`, `castep <seed_name>_DOS`, `castep <seed_name>_EField` sequentially to complete jobs
     in the same folder.
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
   - crate dependencies: `castep_cell_io-0.3.0` is my crate designed to parse/create/modify/save `CASTEP`'s `.cell` and `.param` files.
   - example: A user wants to experiment the effect of using different energy cutoff values to convergence of the calculation. He decides to use a range of `(500, 900)` with an increment of 100. This crate will use `castep_cell_io-0.3.0` to update the `CutOffEnergy` accordingly to create corresponding `.param` files.
6. Support extensions by calling external scripts/programs for data pre/post-processing.
7. Implement graceful shutdown: When terminated, the framework cancels all running jobs through appropriate mechanisms (e.g., `SIGTERM` for local jobs, `scancel` for SLURM jobs).
