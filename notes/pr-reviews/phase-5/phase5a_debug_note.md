# Debug note for PHASE-5A implementation

The test example shows these issues:

1. `sbatch` must be invoked at the working directory (resolved)
1. The `castep` command should be followed by `seed_name` instead of `task_id`
   (resolved by me)
1. I ran the example binary in a directory that only the local node has access.
   When the task was allocated to other nodes and failed because they didn't
   have access to the files, the binary marked the job completed incorrectly,
   though it found there was no `ZnO.castep` generated in the failed task's working
   directory.
1. Have to modify `example/hubbard_u_sweep_slurm/src/job_script.rs` to change
   the script settings about `nodelist`

I think only the issue 1 and 3 matter to the architectural design of our workflow
framework. Other issues are the layer-3 application level.
