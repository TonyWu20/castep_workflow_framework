use crate::config::SweepConfig;

pub fn generate_job_script(config: &SweepConfig, task_id: &str, seed_name: &str) -> String {
    format!(
        "\
#!/usr/bin/env bash
#SBATCH --job-name=\"{task_id}\"
#SBATCH --output=slurm_output_%j.txt
#SBATCH --partition={partition}
#SBATCH --nodes=1
#SBATCH --ntasks-per-node={ntasks}
#SBATCH --cpus-per-task=1
#SBATCH --mem=30000m
#SBATCH --nodelist=nixos
nix develop {nix_flake} --command bash -c \\
    \"mpirun --mca plm slurm \\
        -x OMPI_MCA_btl_tcp_if_include={mpi_if} \\
        -x OMPI_MCA_orte_keep_fqdn_hostnames=true \\
        --mca pmix s1 \\
        --mca btl tcp,self \\
        --map-by numa --bind-to numa \\
    castep.mpi {seed_name}\"
",
        task_id = task_id,
        partition = config.partition,
        ntasks = config.ntasks,
        nix_flake = config.nix_flake,
        mpi_if = config.mpi_if,
    )
}
