use std::path::PathBuf;

use clap::Parser;

/// NiO SinglePoint test seed -- framework job management plus SCF stall
/// watcher. The seed files come from ~/castep_jobs/NiO_SinglePoint_finer_grid
/// (pseudopotentials are copied from --seed-dir at setup time).
#[derive(Parser, Debug, Clone)]
#[command(name = "nio_single_point")]
pub struct NiOConfig {
    /// CASTEP seed name (the .cell/.param/.castep file stem).
    #[arg(long, default_value = "NiO")]
    pub seed_name: String,

    /// Directory holding the real seed files. Pseudopotential files from
    /// the cell's SPECIES_POT block are copied from here into the
    /// task workdir. Required for execution (not for --dry-run).
    #[arg(long, env = "CASTEP_NIO_SEED_DIR")]
    pub seed_dir: Option<PathBuf>,

    /// Submit to SLURM instead of running locally.
    #[arg(long)]
    pub slurm: bool,

    /// Print the topological order and exit without executing.
    #[arg(long)]
    pub dry_run: bool,

    /// CASTEP binary (local mode).
    #[arg(long, default_value = "castep.mpi")]
    pub castep_command: String,

    /// MPI ranks for the local run; values above 1 wrap the command in
    /// `mpirun -np N`.
    #[arg(long, default_value_t = 1)]
    pub mpi_np: u32,

    /// Watcher poll interval in seconds (periodic MonitoringHook).
    #[arg(long, default_value_t = 30)]
    pub watch_interval: u64,

    /// Let the watcher terminate a stalled local run via
    /// `pkill -f <castep binary> <seed>`.
    #[arg(long)]
    pub watch_kill: bool,

    /// Watcher stall threshold multiplier on the energy tolerance.
    #[arg(long, default_value_t = 3)]
    pub watch_tol_mult: u32,

    /// Max SCF cycles per BFGS block before the watcher flags a stall.
    #[arg(long, default_value_t = 500)]
    pub watch_max_scf: u32,

    /// Root directory for run directories.
    #[arg(long, default_value = "runs")]
    pub workdir: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn defaults() {
        let config = NiOConfig::parse_from(["test"]);
        assert_eq!(config.seed_name, "NiO");
        assert!(!config.slurm);
        assert!(!config.dry_run);
        assert_eq!(config.castep_command, "castep.mpi");
        assert_eq!(config.mpi_np, 1);
        assert_eq!(config.watch_interval, 30);
        assert!(!config.watch_kill);
        assert_eq!(config.watch_tol_mult, 3);
        assert_eq!(config.watch_max_scf, 500);
        assert!(config.seed_dir.is_none());
    }

    #[test]
    fn slurm_flag_parses() {
        let config = NiOConfig::parse_from(["test", "--slurm"]);
        assert!(config.slurm);
    }
}
