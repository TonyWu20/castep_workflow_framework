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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SweepConfig;
    use clap::Parser;

    fn default_config() -> SweepConfig {
        SweepConfig::parse_from(["test"])
    }

    #[test]
    fn contains_sbatch_directives() {
        let config = default_config();
        let script = generate_job_script(&config, "scf_U1.0", "ZnO");
        assert!(script.contains("#SBATCH --job-name=\"scf_U1.0\""));
        assert!(script.contains("#SBATCH --partition=debug"));
        assert!(script.contains("#SBATCH --ntasks-per-node=16"));
        assert!(script.contains("#SBATCH --mem=30000m"));
    }

    #[test]
    fn contains_seed_name() {
        let config = default_config();
        let script = generate_job_script(&config, "scf_U0.0", "ZnO");
        assert!(script.contains("castep.mpi ZnO"));
    }

    #[test]
    fn no_literal_tabs() {
        let config = default_config();
        let script = generate_job_script(&config, "scf_U0.0", "ZnO");
        assert!(!script.contains('\t'), "job script should not contain literal tab characters");
    }

    #[test]
    fn starts_with_shebang() {
        let config = default_config();
        let script = generate_job_script(&config, "scf_U0.0", "ZnO");
        assert!(script.starts_with("#!/usr/bin/env bash"));
    }

    #[test]
    fn contains_nix_develop() {
        let config = default_config();
        let script = generate_job_script(&config, "scf_U0.0", "ZnO");
        assert!(script.contains("nix develop"));
        assert!(script.contains(&config.nix_flake));
    }

    #[test]
    fn contains_mpi_interface() {
        let config = default_config();
        let script = generate_job_script(&config, "scf_U0.0", "ZnO");
        assert!(script.contains(&format!("OMPI_MCA_btl_tcp_if_include={mpi_if}", mpi_if = config.mpi_if)));
    }
}
