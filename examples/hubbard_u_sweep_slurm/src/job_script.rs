use crate::config::SweepConfig;

pub fn generate_job_script(config: &SweepConfig, task_id: &str) -> String {
    format!(
        "#!/bin/bash\n\
         #SBATCH --job-name={task_id}\n\
         #SBATCH --account={account}\n\
         #SBATCH --partition={partition}\n\
         #SBATCH --ntasks={ntasks}\n\
         #SBATCH --time={walltime}\n\
         \n\
         {modules}\n\
         \n\
         {command} {seed}\n",
        task_id = task_id,
        account = config.account,
        partition = config.partition,
        ntasks = config.ntasks,
        walltime = config.walltime,
        modules = config.module_load_lines(),
        command = config.castep_command,
        seed = config.seed_name,
    )
}
