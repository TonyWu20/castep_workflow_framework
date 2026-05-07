use anyhow::{anyhow, Result};
use clap::Parser;

/// Configuration for multi-parameter sweep over Hubbard U, k-points, and cutoffs.
#[derive(Parser, Debug, Clone)]
#[command(name = "multi_param_sweep")]
pub struct SweepConfig {
    /// Comma-separated list of Hubbard U values to sweep.
    #[arg(long, default_value = "0.0,1.0,2.0,3.0,4.0,5.0")]
    pub u_values: String,

    /// Comma-separated list of k-point grids (e.g. "8x8x8,6x6x6").
    #[arg(long)]
    pub kpoints: Option<String>,

    /// Comma-separated list of plane-wave cutoffs in eV.
    #[arg(long)]
    pub cutoffs: Option<String>,

    /// Sweep mode: "product" for Cartesian product, "zip" for pairwise.
    #[arg(long, default_value = "product")]
    pub sweep_mode: String,

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
    #[arg(long, default_value_t = 4)]
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

/// Parse a comma-separated list of k-point grids into a vector of [u32; 3].
///
/// Each grid is specified as "NxNxN".  Empty input produces an error
/// containing "empty".  Wrong axis counts (not exactly 3) produce errors
/// mentioning the expected and actual count.  Non-numeric axes produce errors
/// mentioning the invalid input.
pub fn parse_kpoints(s: &str) -> Result<Vec<[u32; 3]>> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("kpoints list is empty"));
    }
    trimmed
        .split(',')
        .map(|segment| {
            let seg = segment.trim();
            let parts: Vec<&str> = seg.split('x').collect();
            if parts.len() != 3 {
                return Err(anyhow!(
                    "invalid kpoint '{}': expected 3 axes, got {}",
                    seg,
                    parts.len()
                ));
            }
            let x = parts[0]
                .parse::<u32>()
                .map_err(|_| anyhow!("invalid kpoint value: '{}'", parts[0]))?;
            let y = parts[1]
                .parse::<u32>()
                .map_err(|_| anyhow!("invalid kpoint value: '{}'", parts[1]))?;
            let z = parts[2]
                .parse::<u32>()
                .map_err(|_| anyhow!("invalid kpoint value: '{}'", parts[2]))?;
            Ok([x, y, z])
        })
        .collect()
}

/// Parse a comma-separated list of plane-wave cutoff values into a vector of
/// f64.
///
/// Each segment is trimmed of whitespace.  Empty input produces an error
/// containing "empty".  Consecutive commas or non-numeric tokens produce an
/// error mentioning the offending token.
pub fn parse_cutoffs(s: &str) -> Result<Vec<f64>> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("cutoffs list is empty"));
    }
    trimmed
        .split(',')
        .map(|token| {
            let t = token.trim();
            t.parse::<f64>()
                .map_err(|_| anyhow!("invalid cutoff value: '{}'", t))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- parse_u_values tests ---

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

    // --- parse_kpoints tests ---

    #[test]
    fn parse_kpoints_basic() {
        let kpts = parse_kpoints("8x8x8,6x6x6").unwrap();
        assert_eq!(kpts, vec![[8, 8, 8], [6, 6, 6]]);
    }

    #[test]
    fn parse_kpoints_single() {
        let kpts = parse_kpoints("4x4x4").unwrap();
        assert_eq!(kpts, vec![[4, 4, 4]]);
    }

    #[test]
    fn parse_kpoints_whitespace() {
        let kpts = parse_kpoints(" 8x8x8 , 6x6x6 ").unwrap();
        assert_eq!(kpts, vec![[8, 8, 8], [6, 6, 6]]);
    }

    #[test]
    fn parse_kpoints_wrong_axes() {
        let err = parse_kpoints("8x8").unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("3 axes") || msg.contains("got 2") || msg.contains("expected 3"), "error should mention wrong axis count: {msg}");
    }

    #[test]
    fn parse_kpoints_non_numeric() {
        let err = parse_kpoints("abc").unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("invalid") || msg.contains("abc") || msg.contains("expected 3 axes"), "error should report the problem: {msg}");
    }

    #[test]
    fn parse_kpoints_empty() {
        let err = parse_kpoints("").unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("empty"), "error should say kpoints list is empty: {msg}");
    }

    #[test]
    fn parse_kpoints_large() {
        let kpts = parse_kpoints("12x12x12").unwrap();
        assert_eq!(kpts, vec![[12, 12, 12]]);
    }

    // --- parse_cutoffs tests ---

    #[test]
    fn parse_cutoffs_basic() {
        let cuts = parse_cutoffs("300,500,800").unwrap();
        assert_eq!(cuts, vec![300.0, 500.0, 800.0]);
    }

    #[test]
    fn parse_cutoffs_single() {
        let cuts = parse_cutoffs("450").unwrap();
        assert_eq!(cuts, vec![450.0]);
    }

    #[test]
    fn parse_cutoffs_whitespace() {
        let cuts = parse_cutoffs(" 300 , 500 , 800 ").unwrap();
        assert_eq!(cuts, vec![300.0, 500.0, 800.0]);
    }

    #[test]
    fn parse_cutoffs_invalid_token() {
        let err = parse_cutoffs("300,abc,800").unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("abc"), "error should mention the invalid token: {msg}");
    }

    #[test]
    fn parse_cutoffs_empty() {
        let err = parse_cutoffs("").unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("empty"), "error should say cutoffs list is empty: {msg}");
    }

    #[test]
    fn parse_cutoffs_negative() {
        let cuts = parse_cutoffs("-100.5,200.0").unwrap();
        assert_eq!(cuts, vec![-100.5, 200.0]);
    }
}
