use crate::config::NiOConfig;

/// Generate the SLURM job script for a queued NiO SinglePoint run.
///
/// `#SBATCH --time` carries the 4-day hard cap from the CeO2 stall
/// diagnosis: a healthy BFGS/SCF block converges in under 200 cycles,
/// so a stalled run must not run unbounded.
pub fn generate_job_script(config: &NiOConfig, task_id: &str, seed_name: &str) -> String {
    let ntasks = config.mpi_np.max(1);
    format!(
        "\
#!/usr/bin/env bash
#SBATCH --job-name=\"{task_id}\"
#SBATCH --output=slurm_output_%j.txt
#SBATCH --partition=debug
#SBATCH --nodes=1
#SBATCH --ntasks-per-node={ntasks}
#SBATCH --cpus-per-task=1
#SBATCH --mem=30000m
#SBATCH --time=96:00:00
castep.mpi {seed_name}
",
        task_id = task_id,
        ntasks = ntasks,
        seed_name = seed_name,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::NiOConfig;
    use clap::Parser;

    fn default_config() -> NiOConfig {
        NiOConfig::parse_from(["test"])
    }

    #[test]
    fn contains_sbatch_directives() {
        let config = default_config();
        let script = generate_job_script(&config, "nio", "NiO");
        assert!(script.contains(r#"#SBATCH --job-name="nio""#));
        assert!(script.contains("#SBATCH --time=96:00:00"));
    }

    #[test]
    fn contains_seed_name() {
        let config = default_config();
        let script = generate_job_script(&config, "nio", "NiO");
        assert!(script.contains("castep.mpi NiO"));
    }

    #[test]
    fn no_literal_tabs() {
        let config = default_config();
        let script = generate_job_script(&config, "nio", "NiO");
        assert!(!script.contains('\t'));
    }

    #[test]
    fn starts_with_shebang() {
        let config = default_config();
        let script = generate_job_script(&config, "nio", "NiO");
        assert!(script.starts_with("#!/usr/bin/env bash"));
    }
}
