// ChainConfig and parsing functions for scf_dos_chain binary.

use anyhow::{anyhow, Result};
use clap::Parser;

/// Configuration for CASTEP SCF + DOS chain workflow.
#[derive(Parser, Debug)]
#[command(name = "scf_dos_chain")]
pub struct ChainConfig {
    /// Hubbard U value.
    #[arg(long, default_value_t = 3.0)]
    pub u_value: f64,

    /// Element symbol for the Hubbard site.
    #[arg(long, default_value = "Zn")]
    pub element: String,

    /// Orbital label for the Hubbard manifold.
    #[arg(long, default_value = "d")]
    pub orbital: char,

    /// CASTEP seed name for input/output files.
    #[arg(long, default_value = "ZnO")]
    pub seed_name: String,

    /// Maximum number of parallel jobs.
    #[arg(long, default_value_t = 1)]
    pub max_parallel: usize,

    /// Run locally instead of submitting to Slurm.
    #[arg(long)]
    pub local: bool,

    /// Print what would be done without executing.
    #[arg(long)]
    pub dry_run: bool,

    /// CASTEP binary command.
    #[arg(long, default_value = "castep")]
    pub castep_command: String,

    /// Working directory for CASTEP runs.
    #[arg(long, default_value = ".")]
    pub workdir: String,

    /// Slurm partition name.
    #[arg(long, env = "CASTEP_SLURM_PARTITION", default_value = "debug")]
    pub partition: String,

    /// Number of MPI tasks per job.
    #[arg(long, default_value_t = 16)]
    pub ntasks: u32,

    /// Nix flake URL for the CASTEP environment.
    #[arg(
        long,
        env = "CASTEP_NIX_FLAKE",
        default_value = "git+ssh://git@github.com/TonyWu20/CASTEP-25.12-nixos#castep_25_mkl"
    )]
    pub nix_flake: String,

    /// Network interface for MPI communication.
    #[arg(long, env = "CASTEP_MPI_IF", default_value = "enp6s0")]
    pub mpi_if: String,
}

/// Parse a comma-separated list of Hubbard U values into a vector of f64.
///
/// Each segment is trimmed of whitespace.  Empty input produces an error
/// containing "empty".  Consecutive commas or non-numeric tokens produce an
/// error containing the offending token.
pub fn parse_u_values(s: &str) -> Result<Vec<f64>> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("empty u values input"));
    }
    trimmed
        .split(',')
        .map(|token| {
            let t = token.trim();
            t.parse::<f64>()
                .map_err(|_| anyhow!("invalid u value: '{}'", t))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    /// Helper: default config for job_script tests
    fn default_chain_config() -> ChainConfig {
        ChainConfig::parse_from(["test"])
    }

    // --- parse_u_values tests (ported, adapted for anyhow::Result) ---

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
        let vals = parse_u_values("42.0").unwrap();
        assert_eq!(vals, vec![42.0]);
    }

    #[test]
    fn parse_invalid_token() {
        let err = parse_u_values("1.0,abc,2.0").unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("abc"), "error should mention the invalid token: {msg}");
    }

    #[test]
    fn parse_empty_token() {
        let err = parse_u_values("1.0,,2.0").unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("invalid"), "error should report parse failure: {msg}");
    }

    #[test]
    fn parse_empty_string() {
        let err = parse_u_values("").unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("empty") || msg.contains("invalid"), "expected parse failure on empty input, got: {msg}");
    }

    #[test]
    fn parse_negative_values() {
        let vals = parse_u_values("-1.0,2.0").unwrap();
        assert_eq!(vals, vec![-1.0, 2.0]);
    }
}
