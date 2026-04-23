use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "hubbard_u_sweep_slurm")]
pub struct SweepConfig {
    /// SLURM partition
    #[arg(long, env = "CASTEP_SLURM_PARTITION", default_value = "debug")]
    pub partition: String,

    /// Number of MPI tasks (cores) per job
    #[arg(long, default_value_t = 16)]
    pub ntasks: u32,

    /// Nix flake URI for the CASTEP environment
    #[arg(
        long,
        env = "CASTEP_NIX_FLAKE",
        default_value = "git+ssh://git@github.com/TonyWu20/CASTEP-25.12-nixos#castep_25_mkl"
    )]
    pub nix_flake: String,

    /// Network interface for OpenMPI TCP (e.g. "enp6s0")
    #[arg(long, env = "CASTEP_MPI_IF", default_value = "enp6s0")]
    pub mpi_if: String,

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

    /// Run tasks locally via direct process execution instead of SLURM
    #[arg(long)]
    pub local: bool,

    /// CASTEP binary name or path (used in --local mode)
    #[arg(long, default_value = "castep")]
    pub castep_command: String,
}

/// Parses a comma-separated string of f64 values.
///
/// Each segment is trimmed before parsing.
/// Returns an error string identifying the offending token on failure.
pub fn parse_u_values(s: &str) -> Result<Vec<f64>, String> {
    s.split(',')
        .map(|segment| {
            let trimmed = segment.trim();
            trimmed
                .parse::<f64>()
                .map_err(|e| format!("invalid U value '{}': {}", trimmed, e))
        })
        .collect::<Result<Vec<_>, _>>()
}

impl SweepConfig {
    pub fn parse_u_values(&self) -> Result<Vec<f64>, String> {
        parse_u_values(&self.u_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_values() {
        let vals = parse_u_values("0.0,1.0,2.0").unwrap();
        assert_eq!(vals, vec![0.0, 1.0, 2.0]);
    }

    #[test]
    fn parse_with_whitespace() {
        let vals = parse_u_values("  0.0 , 1.0 , 2.0  ").unwrap();
        assert_eq!(vals, vec![0.0, 1.0, 2.0]);
    }

    #[test]
    fn parse_single_value() {
        let vals = parse_u_values("3.14").unwrap();
        assert_eq!(vals, vec![3.14]);
    }

    #[test]
    fn parse_invalid_token() {
        let err = parse_u_values("1.0,abc,2.0").unwrap_err();
        assert!(err.contains("abc"), "error should mention the invalid token: {}", err);
    }

    #[test]
    fn parse_empty_token() {
        let err = parse_u_values("1.0,,2.0").unwrap_err();
        assert!(err.contains("invalid"), "error should report parse failure: {}", err);
    }
}
