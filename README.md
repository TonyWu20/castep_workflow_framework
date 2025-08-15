# castep_workflow_framework

A workflow framework for automatic `CASTEP` job pipelines, written in rust.

## Documentations

### Terms

- `seed_name`: The stem part of the filename without file extensions.
- `seed files`: The files required for `CASTEP` to run a calculation jobs. Files for a specific job share the same `<seed_name>`, but with different extensions. This is required by `CASTEP`.
  A `<seed_name>` may have suffix like `_DOS`, `_BandStr`, etc.. These are generally calculations of specific properties, and they're the child jobs of the `<seed_name>` only job. They usually should inherit the `<seed_name>.check` generated from the run of `<seed_name>`, to calculate the properties from the calculated/optimized solution of electron density.

### Expected outcome

In general, our crate has following features:

1. Allow users to define their own calculation pipelines including dependency relationship among the jobs, and then automatically execute jobs in the order respecting the dependency requirements as defined.
2. Automatic dependency handling as declared: In `CASTEP`, some types of calculations are expected to be continued on a check file `<seed_name>.check` produced by either `SinglePoint` or `GeometryOptimization`. For example: If the user declared `<seed_name>_DOS` is dependent on `<seed_name>`, we will duplicate the `<seed_name>.check` and rename to `<seed_name>_DOS.check`, then execute the call of `CASTEP` for `<seed_name>_DOS`. (It seems this could be handled by a separate crate, see point 2 in section "Prerequisites")
3. Monitor the job status, either by tracking the process ID in system processes, tracking the job ID in job schedulers (e.g., slurm, torque, openpbs), or checking keywords in the `CASTEP`'s output file `<seed_name>.castep`.
4. Build a directional acyclic graph (DAG) to demonstrate the calculation pipelines.
5. Provide interfaces for the consumer crates to define workflows in a declarative manner.
6. Support extensions by calling external scripts/programs or additional rust crates to achieve data pre/post-processing.

### Prerequisites

1. Users should prepare all the necessary files for `CASTEP` on their own.
2. The modifications and specific file management operations (e.g, make a directory to put the modified `seed files` and other necessary files, to be the next working directory) on existing `CASTEP` `seed files` are handled by the consumer's crate, and is out of this crate's responsibility.

### Example usage:

1. A user wants to test convergence with different cut-off energy settings. He provides a range of cut-off energies to test. Our crate takes the instructions, and orchestrate the runs in the order defined by the instructions.
2. A user wants to run `CASTEP` jobs on the same system, but with different settings of specific parameters. For example, he wants to change the input of Hubbard U value in a defined sequence.
3. A user wants to automatically calls an external script or program to analyze the results after each successfully completed run. Using our crate, he provides the path and necessary arguments to execute the program/script, and our crate will call it once the job successfully completed.
