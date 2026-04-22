use crate::config::SweepConfig;

pub fn generate_job_script(config: &SweepConfig, task_id: &str) -> String {
    format!(
        "#!/usr/bin/env bash\n\
#SBATCH --job-name=\"{task_id}\"\n\
#SBATCH --output=slurm_output_%j.txt\n\
#SBATCH --partition={partition}\n\
#SBATCH --nodes=1\n\
#SBATCH --ntasks-per-node={ntasks}\n\
#SBATCH --cpus-per-task=1\n\
#SBATCH --mem=30000m\n\
nix develop {nix_flake} --command bash -c \\\n\
    \"mpirun --mca plm slurm \\\n\
        -x OMPI_MCA_btl_tcp_if_include={mpi_if} \\\n\
        -x OMPI_MCA_orte_keep_fqdn_hostnames=true \\\n\
       --mca pmix s1 \\\n\
       --mca btl tcp,self \\\n\
\t--map-by numa --bind-to numa \\\n\
    castep.mpi {task_id}\"\n",
        task_id = task_id,
        partition = config.partition,
        ntasks = config.ntasks,
        nix_flake = config.nix_flake,
        mpi_if = config.mpi_if,
    )
}
