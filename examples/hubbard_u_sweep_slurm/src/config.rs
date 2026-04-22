use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "hubbard_u_sweep_slurm")]
pub struct SweepConfig {
    /// SLURM account name
    #[arg(long, env = "CASTEP_SLURM_ACCOUNT")]
    pub account: String,

    /// SLURM partition
    #[arg(long, env = "CASTEP_SLURM_PARTITION", default_value = "standard")]
    pub partition: String,

    /// Number of MPI tasks (cores) per job
    #[arg(long, default_value_t = 16)]
    pub ntasks: u32,

    /// Walltime per job (HH:MM:SS)
    #[arg(long, default_value = "01:00:00")]
    pub walltime: String,

    /// Module load commands, comma-separated (e.g. "castep/24.1,intel/2024")
    #[arg(long, env = "CASTEP_MODULES", value_delimiter = ',')]
    pub modules: Vec<String>,

    /// CASTEP executable command (e.g. "castep.mpi" or "mpirun -np 16 castep.mpi")
    #[arg(long, env = "CASTEP_COMMAND", default_value = "castep.mpi")]
    pub castep_command: String,

    /// Seed name (CASTEP input file prefix, without extension)
    #[arg(long, default_value = "ZnO")]
    pub seed_name: String,

    /// U values to sweep, comma-separated (eV)
    #[arg(long, default_value = "0.0,1.0,2.0,3.0,4.0,5.0")]
    pub u_values: String,

    /// Maximum number of concurrent SLURM jobs
    #[arg(long, default_value_t = 4)]
    pub max_parallel: usize,

    /// Element to apply Hubbard U to
    #[arg(long, default_value = "Zn")]
    pub element: String,

    /// Orbital for Hubbard U: 'd' or 'f'
    #[arg(long, default_value = "d")]
    pub orbital: char,

    /// Dry-run mode: print topological order and exit without submitting
    #[arg(long)]
    pub dry_run: bool,
}

impl SweepConfig {
    pub fn parse_u_values(&self) -> Vec<f64> {
        self.u_values
            .split(',')
            .filter_map(|s| s.trim().parse::<f64>().ok())
            .collect()
    }

    pub fn module_load_lines(&self) -> String {
        self.modules
            .iter()
            .map(|m| format!("module load {}", m))
            .collect::<Vec<_>>()
            .join("\n")
    }
}