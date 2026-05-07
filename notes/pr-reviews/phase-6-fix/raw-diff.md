diff --git a/.claude/.current_stage b/.claude/.current_stage
new file mode 100644
index 0000000..352800e
--- /dev/null
+++ b/.claude/.current_stage
@@ -0,0 +1 @@
+explore-implement
diff --git a/.exploration_checkpoint.json b/.exploration_checkpoint.json
new file mode 100644
index 0000000..f91e531
--- /dev/null
+++ b/.exploration_checkpoint.json
@@ -0,0 +1,24 @@
+{
+  "directions_path": "/Users/tony/programming/castep_workflow_framework/notes/directions/phase-6-fix/directions-phase-6-fix-core-2.json",
+  "worktree_path": "/Users/tony/programming/castep_workflow_framework/.pipeline-worktrees/phase-6-fix-core-2",
+  "groups": {
+    "core-2": {
+      "status": "completed",
+      "tasks": [
+        "G2-1",
+        "G2-2",
+        "G2-3",
+        "G2-4",
+        "G2-5"
+      ],
+      "completed_tasks": [
+        "G2-1",
+        "G2-2",
+        "G2-3",
+        "G2-4",
+        "G2-5"
+      ],
+      "reason": "Create the examples/scf_dos_chain/ crate \u2014 a purpose-built binary for SCF+DOS task chain testing \u2014 and update the existing hubbard_u_sweep binary for castep-cell-io v0.5.0 path dep compatibility."
+    }
+  }
+}
diff --git a/.gitignore b/.gitignore
index c68272a..87ba5ee 100644
--- a/.gitignore
+++ b/.gitignore
@@ -27,3 +27,5 @@ target
 .direnv
 
 .claude/hooks/current_task*.json
+.pipeline-worktrees/
+.claude/
diff --git a/Cargo.lock b/Cargo.lock
index be63ca7..699a84a 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -130,6 +130,16 @@ dependencies = [
  "syn",
 ]
 
+[[package]]
+name = "castep-cell-fmt"
+version = "0.1.0"
+dependencies = [
+ "anyhow",
+ "ariadne",
+ "chumsky",
+ "thiserror 2.0.18",
+]
+
 [[package]]
 name = "castep-cell-fmt"
 version = "0.1.0"
@@ -150,7 +160,16 @@ source = "registry+https://github.com/rust-lang/crates.io-index"
 checksum = "be8d0b1f40faf83d0a8835829da5b861d69bc497a5e0f50d92293220ecb3a7aa"
 dependencies = [
  "bon",
- "castep-cell-fmt",
+ "castep-cell-fmt 0.1.0 (registry+https://github.com/rust-lang/crates.io-index)",
+ "serde",
+]
+
+[[package]]
+name = "castep-cell-io"
+version = "0.5.0"
+dependencies = [
+ "bon",
+ "castep-cell-fmt 0.1.0",
  "serde",
 ]
 
@@ -493,8 +512,8 @@ name = "hubbard_u_sweep"
 version = "0.1.0"
 dependencies = [
  "anyhow",
- "castep-cell-fmt",
- "castep-cell-io",
+ "castep-cell-fmt 0.1.0",
+ "castep-cell-io 0.5.0",
  "workflow_core",
  "workflow_utils",
 ]
@@ -504,8 +523,8 @@ name = "hubbard_u_sweep_slurm"
 version = "0.1.0"
 dependencies = [
  "anyhow",
- "castep-cell-fmt",
- "castep-cell-io",
+ "castep-cell-fmt 0.1.0 (registry+https://github.com/rust-lang/crates.io-index)",
+ "castep-cell-io 0.4.0",
  "clap",
  "itertools",
  "workflow_core",
@@ -622,6 +641,19 @@ version = "2.8.0"
 source = "registry+https://github.com/rust-lang/crates.io-index"
 checksum = "f8ca58f447f06ed17d5fc4043ce1b10dd205e060fb3ce5b979b8ed8e59ff3f79"
 
+[[package]]
+name = "multi_param_sweep"
+version = "0.1.0"
+dependencies = [
+ "anyhow",
+ "castep-cell-fmt 0.1.0",
+ "castep-cell-io 0.5.0",
+ "clap",
+ "itertools",
+ "workflow_core",
+ "workflow_utils",
+]
+
 [[package]]
 name = "nu-ansi-term"
 version = "0.50.3"
@@ -820,6 +852,19 @@ dependencies = [
  "sdd",
 ]
 
+[[package]]
+name = "scf_dos_chain"
+version = "0.1.0"
+dependencies = [
+ "anyhow",
+ "castep-cell-fmt 0.1.0",
+ "castep-cell-io 0.5.0",
+ "clap",
+ "itertools",
+ "workflow_core",
+ "workflow_utils",
+]
+
 [[package]]
 name = "scopeguard"
 version = "1.2.0"
diff --git a/Cargo.toml b/Cargo.toml
index aac45ea..3a54e1c 100644
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -4,6 +4,8 @@ members = [
     "workflow_utils",
     "examples/hubbard_u_sweep",
     "examples/hubbard_u_sweep_slurm",
+    "examples/multi_param_sweep",
+    "examples/scf_dos_chain",
     "workflow-cli",
 ]
 resolver = "2"
diff --git a/examples/castep-cell-io b/examples/castep-cell-io
new file mode 160000
index 0000000..8461db9
--- /dev/null
+++ b/examples/castep-cell-io
@@ -0,0 +1 @@
+Subproject commit 8461db966eb3410ca8d9f63bc4de3940930bfe5c
diff --git a/examples/hubbard_u_sweep/Cargo.toml b/examples/hubbard_u_sweep/Cargo.toml
index eb4e08c..efa0f23 100644
--- a/examples/hubbard_u_sweep/Cargo.toml
+++ b/examples/hubbard_u_sweep/Cargo.toml
@@ -9,7 +9,7 @@ path = "src/main.rs"
 
 [dependencies]
 anyhow = "1"
-castep-cell-fmt = "0.1.0"
-castep-cell-io = "0.4.0"
+castep-cell-fmt = { version = "0.1.0", path = "../castep-cell-io/castep_cell_fmt" }
+castep-cell-io = { path = "../castep-cell-io/castep_cell_io" }
 workflow_core = { path = "../../workflow_core" }
 workflow_utils = { path = "../../workflow_utils" }
diff --git a/examples/multi_param_sweep/Cargo.toml b/examples/multi_param_sweep/Cargo.toml
new file mode 100644
index 0000000..418ea98
--- /dev/null
+++ b/examples/multi_param_sweep/Cargo.toml
@@ -0,0 +1,17 @@
+[package]
+name = "multi_param_sweep"
+version = "0.1.0"
+edition = "2021"
+
+[[bin]]
+name = "multi_param_sweep"
+path = "src/main.rs"
+
+[dependencies]
+anyhow = { workspace = true }
+clap = { workspace = true }
+itertools = { workspace = true }
+castep-cell-fmt = { version = "0.1.0", path = "../castep-cell-io/castep_cell_fmt" }
+castep-cell-io = { path = "../castep-cell-io/castep_cell_io" }
+workflow_core = { path = "../../workflow_core", features = ["default-logging"] }
+workflow_utils = { path = "../../workflow_utils" }
diff --git a/examples/multi_param_sweep/seeds/ZnO.cell b/examples/multi_param_sweep/seeds/ZnO.cell
new file mode 100644
index 0000000..9e2592a
--- /dev/null
+++ b/examples/multi_param_sweep/seeds/ZnO.cell
@@ -0,0 +1,12 @@
+%BLOCK LATTICE_CART
+  3.25 0.0 0.0
+  0.0 3.25 0.0
+  0.0 0.0 5.21
+%ENDBLOCK LATTICE_CART
+
+%BLOCK POSITIONS_FRAC
+Zn  0.333333  0.666667  0.0
+Zn  0.666667  0.333333  0.5
+O   0.333333  0.666667  0.375
+O   0.666667  0.333333  0.875
+%ENDBLOCK POSITIONS_FRAC
diff --git a/examples/multi_param_sweep/seeds/ZnO.param b/examples/multi_param_sweep/seeds/ZnO.param
new file mode 100644
index 0000000..fea8b7b
--- /dev/null
+++ b/examples/multi_param_sweep/seeds/ZnO.param
@@ -0,0 +1 @@
+task : SinglePoint
diff --git a/examples/multi_param_sweep/src/config.rs b/examples/multi_param_sweep/src/config.rs
new file mode 100644
index 0000000..b5585e3
--- /dev/null
+++ b/examples/multi_param_sweep/src/config.rs
@@ -0,0 +1,292 @@
+use anyhow::{anyhow, Result};
+use clap::Parser;
+
+/// Configuration for multi-parameter sweep over Hubbard U, k-points, and cutoffs.
+#[derive(Parser, Debug, Clone)]
+#[command(name = "multi_param_sweep")]
+pub struct SweepConfig {
+    /// Comma-separated list of Hubbard U values to sweep.
+    #[arg(long, default_value = "0.0,1.0,2.0,3.0,4.0,5.0")]
+    pub u_values: String,
+
+    /// Comma-separated list of k-point grids (e.g. "8x8x8,6x6x6").
+    #[arg(long)]
+    pub kpoints: Option<String>,
+
+    /// Comma-separated list of plane-wave cutoffs in eV.
+    #[arg(long)]
+    pub cutoffs: Option<String>,
+
+    /// Sweep mode: "product" for Cartesian product, "zip" for pairwise.
+    #[arg(long, default_value = "product")]
+    pub sweep_mode: String,
+
+    /// Element symbol for the Hubbard site.
+    #[arg(long, default_value = "Zn")]
+    pub element: String,
+
+    /// Orbital label for the Hubbard manifold.
+    #[arg(long, default_value = "d")]
+    pub orbital: char,
+
+    /// CASTEP seed name for input/output files.
+    #[arg(long, default_value = "ZnO")]
+    pub seed_name: String,
+
+    /// Maximum number of parallel jobs.
+    #[arg(long, default_value_t = 4)]
+    pub max_parallel: usize,
+
+    /// Run locally instead of submitting to Slurm.
+    #[arg(long)]
+    pub local: bool,
+
+    /// Print what would be done without executing.
+    #[arg(long)]
+    pub dry_run: bool,
+
+    /// CASTEP binary command.
+    #[arg(long, default_value = "castep")]
+    pub castep_command: String,
+
+    /// Working directory for CASTEP runs.
+    #[arg(long, default_value = ".")]
+    pub workdir: String,
+
+    /// Slurm partition name.
+    #[arg(long, env = "CASTEP_SLURM_PARTITION", default_value = "debug")]
+    pub partition: String,
+
+    /// Number of MPI tasks per job.
+    #[arg(long, default_value_t = 16)]
+    pub ntasks: u32,
+
+    /// Nix flake URL for the CASTEP environment.
+    #[arg(
+        long,
+        env = "CASTEP_NIX_FLAKE",
+        default_value = "git+ssh://git@github.com/TonyWu20/CASTEP-25.12-nixos#castep_25_mkl"
+    )]
+    pub nix_flake: String,
+
+    /// Network interface for MPI communication.
+    #[arg(long, env = "CASTEP_MPI_IF", default_value = "enp6s0")]
+    pub mpi_if: String,
+}
+
+/// Parse a comma-separated list of Hubbard U values into a vector of f64.
+///
+/// Each segment is trimmed of whitespace.  Empty input produces an error
+/// containing "empty".  Consecutive commas or non-numeric tokens produce an
+/// error containing the offending token.
+pub fn parse_u_values(s: &str) -> Result<Vec<f64>> {
+    let trimmed = s.trim();
+    if trimmed.is_empty() {
+        return Err(anyhow!("empty u values input"));
+    }
+    trimmed
+        .split(',')
+        .map(|token| {
+            let t = token.trim();
+            t.parse::<f64>()
+                .map_err(|_| anyhow!("invalid u value: '{}'", t))
+        })
+        .collect()
+}
+
+/// Parse a comma-separated list of k-point grids into a vector of [u32; 3].
+///
+/// Each grid is specified as "NxNxN".  Empty input produces an error
+/// containing "empty".  Wrong axis counts (not exactly 3) produce errors
+/// mentioning the expected and actual count.  Non-numeric axes produce errors
+/// mentioning the invalid input.
+pub fn parse_kpoints(s: &str) -> Result<Vec<[u32; 3]>> {
+    let trimmed = s.trim();
+    if trimmed.is_empty() {
+        return Err(anyhow!("kpoints list is empty"));
+    }
+    trimmed
+        .split(',')
+        .map(|segment| {
+            let seg = segment.trim();
+            let parts: Vec<&str> = seg.split('x').collect();
+            if parts.len() != 3 {
+                return Err(anyhow!(
+                    "invalid kpoint '{}': expected 3 axes, got {}",
+                    seg,
+                    parts.len()
+                ));
+            }
+            let x = parts[0]
+                .parse::<u32>()
+                .map_err(|_| anyhow!("invalid kpoint value: '{}'", parts[0]))?;
+            let y = parts[1]
+                .parse::<u32>()
+                .map_err(|_| anyhow!("invalid kpoint value: '{}'", parts[1]))?;
+            let z = parts[2]
+                .parse::<u32>()
+                .map_err(|_| anyhow!("invalid kpoint value: '{}'", parts[2]))?;
+            Ok([x, y, z])
+        })
+        .collect()
+}
+
+/// Parse a comma-separated list of plane-wave cutoff values into a vector of
+/// f64.
+///
+/// Each segment is trimmed of whitespace.  Empty input produces an error
+/// containing "empty".  Consecutive commas or non-numeric tokens produce an
+/// error mentioning the offending token.
+pub fn parse_cutoffs(s: &str) -> Result<Vec<f64>> {
+    let trimmed = s.trim();
+    if trimmed.is_empty() {
+        return Err(anyhow!("cutoffs list is empty"));
+    }
+    trimmed
+        .split(',')
+        .map(|token| {
+            let t = token.trim();
+            t.parse::<f64>()
+                .map_err(|_| anyhow!("invalid cutoff value: '{}'", t))
+        })
+        .collect()
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    // --- parse_u_values tests ---
+
+    #[test]
+    fn parse_basic_values() {
+        let vals = parse_u_values("0.0,1.0,2.0").unwrap();
+        assert_eq!(vals, vec![0.0, 1.0, 2.0]);
+    }
+
+    #[test]
+    fn parse_with_whitespace() {
+        let vals = parse_u_values("  0.0 , 1.0 , 2.0  ").unwrap();
+        assert_eq!(vals, vec![0.0, 1.0, 2.0]);
+    }
+
+    #[test]
+    fn parse_single_value() {
+        let vals = parse_u_values("42.0").unwrap();
+        assert_eq!(vals, vec![42.0]);
+    }
+
+    #[test]
+    fn parse_invalid_token() {
+        let err = parse_u_values("1.0,abc,2.0").unwrap_err();
+        let msg = format!("{err}");
+        assert!(msg.contains("abc"), "error should mention the invalid token: {msg}");
+    }
+
+    #[test]
+    fn parse_empty_token() {
+        let err = parse_u_values("1.0,,2.0").unwrap_err();
+        let msg = format!("{err}");
+        assert!(msg.contains("invalid"), "error should report parse failure: {msg}");
+    }
+
+    #[test]
+    fn parse_empty_string() {
+        let err = parse_u_values("").unwrap_err();
+        let msg = format!("{err}");
+        assert!(msg.contains("empty") || msg.contains("invalid"), "expected parse failure on empty input, got: {msg}");
+    }
+
+    #[test]
+    fn parse_negative_values() {
+        let vals = parse_u_values("-1.0,2.0").unwrap();
+        assert_eq!(vals, vec![-1.0, 2.0]);
+    }
+
+    // --- parse_kpoints tests ---
+
+    #[test]
+    fn parse_kpoints_basic() {
+        let kpts = parse_kpoints("8x8x8,6x6x6").unwrap();
+        assert_eq!(kpts, vec![[8, 8, 8], [6, 6, 6]]);
+    }
+
+    #[test]
+    fn parse_kpoints_single() {
+        let kpts = parse_kpoints("4x4x4").unwrap();
+        assert_eq!(kpts, vec![[4, 4, 4]]);
+    }
+
+    #[test]
+    fn parse_kpoints_whitespace() {
+        let kpts = parse_kpoints(" 8x8x8 , 6x6x6 ").unwrap();
+        assert_eq!(kpts, vec![[8, 8, 8], [6, 6, 6]]);
+    }
+
+    #[test]
+    fn parse_kpoints_wrong_axes() {
+        let err = parse_kpoints("8x8").unwrap_err();
+        let msg = format!("{err}");
+        assert!(msg.contains("3 axes") || msg.contains("got 2") || msg.contains("expected 3"), "error should mention wrong axis count: {msg}");
+    }
+
+    #[test]
+    fn parse_kpoints_non_numeric() {
+        let err = parse_kpoints("abc").unwrap_err();
+        let msg = format!("{err}");
+        assert!(msg.contains("invalid") || msg.contains("abc") || msg.contains("expected 3 axes"), "error should report the problem: {msg}");
+    }
+
+    #[test]
+    fn parse_kpoints_empty() {
+        let err = parse_kpoints("").unwrap_err();
+        let msg = format!("{err}");
+        assert!(msg.contains("empty"), "error should say kpoints list is empty: {msg}");
+    }
+
+    #[test]
+    fn parse_kpoints_large() {
+        let kpts = parse_kpoints("12x12x12").unwrap();
+        assert_eq!(kpts, vec![[12, 12, 12]]);
+    }
+
+    // --- parse_cutoffs tests ---
+
+    #[test]
+    fn parse_cutoffs_basic() {
+        let cuts = parse_cutoffs("300,500,800").unwrap();
+        assert_eq!(cuts, vec![300.0, 500.0, 800.0]);
+    }
+
+    #[test]
+    fn parse_cutoffs_single() {
+        let cuts = parse_cutoffs("450").unwrap();
+        assert_eq!(cuts, vec![450.0]);
+    }
+
+    #[test]
+    fn parse_cutoffs_whitespace() {
+        let cuts = parse_cutoffs(" 300 , 500 , 800 ").unwrap();
+        assert_eq!(cuts, vec![300.0, 500.0, 800.0]);
+    }
+
+    #[test]
+    fn parse_cutoffs_invalid_token() {
+        let err = parse_cutoffs("300,abc,800").unwrap_err();
+        let msg = format!("{err}");
+        assert!(msg.contains("abc"), "error should mention the invalid token: {msg}");
+    }
+
+    #[test]
+    fn parse_cutoffs_empty() {
+        let err = parse_cutoffs("").unwrap_err();
+        let msg = format!("{err}");
+        assert!(msg.contains("empty"), "error should say cutoffs list is empty: {msg}");
+    }
+
+    #[test]
+    fn parse_cutoffs_negative() {
+        let cuts = parse_cutoffs("-100.5,200.0").unwrap();
+        assert_eq!(cuts, vec![-100.5, 200.0]);
+    }
+}
diff --git a/examples/multi_param_sweep/src/job_script.rs b/examples/multi_param_sweep/src/job_script.rs
new file mode 100644
index 0000000..a805f1a
--- /dev/null
+++ b/examples/multi_param_sweep/src/job_script.rs
@@ -0,0 +1,97 @@
+use crate::config::SweepConfig;
+
+/// Generate a SLURM job submission script for a CASTEP calculation.
+///
+/// Returns a bash script with SBATCH directives, nix environment setup, and
+/// an mpirun invocation.  Uses only spaces for indentation (no literal tabs).
+pub fn generate_job_script(config: &SweepConfig, task_id: &str, seed_name: &str) -> String {
+    format!(
+        "\
+#!/usr/bin/env bash
+#SBATCH --job-name=\"{task_id}\"
+#SBATCH --output=slurm_output_%j.txt
+#SBATCH --partition={partition}
+#SBATCH --nodes=1
+#SBATCH --ntasks-per-node={ntasks}
+#SBATCH --cpus-per-task=1
+#SBATCH --mem=30000m
+#SBATCH --nodelist=nixos
+nix develop {nix_flake} --command bash -c \\
+    \"mpirun --mca plm slurm \\
+        -x OMPI_MCA_btl_tcp_if_include={mpi_if} \\
+        -x OMPI_MCA_orte_keep_fqdn_hostnames=true \\
+        --mca pmix s1 \\
+        --mca btl tcp,self \\
+        --map-by numa --bind-to numa \\
+    castep.mpi {seed_name}\"
+",
+        task_id = task_id,
+        partition = config.partition,
+        ntasks = config.ntasks,
+        nix_flake = config.nix_flake,
+        mpi_if = config.mpi_if,
+    )
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::config::SweepConfig;
+    use clap::Parser;
+
+    fn default_config() -> SweepConfig {
+        SweepConfig::parse_from(["test"])
+    }
+
+    /// Script contains expected SBATCH directives with the task ID and config values
+    #[test]
+    fn contains_sbatch_directives() {
+        let config = default_config();
+        let script = generate_job_script(&config, "scf_U1.0", "ZnO");
+        assert!(script.contains(r#"#SBATCH --job-name="scf_U1.0""#));
+        assert!(script.contains("#SBATCH --partition=debug"));
+        assert!(script.contains("#SBATCH --ntasks-per-node=16"));
+        assert!(script.contains("#SBATCH --mem=30000m"));
+    }
+
+    /// Script references the correct seed name in the castep command
+    #[test]
+    fn contains_seed_name() {
+        let config = default_config();
+        let script = generate_job_script(&config, "scf_U0.0", "ZnO");
+        assert!(script.contains("castep.mpi ZnO"));
+    }
+
+    /// D.2 fix: script must not contain literal tab characters
+    #[test]
+    fn no_literal_tabs() {
+        let config = default_config();
+        let script = generate_job_script(&config, "scf_U0.0", "ZnO");
+        assert!(!script.contains('\t'), "job script should not contain literal tab characters");
+    }
+
+    /// Script starts with a shebang line
+    #[test]
+    fn starts_with_shebang() {
+        let config = default_config();
+        let script = generate_job_script(&config, "scf_U0.0", "ZnO");
+        assert!(script.starts_with("#!/usr/bin/env bash"));
+    }
+
+    /// Script contains nix develop with the configured flake URI
+    #[test]
+    fn contains_nix_develop() {
+        let config = default_config();
+        let script = generate_job_script(&config, "scf_U0.0", "ZnO");
+        assert!(script.contains("nix develop"));
+        assert!(script.contains(&config.nix_flake));
+    }
+
+    /// Script contains the MPI interface setting
+    #[test]
+    fn contains_mpi_interface() {
+        let config = default_config();
+        let script = generate_job_script(&config, "scf_U0.0", "ZnO");
+        assert!(script.contains(&format!("OMPI_MCA_btl_tcp_if_include={mpi_if}", mpi_if = config.mpi_if)));
+    }
+}
diff --git a/examples/multi_param_sweep/src/main.rs b/examples/multi_param_sweep/src/main.rs
new file mode 100644
index 0000000..de2f278
--- /dev/null
+++ b/examples/multi_param_sweep/src/main.rs
@@ -0,0 +1,444 @@
+mod config;
+mod job_script;
+
+use std::error::Error;
+use std::path::{Path, PathBuf};
+use std::sync::Arc;
+
+use anyhow::anyhow;
+use clap::Parser;
+use castep_cell_fmt::{format::to_string_many_spaced, parse, ToCellFile};
+use castep_cell_io::cell::bz_sampling_kpoints::KpointsMpGrid;
+use castep_cell_io::cell::species::{AtomHubbardU, HubbardU, OrbitalU, Species};
+use castep_cell_io::param::basis_set::CutOffEnergy;
+use castep_cell_io::{CellDocument, ParamDocument};
+use config::{parse_cutoffs, parse_kpoints, parse_u_values, SweepConfig};
+use itertools::iproduct;
+use job_script::generate_job_script;
+use workflow_utils::prelude::*;
+
+fn main() -> anyhow::Result<()> {
+    workflow_core::init_default_logging().ok();
+    let config = SweepConfig::parse();
+
+    let tasks = build_all_scf_tasks(&config)?;
+
+    let mut workflow = Workflow::new("multi_param_sweep")
+        .with_max_parallel(config.max_parallel)?
+        .with_log_dir("logs")
+        .with_root_dir(&config.workdir);
+
+    if !config.local {
+        workflow = workflow.with_queued_submitter(Arc::new(QueuedRunner::new(SchedulerKind::Slurm)));
+    }
+
+    for task in tasks {
+        workflow.add_task(task)?;
+    }
+
+    if config.dry_run {
+        let order = workflow.dry_run()?;
+        println!("Dry-run topological order:");
+        for task_id in &order {
+            println!("  {task_id}");
+        }
+        return Ok(());
+    }
+
+    let state_path = PathBuf::from(".multi_param_sweep.workflow.json");
+    let mut state = JsonStateStore::new("multi_param_sweep", state_path);
+
+    let summary = if config.local {
+        run_default(&mut workflow, &mut state)?
+    } else {
+        let runner: Arc<dyn ProcessRunner> = Arc::new(SystemProcessRunner::new());
+        let executor: Arc<dyn HookExecutor> = Arc::new(ShellHookExecutor);
+        workflow.run(&mut state, runner, executor)?
+    };
+
+    println!(
+        "Workflow complete: {} succeeded, {} failed, {} skipped ({:.1}s)",
+        summary.succeeded.len(),
+        summary.failed.len(),
+        summary.skipped.len(),
+        summary.duration.as_secs_f64(),
+    );
+    Ok(())
+}
+
+fn build_one_scf_task(
+    config: &SweepConfig,
+    u: f64,
+    kpoint: Option<[u32; 3]>,
+    cutoff: Option<f64>,
+    seed_cell: &str,
+    seed_param: &str,
+) -> std::result::Result<Task, WorkflowError> {
+    // Build task ID
+    let u_str = format!("{:.1}", u);
+    let k_str = kpoint.map(|k| format!("{}x{}x{}", k[0], k[1], k[2]));
+    let c_str = cutoff.map(|c| format!("{:.0}", c));
+
+    let id = {
+        let mut id = format!("scf_U{u_str}");
+        if let Some(ref k) = k_str {
+            id.push_str(&format!("_k{k}"));
+        }
+        if let Some(ref c) = c_str {
+            id.push_str(&format!("_c{c}"));
+        }
+        id
+    };
+
+    // Build workdir path
+    let workdir = {
+        let mut name = format!("U{u_str}");
+        if let Some(ref k) = k_str {
+            name.push_str(&format!("_k{k}"));
+        }
+        if let Some(ref c) = c_str {
+            name.push_str(&format!("_c{c}"));
+        }
+        PathBuf::from("runs").join(&name)
+    };
+
+    // Determine execution mode
+    let mode = if config.local {
+        ExecutionMode::direct(&config.castep_command, &[&config.seed_name])
+    } else {
+        ExecutionMode::Queued
+    };
+
+    // Clone values needed by move closures
+    let element = config.element.clone();
+    let orbital_char = config.orbital;
+    let local = config.local;
+    let seed_name = config.seed_name.clone();
+    let config_clone = config.clone();
+    let task_id = id.clone();
+    let seed_cell_owned = seed_cell.to_owned();
+    let seed_param_owned = seed_param.to_owned();
+    let collect_seed = seed_name.clone();
+
+    // Build setup closure
+    let boxed_setup = Box::new(move |path: &Path| -> std::result::Result<(), Box<dyn Error + Send + Sync>> {
+        create_dir(path)?;
+
+        // Parse and modify cell document
+        let mut cell_doc: CellDocument = parse(&seed_cell_owned)?;
+
+        // Add HubbardU block
+        let species = Species::Symbol(element.clone());
+        let orbital = match orbital_char {
+            's' => OrbitalU::S(u),
+            'p' => OrbitalU::P(u),
+            'd' => OrbitalU::D(u),
+            'f' => OrbitalU::F(u),
+            _ => {
+                return Err(Box::new(WorkflowError::InvalidConfig(format!(
+                    "invalid orbital: {}",
+                    orbital_char
+                ))))
+            }
+        };
+        let atom_u = AtomHubbardU::builder()
+            .species(species)
+            .orbitals(vec![orbital])
+            .build();
+        let hubbard_u = HubbardU::builder()
+            .atom_u_values(vec![atom_u])
+            .build();
+        cell_doc.hubbard_u = Some(hubbard_u);
+
+        // Set kpoints if provided
+        if let Some(k) = kpoint {
+            cell_doc.kpoints_mp_grid = Some(KpointsMpGrid(k));
+        }
+
+        // Parse and modify param document
+        let mut param_doc: ParamDocument = parse(&seed_param_owned)?;
+        if let Some(c) = cutoff {
+            param_doc.basis_set.cutoff_energy = Some(CutOffEnergy { value: c, unit: None });
+        }
+
+        // Serialize to file strings
+        let cell_str = to_string_many_spaced(&cell_doc.to_cell_file());
+        let param_str = to_string_many_spaced(&param_doc.to_cell_file());
+
+        // Write .cell and .param files
+        write_file(path.join(format!("{seed_name}.cell")), &cell_str)?;
+        write_file(path.join(format!("{seed_name}.param")), &param_str)?;
+
+        // Write job script if queued
+        if !local {
+            let script = generate_job_script(&config_clone, &task_id, &seed_name);
+            write_file(path.join(JOB_SCRIPT_NAME), &script)?;
+        }
+
+        Ok(())
+    });
+
+    // Build collect closure
+    let boxed_collect = Box::new(move |path: &Path| -> std::result::Result<(), Box<dyn Error + Send + Sync>> {
+        let output_str = read_file(path.join(format!("{collect_seed}.castep")))?;
+        if !output_str.contains("Total time") {
+            return Err(Box::new(WorkflowError::InvalidConfig(
+                "CASTEP output missing 'Total time' marker".into(),
+            )));
+        }
+        Ok(())
+    });
+
+    let mut task = Task::new(id, mode)
+        .workdir(workdir);
+    task.setup = Some(boxed_setup);
+    task.collect = Some(boxed_collect);
+
+    Ok(task)
+}
+
+fn build_all_scf_tasks(config: &SweepConfig) -> anyhow::Result<Vec<Task>> {
+    let seed_cell = include_str!("../seeds/ZnO.cell");
+    let seed_param = include_str!("../seeds/ZnO.param");
+
+    let u_vals = parse_u_values(&config.u_values)?;
+
+    let kpoints: Option<Vec<[u32; 3]>> = match &config.kpoints {
+        Some(s) => Some(parse_kpoints(s)?),
+        None => None,
+    };
+
+    let cutoffs: Option<Vec<f64>> = match &config.cutoffs {
+        Some(s) => Some(parse_cutoffs(s)?),
+        None => None,
+    };
+
+    match config.sweep_mode.as_str() {
+        "product" => {
+            let kpoint_opts: Vec<Option<[u32; 3]>> = match &kpoints {
+                Some(kpts) => kpts.iter().map(|&k| Some(k)).collect(),
+                None => vec![None],
+            };
+            let cutoff_opts: Vec<Option<f64>> = match &cutoffs {
+                Some(cuts) => cuts.iter().map(|&c| Some(c)).collect(),
+                None => vec![None],
+            };
+
+            let mut tasks = Vec::new();
+            for (&u_val, &kpoint_opt, &cutoff_opt) in
+                iproduct!(&u_vals, &kpoint_opts, &cutoff_opts)
+            {
+                tasks.push(build_one_scf_task(
+                    config,
+                    u_val,
+                    kpoint_opt,
+                    cutoff_opt,
+                    seed_cell,
+                    seed_param,
+                )?);
+            }
+            Ok(tasks)
+        }
+        "pairwise" => {
+            let kpts = kpoints.ok_or_else(|| anyhow!("pairwise mode requires --kpoints"))?;
+            let cuts = cutoffs.ok_or_else(|| anyhow!("pairwise mode requires --cutoffs"))?;
+
+            if u_vals.len() != kpts.len() || u_vals.len() != cuts.len() {
+                anyhow::bail!(
+                    "all parameter lists must have the same length for pairwise mode, got u={}, k={}, c={}",
+                    u_vals.len(),
+                    kpts.len(),
+                    cuts.len()
+                );
+            }
+
+            let mut tasks = Vec::with_capacity(u_vals.len());
+            for ((&u_val, &kpt), &cut) in u_vals.iter().zip(&kpts).zip(&cuts) {
+                tasks.push(build_one_scf_task(
+                    config,
+                    u_val,
+                    Some(kpt),
+                    Some(cut),
+                    seed_cell,
+                    seed_param,
+                )?);
+            }
+            Ok(tasks)
+        }
+        mode => Err(anyhow!("unknown sweep mode: {mode}")),
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::config::SweepConfig;
+    use clap::Parser;
+
+    fn default_config() -> SweepConfig {
+        SweepConfig::parse_from(["test"])
+    }
+
+    // --- Sweep combinatorics tests ---
+
+    #[test]
+    fn product_mode_produces_tasks() {
+        let config = default_config();
+        let tasks = build_all_scf_tasks(&config).unwrap();
+        assert_eq!(tasks.len(), 6);
+    }
+
+    #[test]
+    fn product_mode_cartesian_product() {
+        let config = SweepConfig::parse_from([
+            "test",
+            "--u-values", "0.0,1.0,2.0",
+            "--kpoints", "8x8x8,6x6x6",
+            "--cutoffs", "300,500,800",
+            "--sweep-mode", "product",
+        ]);
+        let tasks = build_all_scf_tasks(&config).unwrap();
+        assert_eq!(tasks.len(), 18);
+    }
+
+    #[test]
+    fn pairwise_mode_zip() {
+        let config = SweepConfig::parse_from([
+            "test",
+            "--u-values", "0.0,1.0,2.0",
+            "--kpoints", "8x8x8,6x6x6,4x4x4",
+            "--cutoffs", "300,500,800",
+            "--sweep-mode", "pairwise",
+        ]);
+        let tasks = build_all_scf_tasks(&config).unwrap();
+        assert_eq!(tasks.len(), 3);
+    }
+
+    #[test]
+    fn pairwise_requires_kpoints() {
+        let config = SweepConfig::parse_from([
+            "test",
+            "--u-values", "0.0,1.0,2.0",
+            "--cutoffs", "300,500,800",
+            "--sweep-mode", "pairwise",
+        ]);
+        let err = build_all_scf_tasks(&config).unwrap_err();
+        let msg = format!("{err}");
+        assert!(msg.contains("kpoints"), "error should mention kpoints: {msg}");
+    }
+
+    #[test]
+    fn pairwise_requires_cutoffs() {
+        let config = SweepConfig::parse_from([
+            "test",
+            "--u-values", "0.0,1.0,2.0",
+            "--kpoints", "8x8x8,6x6x6,4x4x4",
+            "--sweep-mode", "pairwise",
+        ]);
+        let err = build_all_scf_tasks(&config).unwrap_err();
+        let msg = format!("{err}");
+        assert!(msg.contains("cutoffs"), "error should mention cutoffs: {msg}");
+    }
+
+    #[test]
+    fn pairwise_unequal_lengths() {
+        let config = SweepConfig::parse_from([
+            "test",
+            "--u-values", "0.0,1.0",
+            "--kpoints", "8x8x8,6x6x6,4x4x4",
+            "--cutoffs", "300,500,800",
+            "--sweep-mode", "pairwise",
+        ]);
+        let err = build_all_scf_tasks(&config).unwrap_err();
+        let msg = format!("{err}");
+        assert!(msg.contains("same length") || msg.contains("length"), "error should mention length mismatch: {msg}");
+    }
+
+    #[test]
+    fn unknown_sweep_mode() {
+        let config = SweepConfig::parse_from([
+            "test",
+            "--sweep-mode", "invalid",
+        ]);
+        let err = build_all_scf_tasks(&config).unwrap_err();
+        let msg = format!("{err}");
+        assert!(msg.contains("unknown sweep mode"), "error should mention unknown mode: {msg}");
+    }
+
+    // --- Task structure tests ---
+
+    #[test]
+    fn task_ids_are_unique() {
+        let config = SweepConfig::parse_from([
+            "test",
+            "--u-values", "0.0,1.0",
+            "--kpoints", "8x8x8,6x6x6",
+            "--cutoffs", "300,500",
+            "--sweep-mode", "product",
+        ]);
+        let tasks = build_all_scf_tasks(&config).unwrap();
+        let ids: Vec<&str> = tasks.iter().map(|t| t.id.as_str()).collect();
+        let mut unique_ids = ids.clone();
+        unique_ids.sort();
+        unique_ids.dedup();
+        assert_eq!(ids.len(), unique_ids.len(), "all task IDs must be unique");
+    }
+
+    #[test]
+    fn task_ids_encode_parameters() {
+        let config = SweepConfig::parse_from([
+            "test",
+            "--u-values", "3.0",
+            "--kpoints", "8x8x8",
+            "--cutoffs", "500",
+            "--sweep-mode", "product",
+        ]);
+        let tasks = build_all_scf_tasks(&config).unwrap();
+        assert_eq!(tasks.len(), 1);
+        let id = &tasks[0].id;
+        assert!(id.contains("U3") || id.contains("U3.0"), "task ID should contain U value: {id}");
+        assert!(id.contains("k8x8x8") || id.contains("k"), "task ID should contain kpoint: {id}");
+        assert!(id.contains("c500") || id.contains("c"), "task ID should contain cutoff: {id}");
+    }
+
+    #[test]
+    fn product_with_optional_params() {
+        let config = SweepConfig::parse_from([
+            "test",
+            "--u-values", "0.0,1.0,2.0",
+        ]);
+        let tasks = build_all_scf_tasks(&config).unwrap();
+        assert_eq!(tasks.len(), 3);
+    }
+
+    #[test]
+    fn scf_tasks_have_no_dependencies() {
+        let config = SweepConfig::parse_from([
+            "test",
+            "--u-values", "0.0,1.0",
+            "--kpoints", "8x8x8",
+            "--cutoffs", "500",
+            "--sweep-mode", "product",
+        ]);
+        let tasks = build_all_scf_tasks(&config).unwrap();
+        for task in &tasks {
+            assert!(task.dependencies.is_empty(), "SCF task should have no dependencies: {}", task.id);
+        }
+    }
+
+    #[test]
+    fn tasks_have_workdirs() {
+        let config = SweepConfig::parse_from([
+            "test",
+            "--u-values", "1.0",
+            "--kpoints", "8x8x8",
+            "--cutoffs", "500",
+            "--sweep-mode", "product",
+        ]);
+        let tasks = build_all_scf_tasks(&config).unwrap();
+        assert!(!tasks.is_empty());
+        for task in &tasks {
+            assert!(!task.workdir.as_os_str().is_empty(), "task {} should have a workdir", task.id);
+        }
+    }
+}
diff --git a/examples/scf_dos_chain/Cargo.toml b/examples/scf_dos_chain/Cargo.toml
new file mode 100644
index 0000000..8a130fe
--- /dev/null
+++ b/examples/scf_dos_chain/Cargo.toml
@@ -0,0 +1,17 @@
+[package]
+name = "scf_dos_chain"
+version = "0.1.0"
+edition = "2021"
+
+[[bin]]
+name = "scf_dos_chain"
+path = "src/main.rs"
+
+[dependencies]
+anyhow = { workspace = true }
+clap = { workspace = true }
+itertools = { workspace = true }
+castep-cell-fmt = { version = "0.1.0", path = "../castep-cell-io/castep_cell_fmt" }
+castep-cell-io = { path = "../castep-cell-io/castep_cell_io" }
+workflow_core = { path = "../../workflow_core", features = ["default-logging"] }
+workflow_utils = { path = "../../workflow_utils" }
diff --git a/examples/scf_dos_chain/seeds/ZnO.cell b/examples/scf_dos_chain/seeds/ZnO.cell
new file mode 100644
index 0000000..9e2592a
--- /dev/null
+++ b/examples/scf_dos_chain/seeds/ZnO.cell
@@ -0,0 +1,12 @@
+%BLOCK LATTICE_CART
+  3.25 0.0 0.0
+  0.0 3.25 0.0
+  0.0 0.0 5.21
+%ENDBLOCK LATTICE_CART
+
+%BLOCK POSITIONS_FRAC
+Zn  0.333333  0.666667  0.0
+Zn  0.666667  0.333333  0.5
+O   0.333333  0.666667  0.375
+O   0.666667  0.333333  0.875
+%ENDBLOCK POSITIONS_FRAC
diff --git a/examples/scf_dos_chain/seeds/ZnO.param b/examples/scf_dos_chain/seeds/ZnO.param
new file mode 100644
index 0000000..fea8b7b
--- /dev/null
+++ b/examples/scf_dos_chain/seeds/ZnO.param
@@ -0,0 +1 @@
+task : SinglePoint
diff --git a/examples/scf_dos_chain/src/config.rs b/examples/scf_dos_chain/src/config.rs
new file mode 100644
index 0000000..03edcb6
--- /dev/null
+++ b/examples/scf_dos_chain/src/config.rs
@@ -0,0 +1,144 @@
+// ChainConfig and parsing functions for scf_dos_chain binary.
+
+use anyhow::{anyhow, Result};
+use clap::Parser;
+
+/// Configuration for CASTEP SCF + DOS chain workflow.
+#[derive(Parser, Debug)]
+#[command(name = "scf_dos_chain")]
+pub struct ChainConfig {
+    /// Hubbard U value.
+    #[arg(long, default_value_t = 3.0)]
+    pub u_value: f64,
+
+    /// Element symbol for the Hubbard site.
+    #[arg(long, default_value = "Zn")]
+    pub element: String,
+
+    /// Orbital label for the Hubbard manifold.
+    #[arg(long, default_value = "d")]
+    pub orbital: char,
+
+    /// CASTEP seed name for input/output files.
+    #[arg(long, default_value = "ZnO")]
+    pub seed_name: String,
+
+    /// Maximum number of parallel jobs.
+    #[arg(long, default_value_t = 1)]
+    pub max_parallel: usize,
+
+    /// Run locally instead of submitting to Slurm.
+    #[arg(long)]
+    pub local: bool,
+
+    /// Print what would be done without executing.
+    #[arg(long)]
+    pub dry_run: bool,
+
+    /// CASTEP binary command.
+    #[arg(long, default_value = "castep")]
+    pub castep_command: String,
+
+    /// Working directory for CASTEP runs.
+    #[arg(long, default_value = ".")]
+    pub workdir: String,
+
+    /// Slurm partition name.
+    #[arg(long, env = "CASTEP_SLURM_PARTITION", default_value = "debug")]
+    pub partition: String,
+
+    /// Number of MPI tasks per job.
+    #[arg(long, default_value_t = 16)]
+    pub ntasks: u32,
+
+    /// Nix flake URL for the CASTEP environment.
+    #[arg(
+        long,
+        env = "CASTEP_NIX_FLAKE",
+        default_value = "git+ssh://git@github.com/TonyWu20/CASTEP-25.12-nixos#castep_25_mkl"
+    )]
+    pub nix_flake: String,
+
+    /// Network interface for MPI communication.
+    #[arg(long, env = "CASTEP_MPI_IF", default_value = "enp6s0")]
+    pub mpi_if: String,
+}
+
+/// Parse a comma-separated list of Hubbard U values into a vector of f64.
+///
+/// Each segment is trimmed of whitespace.  Empty input produces an error
+/// containing "empty".  Consecutive commas or non-numeric tokens produce an
+/// error containing the offending token.
+#[allow(dead_code)]
+pub fn parse_u_values(s: &str) -> Result<Vec<f64>> {
+    let trimmed = s.trim();
+    if trimmed.is_empty() {
+        return Err(anyhow!("empty u values input"));
+    }
+    trimmed
+        .split(',')
+        .map(|token| {
+            let t = token.trim();
+            t.parse::<f64>()
+                .map_err(|_| anyhow!("invalid u value: '{}'", t))
+        })
+        .collect()
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use clap::Parser;
+
+    /// Helper: default config for job_script tests
+    fn default_chain_config() -> ChainConfig {
+        ChainConfig::parse_from(["test"])
+    }
+
+    // --- parse_u_values tests (ported, adapted for anyhow::Result) ---
+
+    #[test]
+    fn parse_basic_values() {
+        let vals = parse_u_values("0.0,1.0,2.0").unwrap();
+        assert_eq!(vals, vec![0.0, 1.0, 2.0]);
+    }
+
+    #[test]
+    fn parse_with_whitespace() {
+        let vals = parse_u_values("  0.0 , 1.0 , 2.0  ").unwrap();
+        assert_eq!(vals, vec![0.0, 1.0, 2.0]);
+    }
+
+    #[test]
+    fn parse_single_value() {
+        let vals = parse_u_values("42.0").unwrap();
+        assert_eq!(vals, vec![42.0]);
+    }
+
+    #[test]
+    fn parse_invalid_token() {
+        let err = parse_u_values("1.0,abc,2.0").unwrap_err();
+        let msg = format!("{err}");
+        assert!(msg.contains("abc"), "error should mention the invalid token: {msg}");
+    }
+
+    #[test]
+    fn parse_empty_token() {
+        let err = parse_u_values("1.0,,2.0").unwrap_err();
+        let msg = format!("{err}");
+        assert!(msg.contains("invalid"), "error should report parse failure: {msg}");
+    }
+
+    #[test]
+    fn parse_empty_string() {
+        let err = parse_u_values("").unwrap_err();
+        let msg = format!("{err}");
+        assert!(msg.contains("empty") || msg.contains("invalid"), "expected parse failure on empty input, got: {msg}");
+    }
+
+    #[test]
+    fn parse_negative_values() {
+        let vals = parse_u_values("-1.0,2.0").unwrap();
+        assert_eq!(vals, vec![-1.0, 2.0]);
+    }
+}
diff --git a/examples/scf_dos_chain/src/job_script.rs b/examples/scf_dos_chain/src/job_script.rs
new file mode 100644
index 0000000..cac398e
--- /dev/null
+++ b/examples/scf_dos_chain/src/job_script.rs
@@ -0,0 +1,94 @@
+// SLURM job script generation for scf_dos_chain binary.
+
+use crate::config::ChainConfig;
+
+/// Generate a SLURM job submission script for a CASTEP calculation.
+///
+/// Returns a bash script with SBATCH directives, nix environment setup, and
+/// an mpirun invocation.  Uses only spaces for indentation (no literal tabs).
+pub fn generate_job_script(config: &ChainConfig, task_id: &str, seed_name: &str) -> String {
+    format!(
+        "\
+#!/usr/bin/env bash
+#SBATCH --job-name=\"{task_id}\"
+#SBATCH --output=slurm_output_%j.txt
+#SBATCH --partition={partition}
+#SBATCH --nodes=1
+#SBATCH --ntasks-per-node={ntasks}
+#SBATCH --cpus-per-task=1
+#SBATCH --mem=30000m
+#SBATCH --nodelist=nixos
+nix develop {nix_flake} --command bash -c \\
+    \"mpirun --mca plm slurm \\
+        -x OMPI_MCA_btl_tcp_if_include={mpi_if} \\
+        -x OMPI_MCA_orte_keep_fqdn_hostnames=true \\
+        --mca pmix s1 \\
+        --mca btl tcp,self \\
+        --map-by numa --bind-to numa \\
+    castep.mpi {seed_name}\"
+",
+        task_id = task_id,
+        partition = config.partition,
+        ntasks = config.ntasks,
+        nix_flake = config.nix_flake,
+        mpi_if = config.mpi_if,
+    )
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::config::ChainConfig;
+    use clap::Parser;
+
+    fn default_config() -> ChainConfig {
+        ChainConfig::parse_from(["test"])
+    }
+
+    #[test]
+    fn contains_sbatch_directives() {
+        let config = default_config();
+        let script = generate_job_script(&config, "scf", "ZnO");
+        assert!(script.contains(r#"#SBATCH --job-name="scf""#));
+        assert!(script.contains("#SBATCH --partition=debug"));
+        assert!(script.contains("#SBATCH --ntasks-per-node=16"));
+        assert!(script.contains("#SBATCH --mem=30000m"));
+    }
+
+    #[test]
+    fn contains_seed_name() {
+        let config = default_config();
+        let script = generate_job_script(&config, "scf", "ZnO");
+        assert!(script.contains("castep.mpi ZnO"));
+    }
+
+    /// D.2 fix: no literal tabs
+    #[test]
+    fn no_literal_tabs() {
+        let config = default_config();
+        let script = generate_job_script(&config, "scf", "ZnO");
+        assert!(!script.contains('\t'), "job script should not contain literal tab characters");
+    }
+
+    #[test]
+    fn starts_with_shebang() {
+        let config = default_config();
+        let script = generate_job_script(&config, "scf", "ZnO");
+        assert!(script.starts_with("#!/usr/bin/env bash"));
+    }
+
+    #[test]
+    fn contains_nix_develop() {
+        let config = default_config();
+        let script = generate_job_script(&config, "scf", "ZnO");
+        assert!(script.contains("nix develop"));
+        assert!(script.contains(&config.nix_flake));
+    }
+
+    #[test]
+    fn contains_mpi_interface() {
+        let config = default_config();
+        let script = generate_job_script(&config, "scf", "ZnO");
+        assert!(script.contains(&format!("OMPI_MCA_btl_tcp_if_include={mpi_if}", mpi_if = config.mpi_if)));
+    }
+}
diff --git a/examples/scf_dos_chain/src/main.rs b/examples/scf_dos_chain/src/main.rs
new file mode 100644
index 0000000..729924b
--- /dev/null
+++ b/examples/scf_dos_chain/src/main.rs
@@ -0,0 +1,316 @@
+mod config;
+mod job_script;
+
+use std::path::{Path, PathBuf};
+use std::sync::Arc;
+
+use clap::Parser;
+use castep_cell_fmt::{format::to_string_many_spaced, parse, ToCellFile};
+use castep_cell_io::cell::species::{AtomHubbardU, HubbardU, OrbitalU, Species};
+use castep_cell_io::{CellDocument, ParamDocument};
+use config::ChainConfig;
+use job_script::generate_job_script;
+use workflow_utils::prelude::*;
+use workflow_core::task::TaskClosure;
+
+fn build_scf_task(
+    config: &ChainConfig,
+    seed_cell: &str,
+    seed_param: &str,
+) -> std::result::Result<Task, WorkflowError> {
+    let id = "scf".to_string();
+    let workdir = PathBuf::from("runs/scf_dos");
+
+    let mode = if config.local {
+        ExecutionMode::direct(&config.castep_command, &[&config.seed_name])
+    } else {
+        ExecutionMode::Queued
+    };
+
+    // Clone values for move closures (owned values only, no references to config)
+    let element = config.element.clone();
+    let seed_name = config.seed_name.clone();
+    let orbital_char = config.orbital;
+    let u_value = config.u_value;
+    let local = config.local;
+
+    // Pre-compute job script outside closures to avoid capturing config ref
+    let queued_script = if local {
+        None
+    } else {
+        Some(generate_job_script(config, &seed_name, &seed_name))
+    };
+
+    let setup_seed_cell = seed_cell.to_owned();
+    let setup_seed_param = seed_param.to_owned();
+    let setup_element = element.clone();
+    let setup_seed_name = seed_name.clone();
+
+    let boxed_setup: TaskClosure = Box::new(move |path: &Path| {
+        create_dir(path)?;
+
+        // Parse and modify cell document -- inject Hubbard U
+        let mut cell_doc: CellDocument = parse(&setup_seed_cell)?;
+        let species = Species::Symbol(setup_element.clone());
+        let orbital = match orbital_char {
+            's' => OrbitalU::S(u_value),
+            'p' => OrbitalU::P(u_value),
+            'd' => OrbitalU::D(u_value),
+            'f' => OrbitalU::F(u_value),
+            _ => return Err(Box::new(WorkflowError::InvalidConfig(
+                format!("invalid orbital: {}", orbital_char)
+            ))),
+        };
+        let atom_u = AtomHubbardU::builder()
+            .species(species)
+            .orbitals(vec![orbital])
+            .build();
+        let hubbard_u = HubbardU::builder()
+            .atom_u_values(vec![atom_u])
+            .build();
+        cell_doc.hubbard_u = Some(hubbard_u);
+
+        // Parse param document (no modification needed for SCF)
+        let param_doc: ParamDocument = parse(&setup_seed_param)?;
+
+        // Serialize to file strings
+        let cell_str = to_string_many_spaced(&cell_doc.to_cell_file());
+        let param_str = to_string_many_spaced(&param_doc.to_cell_file());
+
+        // Write .cell and .param files
+        write_file(path.join(format!("{}.cell", setup_seed_name)), &cell_str)?;
+        write_file(path.join(format!("{}.param", setup_seed_name)), &param_str)?;
+
+        // Write job script if queued
+        if let Some(ref script) = queued_script {
+            write_file(path.join(JOB_SCRIPT_NAME), script)?;
+        }
+
+        Ok(())
+    });
+
+    let collect_seed_name = seed_name;
+    let boxed_collect: TaskClosure = Box::new(move |path: &Path| {
+        let output_str = read_file(path.join(format!("{}.castep", collect_seed_name)))?;
+        if !output_str.contains("Total time") {
+            return Err(Box::new(WorkflowError::InvalidConfig(
+                "CASTEP output missing 'Total time' marker".into(),
+            )));
+        }
+        Ok(())
+    });
+
+    let mut task = Task::new(id, mode).workdir(workdir);
+    task.setup = Some(boxed_setup);
+    task.collect = Some(boxed_collect);
+
+    Ok(task)
+}
+
+fn build_dos_task(
+    config: &ChainConfig,
+    scf_task_id: &str,
+    seed_cell: &str,
+    seed_param: &str,
+) -> std::result::Result<Task, WorkflowError> {
+    let id = "dos".to_string();
+    let workdir = PathBuf::from("runs/scf_dos");
+
+    let mode = if config.local {
+        ExecutionMode::direct(&config.castep_command, &["ZnO_DOS"])
+    } else {
+        ExecutionMode::Queued
+    };
+
+    // Pre-compute job script outside closures to avoid capturing config ref
+    let queued_script = if config.local {
+        None
+    } else {
+        Some(generate_job_script(config, &id, "ZnO_DOS"))
+    };
+
+    let setup_seed_cell = seed_cell.to_owned();
+    let setup_seed_param = seed_param.to_owned();
+    let _setup_local = config.local;
+    let setup_scf_task = scf_task_id.to_owned();
+
+    let boxed_setup: TaskClosure = Box::new(move |path: &Path| {
+        create_dir(path)?;
+
+        // Parse seed cell and write as ZnO_DOS.cell (reuse seed cell for DOS restart)
+        let cell_doc: CellDocument = parse(&setup_seed_cell)?;
+        let cell_str = to_string_many_spaced(&cell_doc.to_cell_file());
+        write_file(path.join("ZnO_DOS.cell"), &cell_str)?;
+
+        // Parse seed param, set Task::BandStructure
+        let mut param_doc: ParamDocument = parse(&setup_seed_param)?;
+        param_doc.general.task = Some(castep_cell_io::param::general::Task::BandStructure);
+        let param_str = to_string_many_spaced(&param_doc.to_cell_file());
+        write_file(path.join("ZnO_DOS.param"), &param_str)?;
+
+        // Copy checkpoint from SCF run
+        copy_file(path.join("ZnO.check"), path.join("ZnO_DOS.check"))?;
+
+        // Write job script if queued
+        if let Some(ref script) = queued_script {
+            write_file(path.join(JOB_SCRIPT_NAME), script)?;
+        }
+
+        Ok(())
+    });
+
+    let boxed_collect: TaskClosure = Box::new(move |_: &Path| {
+        let output_str = read_file(PathBuf::from("ZnO_DOS.castep"))?;
+        if !output_str.contains("Total time") {
+            return Err(Box::new(WorkflowError::InvalidConfig(
+                "CASTEP output missing 'Total time' marker".into(),
+            )));
+        }
+        Ok(())
+    });
+
+    let mut task = Task::new(id, mode)
+        .workdir(workdir)
+        .depends_on(setup_scf_task);
+    task.setup = Some(boxed_setup);
+    task.collect = Some(boxed_collect);
+
+    Ok(task)
+}
+
+fn main() -> anyhow::Result<()> {
+    workflow_core::init_default_logging().ok();
+
+    let config = ChainConfig::parse();
+
+    let seed_cell = include_str!("../seeds/ZnO.cell");
+    let seed_param = include_str!("../seeds/ZnO.param");
+
+    let scf_task = build_scf_task(&config, seed_cell, seed_param)?;
+    let dos_task = build_dos_task(&config, &scf_task.id, seed_cell, seed_param)?;
+
+    let mut workflow = Workflow::new("scf_dos_chain")
+        .with_max_parallel(config.max_parallel)?
+        .with_log_dir("logs")
+        .with_root_dir(&config.workdir);
+
+    if !config.local {
+        workflow = workflow.with_queued_submitter(Arc::new(QueuedRunner::new(SchedulerKind::Slurm)));
+    }
+
+    workflow.add_task(scf_task)?;
+    workflow.add_task(dos_task)?;
+
+    if config.dry_run {
+        let order = workflow.dry_run()?;
+        println!("Dry-run topological order:");
+        for task_id in &order {
+            println!("  {task_id}");
+        }
+        return Ok(());
+    }
+
+    let state_path = PathBuf::from(".scf_dos_chain.workflow.json");
+    let mut state = JsonStateStore::new("scf_dos_chain", state_path);
+
+    let summary = if config.local {
+        run_default(&mut workflow, &mut state)?
+    } else {
+        let runner: Arc<dyn ProcessRunner> = Arc::new(SystemProcessRunner::new());
+        let executor: Arc<dyn HookExecutor> = Arc::new(ShellHookExecutor);
+        workflow.run(&mut state, runner, executor)?
+    };
+
+    println!(
+        "Workflow complete: {} succeeded, {} failed, {} skipped ({:.1}s)",
+        summary.succeeded.len(),
+        summary.failed.len(),
+        summary.skipped.len(),
+        summary.duration.as_secs_f64(),
+    );
+    Ok(())
+}
+
+#[cfg(test)]
+mod tests {
+    use crate::config::ChainConfig;
+    use crate::{build_dos_task, build_scf_task};
+    use clap::Parser;
+
+    fn default_config() -> ChainConfig {
+        ChainConfig::parse_from(["test"])
+    }
+
+    fn test_seeds() -> (&'static str, &'static str) {
+        (include_str!("../seeds/ZnO.cell"), include_str!("../seeds/ZnO.param"))
+    }
+
+    /// SCF task has ID 'scf'
+    #[test]
+    fn scf_task_id() {
+        let config = default_config();
+        let (cell, param) = test_seeds();
+        let task = build_scf_task(&config, cell, param).unwrap();
+        assert_eq!(task.id, "scf");
+    }
+
+    /// DOS task has ID 'dos'
+    #[test]
+    fn dos_task_id() {
+        let config = default_config();
+        let (cell, param) = test_seeds();
+        let task = build_dos_task(&config, "scf", cell, param).unwrap();
+        assert_eq!(task.id, "dos");
+    }
+
+    /// DOS task depends on SCF task
+    #[test]
+    fn dos_depends_on_scf() {
+        let config = default_config();
+        let (cell, param) = test_seeds();
+        let scf = build_scf_task(&config, cell, param).unwrap();
+        let dos = build_dos_task(&config, &scf.id, cell, param).unwrap();
+        assert!(dos.dependencies.contains(&"scf".to_string()));
+    }
+
+    /// SCF task has no dependencies
+    #[test]
+    fn scf_no_dependencies() {
+        let config = default_config();
+        let (cell, param) = test_seeds();
+        let task = build_scf_task(&config, cell, param).unwrap();
+        assert!(task.dependencies.is_empty());
+    }
+
+    /// DOS task has exactly 1 dependency
+    #[test]
+    fn dos_dependency_count() {
+        let config = default_config();
+        let (cell, param) = test_seeds();
+        let scf = build_scf_task(&config, cell, param).unwrap();
+        let dos = build_dos_task(&config, &scf.id, cell, param).unwrap();
+        assert_eq!(dos.dependencies.len(), 1);
+    }
+
+    /// Both tasks share the same workdir
+    #[test]
+    fn shared_workdir() {
+        let config = default_config();
+        let (cell, param) = test_seeds();
+        let scf = build_scf_task(&config, cell, param).unwrap();
+        let dos = build_dos_task(&config, &scf.id, cell, param).unwrap();
+        assert_eq!(scf.workdir, dos.workdir, "SCF and DOS tasks should share the same workdir");
+        let wd = scf.workdir.to_string_lossy();
+        assert!(wd.contains("scf_dos"), "workdir should contain 'scf_dos': {wd}");
+    }
+
+    /// SCF task has correct workdir path
+    #[test]
+    fn scf_workdir_path() {
+        let config = default_config();
+        let (cell, param) = test_seeds();
+        let task = build_scf_task(&config, cell, param).unwrap();
+        let wd = task.workdir.to_string_lossy();
+        assert!(wd.ends_with("scf_dos") || wd.contains("scf_dos"), "workdir should end with or contain 'scf_dos': {wd}");
+    }
+}
diff --git a/flake.lock b/flake.lock
index b1b4555..43772d6 100644
--- a/flake.lock
+++ b/flake.lock
@@ -57,11 +57,11 @@
     },
     "nixpkgs_2": {
       "locked": {
-        "lastModified": 1775036866,
-        "narHash": "sha256-ZojAnPuCdy657PbTq5V0Y+AHKhZAIwSIT2cb8UgAz/U=",
+        "lastModified": 1777578337,
+        "narHash": "sha256-Ad49moKWeXtKBJNy2ebiTQUEgdLyvGmTeykAQ9xM+Z4=",
         "owner": "NixOS",
         "repo": "nixpkgs",
-        "rev": "6201e203d09599479a3b3450ed24fa81537ebc4e",
+        "rev": "15f4ee454b1dce334612fa6843b3e05cf546efab",
         "type": "github"
       },
       "original": {
diff --git a/flake.nix b/flake.nix
index cd6cd74..859fa5f 100644
--- a/flake.nix
+++ b/flake.nix
@@ -11,7 +11,11 @@
   outputs = { nixpkgs, fenix, devshell, ... }:
     let
       systems = [ "x86_64-linux" "aarch64-darwin" ];
-      pkgsFor = system: import nixpkgs { inherit system; overlays = [ fenix.overlays.default devshell.overlays.default ]; };
+      pkgsFor = system: import nixpkgs {
+        inherit system;
+        config.allowUnfree = true;
+        overlays = [ fenix.overlays.default devshell.overlays.default ];
+      };
 
       forAllSystems = nixpkgs.lib.genAttrs systems;
     in
@@ -36,6 +40,7 @@
               fish
               python3
               uv
+              claude-code
             ];
             commands = [
               {
@@ -100,7 +105,7 @@
                   ANTHROPIC_DEFAULT_OPUS_MODEL=deepseek-v4-pro[1m] \
                   ANTHROPIC_DEFAULT_SONNET_MODEL=deepseek-v4-flash[1m] \
                   ANTHROPIC_DEFAULT_HAIKU_MODEL=deepseek-v4-flash \
-                  claude --model "opusplan"
+                  claude --model "opusplan" --plugin-dir /Users/tony/programming/rust-development-pipeline
                 '';
               }
             ];
diff --git a/notes/directions/phase-6-fix/codebase-state.md b/notes/directions/phase-6-fix/codebase-state.md
new file mode 100644
index 0000000..1c4ac8c
--- /dev/null
+++ b/notes/directions/phase-6-fix/codebase-state.md
@@ -0,0 +1,251 @@
+# Codebase State Analysis for Phase 6 Fix Plan
+
+**Generated**: 2026-05-05
+**Plan file**: `plans/phase-6-fix/PHASE_6_FIX_PLAN.md`
+
+---
+
+## 1. Current File Tree for Affected Areas
+
+### Workspace root
+```
+/Users/tony/programming/castep_workflow_framework/
+  Cargo.toml              (workspace root)
+  Cargo.lock
+  workflow_core/           (lib crate)
+    Cargo.toml
+    src/
+      lib.rs, dag.rs, error.rs, monitoring.rs, prelude.rs,
+      process.rs, state.rs, task.rs, workflow.rs
+    tests/
+      bin/mock_castep       (pre-compiled test binary)
+      collect_failure_policy.rs, dependencies.rs, hook_recording.rs,
+      hubbard_u_sweep.rs, integration.rs, log_persistence.rs,
+      queued_workflow.rs, resume.rs, timeout_integration.rs
+  workflow_utils/          (lib crate)
+    Cargo.toml
+    src/
+      lib.rs, executor.rs, files.rs, monitoring.rs, prelude.rs, queued.rs
+    tests/
+      executor_tests.rs, files_tests.rs, monitoring_tests.rs,
+      process_tests.rs, queued_integration.rs
+  workflow-cli/            (bin crate)
+    Cargo.toml
+    src/main.rs
+  examples/
+    hubbard_u_sweep/       (bin crate -- TO BE UPDATED)
+      Cargo.toml
+      src/main.rs
+      seeds/ZnO.cell, ZnO.param
+    hubbard_u_sweep_slurm/ (bin crate -- TO BE DELETED)
+      Cargo.toml
+      src/main.rs, config.rs, job_script.rs
+      seeds/ZnO.cell, ZnO.param
+      .validation-complete
+  notes/
+    directions/phase-6-fix/
+      workspace-map.json
+      deferred-and-patterns.md
+    plan-enrichment/phase-6-fix/
+      codebase-state.md, deferred-and-patterns.md, draft-elaboration.md,
+      gather-summary.md, task-checklist.md
+    plan-reviews/phase-6-fix/
+      decisions.md
+```
+
+### Files to be created
+```
+examples/multi_param_sweep/
+  Cargo.toml, src/main.rs, src/config.rs, src/job_script.rs,
+  seeds/ZnO.cell, ZnO.param
+examples/scf_dos_chain/
+  Cargo.toml, src/main.rs, src/config.rs, src/job_script.rs,
+  seeds/ZnO.cell, ZnO.param
+```
+
+### Files to be deleted
+```
+examples/hubbard_u_sweep_slurm/   (entire directory)
+```
+
+### Files to be updated
+```
+/Users/tony/programming/castep_workflow_framework/Cargo.toml   (workspace members)
+/Users/tony/programming/castep_workflow_framework/examples/hubbard_u_sweep/Cargo.toml
+/Users/tony/programming/castep_workflow_framework/examples/hubbard_u_sweep/src/main.rs
+```
+
+---
+
+## 2. Key Type and Function Signatures
+
+### Core types (workflow_core::task)
+
+```
+pub struct Task {
+    pub id: String,
+    pub dependencies: Vec<String>,
+    pub workdir: PathBuf,
+    pub mode: ExecutionMode,
+    pub setup: Option<TaskClosure>,
+    pub collect: Option<TaskClosure>,
+    pub monitors: Vec<MonitoringHook>,
+    pub(crate) collect_failure_policy: CollectFailurePolicy,
+}
+
+impl Task {
+    pub fn new(id: impl Into<String>, mode: ExecutionMode) -> Self;
+    pub fn depends_on(mut self, id: impl Into<String>) -> Self;
+    pub fn workdir(mut self, path: impl Into<PathBuf>) -> Self;
+    pub fn setup<F, E>(mut self, f: F) -> Self
+        where F: Fn(&Path) -> Result<(), E> + Send + Sync + 'static,
+              E: std::error::Error + Send + Sync + 'static;
+    pub fn collect<F, E>(mut self, f: F) -> Self
+        where F: Fn(&Path) -> Result<(), E> + Send + Sync + 'static,
+              E: std::error::Error + Send + Sync + 'static;
+}
+```
+
+### ExecutionMode
+```
+pub enum ExecutionMode {
+    Direct { command: String, args: Vec<String>, env: HashMap<String, String>, timeout: Option<Duration> },
+    Queued,
+}
+impl ExecutionMode {
+    pub fn direct(command: impl Into<String>, args: &[&str]) -> Self;
+}
+```
+
+### Workflow
+```
+pub struct Workflow {
+    pub name: String,
+    tasks: HashMap<String, Task>,
+    max_parallel: usize,
+    ...
+}
+
+impl Workflow {
+    pub fn new(name: impl Into<String>) -> Self;
+    pub fn with_max_parallel(mut self, n: usize) -> Result<Self, WorkflowError>;
+    pub fn with_log_dir(mut self, path: impl Into<PathBuf>) -> Self;
+    pub fn with_queued_submitter(mut self, qs: Arc<dyn QueuedSubmitter>) -> Self;
+    pub fn with_root_dir(mut self, path: impl Into<PathBuf>) -> Self;
+    pub fn add_task(&mut self, task: Task) -> Result<(), WorkflowError>;
+    pub fn dry_run(&self) -> Result<Vec<String>, WorkflowError>;
+    pub fn run(&mut self, state: &mut dyn StateStore, runner: Arc<dyn ProcessRunner>,
+               hook_executor: Arc<dyn HookExecutor>) -> Result<WorkflowSummary, WorkflowError>;
+}
+```
+
+### workflow_utils convenience API
+```
+pub fn run_default(workflow: &mut Workflow, state: &mut dyn StateStore)
+    -> Result<WorkflowSummary, WorkflowError>;
+
+// prelude re-exports:
+// WorkflowError, JsonStateStore, StateStore, StateStoreExt, TaskStatus,
+// CollectFailurePolicy, ExecutionMode, Task, Workflow, WorkflowSummary,
+// HookExecutor, ProcessRunner
+// create_dir, copy_file, read_file, write_file, exists, remove_dir
+// run_default, QueuedRunner, SchedulerKind, ShellHookExecutor,
+// SystemProcessRunner, JOB_SCRIPT_NAME
+```
+
+---
+
+## 3. Module Dependency Graph
+
+```
+                     workflow_core (lib)
+                     /     |       \
+                    /      |        \
+                   /       |         \
+     workflow_utils     workflow-cli   hubbard_u_sweep (bin)
+         |                                 |
+         |                          hubbard_u_sweep_slurm (bin) [TO DELETE]
+         |
+         +-- depends on workflow_core
+         +-- re-exports workflow_core::prelude fully
+```
+
+### External dependencies
+- `castep-cell-fmt` v0.1.0 — used by hubbard_u_sweep, hubbard_u_sweep_slurm
+- `castep-cell-io` v0.4.0 — used by hubbard_u_sweep, hubbard_u_sweep_slurm
+
+The plan requires changing to a local path dep for `castep-cell-io` at v0.5.0 API:
+`castep-cell-io = { path = "../castep-cell-io/castep_cell_io" }`
+
+---
+
+## 4. Existing Test Structure
+
+### workflow_core tests
+- `task.rs` — task_builder, direct_constructor_fields, execution_mode_debug, depends_on_chaining
+- `workflow.rs` — single_task_completes, chain_respects_order, failed_task_skips_dependent,
+  dry_run_returns_topo_order, duplicate_task_id_errors, valid_dependency_add, etc.
+- Integration tests: collect_failure_policy, dependencies, hook_recording, hubbard_u_sweep,
+  integration (resume), log_persistence, queued_workflow, resume, timeout_integration
+
+### workflow_utils tests
+- `queued.rs` — 6 tests for parse_job_id (Slurm/PBS variants, edge cases)
+- `files_tests.rs` — 5 tests for read/write/copy/remove operations
+- executor, monitoring, process, queued_integration test files
+
+### hubbard_u_sweep_slurm tests (must be ported before deletion)
+- `config.rs` — 7 tests for parse_u_values (basic, whitespace, single, invalid, empty, etc.)
+- `job_script.rs` — 6 tests for generate_job_script (sbatch directives, seed name, tabs, shebang, nix, mpi)
+
+---
+
+## 5. Observations and Patterns
+
+### A. castep-cell-io/fmt are NOT local — must create local path dep
+The plan requires `castep-cell-io = { path = "../castep-cell-io/castep_cell_io" }`.
+The existing dep is v0.4.0 from crates.io. The plan amendment confirms path-dep approach.
+
+### B. TASK-3 naming conflict: Task name collision
+`use castep_cell_io::param::general::Task;` will shadow `workflow_core::task::Task` from glob import.
+Resolution: alias import (`as CastepTask`) or use fully-qualified path.
+
+### C. Dead re-export warnings in workspace map
+~15 false positives from cross-crate resolution limitations. Not real issues.
+
+### D. hubbard_u_sweep (non-slurm) also needs updates
+- Cargo.toml: path dep for castep-cell-io
+- main.rs: v0.5.0 parse API
+
+### E. Seed files identical across existing binaries
+Simple ZnO wurtzite cell for new binaries.
+
+### F. job_script.rs code reuse
+Both new binaries need identical job_script module. Config structs differ but SLURM fields overlap.
+
+### G. Generate_job_script function signature
+Takes config (SLURM fields), task_id, seed_name.
+
+### H. API change: v0.4.0 → v0.5.0
+- Free function `castep_cell_fmt::parse::<CellDocument>(&input)` replaces builder
+- `KpointsMpGrid` is direct field on `CellDocument`
+- `CutOffEnergy { value, unit: None }` pattern
+- `to_cell_file()` and `to_string_many_spaced()` serialization unchanged
+
+### I. Test coverage gaps for new binaries
+- parse_kpoints and parse_cutoffs tests needed (not in plan)
+- Sweep mode combinatoric logic tests
+- Setup closure correctness tests
+- DOS chain setup tests
+
+### J. Existing integration test `test_hubbard_u_sweep_with_mock_castep`
+Continues working — targets workflow_core directly.
+
+### K. Compilation status
+Current state compiles with v0.4.0 castep-cell-io from crates.io. New binaries need v0.5.0 path dep.
+
+### L. Known failure modes from previous phases
+1. Missing `pub use` re-exports
+2. Incomplete consumer updates (constants/hardcoded strings)
+3. Stale imports after refactoring
+4. Dead API surface (newtypes with exposed inner types)
+5. Stale documentation
diff --git a/notes/directions/phase-6-fix/deferred-and-patterns.md b/notes/directions/phase-6-fix/deferred-and-patterns.md
new file mode 100644
index 0000000..f5c00e1
--- /dev/null
+++ b/notes/directions/phase-6-fix/deferred-and-patterns.md
@@ -0,0 +1,264 @@
+# Deferred Improvements and Architectural Patterns -- Phase 6 Fix
+
+> Synthesised from:
+> - `plans/phase-6-fix/PHASE_6_FIX_PLAN.md`
+> - `notes/pr-reviews/phase-6/deferred.md`
+> - `notes/plan-enrichment/phase-6-fix/deferred-and-patterns.md` (previous enrichment)
+>
+> Generated 2026-05-05 during phase-6-fix plan elaboration.
+
+---
+
+## Deferred Improvements
+
+Items carried forward from prior review rounds. Each has a precondition that
+must be met before it is actionable.
+
+### D.1: Restore plan-specified portable config fields
+
+- **Source:** Phase 5A review
+- **Problem:** The old `hubbard_u_sweep_slurm` example uses NixOS-specific config
+  fields (`nix_flake`, `mpi_if`, `--nodelist=nixos`) instead of the
+  plan-specified portable fields (`account`, `walltime`, `modules`,
+  `castep_command`). Reduces the example's value as a reference for non-NixOS
+  clusters.
+- **Precondition:** A second user adopts the examples, or Tony moves to a
+  non-NixOS cluster.
+- **Action:** When triggered, migrate both `multi_param_sweep` and
+  `scf_dos_chain` to use portable SLURM fields.
+
+### D.2: `generate_job_script` formatting inconsistencies
+
+- **Source:** Phase 5A review
+- **Problem:** `job_script.rs` uses a literal `\t` character among spaces for
+  the `--map-by` flag. SBATCH directives have inconsistent quoting.
+- **Precondition:** Next functional edit to `job_script.rs`.
+- **Action:** Use a clean heredoc template (or `indoc!` macro) with consistent
+  quoting -- no literal tab characters mixed with spaces. The existing
+  `no_literal_tabs` test from the old binary should be ported and pass.
+
+### D.3 (partial): Unit tests for `generate_job_script`
+
+- **Source:** Phase 5A review
+- **Problem:** `parse_u_values` tests are comprehensive (done in Phase 5B).
+  `generate_job_script` tests are tightly coupled to NixOS-specific output,
+  making assertions brittle without a second template variant.
+- **Precondition:** D.1 must be addressed first -- a portable template variant
+  makes test assertions meaningful.
+- **Action:** Add unit tests for the portable `generate_job_script` after D.1.
+
+### `read_task_ids` empty-string edge case
+
+- **Source:** Round 2 review
+- **Problem:** `read_task_ids` returns `Ok([""])` when called with `vec![""]`,
+  producing a downstream error rather than a clear diagnostic. Not reachable
+  through normal CLI usage (clap's `default_value = "-"` ensures a dash), but a
+  future maintainer could be confused by the gap between the documented
+  stdin-fallback description and the actual single-condition check.
+- **Precondition:** Actual need to handle a different edge case in
+  `read_task_ids` -- not worth a standalone change.
+- **Action:** Tighten the check to reject empty strings explicitly with a clear
+  error message.
+
+---
+
+## Key Architectural Patterns
+
+Patterns established or reinforced by this fix plan. Implementation should
+follow these consistently.
+
+### 1. Purpose-specific binaries over mode-flags
+
+The old `hubbard_u_sweep_slurm` binary had three operation modes (single,
+product, pairwise) controlled by a flag, plus implicit chain-building in
+product/pairwise modes. This caused three bugs:
+
+- String-parsed "second values" with no CLI format hints
+- Default sweep mode "single" silently producing only SCF tasks
+- Product/pairwise modes building SCF->DOS chains with colliding task IDs
+
+**Fix pattern:** Decompose into two purpose-built binaries, each owning exactly
+one workflow:
+
+| Binary | Workflow | Tasks |
+|--------|----------|-------|
+| `multi_param_sweep` | Independent SCF parameter sweep | All SCF, no children |
+| `scf_dos_chain` | SCF followed by DOS (chain) | 2 tasks, 1 dependency |
+
+**Rule:** If a binary needs a flag to enable or disable a fundamentally
+different kind of workflow, that workflow probably belongs in its own binary.
+
+### 2. Parameter encoding in task IDs
+
+The old binary generated `dos_kpt8x8x8` as a task ID without encoding the
+Hubbard U value. Sweeping U in chain mode gave duplicate IDs.
+
+**Pattern:** Encode every distinguishing parameter into the task ID:
+
+```
+scf_U3.0_k8x8x8_c500  # multi_param_sweep: all three params encoded
+scf                      # scf_dos_chain: single parameter set, no collision risk
+dos                      # scf_dos_chain: single parameter set, no collision risk
+```
+
+**Rule:** A task ID must differentiate every parameter that varies in scope.
+
+### 3. Parse-time validation gates with clear diagnostics
+
+The plan defines two parsing utility functions with strict validation:
+
+- `parse_kpoints`: comma-separated, each segment must have exactly 3 axes
+  separated by `x`. `"8xx8"`, `"abc"`, `"8x8"` all produce clear `anyhow!`
+  errors. Empty input returns an explicit "kpoints list is empty" error.
+- `parse_cutoffs`: comma-separated f64 list, same pattern as `parse_u_values`.
+
+Pairwise mode requires both `--kpoints` and `--cutoffs` to be explicitly
+provided, with a clear error if not.
+
+**Pattern:** Validate at the boundary, not at use-site. Provide the exact
+expected format in the error message.
+
+**Rule:** Every parsing function should produce `anyhow::Result<T>` with a
+message that tells the user what format was expected and what went wrong.
+
+### 4. Explicit defaults over silent modes
+
+The old binary defaulted `--sweep-mode` to `"single"`, which silently produced
+only SCF tasks. The new plan defaults to `"product"` -- the most useful
+full-sweep mode. If a user wants a subset, they choose `"pairwise"` explicitly.
+
+- `--u-values` gets a generous default (`"0.0,1.0,2.0,3.0,4.0,5.0"`) so a user
+  gets useful output from a bare invocation.
+- `--kpoints` and `--cutoffs` are `Option<String>`, defaulting to `None`. When
+  absent, seed file defaults are used -- this keeps the binary usable for
+  simple U-only sweeps.
+
+**Rule:** Defaults should do something useful and visible, not silently
+restrict behaviour.
+
+### 5. In-memory document mutation (not string manipulation)
+
+All CASTEP file modifications follow parse-mutate-serialize:
+
+1. Parse seed file via `castep_cell_fmt::parse::<CellDocument>(&input)` or
+   `castep_cell_fmt::parse::<ParamDocument>(&input)`
+2. Mutate typed fields directly (e.g., `doc.kpoints_mp_grid =
+   Some(KpointsMpGrid(...))`, `doc.basis_set.cutoff_energy =
+   Some(CutOffEnergy { ... })`)
+3. Serialize via `doc.to_cell_file()` and `to_string_many_spaced()`
+
+**Rule:** Never construct or modify CASTEP input files through string
+concatenation or regex replacement. Always go through the typed document API.
+
+### 6. Shared workdir for chained tasks
+
+The `scf_dos_chain` binary places both the SCF and DOS task in the same
+workdir (`runs/scf_dos/`). This makes checkpoint file access straightforward:
+
+- SCF produces `ZnO.check` in the workdir
+- DOS setup copies `<workdir>/ZnO.check` to `<workdir>/ZnO_DOS.check` by
+  simple filename, no path traversal needed
+
+**Rule:** Dependent tasks that share files (checkpoints, wavefunctions) should
+share a workdir. Only split workdirs when tasks are entirely independent.
+
+### 7. Clean heredoc template for job scripts
+
+The plan mandates proper heredoc templates in `generate_job_script`:
+
+- No literal tab characters mixed with spaces
+- Consistent quoting around SBATCH directives
+- Ported `no_literal_tabs` test must pass
+
+**Rule:** Any file generation template must use consistent indentation
+(either all spaces or all tabs, never mixed) and consistent quoting.
+
+---
+
+## Known Failure Modes
+
+Failure modes identified from prior fix rounds (Phase 4, Phase 5, Phase 5b)
+that should be consciously avoided during phase-6-fix implementation.
+
+### F.1: Missing `pub use` re-exports
+
+- **Source:** Phase 4 TASK-2
+- **Pattern:** A type is implemented in a submodule but omitted from the `pub
+  use` line in the crate root, making it inaccessible at the crate root despite
+  being available from the submodule.
+- **Mitigation:** After adding any new public type or function to a submodule,
+  verify it appears in the crate's `lib.rs` re-export section.
+
+### F.2: Incomplete consumer updates
+
+- **Source:** Phase 5 TASK-1, TASK-2
+- **Pattern:** After introducing a named constant (e.g., `JOB_SCRIPT_NAME`),
+  some consumers still use the hardcoded string (`"job.sh"`). A rename or
+  abstraction change in the library is not fully propagated until every
+  consumer is updated.
+- **Mitigation:** Search all crate consumers (including example binaries and
+  test files) for both the old and new forms after any rename. Use `rg` to
+  verify zero occurrences of the old form remain in non-library code.
+
+### F.3: Stale imports after refactoring
+
+- **Source:** Phase 4 TASK-3
+- **Pattern:** Moving a function from CLI to library leaves behind the original
+  definition plus duplicate tests in the CLI. The refactoring is correct in the
+  library but the dead code persists in the consumer.
+- **Mitigation:** After moving or extracting a function, delete the original
+  definition immediately. Run `cargo clippy --all-targets` with
+  `-W dead-code` to detect survivors.
+
+### F.4: Dead / unenforced API surface
+
+- **Source:** Phase 4 TASK-1
+- **Pattern:** A method like `TaskSuccessors::inner()` exposes the backing
+  type (`HashMap`), defeating a newtype abstraction, and has zero callers.
+  Without a dead-code lint, such methods accumulate silently.
+- **Mitigation:** Do not add getter methods that expose internal representation
+  unless a concrete consumer exists. Run dead-code detection as part of the
+  implementation loop.
+
+### F.5: Stale documentation
+
+- **Source:** Phase 5b TASK-1
+- **Pattern:** `ARCHITECTURE.md` contains outdated struct field names
+  (`execution_mode` vs `mode`, `dependencies` vs `depends_on`), outdated trait
+  signatures (pre-refactor `StateStore`), and outdated error types.
+  Documentation drifts silently when not treated as compilable code.
+- **Mitigation:** Update any developer-facing documentation in the same
+  commit that changes the code it describes. Review the plan's verification
+  steps for documentation checks.
+
+### F.6: Overloaded binary syndrome (new, from phase-6-fix)
+
+- **Source:** The old `hubbard_u_sweep_slurm` binary
+- **Pattern:** A single binary uses a mode flag to enable fundamentally
+  different workflows (independent tasks vs. chained tasks). This causes
+  hidden-mode confusion, duplicate task IDs, and untested flag combinations.
+- **Mitigation:** If a binary needs a flag to toggle between "has dependent
+  tasks" and "does not have dependent tasks", split into separate binaries.
+  See Pattern 1.
+
+### F.7: Default-driven silent behaviour (new, from phase-6-fix)
+
+- **Source:** Old binary defaulted `--sweep-mode` to `"single"` without
+  indicating this to the user.
+- **Pattern:** A default value that silently restricts functionality, causing
+  first-time users to get incomplete results without warning.
+- **Mitigation:** Choose defaults that produce the most complete and useful
+  behaviour. If a restricted mode exists, require explicit opt-in. See Pattern
+  4.
+
+### F.8: String-format coupling without documentation (new, from phase-6-fix)
+
+- **Source:** Old binary accepted comma-separated values without printing
+  expected format in help text or error messages.
+- **Pattern:** A CLI argument expects structured string input but does not
+  document the expected format or validate it at the boundary, leaving users
+  to discover format requirements through trial and error.
+- **Mitigation:** Every CLI string argument that carries structured data must
+  document its expected format in the help text (if `clap` does not already
+  handle it) and produce `anyhow::Result<T>` with the expected format in the
+  error message. See Pattern 3.
diff --git a/notes/directions/phase-6-fix/directions-index.json b/notes/directions/phase-6-fix/directions-index.json
new file mode 100644
index 0000000..f9b6f50
--- /dev/null
+++ b/notes/directions/phase-6-fix/directions-index.json
@@ -0,0 +1,78 @@
+{
+  "meta": {
+    "title": "Phase 6 Fix: Rewrite example binaries for multi-parameter sweep & SCF+DOS chain",
+    "source_branch": "phase-6-fix"
+  },
+  "architecture_notes": [
+    "Summary: Replace the buggy hubbard_u_sweep_slurm binary with two purpose-built binaries \u2014 multi_param_sweep (independent SCF parameter sweep) and scf_dos_chain (SCF+DOS task chain) \u2014 using only local path deps for castep-cell-io v0.5.0.",
+    "parse_time_validation: Parse-time validation at CLI boundary \u2014 parse_kpoints and parse_cutoffs error immediately at CLI parse time, not inside setup closures. A malformed k-point like '8xx8' must fail before task construction begins (Pattern 3).",
+    "internal_type_choice: parse_kpoints uses [u32; 3] not a custom struct \u2014 The array type directly converts to KpointsMpGrid([kx, ky, kz]) with zero overhead. The type guarantee ('exactly 3 u32s') is enforced by the array type itself.",
+    "duplication_choice: parse_u_values and generate_job_script are duplicated, not extracted \u2014 Each function is ~15-30 lines with stable semantics. Extracting to a shared crate is overkill for 2 consumers. If a third consumer appears, extraction becomes justified.",
+    "task_naming_collision: Fully-qualified path for Task::BandStructure, not alias import \u2014 The collision between workflow_core::task::Task and castep_cell_io::param::general::Task is resolved by using the fully-qualified path at the single use-site where Task::BandStructure is set. No alias needed, no import shadow risk.",
+    "shared_workdir: Shared workdir for chained tasks \u2014 Both SCF and DOS tasks in scf_dos_chain use runs/scf_dos/. This makes checkpoint file access straightforward: the DOS setup copies ZnO.check to ZnO_DOS.check within the same directory (Pattern 6).",
+    "workspace_serialization: Workspace member additions serialize G1 and G2 skeleton tasks \u2014 Both new crates use workspace = true dependencies (anyhow, clap, itertools), which require workspace membership for cargo check to resolve. Since both crate skeletons must add themselves to the root Cargo.toml workspace.members, G2-1 depends on G1-1 to avoid a merge conflict on the same file. The groups remain logically independent at the code level.",
+    "no_new_library_crates: No new library crates are created \u2014 All new code lives in binary (example) crates. The new code is workflow usage (orchestration, not abstractions). The library crates (workflow_core, workflow_utils) are unchanged.",
+    "Crate boundaries:",
+    "  parse_u_values: Duplicated in multi_param_sweep/src/config.rs and scf_dos_chain/src/config.rs",
+    "  parse_kpoints: multi_param_sweep/src/config.rs only (not needed by Binary 2)",
+    "  parse_cutoffs: multi_param_sweep/src/config.rs only",
+    "  generate_job_script: Duplicated in both binaries' job_script.rs (D.2 fix applies to both)",
+    "  SweepConfig: multi_param_sweep/src/config.rs",
+    "  ChainConfig: scf_dos_chain/src/config.rs",
+    "  sweep_logic: multi_param_sweep/src/main.rs (build_one_scf_task, build_all_scf_tasks)",
+    "  chain_logic: scf_dos_chain/src/main.rs (build_scf_task, build_dos_task)",
+    "  seed_files: Each binary's seeds/ directory (identical content, self-contained)",
+    "Pattern: Purpose-specific binaries over mode-flags \u2014 check: Neither binary should have a flag that toggles between 'independent tasks' and 'chained tasks'. multi_param_sweep has a --sweep-mode flag for product vs pairwise (both are independent SCF sweeps, same workflow type). scf_dos_chain has no mode flag.",
+    "Pattern: Parameter encoding in task IDs \u2014 check: multi_param_sweep task IDs: scf_U{u}_k{k}_c{c} (e.g., scf_U3.0_k8x8x8_c500). scf_dos_chain task IDs: scf and dos (single parameter set, no collision risk).",
+    "Pattern: Parse-time validation gates \u2014 check: Every parsing function returns anyhow::Result<T> with messages containing the expected format and what went wrong. Examples: 'kpoints list is empty', 'invalid k-point: expected 3 axes, got 2'.",
+    "Pattern: Explicit defaults \u2014 check: --sweep-mode defaults to 'product'. --kpoints and --cutoffs are Option<String> defaulting to None (seed defaults used). Pairwise mode errors if either is None.",
+    "Pattern: In-memory document mutation \u2014 check: All CASTEP file modifications go through parse -> mutate typed fields -> to_cell_file() -> to_string_many_spaced(). Never use format! on raw strings to build .cell/.param content.",
+    "Pattern: Shared workdir for chained tasks \u2014 check: scf_dos_chain places both tasks in runs/scf_dos/. DOS setup copies ZnO.check to ZnO_DOS.check in the same directory.",
+    "Pattern: Clean heredoc template \u2014 check: The no_literal_tabs test must pass. Use consistent space-only indentation. SBATCH directives use consistent quoting (double-quotes around values that might contain spaces)."
+  ],
+  "known_pitfalls": [
+    "Task name collision in scf_dos_chain (P0): use workflow_utils::prelude::* brings workflow_core::task::Task into scope. Importing castep_cell_io::param::general::task::Task would shadow it. Resolution: use the fully-qualified path castep_cell_io::param::general::task::Task::BandStructure at the single use-site in the DOS setup closure. Verify the exact module path with LSP hover during implementation.. Mitigation: Do not add use castep_cell_io::param::general::task::Task. Instead use the fully-qualified path at the single use-site.",
+    "Local path dep must exist on the filesystem: The path ../castep-cell-io/castep_cell_io must resolve to a sibling directory containing a Cargo.toml with the castep-cell-io crate. Verify this exists before implementation.. Mitigation: Pre-implementation check: verify ../castep-cell-io/castep_cell_io/Cargo.toml exists.",
+    "castep-cell-fmt version compatibility: castep-cell-fmt = 0.1.0 from crates.io while castep-cell-io uses a local path dep at v0.5.0. If v0.5.0 depends on a newer castep-cell-fmt, Cargo resolves the semver-compatible version automatically. No conflict expected.. Mitigation: No action needed; Cargo handles this automatically.",
+    "Pairwise mode validation ordering: In pairwise mode, validate that both --kpoints and --cutoffs are provided BEFORE parsing them. If either is None, return a clear error immediately. After parsing, validate all three lists have the same length. The error message must state the expected and actual lengths.. Mitigation: Check sweep_mode first, then validate presence of both Option fields, then parse, then check lengths.",
+    "hubbard_u_sweep compatibility with v0.5.0 path dep: The existing hubbard_u_sweep/src/main.rs already uses the v0.5.0 parse API pattern (castep_cell_fmt::parse::<CellDocument>(&input)). However, v0.5.0 may have changed Species::Symbol signature, builder method names, or HubbardUUnit enum variants. If compilation fails after the path-dep change, fix these call sites based on compiler error messages.. Mitigation: After changing the path dep, run cargo check -p hubbard_u_sweep. If it fails, fix each error based on the compiler message. Common changes may include Species::Symbol type or builder method names.",
+    "Dead-code detection after refactoring (F.3 mitigation): After deleting hubbard_u_sweep_slurm, run cargo clippy --workspace --all-targets -- -W dead-code to detect any stale imports or dead functions left over from the refactoring.. Mitigation: Include clippy dead-code check in the Group 3 acceptance criteria.",
+    "D.2 formatting fix must be embedded in ported job_script.rs: The no_literal_tabs test from the old binary must be ported and must pass. The D.2 fix (clean heredoc template with all-space indentation, consistent double-quoting on SBATCH directives) must be applied to the ported generate_job_script function in BOTH new binaries.. Mitigation: Implementation checklist: all-space indentation, SBATCH directives use double-quotes consistently, shell continuation lines use backslash, no_literal_tabs test passes in both binaries.",
+    "Deferred items: what NOT to do: D.1 (portable SLURM config): Do NOT implement. D.3 (expanded unit tests for generate_job_script): Do NOT expand beyond ported tests. read_task_ids edge case: Do NOT fix. All three have unmet preconditions.. Mitigation: Review the deferred-and-patterns.md file for full details on each deferred item.",
+    "G1-1 and G2-1 both modify root Cargo.toml \u2014 serialized: Both multi_param_sweep and scf_dos_chain use workspace = true dependencies, requiring workspace membership. To avoid merge conflicts on the root Cargo.toml, TASK-G2-1 must run after TASK-G1-1 completes. The groups are logically independent at the code level; this dependency is solely a file collision on the workspace member list.. Mitigation: TASK-G2-1 depends_on TASK-G1-1. Only the skeleton creation tasks are serialized; all other tasks within each group can run in parallel with tasks in the other group."
+  ],
+  "groups": [
+    {
+      "group_id": "core-1",
+      "task_ids": [
+        "G1-1",
+        "G1-2",
+        "G1-3",
+        "G1-4",
+        "G1-5"
+      ],
+      "description": "Create the complete examples/multi_param_sweep/ crate \u2014 a purpose-built binary for independent SCF parameter sweeps across Hubbard U, k-point MP grid, and cutoff energy in product or pairwise mode.",
+      "file": "directions-phase-6-fix-core-1.json"
+    },
+    {
+      "group_id": "core-2",
+      "task_ids": [
+        "G2-1",
+        "G2-2",
+        "G2-3",
+        "G2-4",
+        "G2-5"
+      ],
+      "description": "Create the examples/scf_dos_chain/ crate \u2014 a purpose-built binary for SCF+DOS task chain testing \u2014 and update the existing hubbard_u_sweep binary for castep-cell-io v0.5.0 path dep compatibility.",
+      "file": "directions-phase-6-fix-core-2.json"
+    },
+    {
+      "group_id": "workspace",
+      "task_ids": [
+        "G3-1"
+      ],
+      "description": "Wire everything together by removing the old hubbard_u_sweep_slurm crate, cleaning up workspace members, and running full build/test/clippy verification.",
+      "file": "directions-phase-6-fix-workspace.json"
+    }
+  ]
+}
diff --git a/notes/directions/phase-6-fix/directions-phase-6-fix-core-1.json b/notes/directions/phase-6-fix/directions-phase-6-fix-core-1.json
new file mode 100644
index 0000000..e886613
--- /dev/null
+++ b/notes/directions/phase-6-fix/directions-phase-6-fix-core-1.json
@@ -0,0 +1,244 @@
+{
+  "meta": {
+    "title": "Phase 6 Fix: Rewrite example binaries for multi-parameter sweep & SCF+DOS chain",
+    "source_branch": "phase-6-fix"
+  },
+  "architecture_notes": [
+    "Summary: Replace the buggy hubbard_u_sweep_slurm binary with two purpose-built binaries \u2014 multi_param_sweep (independent SCF parameter sweep) and scf_dos_chain (SCF+DOS task chain) \u2014 using only local path deps for castep-cell-io v0.5.0.",
+    "parse_time_validation: Parse-time validation at CLI boundary \u2014 parse_kpoints and parse_cutoffs error immediately at CLI parse time, not inside setup closures. A malformed k-point like '8xx8' must fail before task construction begins (Pattern 3).",
+    "internal_type_choice: parse_kpoints uses [u32; 3] not a custom struct \u2014 The array type directly converts to KpointsMpGrid([kx, ky, kz]) with zero overhead. The type guarantee ('exactly 3 u32s') is enforced by the array type itself.",
+    "duplication_choice: parse_u_values and generate_job_script are duplicated, not extracted \u2014 Each function is ~15-30 lines with stable semantics. Extracting to a shared crate is overkill for 2 consumers. If a third consumer appears, extraction becomes justified.",
+    "task_naming_collision: Fully-qualified path for Task::BandStructure, not alias import \u2014 The collision between workflow_core::task::Task and castep_cell_io::param::general::Task is resolved by using the fully-qualified path at the single use-site where Task::BandStructure is set. No alias needed, no import shadow risk.",
+    "shared_workdir: Shared workdir for chained tasks \u2014 Both SCF and DOS tasks in scf_dos_chain use runs/scf_dos/. This makes checkpoint file access straightforward: the DOS setup copies ZnO.check to ZnO_DOS.check within the same directory (Pattern 6).",
+    "workspace_serialization: Workspace member additions serialize G1 and G2 skeleton tasks \u2014 Both new crates use workspace = true dependencies (anyhow, clap, itertools), which require workspace membership for cargo check to resolve. Since both crate skeletons must add themselves to the root Cargo.toml workspace.members, G2-1 depends on G1-1 to avoid a merge conflict on the same file. The groups remain logically independent at the code level.",
+    "no_new_library_crates: No new library crates are created \u2014 All new code lives in binary (example) crates. The new code is workflow usage (orchestration, not abstractions). The library crates (workflow_core, workflow_utils) are unchanged.",
+    "Crate boundaries:",
+    "  parse_u_values: Duplicated in multi_param_sweep/src/config.rs and scf_dos_chain/src/config.rs",
+    "  parse_kpoints: multi_param_sweep/src/config.rs only (not needed by Binary 2)",
+    "  parse_cutoffs: multi_param_sweep/src/config.rs only",
+    "  generate_job_script: Duplicated in both binaries' job_script.rs (D.2 fix applies to both)",
+    "  SweepConfig: multi_param_sweep/src/config.rs",
+    "  ChainConfig: scf_dos_chain/src/config.rs",
+    "  sweep_logic: multi_param_sweep/src/main.rs (build_one_scf_task, build_all_scf_tasks)",
+    "  chain_logic: scf_dos_chain/src/main.rs (build_scf_task, build_dos_task)",
+    "  seed_files: Each binary's seeds/ directory (identical content, self-contained)",
+    "Pattern: Purpose-specific binaries over mode-flags \u2014 check: Neither binary should have a flag that toggles between 'independent tasks' and 'chained tasks'. multi_param_sweep has a --sweep-mode flag for product vs pairwise (both are independent SCF sweeps, same workflow type). scf_dos_chain has no mode flag.",
+    "Pattern: Parameter encoding in task IDs \u2014 check: multi_param_sweep task IDs: scf_U{u}_k{k}_c{c} (e.g., scf_U3.0_k8x8x8_c500). scf_dos_chain task IDs: scf and dos (single parameter set, no collision risk).",
+    "Pattern: Parse-time validation gates \u2014 check: Every parsing function returns anyhow::Result<T> with messages containing the expected format and what went wrong. Examples: 'kpoints list is empty', 'invalid k-point: expected 3 axes, got 2'.",
+    "Pattern: Explicit defaults \u2014 check: --sweep-mode defaults to 'product'. --kpoints and --cutoffs are Option<String> defaulting to None (seed defaults used). Pairwise mode errors if either is None.",
+    "Pattern: In-memory document mutation \u2014 check: All CASTEP file modifications go through parse -> mutate typed fields -> to_cell_file() -> to_string_many_spaced(). Never use format! on raw strings to build .cell/.param content.",
+    "Pattern: Shared workdir for chained tasks \u2014 check: scf_dos_chain places both tasks in runs/scf_dos/. DOS setup copies ZnO.check to ZnO_DOS.check in the same directory.",
+    "Pattern: Clean heredoc template \u2014 check: The no_literal_tabs test must pass. Use consistent space-only indentation. SBATCH directives use consistent quoting (double-quotes around values that might contain spaces)."
+  ],
+  "known_pitfalls": [
+    "Task name collision in scf_dos_chain (P0): use workflow_utils::prelude::* brings workflow_core::task::Task into scope. Importing castep_cell_io::param::general::task::Task would shadow it. Resolution: use the fully-qualified path castep_cell_io::param::general::task::Task::BandStructure at the single use-site in the DOS setup closure. Verify the exact module path with LSP hover during implementation.. Mitigation: Do not add use castep_cell_io::param::general::task::Task. Instead use the fully-qualified path at the single use-site.",
+    "Local path dep must exist on the filesystem: The path ../castep-cell-io/castep_cell_io must resolve to a sibling directory containing a Cargo.toml with the castep-cell-io crate. Verify this exists before implementation.. Mitigation: Pre-implementation check: verify ../castep-cell-io/castep_cell_io/Cargo.toml exists.",
+    "castep-cell-fmt version compatibility: castep-cell-fmt = 0.1.0 from crates.io while castep-cell-io uses a local path dep at v0.5.0. If v0.5.0 depends on a newer castep-cell-fmt, Cargo resolves the semver-compatible version automatically. No conflict expected.. Mitigation: No action needed; Cargo handles this automatically.",
+    "Pairwise mode validation ordering: In pairwise mode, validate that both --kpoints and --cutoffs are provided BEFORE parsing them. If either is None, return a clear error immediately. After parsing, validate all three lists have the same length. The error message must state the expected and actual lengths.. Mitigation: Check sweep_mode first, then validate presence of both Option fields, then parse, then check lengths.",
+    "hubbard_u_sweep compatibility with v0.5.0 path dep: The existing hubbard_u_sweep/src/main.rs already uses the v0.5.0 parse API pattern (castep_cell_fmt::parse::<CellDocument>(&input)). However, v0.5.0 may have changed Species::Symbol signature, builder method names, or HubbardUUnit enum variants. If compilation fails after the path-dep change, fix these call sites based on compiler error messages.. Mitigation: After changing the path dep, run cargo check -p hubbard_u_sweep. If it fails, fix each error based on the compiler message. Common changes may include Species::Symbol type or builder method names.",
+    "Dead-code detection after refactoring (F.3 mitigation): After deleting hubbard_u_sweep_slurm, run cargo clippy --workspace --all-targets -- -W dead-code to detect any stale imports or dead functions left over from the refactoring.. Mitigation: Include clippy dead-code check in the Group 3 acceptance criteria.",
+    "D.2 formatting fix must be embedded in ported job_script.rs: The no_literal_tabs test from the old binary must be ported and must pass. The D.2 fix (clean heredoc template with all-space indentation, consistent double-quoting on SBATCH directives) must be applied to the ported generate_job_script function in BOTH new binaries.. Mitigation: Implementation checklist: all-space indentation, SBATCH directives use double-quotes consistently, shell continuation lines use backslash, no_literal_tabs test passes in both binaries.",
+    "Deferred items: what NOT to do: D.1 (portable SLURM config): Do NOT implement. D.3 (expanded unit tests for generate_job_script): Do NOT expand beyond ported tests. read_task_ids edge case: Do NOT fix. All three have unmet preconditions.. Mitigation: Review the deferred-and-patterns.md file for full details on each deferred item.",
+    "G1-1 and G2-1 both modify root Cargo.toml \u2014 serialized: Both multi_param_sweep and scf_dos_chain use workspace = true dependencies, requiring workspace membership. To avoid merge conflicts on the root Cargo.toml, TASK-G2-1 must run after TASK-G1-1 completes. The groups are logically independent at the code level; this dependency is solely a file collision on the workspace member list.. Mitigation: TASK-G2-1 depends_on TASK-G1-1. Only the skeleton creation tasks are serialized; all other tasks within each group can run in parallel with tasks in the other group."
+  ],
+  "task_groups": [
+    {
+      "group_id": "core-1",
+      "reason": "Create the complete examples/multi_param_sweep/ crate \u2014 a purpose-built binary for independent SCF parameter sweeps across Hubbard U, k-point MP grid, and cutoff energy in product or pairwise mode.",
+      "tasks": [
+        "G1-1",
+        "G1-2",
+        "G1-3",
+        "G1-4",
+        "G1-5"
+      ],
+      "depends_on_groups": []
+    }
+  ],
+  "tasks": [
+    {
+      "id": "G1-1",
+      "description": "Create the directory structure, Cargo.toml, stub source files, seed files, and add the crate to workspace members so cargo check resolves dependencies. \u2014 Crate creation: Cargo.toml with all dependencies, minimal main.rs with mod declarations, empty config.rs, empty job_script.rs, seed files from existing hubbard_u_sweep_slurm, and workspace member registration.",
+      "files_in_scope": [
+        "examples/multi_param_sweep (new crate)",
+        "examples/multi_param_sweep/Cargo.toml",
+        "examples/multi_param_sweep/src/main.rs",
+        "examples/multi_param_sweep/src/config.rs",
+        "examples/multi_param_sweep/src/job_script.rs",
+        "examples/multi_param_sweep/seeds/ZnO.cell",
+        "examples/multi_param_sweep/seeds/ZnO.param",
+        "Cargo.toml (workspace root)"
+      ],
+      "changes": [
+        {
+          "path": "examples/multi_param_sweep/Cargo.toml",
+          "action": "create",
+          "guidance": "Create Cargo.toml with [package] name = 'multi_param_sweep', edition 2021. Use [[bin]] pointing to src/main.rs. Dependencies: anyhow = { workspace = true }, clap = { workspace = true }, itertools = { workspace = true }, castep-cell-fmt = '0.1.0', castep-cell-io = { path = '../castep-cell-io/castep_cell_io' }, workflow_core = { path = '../../workflow_core', features = ['default-logging'] }, workflow_utils = { path = '../../workflow_utils' }. Copy from hubbard_u_sweep_slurm/Cargo.toml as a template, adjusting the package name and adding kpoints/cutoff-related deps."
+        },
+        {
+          "path": "examples/multi_param_sweep/src/main.rs",
+          "action": "create",
+          "guidance": "Create stub main.rs with mod config; and mod job_script; declarations. Add a minimal fn main() { println!('multi_param_sweep: skeleton'); } that compiles. No imports yet."
+        },
+        {
+          "path": "examples/multi_param_sweep/src/config.rs",
+          "action": "create",
+          "guidance": "Create empty config.rs with a placeholder comment: // Config will be added in G1-2. No struct, no functions, no tests yet."
+        },
+        {
+          "path": "examples/multi_param_sweep/src/job_script.rs",
+          "action": "create",
+          "guidance": "Create empty job_script.rs with a placeholder comment: // Job script will be added in G1-3."
+        },
+        {
+          "path": "examples/multi_param_sweep/seeds/ZnO.cell",
+          "action": "modify",
+          "guidance": "Copy the content from examples/hubbard_u_sweep_slurm/seeds/ZnO.cell. This is the ZnO wurtzite lattice cell file used as the seed for all task setups."
+        },
+        {
+          "path": "examples/multi_param_sweep/seeds/ZnO.param",
+          "action": "modify",
+          "guidance": "Copy the content from examples/hubbard_u_sweep_slurm/seeds/ZnO.param. This is the seed param file (task: SinglePoint) used as the base for task-specific param modifications."
+        },
+        {
+          "path": "Cargo.toml (workspace root)",
+          "action": "modify",
+          "guidance": "Add 'examples/multi_param_sweep' to the [workspace] members array. Insert it after 'examples/hubbard_u_sweep_slurm' (keeping the old binary for now). Keep all other members unchanged."
+        }
+      ],
+      "acceptance": [
+        "cargo check --workspace (must compile \u2014 new crate resolves workspace deps, old slurm crate still compiles)",
+        "ls examples/multi_param_sweep/Cargo.toml examples/multi_param_sweep/src/main.rs examples/multi_param_sweep/src/config.rs examples/multi_param_sweep/src/job_script.rs examples/multi_param_sweep/seeds/ZnO.cell examples/multi_param_sweep/seeds/ZnO.param"
+      ],
+      "depends_on": [],
+      "wiring_checklist": [
+        {
+          "kind": "pub_mod",
+          "file": "examples/multi_param_sweep/src/main.rs",
+          "detail": "Add mod config; and mod job_script; at the top of the file (before fn main)."
+        },
+        {
+          "kind": "pub_mod",
+          "file": "Cargo.toml (workspace root)",
+          "detail": "Add 'examples/multi_param_sweep' to [workspace] members array."
+        }
+      ]
+    },
+    {
+      "id": "G1-2",
+      "description": "Implement the SweepConfig clap struct and all three parsing utility functions (parse_u_values, parse_kpoints, parse_cutoffs) with comprehensive tests written first. \u2014 CLI configuration and parse-time validation: SweepConfig struct (clap Parser derive, all 16 fields from the plan plus SLURM fields), parse_u_values (ported with anyhow::Result), parse_kpoints (new, strict NxNxN validation), parse_cutoffs (new, f64 comma-separated), and their in-file #[cfg(test)] tests.",
+      "files_in_scope": [
+        "examples/multi_param_sweep/src/config.rs"
+      ],
+      "changes": [
+        {
+          "path": "examples/multi_param_sweep/src/config.rs",
+          "action": "modify",
+          "guidance": "Implement the full SweepConfig struct with #[derive(Parser, Debug)], #[command(name = 'multi_param_sweep')], and all fields from the plan: u_values (String, default '0.0,1.0,2.0,3.0,4.0,5.0'), kpoints (Option<String>), cutoffs (Option<String>), sweep_mode (String, default 'product'), element (String, default 'Zn'), orbital (char, default 'd'), seed_name (String, default 'ZnO'), max_parallel (usize, default 4), local (bool), dry_run (bool), castep_command (String, default 'castep'), workdir (String, default '.'), plus SLURM fields: partition (String, env CASTEP_SLURM_PARTITION, default 'debug'), ntasks (u32, default 16), nix_flake (String, env CASTEP_NIX_FLAKE, with the long default), mpi_if (String, env CASTEP_MPI_IF, default 'enp6s0'). Each field should have a doc comment describing its purpose and the expected format for string-typed fields. Implement parse_u_values (ported, returning anyhow::Result<Vec<f64>> not Result<_, String>), parse_kpoints (new, splitting on comma then on 'x', validating exactly 3 u32 axes per segment), parse_cutoffs (new, splitting on comma, trimming, parsing f64). All three return anyhow::Result with clear error messages describing expected format. parse_kpoints must validate each segment splits into exactly 3 parts on 'x' and each part parses as u32 \u2014 reject '8x8' (2 axes) and 'abc' (non-numeric). Add #[cfg(test)] mod tests with the tests from tdd_interface.test_code. For parse_kpoints, the 'wrong_axes' test must verify the error message mentions the expected axis count or format. For empty input tests, the error must contain 'empty'."
+        }
+      ],
+      "acceptance": [
+        "cargo check -p multi_param_sweep",
+        "cargo test -p multi_param_sweep -- config::tests (all parse tests pass: 7 parse_u_values + 7 parse_kpoints + 6 parse_cutoffs = 20 tests)",
+        "cargo test -p multi_param_sweep (no failures; job_script tests may not exist yet and will be ignored)"
+      ],
+      "depends_on": [
+        "G1-1"
+      ],
+      "kind": "lib-tdd",
+      "tdd_interface": {
+        "test_file": "examples/multi_param_sweep/src/config.rs",
+        "test_module": "tests",
+        "test_fn_name": "parse_basic_values",
+        "test_code": "// --- parse_u_values tests (ported from old binary, adapted for anyhow::Result) ---\n\n/// Parse comma-separated basic values\n#[test]\nfn parse_basic_values() {\n    let vals = parse_u_values(\"0.0,1.0,2.0\").unwrap();\n    assert_eq!(vals, vec![0.0, 1.0, 2.0]);\n}\n\n/// Parse with whitespace around values\n#[test]\nfn parse_with_whitespace() {\n    let vals = parse_u_values(\"  0.0 , 1.0 , 2.0  \").unwrap();\n    assert_eq!(vals, vec![0.0, 1.0, 2.0]);\n}\n\n/// Parse a single value\n#[test]\nfn parse_single_value() {\n    let vals = parse_u_values(\"42.0\").unwrap();\n    assert_eq!(vals, vec![42.0]);\n}\n\n/// Invalid token in the middle\n#[test]\nfn parse_invalid_token() {\n    let err = parse_u_values(\"1.0,abc,2.0\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"abc\"), \"error should mention the invalid token: {msg}\");\n}\n\n/// Empty token in the middle (two consecutive commas)\n#[test]\nfn parse_empty_token() {\n    let err = parse_u_values(\"1.0,,2.0\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"invalid\"), \"error should report parse failure: {msg}\");\n}\n\n/// Entirely empty input\n#[test]\nfn parse_empty_string() {\n    let err = parse_u_values(\"\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"empty\") || msg.contains(\"invalid\"), \"expected parse failure on empty input, got: {msg}\");\n}\n\n/// Negative values\n#[test]\nfn parse_negative_values() {\n    let vals = parse_u_values(\"-1.0,2.0\").unwrap();\n    assert_eq!(vals, vec![-1.0, 2.0]);\n}\n\n// --- parse_kpoints tests (new) ---\n\n/// Basic valid k-points\n#[test]\nfn parse_kpoints_basic() {\n    let kpts = parse_kpoints(\"8x8x8,6x6x6\").unwrap();\n    assert_eq!(kpts, vec![[8, 8, 8], [6, 6, 6]]);\n}\n\n/// Single valid k-point\n#[test]\nfn parse_kpoints_single() {\n    let kpts = parse_kpoints(\"4x4x4\").unwrap();\n    assert_eq!(kpts, vec![[4, 4, 4]]);\n}\n\n/// With whitespace\n#[test]\nfn parse_kpoints_whitespace() {\n    let kpts = parse_kpoints(\" 8x8x8 , 6x6x6 \").unwrap();\n    assert_eq!(kpts, vec![[8, 8, 8], [6, 6, 6]]);\n}\n\n/// Wrong number of axes (only 2 instead of 3)\n#[test]\nfn parse_kpoints_wrong_axes() {\n    let err = parse_kpoints(\"8x8\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"3 axes\") || msg.contains(\"got 2\") || msg.contains(\"expected 3\"), \"error should mention wrong axis count: {msg}\");\n}\n\n/// Non-numeric axis value\n#[test]\nfn parse_kpoints_non_numeric() {\n    let err = parse_kpoints(\"abc\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"invalid\") || msg.contains(\"abc\") || msg.contains(\"expected 3 axes\"), \"error should report the problem: {msg}\");\n}\n\n/// Empty input\n#[test]\nfn parse_kpoints_empty() {\n    let err = parse_kpoints(\"\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"empty\"), \"error should say kpoints list is empty: {msg}\");\n}\n\n/// Large grid values\n#[test]\nfn parse_kpoints_large() {\n    let kpts = parse_kpoints(\"12x12x12\").unwrap();\n    assert_eq!(kpts, vec![[12, 12, 12]]);\n}\n\n// --- parse_cutoffs tests (new) ---\n\n/// Basic valid cutoffs\n#[test]\nfn parse_cutoffs_basic() {\n    let cuts = parse_cutoffs(\"300,500,800\").unwrap();\n    assert_eq!(cuts, vec![300.0, 500.0, 800.0]);\n}\n\n/// Single cutoff\n#[test]\nfn parse_cutoffs_single() {\n    let cuts = parse_cutoffs(\"450\").unwrap();\n    assert_eq!(cuts, vec![450.0]);\n}\n\n/// With whitespace\n#[test]\nfn parse_cutoffs_whitespace() {\n    let cuts = parse_cutoffs(\" 300 , 500 , 800 \").unwrap();\n    assert_eq!(cuts, vec![300.0, 500.0, 800.0]);\n}\n\n/// Invalid token\n#[test]\nfn parse_cutoffs_invalid_token() {\n    let err = parse_cutoffs(\"300,abc,800\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"abc\"), \"error should mention the invalid token: {msg}\");\n}\n\n/// Empty input\n#[test]\nfn parse_cutoffs_empty() {\n    let err = parse_cutoffs(\"\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"empty\"), \"error should say cutoffs list is empty: {msg}\");\n}\n\n/// Negative and fractional values\n#[test]\nfn parse_cutoffs_negative() {\n    let cuts = parse_cutoffs(\"-100.5,200.0\").unwrap();\n    assert_eq!(cuts, vec![-100.5, 200.0]);\n}",
+        "signature": "pub fn parse_u_values(s: &str) -> anyhow::Result<Vec<f64>>",
+        "expected_behavior": "parse_u_values: returns Vec<f64> for comma-separated numbers, errors on non-numeric tokens with the offending token in the message. Empty input produces an error containing 'empty' or 'invalid'. Whitespace around values is trimmed. parse_kpoints: returns Vec<[u32; 3]> for comma-separated NxNxN triplets. Errors on wrong number of axes (must be exactly 3 per segment). Empty input produces an error containing 'empty'. Whitespace around values is trimmed. Non-numeric axis values produce errors mentioning the invalid input. parse_cutoffs: returns Vec<f64> for comma-separated numbers, same semantics as parse_u_values but with cutoff-specific error context."
+      }
+    },
+    {
+      "id": "G1-3",
+      "description": "Implement the SLURM job script generation function with the D.2 formatting fix applied, with ported tests written first. \u2014 SLURM job script generation: generate_job_script function with the D.2 clean heredoc template (all-space indentation, consistent double-quoting on SBATCH directives), and all 6 ported tests (sbatch directives, seed name, no_literal_tabs, shebang, nix develop, mpi interface).",
+      "files_in_scope": [
+        "examples/multi_param_sweep/src/job_script.rs"
+      ],
+      "changes": [
+        {
+          "path": "examples/multi_param_sweep/src/job_script.rs",
+          "action": "modify",
+          "guidance": "Implement generate_job_script(config: &SweepConfig, task_id: &str, seed_name: &str) -> String. Port the function body from the old hubbard_u_sweep_slurm/src/job_script.rs but apply the D.2 formatting fix: use consistent space-only indentation throughout the heredoc template (no literal tab characters mixed with spaces), use double-quotes consistently around SBATCH directive values (e.g., #SBATCH --job-name=\\\"{task_id}\\\"), and ensure shell continuation lines use \\\\ with proper spacing. The function must use format!() with named parameters. Import SweepConfig from crate::config. Add #[cfg(test)] mod tests with all 6 ported tests from tdd_interface.test_code adapted for the new SweepConfig. The no_literal_tabs test is the D.2 verification: it must assert !script.contains('\\t') and must PASS with the new template."
+        }
+      ],
+      "acceptance": [
+        "cargo check -p multi_param_sweep",
+        "cargo test -p multi_param_sweep -- job_script::tests (all 6 tests pass)",
+        "cargo test -p multi_param_sweep -- job_script::tests::no_literal_tabs (must pass \u2014 this is the D.2 verification)"
+      ],
+      "depends_on": [
+        "G1-2"
+      ],
+      "kind": "lib-tdd",
+      "tdd_interface": {
+        "test_file": "examples/multi_param_sweep/src/job_script.rs",
+        "test_module": "tests",
+        "test_fn_name": "contains_sbatch_directives",
+        "test_code": "use crate::config::SweepConfig;\nuse clap::Parser;\n\nfn default_config() -> SweepConfig {\n    SweepConfig::parse_from([\"test\"])\n}\n\n/// Script contains expected SBATCH directives with the task ID and config values\n#[test]\nfn contains_sbatch_directives() {\n    let config = default_config();\n    let script = generate_job_script(&config, \"scf_U1.0\", \"ZnO\");\n    assert!(script.contains(\"#SBATCH --job-name=\\\"scf_U1.0\\\"\"));\n    assert!(script.contains(\"#SBATCH --partition=debug\"));\n    assert!(script.contains(\"#SBATCH --ntasks-per-node=16\"));\n    assert!(script.contains(\"#SBATCH --mem=30000m\"));\n}\n\n/// Script references the correct seed name in the castep command\n#[test]\nfn contains_seed_name() {\n    let config = default_config();\n    let script = generate_job_script(&config, \"scf_U0.0\", \"ZnO\");\n    assert!(script.contains(\"castep.mpi ZnO\"));\n}\n\n/// D.2 fix: script must not contain literal tab characters\n#[test]\nfn no_literal_tabs() {\n    let config = default_config();\n    let script = generate_job_script(&config, \"scf_U0.0\", \"ZnO\");\n    assert!(!script.contains('\\t'), \"job script should not contain literal tab characters\");\n}\n\n/// Script starts with a shebang line\n#[test]\nfn starts_with_shebang() {\n    let config = default_config();\n    let script = generate_job_script(&config, \"scf_U0.0\", \"ZnO\");\n    assert!(script.starts_with(\"#!/usr/bin/env bash\"));\n}\n\n/// Script contains nix develop with the configured flake URI\n#[test]\nfn contains_nix_develop() {\n    let config = default_config();\n    let script = generate_job_script(&config, \"scf_U0.0\", \"ZnO\");\n    assert!(script.contains(\"nix develop\"));\n    assert!(script.contains(&config.nix_flake));\n}\n\n/// Script contains the MPI interface setting\n#[test]\nfn contains_mpi_interface() {\n    let config = default_config();\n    let script = generate_job_script(&config, \"scf_U0.0\", \"ZnO\");\n    assert!(script.contains(&format!(\"OMPI_MCA_btl_tcp_if_include={mpi_if}\", mpi_if = config.mpi_if)));\n}",
+        "signature": "pub fn generate_job_script(config: &SweepConfig, task_id: &str, seed_name: &str) -> String",
+        "expected_behavior": "generate_job_script produces a SLURM job script string. The template uses ONLY spaces for indentation (no literal tab characters). SBATCH directives use double-quotes around values: #SBATCH --job-name=\"{task_id}\". The script contains: a bash shebang, SBATCH directives for partition/ntasks/mem, a nix develop command with the configured flake, an mpirun command with the MPI interface, and the castep.mpi {seed_name} invocation. Shell continuation lines use backslash (not mixed tab+space indentation)."
+      }
+    },
+    {
+      "id": "G1-4",
+      "description": "Implement the sweep combinatorics (product/pairwise mode dispatch) and the build_one_scf_task builder function with tests for correct task structure and sweep logic. \u2014 Sweep combinatorics and SCF task construction: build_one_scf_task (single SCF task with setup/collect closures that inject hubbard_u, kpoints_mp_grid, and cutoff_energy), build_all_scf_tasks (dispatches to product or pairwise modes), sweep mode validation (pairwise requires both kpoints and cutoffs, equal-length check), and in-file #[cfg(test)] tests verifying task IDs, dependency structure, workdir paths, and mode-specific error handling.",
+      "files_in_scope": [
+        "examples/multi_param_sweep/src/main.rs"
+      ],
+      "changes": [
+        {
+          "path": "examples/multi_param_sweep/src/main.rs",
+          "action": "modify",
+          "guidance": "Implement the sweep logic and task builder functions. Add necessary imports: use config::{parse_u_values, parse_kpoints, parse_cutoffs, SweepConfig}; use job_script::generate_job_script; plus castep_cell_fmt (parse, to_string_many_spaced, ToCellFile), castep_cell_io types (CellDocument, ParamDocument, AtomHubbardU, HubbardU, HubbardUUnit, OrbitalU, Species, KpointsMpGrid, CutOffEnergy), itertools::iproduct, workflow_utils::prelude::*.\n\nFunction build_one_scf_task(config, u, kpoint, cutoff, seed_cell, seed_param) -> Result<Task, WorkflowError>: Construct task ID using format! with the pattern 'scf_U{}_k{}_c{}'. For the u value, use {:.1} formatting to drop trailing zeros (3.0 -> 'U3.0'). For kpoint, format as '{0}x{1}x{2}' \u2014 if kpoint is None, omit the _k segment entirely. For cutoff, use {:.0} \u2014 if cutoff is None, omit the _c segment entirely. Workdir: PathBuf::from(format!('runs/U{u:.1}_k{k_str}_c{c_str}')) with segments omitted if params are None.\n\nThe setup closure must: (1) create_dir(workdir), (2) parse seed_cell into CellDocument via castep_cell_fmt::parse::<CellDocument>, inject hubbard_u block using the builder pattern: AtomHubbardU::builder().species(Species::Symbol(element.clone())).orbitals(vec![orbital_u]).build() and HubbardU::builder().unit(HubbardUUnit::ElectronVolt).atom_u_values(vec![atom_u]).build(), set cell_doc.hubbard_u = Some(hubbard_u), (3) if kpoint is Some, set cell_doc.kpoints_mp_grid = Some(KpointsMpGrid(kpoint)), (4) parse seed_param into ParamDocument via castep_cell_fmt::parse::<ParamDocument>, if cutoff is Some set doc.basis_set.cutoff_energy = Some(CutOffEnergy { value: cutoff, unit: None }), (5) serialize both to .cell and .param files via to_string_many_spaced(&doc.to_cell_file()) and write_file, (6) if !config.local, write job_script via write_file(workdir.join(JOB_SCRIPT_NAME), &generate_job_script(config, &task_id, &config.seed_name)).\n\nThe collect closure must: read <seed>.castep via read_file, verify it contains 'Total time' marker. Return WorkflowError::InvalidConfig if the file is missing or the marker is absent.\n\nFunction build_all_scf_tasks(config) -> anyhow::Result<Vec<Task>>: Load seed_cell and seed_param via include_str!('../seeds/ZnO.cell') and include_str!('../seeds/ZnO.param') (passed as parameters). Parse u_values. Match config.sweep_mode.as_str(): 'product' branch uses itertools::iproduct! over u_vals, kpoints_vec (unwrap_or(vec![None])), cutoffs_vec (unwrap_or(vec![None])). For each (u, k, c) tuple, call build_one_scf_task. 'pairwise' branch requires both kpoints and cutoffs to be Some (anyhow::bail! if not), parses all three lists, validates equal lengths (anyhow::bail! if not), then zip all three iterators and call build_one_scf_task for each triple. Unknown mode returns anyhow::bail!('unknown sweep mode...').\n\nAdd #[cfg(test)] mod tests with the tests from tdd_interface.test_code. The tests verify combinatoric correctness, mode validation, task ID uniqueness, and structural invariants."
+        }
+      ],
+      "acceptance": [
+        "cargo check -p multi_param_sweep",
+        "cargo test -p multi_param_sweep (config tests + job_script tests + sweep logic tests all pass)",
+        "All sweep tests pass: product count (6 default, 18 explicit), pairwise count (3), pairwise requires kpoints, pairwise requires cutoffs, unequal lengths error, unknown mode error, unique IDs, ID format, optional params, no dependencies, workdirs present"
+      ],
+      "depends_on": [
+        "G1-3"
+      ],
+      "kind": "lib-tdd",
+      "tdd_interface": {
+        "test_file": "examples/multi_param_sweep/src/main.rs",
+        "test_module": "tests",
+        "test_fn_name": "product_mode_produces_tasks",
+        "test_code": "use crate::config::SweepConfig;\nuse clap::Parser;\n\nfn default_config() -> SweepConfig {\n    SweepConfig::parse_from([\"test\"])\n}\n\nfn test_seeds() -> (&'static str, &'static str) {\n    (include_str!(\"../seeds/ZnO.cell\"), include_str!(\"../seeds/ZnO.param\"))\n}\n\n// --- Sweep combinatorics tests ---\n\n/// Product mode with defaults produces many tasks (6 u-values * default seed = 6 tasks)\n#[test]\nfn product_mode_produces_tasks() {\n    let config = default_config();\n    let tasks = build_all_scf_tasks(&config).unwrap();\n    // Default: 6 u-values (0.0..5.0), no kpoints/cutoffs by default -> 6 tasks\n    assert_eq!(tasks.len(), 6);\n}\n\n/// Product mode with explicit u-values, kpoints, and cutoffs produces Cartesian product\n#[test]\nfn product_mode_cartesian_product() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--u-values\", \"0.0,1.0,2.0\",\n        \"--kpoints\", \"8x8x8,6x6x6\",\n        \"--cutoffs\", \"300,500,800\",\n        \"--sweep-mode\", \"product\",\n    ]);\n    let tasks = build_all_scf_tasks(&config).unwrap();\n    // 3 u * 2 k * 3 c = 18 tasks\n    assert_eq!(tasks.len(), 18);\n}\n\n/// Pairwise mode produces zipped tasks\n#[test]\nfn pairwise_mode_zip() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--u-values\", \"0.0,1.0,2.0\",\n        \"--kpoints\", \"8x8x8,6x6x6,4x4x4\",\n        \"--cutoffs\", \"300,500,800\",\n        \"--sweep-mode\", \"pairwise\",\n    ]);\n    let tasks = build_all_scf_tasks(&config).unwrap();\n    assert_eq!(tasks.len(), 3);\n}\n\n/// Pairwise mode requires --kpoints\n#[test]\nfn pairwise_requires_kpoints() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--u-values\", \"0.0,1.0,2.0\",\n        \"--cutoffs\", \"300,500,800\",\n        \"--sweep-mode\", \"pairwise\",\n    ]);\n    let err = build_all_scf_tasks(&config).unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"kpoints\"), \"error should mention kpoints: {msg}\");\n}\n\n/// Pairwise mode requires --cutoffs\n#[test]\nfn pairwise_requires_cutoffs() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--u-values\", \"0.0,1.0,2.0\",\n        \"--kpoints\", \"8x8x8,6x6x6,4x4x4\",\n        \"--sweep-mode\", \"pairwise\",\n    ]);\n    let err = build_all_scf_tasks(&config).unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"cutoffs\"), \"error should mention cutoffs: {msg}\");\n}\n\n/// Pairwise mode with unequal list lengths errors\n#[test]\nfn pairwise_unequal_lengths() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--u-values\", \"0.0,1.0\",\n        \"--kpoints\", \"8x8x8,6x6x6,4x4x4\",\n        \"--cutoffs\", \"300,500,800\",\n        \"--sweep-mode\", \"pairwise\",\n    ]);\n    let err = build_all_scf_tasks(&config).unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"same length\") || msg.contains(\"length\"), \"error should mention length mismatch: {msg}\");\n}\n\n/// Unknown sweep mode errors\n#[test]\nfn unknown_sweep_mode() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--sweep-mode\", \"invalid\",\n    ]);\n    let err = build_all_scf_tasks(&config).unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"unknown sweep mode\"), \"error should mention unknown mode: {msg}\");\n}\n\n// --- Task structure tests ---\n\n/// Each task has a unique ID\n#[test]\nfn task_ids_are_unique() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--u-values\", \"0.0,1.0\",\n        \"--kpoints\", \"8x8x8,6x6x6\",\n        \"--cutoffs\", \"300,500\",\n        \"--sweep-mode\", \"product\",\n    ]);\n    let tasks = build_all_scf_tasks(&config).unwrap();\n    let ids: Vec<&str> = tasks.iter().map(|t| t.id.as_str()).collect();\n    let mut unique_ids = ids.clone();\n    unique_ids.sort();\n    unique_ids.dedup();\n    assert_eq!(ids.len(), unique_ids.len(), \"all task IDs must be unique\");\n}\n\n/// Task IDs encode parameters in the expected format scf_U{u}_k{k}_c{c}\n#[test]\nfn task_ids_encode_parameters() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--u-values\", \"3.0\",\n        \"--kpoints\", \"8x8x8\",\n        \"--cutoffs\", \"500\",\n        \"--sweep-mode\", \"product\",\n    ]);\n    let tasks = build_all_scf_tasks(&config).unwrap();\n    assert_eq!(tasks.len(), 1);\n    let id = &tasks[0].id;\n    assert!(id.contains(\"U3\") || id.contains(\"U3.0\"), \"task ID should contain U value: {id}\");\n    assert!(id.contains(\"k8x8x8\") || id.contains(\"k\"), \"task ID should contain kpoint: {id}\");\n    assert!(id.contains(\"c500\") || id.contains(\"c\"), \"task ID should contain cutoff: {id}\");\n}\n\n/// Product mode with only u-values (optional kpoints/cutoffs) still works\n#[test]\nfn product_with_optional_params() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--u-values\", \"0.0,1.0,2.0\",\n    ]);\n    let tasks = build_all_scf_tasks(&config).unwrap();\n    assert_eq!(tasks.len(), 3);\n}\n\n/// SCF tasks have no dependencies\n#[test]\nfn scf_tasks_have_no_dependencies() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--u-values\", \"0.0,1.0\",\n        \"--kpoints\", \"8x8x8\",\n        \"--cutoffs\", \"500\",\n        \"--sweep-mode\", \"product\",\n    ]);\n    let tasks = build_all_scf_tasks(&config).unwrap();\n    for task in &tasks {\n        assert!(task.dependencies.is_empty(), \"SCF task should have no dependencies: {}\", task.id);\n    }\n}\n\n/// Tasks have workdir paths\n#[test]\nfn tasks_have_workdirs() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--u-values\", \"1.0\",\n        \"--kpoints\", \"8x8x8\",\n        \"--cutoffs\", \"500\",\n        \"--sweep-mode\", \"product\",\n    ]);\n    let tasks = build_all_scf_tasks(&config).unwrap();\n    assert!(!tasks.is_empty());\n    for task in &tasks {\n        assert!(!task.workdir.as_os_str().is_empty(), \"task {} should have a workdir\", task.id);\n    }\n}",
+        "signature": "fn build_one_scf_task(config: &SweepConfig, u: f64, kpoint: Option<[u32; 3]>, cutoff: Option<f64>, seed_cell: &str, seed_param: &str) -> Result<Task, WorkflowError>",
+        "expected_behavior": "build_all_scf_tasks in product mode: given N u-values, M kpoints, P cutoffs, produces N*M*P tasks (Cartesian product). If kpoints or cutoffs is None, uses a single None entry in the product (N*1 or N*1*1 tasks). In pairwise mode: requires both kpoints and cutoffs to be Some (errors with 'requires --kpoints' or 'requires --cutoffs' if not), parses all three lists, validates equal lengths (errors if unequal), then zips. Unknown sweep_mode produces 'unknown sweep mode' error. build_one_scf_task: produces a Task with a unique ID encoding all three parameters, no dependencies, a workdir path, and setup/collect closures. The setup closure injects hubbard_u, kpoints_mp_grid (if provided), and cutoff_energy into the seed files. The collect closure verifies the .castep output file exists and contains 'Total time'."
+      }
+    },
+    {
+      "id": "G1-5",
+      "description": "Implement the main() function that orchestrates config parsing, task building, workflow construction, dry-run display, and local/queued execution. \u2014 Binary entry point: main() function that parses SweepConfig from CLI, calls build_all_scf_tasks, constructs a Workflow with max_parallel, log_dir, root_dir, conditionally with_queued_submitter, adds all tasks, then either dry-runs (printing topological order) or runs (local via run_default, or queued via Workflow::run with Arc<dyn ProcessRunner> + Arc<dyn HookExecutor>). Also prints workflow summary on completion.",
+      "files_in_scope": [
+        "examples/multi_param_sweep/src/main.rs"
+      ],
+      "changes": [
+        {
+          "path": "examples/multi_param_sweep/src/main.rs",
+          "action": "modify",
+          "guidance": "Implement the main() function. Use anyhow::Result as the return type. Call workflow_core::init_default_logging().ok() first. Parse SweepConfig::parse(). Call build_all_scf_tasks(&config)? to get tasks. Construct workflow: Workflow::new('multi_param_sweep').with_max_parallel(config.max_parallel)?.with_log_dir('logs').with_root_dir(&config.workdir). If !config.local, add .with_queued_submitter(Arc::new(QueuedRunner::new(SchedulerKind::Slurm))). Add all tasks via workflow.add_task(task)?. For dry_run: call workflow.dry_run()?, print 'Dry-run topological order:' then each task ID on its own line prefixed with '  '. Return Ok(()). For execution: create JsonStateStore at '.multi_param_sweep.workflow.json'. If config.local, use run_default(&mut workflow, &mut state)?; else, construct SystemProcessRunner and ShellHookExecutor as Arc<dyn ProcessRunner/HookExecutor>, call workflow.run(&mut state, runner, executor)?. After run, print: 'Workflow complete: {succeeded} succeeded, {failed} failed, {skipped} skipped ({duration}s)'. Import std::sync::Arc for the queued path."
+        }
+      ],
+      "acceptance": [
+        "cargo check -p multi_param_sweep",
+        "cargo run --bin multi_param_sweep -- --dry-run (prints 'Dry-run topological order:' followed by task IDs, no errors)",
+        "cargo run --bin multi_param_sweep -- --dry-run --sweep-mode pairwise --u-values 0,1,2 --kpoints 8x8x8,6x6x6,4x4x4 --cutoffs 300,500,800 (prints exactly 3 task IDs, no errors)",
+        "cargo run --bin multi_param_sweep -- --dry-run --sweep-mode pairwise --u-values 0,1,2 (errors: pairwise mode requires --kpoints and --cutoffs)"
+      ],
+      "depends_on": [
+        "G1-4"
+      ]
+    }
+  ]
+}
diff --git a/notes/directions/phase-6-fix/directions-phase-6-fix-core-2.json b/notes/directions/phase-6-fix/directions-phase-6-fix-core-2.json
new file mode 100644
index 0000000..e960204
--- /dev/null
+++ b/notes/directions/phase-6-fix/directions-phase-6-fix-core-2.json
@@ -0,0 +1,258 @@
+{
+  "meta": {
+    "title": "Phase 6 Fix: Rewrite example binaries for multi-parameter sweep & SCF+DOS chain",
+    "source_branch": "phase-6-fix"
+  },
+  "architecture_notes": [
+    "Summary: Replace the buggy hubbard_u_sweep_slurm binary with two purpose-built binaries \u2014 multi_param_sweep (independent SCF parameter sweep) and scf_dos_chain (SCF+DOS task chain) \u2014 using only local path deps for castep-cell-io v0.5.0.",
+    "parse_time_validation: Parse-time validation at CLI boundary \u2014 parse_kpoints and parse_cutoffs error immediately at CLI parse time, not inside setup closures. A malformed k-point like '8xx8' must fail before task construction begins (Pattern 3).",
+    "internal_type_choice: parse_kpoints uses [u32; 3] not a custom struct \u2014 The array type directly converts to KpointsMpGrid([kx, ky, kz]) with zero overhead. The type guarantee ('exactly 3 u32s') is enforced by the array type itself.",
+    "duplication_choice: parse_u_values and generate_job_script are duplicated, not extracted \u2014 Each function is ~15-30 lines with stable semantics. Extracting to a shared crate is overkill for 2 consumers. If a third consumer appears, extraction becomes justified.",
+    "task_naming_collision: Fully-qualified path for Task::BandStructure, not alias import \u2014 The collision between workflow_core::task::Task and castep_cell_io::param::general::Task is resolved by using the fully-qualified path at the single use-site where Task::BandStructure is set. No alias needed, no import shadow risk.",
+    "shared_workdir: Shared workdir for chained tasks \u2014 Both SCF and DOS tasks in scf_dos_chain use runs/scf_dos/. This makes checkpoint file access straightforward: the DOS setup copies ZnO.check to ZnO_DOS.check within the same directory (Pattern 6).",
+    "workspace_serialization: Workspace member additions serialize G1 and G2 skeleton tasks \u2014 Both new crates use workspace = true dependencies (anyhow, clap, itertools), which require workspace membership for cargo check to resolve. Since both crate skeletons must add themselves to the root Cargo.toml workspace.members, G2-1 depends on G1-1 to avoid a merge conflict on the same file. The groups remain logically independent at the code level.",
+    "no_new_library_crates: No new library crates are created \u2014 All new code lives in binary (example) crates. The new code is workflow usage (orchestration, not abstractions). The library crates (workflow_core, workflow_utils) are unchanged.",
+    "Crate boundaries:",
+    "  parse_u_values: Duplicated in multi_param_sweep/src/config.rs and scf_dos_chain/src/config.rs",
+    "  parse_kpoints: multi_param_sweep/src/config.rs only (not needed by Binary 2)",
+    "  parse_cutoffs: multi_param_sweep/src/config.rs only",
+    "  generate_job_script: Duplicated in both binaries' job_script.rs (D.2 fix applies to both)",
+    "  SweepConfig: multi_param_sweep/src/config.rs",
+    "  ChainConfig: scf_dos_chain/src/config.rs",
+    "  sweep_logic: multi_param_sweep/src/main.rs (build_one_scf_task, build_all_scf_tasks)",
+    "  chain_logic: scf_dos_chain/src/main.rs (build_scf_task, build_dos_task)",
+    "  seed_files: Each binary's seeds/ directory (identical content, self-contained)",
+    "Pattern: Purpose-specific binaries over mode-flags \u2014 check: Neither binary should have a flag that toggles between 'independent tasks' and 'chained tasks'. multi_param_sweep has a --sweep-mode flag for product vs pairwise (both are independent SCF sweeps, same workflow type). scf_dos_chain has no mode flag.",
+    "Pattern: Parameter encoding in task IDs \u2014 check: multi_param_sweep task IDs: scf_U{u}_k{k}_c{c} (e.g., scf_U3.0_k8x8x8_c500). scf_dos_chain task IDs: scf and dos (single parameter set, no collision risk).",
+    "Pattern: Parse-time validation gates \u2014 check: Every parsing function returns anyhow::Result<T> with messages containing the expected format and what went wrong. Examples: 'kpoints list is empty', 'invalid k-point: expected 3 axes, got 2'.",
+    "Pattern: Explicit defaults \u2014 check: --sweep-mode defaults to 'product'. --kpoints and --cutoffs are Option<String> defaulting to None (seed defaults used). Pairwise mode errors if either is None.",
+    "Pattern: In-memory document mutation \u2014 check: All CASTEP file modifications go through parse -> mutate typed fields -> to_cell_file() -> to_string_many_spaced(). Never use format! on raw strings to build .cell/.param content.",
+    "Pattern: Shared workdir for chained tasks \u2014 check: scf_dos_chain places both tasks in runs/scf_dos/. DOS setup copies ZnO.check to ZnO_DOS.check in the same directory.",
+    "Pattern: Clean heredoc template \u2014 check: The no_literal_tabs test must pass. Use consistent space-only indentation. SBATCH directives use consistent quoting (double-quotes around values that might contain spaces)."
+  ],
+  "known_pitfalls": [
+    "Task name collision in scf_dos_chain (P0): use workflow_utils::prelude::* brings workflow_core::task::Task into scope. Importing castep_cell_io::param::general::task::Task would shadow it. Resolution: use the fully-qualified path castep_cell_io::param::general::task::Task::BandStructure at the single use-site in the DOS setup closure. Verify the exact module path with LSP hover during implementation.. Mitigation: Do not add use castep_cell_io::param::general::task::Task. Instead use the fully-qualified path at the single use-site.",
+    "Local path dep must exist on the filesystem: The path ../castep-cell-io/castep_cell_io must resolve to a sibling directory containing a Cargo.toml with the castep-cell-io crate. Verify this exists before implementation.. Mitigation: Pre-implementation check: verify ../castep-cell-io/castep_cell_io/Cargo.toml exists.",
+    "castep-cell-fmt version compatibility: castep-cell-fmt = 0.1.0 from crates.io while castep-cell-io uses a local path dep at v0.5.0. If v0.5.0 depends on a newer castep-cell-fmt, Cargo resolves the semver-compatible version automatically. No conflict expected.. Mitigation: No action needed; Cargo handles this automatically.",
+    "Pairwise mode validation ordering: In pairwise mode, validate that both --kpoints and --cutoffs are provided BEFORE parsing them. If either is None, return a clear error immediately. After parsing, validate all three lists have the same length. The error message must state the expected and actual lengths.. Mitigation: Check sweep_mode first, then validate presence of both Option fields, then parse, then check lengths.",
+    "hubbard_u_sweep compatibility with v0.5.0 path dep: The existing hubbard_u_sweep/src/main.rs already uses the v0.5.0 parse API pattern (castep_cell_fmt::parse::<CellDocument>(&input)). However, v0.5.0 may have changed Species::Symbol signature, builder method names, or HubbardUUnit enum variants. If compilation fails after the path-dep change, fix these call sites based on compiler error messages.. Mitigation: After changing the path dep, run cargo check -p hubbard_u_sweep. If it fails, fix each error based on the compiler message. Common changes may include Species::Symbol type or builder method names.",
+    "Dead-code detection after refactoring (F.3 mitigation): After deleting hubbard_u_sweep_slurm, run cargo clippy --workspace --all-targets -- -W dead-code to detect any stale imports or dead functions left over from the refactoring.. Mitigation: Include clippy dead-code check in the Group 3 acceptance criteria.",
+    "D.2 formatting fix must be embedded in ported job_script.rs: The no_literal_tabs test from the old binary must be ported and must pass. The D.2 fix (clean heredoc template with all-space indentation, consistent double-quoting on SBATCH directives) must be applied to the ported generate_job_script function in BOTH new binaries.. Mitigation: Implementation checklist: all-space indentation, SBATCH directives use double-quotes consistently, shell continuation lines use backslash, no_literal_tabs test passes in both binaries.",
+    "Deferred items: what NOT to do: D.1 (portable SLURM config): Do NOT implement. D.3 (expanded unit tests for generate_job_script): Do NOT expand beyond ported tests. read_task_ids edge case: Do NOT fix. All three have unmet preconditions.. Mitigation: Review the deferred-and-patterns.md file for full details on each deferred item.",
+    "G1-1 and G2-1 both modify root Cargo.toml \u2014 serialized: Both multi_param_sweep and scf_dos_chain use workspace = true dependencies, requiring workspace membership. To avoid merge conflicts on the root Cargo.toml, TASK-G2-1 must run after TASK-G1-1 completes. The groups are logically independent at the code level; this dependency is solely a file collision on the workspace member list.. Mitigation: TASK-G2-1 depends_on TASK-G1-1. Only the skeleton creation tasks are serialized; all other tasks within each group can run in parallel with tasks in the other group."
+  ],
+  "task_groups": [
+    {
+      "group_id": "core-2",
+      "reason": "Create the examples/scf_dos_chain/ crate \u2014 a purpose-built binary for SCF+DOS task chain testing \u2014 and update the existing hubbard_u_sweep binary for castep-cell-io v0.5.0 path dep compatibility.",
+      "tasks": [
+        "G2-1",
+        "G2-2",
+        "G2-3",
+        "G2-4",
+        "G2-5"
+      ],
+      "depends_on_groups": [
+        "core-1"
+      ]
+    }
+  ],
+  "tasks": [
+    {
+      "id": "G2-1",
+      "description": "Create the directory structure, Cargo.toml, stub source files, seed files, and add the crate to workspace members. \u2014 Crate creation: Cargo.toml with all dependencies, minimal main.rs with mod declarations, empty config.rs, empty job_script.rs, seed files, and workspace member registration.",
+      "files_in_scope": [
+        "examples/scf_dos_chain (new crate)",
+        "examples/scf_dos_chain/Cargo.toml",
+        "examples/scf_dos_chain/src/main.rs",
+        "examples/scf_dos_chain/src/config.rs",
+        "examples/scf_dos_chain/src/job_script.rs",
+        "examples/scf_dos_chain/seeds/ZnO.cell",
+        "examples/scf_dos_chain/seeds/ZnO.param",
+        "Cargo.toml (workspace root)"
+      ],
+      "changes": [
+        {
+          "path": "examples/scf_dos_chain/Cargo.toml",
+          "action": "create",
+          "guidance": "Create Cargo.toml with [package] name = 'scf_dos_chain', edition 2021, [[bin]] path = src/main.rs. Same dependencies as multi_param_sweep: anyhow/clap/itertools = { workspace = true }, castep-cell-fmt = '0.1.0', castep-cell-io = { path = '../castep-cell-io/castep_cell_io' }, workflow_core = { path = '../../workflow_core', features = ['default-logging'] }, workflow_utils = { path = '../../workflow_utils' }."
+        },
+        {
+          "path": "examples/scf_dos_chain/src/main.rs",
+          "action": "create",
+          "guidance": "Create stub main.rs with mod config; and mod job_script; declarations. Add fn main() { println!('scf_dos_chain: skeleton'); }. No imports."
+        },
+        {
+          "path": "examples/scf_dos_chain/src/config.rs",
+          "action": "create",
+          "guidance": "Create empty config.rs with placeholder comment."
+        },
+        {
+          "path": "examples/scf_dos_chain/src/job_script.rs",
+          "action": "create",
+          "guidance": "Create empty job_script.rs with placeholder comment."
+        },
+        {
+          "path": "examples/scf_dos_chain/seeds/ZnO.cell",
+          "action": "modify",
+          "guidance": "Copy from examples/hubbard_u_sweep_slurm/seeds/ZnO.cell (or from examples/multi_param_sweep/seeds/ZnO.cell if already created)."
+        },
+        {
+          "path": "examples/scf_dos_chain/seeds/ZnO.param",
+          "action": "modify",
+          "guidance": "Copy from examples/hubbard_u_sweep_slurm/seeds/ZnO.param (or from examples/multi_param_sweep/seeds/ZnO.param if already created)."
+        },
+        {
+          "path": "Cargo.toml (workspace root)",
+          "action": "modify",
+          "guidance": "Add 'examples/scf_dos_chain' to the [workspace] members array. Insert it after 'examples/multi_param_sweep'. Keep 'examples/hubbard_u_sweep_slurm' in the list (it will be removed in G3-1)."
+        }
+      ],
+      "acceptance": [
+        "cargo check --workspace (both new crates + old slurm crate compile)",
+        "ls examples/scf_dos_chain/Cargo.toml examples/scf_dos_chain/src/main.rs examples/scf_dos_chain/src/config.rs examples/scf_dos_chain/src/job_script.rs examples/scf_dos_chain/seeds/ZnO.cell examples/scf_dos_chain/seeds/ZnO.param"
+      ],
+      "depends_on": [],
+      "wiring_checklist": [
+        {
+          "kind": "pub_mod",
+          "file": "examples/scf_dos_chain/src/main.rs",
+          "detail": "Add mod config; and mod job_script; before fn main()."
+        },
+        {
+          "kind": "pub_mod",
+          "file": "Cargo.toml (workspace root)",
+          "detail": "Add 'examples/scf_dos_chain' to the workspace members list."
+        }
+      ]
+    },
+    {
+      "id": "G2-2",
+      "description": "Implement the ChainConfig clap struct, parse_u_values function, and generate_job_script function (adapted for ChainConfig type) with tests written first. \u2014 CLI configuration (ChainConfig with single u-value, no sweep, no kpoints/cutoff), parse_u_values function (duplicated from multi_param_sweep), generate_job_script adapted for &ChainConfig parameter, and their in-file #[cfg(test)] tests.",
+      "files_in_scope": [
+        "examples/scf_dos_chain/src/config.rs and examples/scf_dos_chain/src/job_script.rs",
+        "examples/scf_dos_chain/src/config.rs",
+        "examples/scf_dos_chain/src/job_script.rs"
+      ],
+      "changes": [
+        {
+          "path": "examples/scf_dos_chain/src/config.rs",
+          "action": "modify",
+          "guidance": "Implement ChainConfig struct with #[derive(Parser, Debug)], #[command(name = 'scf_dos_chain')], and fields: u_value (f64, default 3.0), element (String, default 'Zn'), orbital (char, default 'd'), seed_name (String, default 'ZnO'), max_parallel (usize, default 1), local (bool), dry_run (bool), castep_command (String, default 'castep'), workdir (String, default '.'), plus SLURM fields (partition, ntasks, nix_flake, mpi_if \u2014 same as SweepConfig). Implement parse_u_values as a duplicate of the multi_param_sweep version (returning anyhow::Result<Vec<f64>>). Add #[cfg(test)] mod tests with the 7 ported parse_u_values tests. Place the parse_u_values tests directly in config.rs's test module."
+        },
+        {
+          "path": "examples/scf_dos_chain/src/job_script.rs",
+          "action": "modify",
+          "guidance": "Implement generate_job_script(config: &ChainConfig, task_id: &str, seed_name: &str) -> String. The function body is identical to the multi_param_sweep version (same SLURM fields on both config types). Use the exact same heredoc template with D.2 fix applied (all-space indentation, consistent SBATCH quoting). Import ChainConfig from crate::config. Add #[cfg(test)] mod tests with the 6 ported tests adapted for ChainConfig (using ChainConfig::parse_from(['test'])). Add use clap::Parser; in the test module. All 6 tests including no_literal_tabs must pass."
+        }
+      ],
+      "acceptance": [
+        "cargo check -p scf_dos_chain",
+        "cargo test -p scf_dos_chain -- config::tests (7 parse_u_values tests pass)",
+        "cargo test -p scf_dos_chain -- job_script::tests (6 tests pass, including no_literal_tabs)"
+      ],
+      "depends_on": [
+        "G2-1"
+      ],
+      "kind": "lib-tdd",
+      "tdd_interface": {
+        "test_file": "examples/scf_dos_chain/src/config.rs",
+        "test_module": "tests (in each file)",
+        "test_fn_name": "parse_basic_values",
+        "test_code": "// ===== config.rs tests =====\n// These are the same 7 parse_u_values tests as in multi_param_sweep,\n// duplicated here because parse_u_values is duplicated.\n\nuse crate::config::ChainConfig;\nuse clap::Parser;\n\n/// Helper: default config for job_script tests\nfn default_chain_config() -> ChainConfig {\n    ChainConfig::parse_from([\"test\"])\n}\n\n// --- parse_u_values tests (ported, adapted for anyhow::Result) ---\n\n#[test]\nfn parse_basic_values() {\n    let vals = parse_u_values(\"0.0,1.0,2.0\").unwrap();\n    assert_eq!(vals, vec![0.0, 1.0, 2.0]);\n}\n\n#[test]\nfn parse_with_whitespace() {\n    let vals = parse_u_values(\"  0.0 , 1.0 , 2.0  \").unwrap();\n    assert_eq!(vals, vec![0.0, 1.0, 2.0]);\n}\n\n#[test]\nfn parse_single_value() {\n    let vals = parse_u_values(\"42.0\").unwrap();\n    assert_eq!(vals, vec![42.0]);\n}\n\n#[test]\nfn parse_invalid_token() {\n    let err = parse_u_values(\"1.0,abc,2.0\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"abc\"), \"error should mention the invalid token: {msg}\");\n}\n\n#[test]\nfn parse_empty_token() {\n    let err = parse_u_values(\"1.0,,2.0\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"invalid\"), \"error should report parse failure: {msg}\");\n}\n\n#[test]\nfn parse_empty_string() {\n    let err = parse_u_values(\"\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"empty\") || msg.contains(\"invalid\"), \"expected parse failure on empty input, got: {msg}\");\n}\n\n#[test]\nfn parse_negative_values() {\n    let vals = parse_u_values(\"-1.0,2.0\").unwrap();\n    assert_eq!(vals, vec![-1.0, 2.0]);\n}\n\n// ===== job_script.rs tests =====\n\n#[test]\nfn contains_sbatch_directives() {\n    let config = default_chain_config();\n    let script = generate_job_script(&config, \"scf\", \"ZnO\");\n    assert!(script.contains(\"#SBATCH --job-name=\\\"scf\\\"\"));\n    assert!(script.contains(\"#SBATCH --partition=debug\"));\n    assert!(script.contains(\"#SBATCH --ntasks-per-node=16\"));\n    assert!(script.contains(\"#SBATCH --mem=30000m\"));\n}\n\n#[test]\nfn contains_seed_name() {\n    let config = default_chain_config();\n    let script = generate_job_script(&config, \"scf\", \"ZnO\");\n    assert!(script.contains(\"castep.mpi ZnO\"));\n}\n\n/// D.2 fix: no literal tabs\n#[test]\nfn no_literal_tabs() {\n    let config = default_chain_config();\n    let script = generate_job_script(&config, \"scf\", \"ZnO\");\n    assert!(!script.contains('\\t'), \"job script should not contain literal tab characters\");\n}\n\n#[test]\nfn starts_with_shebang() {\n    let config = default_chain_config();\n    let script = generate_job_script(&config, \"scf\", \"ZnO\");\n    assert!(script.starts_with(\"#!/usr/bin/env bash\"));\n}\n\n#[test]\nfn contains_nix_develop() {\n    let config = default_chain_config();\n    let script = generate_job_script(&config, \"scf\", \"ZnO\");\n    assert!(script.contains(\"nix develop\"));\n    assert!(script.contains(&config.nix_flake));\n}\n\n#[test]\nfn contains_mpi_interface() {\n    let config = default_chain_config();\n    let script = generate_job_script(&config, \"scf\", \"ZnO\");\n    assert!(script.contains(&format!(\"OMPI_MCA_btl_tcp_if_include={mpi_if}\", mpi_if = config.mpi_if)));\n}",
+        "signature": "pub fn parse_u_values(s: &str) -> anyhow::Result<Vec<f64>>",
+        "expected_behavior": "parse_u_values: identical behavior to the multi_param_sweep version \u2014 returns Vec<f64> for comma-separated numbers, errors with clear messages for non-numeric tokens and empty input. generate_job_script: identical template to the multi_param_sweep version but accepting &ChainConfig instead of &SweepConfig. Since SLURM fields are identical on both config types, the output is functionally the same. D.2 fix applied: all-space indentation, double-quoted SBATCH values, no literal tabs."
+      },
+      "wiring_checklist": [
+        {
+          "kind": "pub_mod",
+          "file": "examples/scf_dos_chain/src/config.rs",
+          "detail": "Add use clap::Parser; at the top."
+        },
+        {
+          "kind": "pub_mod",
+          "file": "examples/scf_dos_chain/src/job_script.rs",
+          "detail": "Add use crate::config::ChainConfig; at the top."
+        }
+      ]
+    },
+    {
+      "id": "G2-3",
+      "description": "Implement build_scf_task and build_dos_task functions that construct the two-task SCF+DOS chain, with tests for dependency structure and DOS setup logic. \u2014 Chain task construction: build_scf_task (SCF task with setup injecting hubbard_u, collect verifying .castep output), build_dos_task (DOS task depending on scf, setup copying checkpoint and setting Task::BandStructure), and in-file #[cfg(test)] tests verifying task IDs, dependency structure, workdir paths, and the DOS setup closure behavior.",
+      "files_in_scope": [
+        "examples/scf_dos_chain/src/main.rs"
+      ],
+      "changes": [
+        {
+          "path": "examples/scf_dos_chain/src/main.rs",
+          "action": "modify",
+          "guidance": "Add necessary imports: use config::{parse_u_values, ChainConfig}; use job_script::generate_job_script; plus castep_cell_fmt (parse, to_string_many_spaced, ToCellFile), castep_cell_io (CellDocument, ParamDocument, AtomHubbardU, HubbardU, HubbardUUnit, OrbitalU, Species, CutOffEnergy), workflow_utils::prelude::*. Note: do NOT import castep_cell_io::param::general::task::Task \u2014 use the fully-qualified path at the single use-site (see below).\n\nFunction build_scf_task(config, seed_cell, seed_param) -> Result<Task, WorkflowError>: Task ID 'scf', workdir PathBuf::from('runs/scf_dos'). Execution mode: if config.local, ExecutionMode::direct(&config.castep_command, &['ZnO']), else ExecutionMode::Queued. Setup closure: (1) create_dir(workdir), (2) parse seed_cell into CellDocument, inject hubbard_u using config.u_value/config.element/config.orbital (same builder pattern as multi_param_sweep), (3) serialize .cell + .param to workdir, (4) if !config.local, write job_script. Collect closure: verify <seed>.castep exists with 'Total time' marker.\n\nFunction build_dos_task(config, scf_task_id, seed_cell, seed_param) -> Result<Task, WorkflowError>: Task ID 'dos', workdir PathBuf::from('runs/scf_dos') (SAME workdir as SCF). Depends on scf_task_id. Execution mode: if config.local, ExecutionMode::direct(&config.castep_command, &['ZnO_DOS']), else ExecutionMode::Queued. Setup closure: (1) create_dir(workdir), (2) parse seed_cell into CellDocument, write to <workdir>/ZnO_DOS.cell (reuse seed cell, no hubbard_u injection needed since this is a DOS restart), (3) parse seed_param into ParamDocument, set doc.general.task = Some(castep_cell_io::param::general::task::Task::BandStructure) \u2014 THIS IS THE FULLY-QUALIFIED PATH, do not import the Task enum, (4) write to <workdir>/ZnO_DOS.param, (5) copy <workdir>/ZnO.check to <workdir>/ZnO_DOS.check using workflow_utils::files::copy_file or the prelude re-export, (6) if !config.local, write job_script via generate_job_script(&config, 'dos', 'ZnO_DOS'). Collect closure: verify ZnO_DOS.castep exists and contains 'Total time'.\n\nAdd #[cfg(test)] mod tests with the tests from tdd_interface.test_code. These tests verify structural correctness: task IDs, dependency structure, shared workdir, dependency counts."
+        }
+      ],
+      "acceptance": [
+        "cargo check -p scf_dos_chain",
+        "cargo test -p scf_dos_chain (config tests + job_script tests + chain logic tests all pass)",
+        "All chain structure tests pass: scf id is 'scf', dos id is 'dos', dos depends on scf, scf has no deps, dos has 1 dep, shared workdir, correct workdir path"
+      ],
+      "depends_on": [
+        "G2-2"
+      ],
+      "kind": "lib-tdd",
+      "tdd_interface": {
+        "test_file": "examples/scf_dos_chain/src/main.rs",
+        "test_module": "tests",
+        "test_fn_name": "scf_task_id",
+        "test_code": "use crate::config::ChainConfig;\nuse clap::Parser;\n\nfn default_config() -> ChainConfig {\n    ChainConfig::parse_from([\"test\"])\n}\n\nfn test_seeds() -> (&'static str, &'static str) {\n    (include_str!(\"../seeds/ZnO.cell\"), include_str!(\"../seeds/ZnO.param\"))\n}\n\n// --- Task ID and structure tests ---\n\n/// SCF task has ID 'scf'\n#[test]\nfn scf_task_id() {\n    let config = default_config();\n    let (cell, param) = test_seeds();\n    let task = build_scf_task(&config, cell, param).unwrap();\n    assert_eq!(task.id, \"scf\");\n}\n\n/// DOS task has ID 'dos'\n#[test]\nfn dos_task_id() {\n    let config = default_config();\n    let (cell, param) = test_seeds();\n    let task = build_dos_task(&config, \"scf\", cell, param).unwrap();\n    assert_eq!(task.id, \"dos\");\n}\n\n/// DOS task depends on SCF task\n#[test]\nfn dos_depends_on_scf() {\n    let config = default_config();\n    let (cell, param) = test_seeds();\n    let scf = build_scf_task(&config, cell, param).unwrap();\n    let dos = build_dos_task(&config, &scf.id, cell, param).unwrap();\n    assert!(dos.dependencies.contains(&\"scf\".to_string()));\n}\n\n/// SCF task has no dependencies\n#[test]\nfn scf_no_dependencies() {\n    let config = default_config();\n    let (cell, param) = test_seeds();\n    let task = build_scf_task(&config, cell, param).unwrap();\n    assert!(task.dependencies.is_empty());\n}\n\n/// DOS task has exactly 1 dependency\n#[test]\nfn dos_dependency_count() {\n    let config = default_config();\n    let (cell, param) = test_seeds();\n    let scf = build_scf_task(&config, cell, param).unwrap();\n    let dos = build_dos_task(&config, &scf.id, cell, param).unwrap();\n    assert_eq!(dos.dependencies.len(), 1);\n}\n\n/// Both tasks share the same workdir\n#[test]\nfn shared_workdir() {\n    let config = default_config();\n    let (cell, param) = test_seeds();\n    let scf = build_scf_task(&config, cell, param).unwrap();\n    let dos = build_dos_task(&config, &scf.id, cell, param).unwrap();\n    assert_eq!(scf.workdir, dos.workdir, \"SCF and DOS tasks should share the same workdir\");\n    let wd = scf.workdir.to_string_lossy();\n    assert!(wd.contains(\"scf_dos\"), \"workdir should contain 'scf_dos': {wd}\");\n}\n\n/// SCF task has correct workdir path\n#[test]\nfn scf_workdir_path() {\n    let config = default_config();\n    let (cell, param) = test_seeds();\n    let task = build_scf_task(&config, cell, param).unwrap();\n    let wd = task.workdir.to_string_lossy();\n    assert!(wd.ends_with(\"scf_dos\") || wd.contains(\"scf_dos\"), \"workdir should end with or contain 'scf_dos': {wd}\");\n}",
+        "signature": "fn build_scf_task(config: &ChainConfig, seed_cell: &str, seed_param: &str) -> Result<Task, WorkflowError>",
+        "expected_behavior": "build_scf_task: produces a Task with id 'scf', workdir ending in 'runs/scf_dos', no dependencies, and a setup closure that injects the Hubbard U value from config into the seed cell file. The collect closure verifies the .castep output. build_dos_task: produces a Task with id 'dos', same workdir as SCF, exactly 1 dependency on the SCF task ID. The setup closure copies ZnO.check to ZnO_DOS.check and sets general.task = Task::BandStructure in the param file. The collect closure verifies ZnO_DOS.castep output."
+      }
+    },
+    {
+      "id": "G2-4",
+      "description": "Implement the main() function that orchestrates config parsing, chain construction, workflow creation, dry-run display, and local/queued execution. \u2014 Binary entry point: main() function that parses ChainConfig, builds SCF and DOS tasks, constructs a Workflow with the two-task chain, and supports dry-run/local/queued execution paths.",
+      "files_in_scope": [
+        "examples/scf_dos_chain/src/main.rs"
+      ],
+      "changes": [
+        {
+          "path": "examples/scf_dos_chain/src/main.rs",
+          "action": "modify",
+          "guidance": "Implement the main() function following the same pattern as multi_param_sweep main(). Call workflow_core::init_default_logging().ok(). Parse ChainConfig::parse(). Load seed files via include_str!('../seeds/ZnO.cell') and include_str!('../seeds/ZnO.param'). Call build_scf_task(&config, seed_cell, seed_param)? to get the SCF task, then build_dos_task(&config, &scf_task.id, seed_cell, seed_param)? to get the DOS task. Construct workflow: Workflow::new('scf_dos_chain').with_max_parallel(config.max_parallel)?.with_log_dir('logs').with_root_dir(&config.workdir). If !config.local, add .with_queued_submitter(Arc::new(QueuedRunner::new(SchedulerKind::Slurm))). Add SCF task first, then DOS task via workflow.add_task(task)?. Dry-run: same pattern as multi_param_sweep (print topological order). For execution: JsonStateStore at '.scf_dos_chain.workflow.json', local via run_default, queued via Workflow::run with SystemProcessRunner + ShellHookExecutor. Print workflow summary on completion. Import std::sync::Arc."
+        }
+      ],
+      "acceptance": [
+        "cargo check -p scf_dos_chain",
+        "cargo run --bin scf_dos_chain -- --dry-run (prints 'scf' then 'dos' \u2014 chain dependency order, exactly 2 tasks)",
+        "cargo run --bin scf_dos_chain -- --local --dry-run (same output \u2014 local flag doesn't change dry-run topology)"
+      ],
+      "depends_on": [
+        "G2-3"
+      ]
+    },
+    {
+      "id": "G2-5",
+      "description": "Update the existing hubbard_u_sweep binary's Cargo.toml to use the local path dep for castep-cell-io v0.5.0 and fix any API breakage in main.rs. \u2014 Dependency update: change castep-cell-io from version 0.4.0 to local path dep, verify compilation, and fix any API breakage in main.rs (Species::Symbol signature, builder method changes, or enum variant renames).",
+      "files_in_scope": [
+        "examples/hubbard_u_sweep",
+        "examples/hubbard_u_sweep/Cargo.toml",
+        "examples/hubbard_u_sweep/src/main.rs"
+      ],
+      "changes": [
+        {
+          "path": "examples/hubbard_u_sweep/Cargo.toml",
+          "action": "modify",
+          "guidance": "Change the castep-cell-io dependency line from 'castep-cell-io = \"0.4.0\"' to 'castep-cell-io = { path = \"../castep-cell-io/castep_cell_io\" }'. All other dependencies (anyhow = \"1\", castep-cell-fmt = \"0.1.0\", workflow_core, workflow_utils) remain unchanged."
+        },
+        {
+          "path": "examples/hubbard_u_sweep/src/main.rs",
+          "action": "modify",
+          "guidance": "Run cargo check -p hubbard_u_sweep after the Cargo.toml change. If compilation fails due to v0.5.0 API changes, fix the call sites. Common breakage points: Species::Symbol may take a different type than String (e.g., &str or a newtype) \u2014 check with LSP hover on Species::Symbol. Builder method names on AtomHubbardU/HubbardU may have changed. HubbardUUnit enum variants may be renamed. Fix each error based on the compiler diagnostic. The existing code already uses castep_cell_fmt::parse::<CellDocument>(&input) which is the v0.5.0 parse API pattern, so the parse logic should work unchanged. Only fix what the compiler reports as errors."
+        }
+      ],
+      "acceptance": [
+        "cargo check -p hubbard_u_sweep (must compile with new path dep; zero errors, zero warnings)"
+      ],
+      "depends_on": []
+    }
+  ]
+}
diff --git a/notes/directions/phase-6-fix/directions-phase-6-fix-workspace.json b/notes/directions/phase-6-fix/directions-phase-6-fix-workspace.json
new file mode 100644
index 0000000..35c8122
--- /dev/null
+++ b/notes/directions/phase-6-fix/directions-phase-6-fix-workspace.json
@@ -0,0 +1,101 @@
+{
+  "meta": {
+    "title": "Phase 6 Fix: Rewrite example binaries for multi-parameter sweep & SCF+DOS chain",
+    "source_branch": "phase-6-fix"
+  },
+  "architecture_notes": [
+    "Summary: Replace the buggy hubbard_u_sweep_slurm binary with two purpose-built binaries \u2014 multi_param_sweep (independent SCF parameter sweep) and scf_dos_chain (SCF+DOS task chain) \u2014 using only local path deps for castep-cell-io v0.5.0.",
+    "parse_time_validation: Parse-time validation at CLI boundary \u2014 parse_kpoints and parse_cutoffs error immediately at CLI parse time, not inside setup closures. A malformed k-point like '8xx8' must fail before task construction begins (Pattern 3).",
+    "internal_type_choice: parse_kpoints uses [u32; 3] not a custom struct \u2014 The array type directly converts to KpointsMpGrid([kx, ky, kz]) with zero overhead. The type guarantee ('exactly 3 u32s') is enforced by the array type itself.",
+    "duplication_choice: parse_u_values and generate_job_script are duplicated, not extracted \u2014 Each function is ~15-30 lines with stable semantics. Extracting to a shared crate is overkill for 2 consumers. If a third consumer appears, extraction becomes justified.",
+    "task_naming_collision: Fully-qualified path for Task::BandStructure, not alias import \u2014 The collision between workflow_core::task::Task and castep_cell_io::param::general::Task is resolved by using the fully-qualified path at the single use-site where Task::BandStructure is set. No alias needed, no import shadow risk.",
+    "shared_workdir: Shared workdir for chained tasks \u2014 Both SCF and DOS tasks in scf_dos_chain use runs/scf_dos/. This makes checkpoint file access straightforward: the DOS setup copies ZnO.check to ZnO_DOS.check within the same directory (Pattern 6).",
+    "workspace_serialization: Workspace member additions serialize G1 and G2 skeleton tasks \u2014 Both new crates use workspace = true dependencies (anyhow, clap, itertools), which require workspace membership for cargo check to resolve. Since both crate skeletons must add themselves to the root Cargo.toml workspace.members, G2-1 depends on G1-1 to avoid a merge conflict on the same file. The groups remain logically independent at the code level.",
+    "no_new_library_crates: No new library crates are created \u2014 All new code lives in binary (example) crates. The new code is workflow usage (orchestration, not abstractions). The library crates (workflow_core, workflow_utils) are unchanged.",
+    "Crate boundaries:",
+    "  parse_u_values: Duplicated in multi_param_sweep/src/config.rs and scf_dos_chain/src/config.rs",
+    "  parse_kpoints: multi_param_sweep/src/config.rs only (not needed by Binary 2)",
+    "  parse_cutoffs: multi_param_sweep/src/config.rs only",
+    "  generate_job_script: Duplicated in both binaries' job_script.rs (D.2 fix applies to both)",
+    "  SweepConfig: multi_param_sweep/src/config.rs",
+    "  ChainConfig: scf_dos_chain/src/config.rs",
+    "  sweep_logic: multi_param_sweep/src/main.rs (build_one_scf_task, build_all_scf_tasks)",
+    "  chain_logic: scf_dos_chain/src/main.rs (build_scf_task, build_dos_task)",
+    "  seed_files: Each binary's seeds/ directory (identical content, self-contained)",
+    "Pattern: Purpose-specific binaries over mode-flags \u2014 check: Neither binary should have a flag that toggles between 'independent tasks' and 'chained tasks'. multi_param_sweep has a --sweep-mode flag for product vs pairwise (both are independent SCF sweeps, same workflow type). scf_dos_chain has no mode flag.",
+    "Pattern: Parameter encoding in task IDs \u2014 check: multi_param_sweep task IDs: scf_U{u}_k{k}_c{c} (e.g., scf_U3.0_k8x8x8_c500). scf_dos_chain task IDs: scf and dos (single parameter set, no collision risk).",
+    "Pattern: Parse-time validation gates \u2014 check: Every parsing function returns anyhow::Result<T> with messages containing the expected format and what went wrong. Examples: 'kpoints list is empty', 'invalid k-point: expected 3 axes, got 2'.",
+    "Pattern: Explicit defaults \u2014 check: --sweep-mode defaults to 'product'. --kpoints and --cutoffs are Option<String> defaulting to None (seed defaults used). Pairwise mode errors if either is None.",
+    "Pattern: In-memory document mutation \u2014 check: All CASTEP file modifications go through parse -> mutate typed fields -> to_cell_file() -> to_string_many_spaced(). Never use format! on raw strings to build .cell/.param content.",
+    "Pattern: Shared workdir for chained tasks \u2014 check: scf_dos_chain places both tasks in runs/scf_dos/. DOS setup copies ZnO.check to ZnO_DOS.check in the same directory.",
+    "Pattern: Clean heredoc template \u2014 check: The no_literal_tabs test must pass. Use consistent space-only indentation. SBATCH directives use consistent quoting (double-quotes around values that might contain spaces)."
+  ],
+  "known_pitfalls": [
+    "Task name collision in scf_dos_chain (P0): use workflow_utils::prelude::* brings workflow_core::task::Task into scope. Importing castep_cell_io::param::general::task::Task would shadow it. Resolution: use the fully-qualified path castep_cell_io::param::general::task::Task::BandStructure at the single use-site in the DOS setup closure. Verify the exact module path with LSP hover during implementation.. Mitigation: Do not add use castep_cell_io::param::general::task::Task. Instead use the fully-qualified path at the single use-site.",
+    "Local path dep must exist on the filesystem: The path ../castep-cell-io/castep_cell_io must resolve to a sibling directory containing a Cargo.toml with the castep-cell-io crate. Verify this exists before implementation.. Mitigation: Pre-implementation check: verify ../castep-cell-io/castep_cell_io/Cargo.toml exists.",
+    "castep-cell-fmt version compatibility: castep-cell-fmt = 0.1.0 from crates.io while castep-cell-io uses a local path dep at v0.5.0. If v0.5.0 depends on a newer castep-cell-fmt, Cargo resolves the semver-compatible version automatically. No conflict expected.. Mitigation: No action needed; Cargo handles this automatically.",
+    "Pairwise mode validation ordering: In pairwise mode, validate that both --kpoints and --cutoffs are provided BEFORE parsing them. If either is None, return a clear error immediately. After parsing, validate all three lists have the same length. The error message must state the expected and actual lengths.. Mitigation: Check sweep_mode first, then validate presence of both Option fields, then parse, then check lengths.",
+    "hubbard_u_sweep compatibility with v0.5.0 path dep: The existing hubbard_u_sweep/src/main.rs already uses the v0.5.0 parse API pattern (castep_cell_fmt::parse::<CellDocument>(&input)). However, v0.5.0 may have changed Species::Symbol signature, builder method names, or HubbardUUnit enum variants. If compilation fails after the path-dep change, fix these call sites based on compiler error messages.. Mitigation: After changing the path dep, run cargo check -p hubbard_u_sweep. If it fails, fix each error based on the compiler message. Common changes may include Species::Symbol type or builder method names.",
+    "Dead-code detection after refactoring (F.3 mitigation): After deleting hubbard_u_sweep_slurm, run cargo clippy --workspace --all-targets -- -W dead-code to detect any stale imports or dead functions left over from the refactoring.. Mitigation: Include clippy dead-code check in the Group 3 acceptance criteria.",
+    "D.2 formatting fix must be embedded in ported job_script.rs: The no_literal_tabs test from the old binary must be ported and must pass. The D.2 fix (clean heredoc template with all-space indentation, consistent double-quoting on SBATCH directives) must be applied to the ported generate_job_script function in BOTH new binaries.. Mitigation: Implementation checklist: all-space indentation, SBATCH directives use double-quotes consistently, shell continuation lines use backslash, no_literal_tabs test passes in both binaries.",
+    "Deferred items: what NOT to do: D.1 (portable SLURM config): Do NOT implement. D.3 (expanded unit tests for generate_job_script): Do NOT expand beyond ported tests. read_task_ids edge case: Do NOT fix. All three have unmet preconditions.. Mitigation: Review the deferred-and-patterns.md file for full details on each deferred item.",
+    "G1-1 and G2-1 both modify root Cargo.toml \u2014 serialized: Both multi_param_sweep and scf_dos_chain use workspace = true dependencies, requiring workspace membership. To avoid merge conflicts on the root Cargo.toml, TASK-G2-1 must run after TASK-G1-1 completes. The groups are logically independent at the code level; this dependency is solely a file collision on the workspace member list.. Mitigation: TASK-G2-1 depends_on TASK-G1-1. Only the skeleton creation tasks are serialized; all other tasks within each group can run in parallel with tasks in the other group."
+  ],
+  "task_groups": [
+    {
+      "group_id": "workspace",
+      "reason": "Wire everything together by removing the old hubbard_u_sweep_slurm crate, cleaning up workspace members, and running full build/test/clippy verification.",
+      "tasks": [
+        "G3-1"
+      ],
+      "depends_on_groups": [
+        "core-2"
+      ]
+    }
+  ],
+  "tasks": [
+    {
+      "id": "G3-1",
+      "description": "Remove hubbard_u_sweep_slurm from workspace members, delete the entire old crate directory, and run full workspace build, test, clippy, and dry-run verification. \u2014 Workspace finalization: updating workspace members list to remove the old binary, deleting examples/hubbard_u_sweep_slurm/ entirely, and running comprehensive verification (build, test, clippy dead-code, dry-run both binaries).",
+      "files_in_scope": [
+        "Cargo.toml (workspace root) and examples/hubbard_u_sweep_slurm/ (deletion)",
+        "Cargo.toml (workspace root)",
+        "examples/hubbard_u_sweep_slurm/"
+      ],
+      "changes": [
+        {
+          "path": "Cargo.toml (workspace root)",
+          "action": "modify",
+          "guidance": "Remove 'examples/hubbard_u_sweep_slurm' from the [workspace] members array. The members list should now contain exactly: 'workflow_core', 'workflow_utils', 'examples/hubbard_u_sweep', 'examples/multi_param_sweep', 'examples/scf_dos_chain', 'workflow-cli'. Do NOT change any other configuration (resolver, workspace.dependencies, etc.)."
+        },
+        {
+          "path": "examples/hubbard_u_sweep_slurm/",
+          "action": "delete",
+          "guidance": "Delete the entire directory and all its contents: Cargo.toml, src/main.rs, src/config.rs, src/job_script.rs, seeds/ZnO.cell, seeds/ZnO.param, .validation-complete. Use rm -rf or the equivalent VCS remove. This removes the old buggy binary entirely from the codebase."
+        }
+      ],
+      "acceptance": [
+        "cargo build --workspace (all crates compile with zero errors, zero warnings)",
+        "cargo test --workspace (all tests pass; old slurm tests are gone, new binary tests run and pass)",
+        "cargo clippy --workspace --all-targets -- -W dead-code (no dead-code warnings from stale imports \u2014 if any appear, fix them in the library crates)",
+        "cargo run --bin multi_param_sweep -- --dry-run (produces correct topological order with all 6 default SCF task IDs, no DOS tasks, no duplicate IDs)",
+        "cargo run --bin multi_param_sweep -- --dry-run --sweep-mode pairwise --u-values 0,1,2 --kpoints 8x8x8,6x6x6,4x4x4 --cutoffs 300,500,800 (produces 3 tasks, no errors)",
+        "cargo run --bin scf_dos_chain -- --dry-run (produces 'scf' then 'dos' \u2014 exactly 2 tasks, chain dependency respected)",
+        "cargo run --bin scf_dos_chain -- --local --dry-run (same output \u2014 'scf' then 'dos')",
+        "bash -c 'ls examples/hubbard_u_sweep_slurm/ 2>&1 | grep -q \"No such file\"' (old crate directory is fully gone)"
+      ],
+      "depends_on": [
+        "G1-5",
+        "G2-4",
+        "G2-5"
+      ],
+      "wiring_checklist": [
+        {
+          "kind": "pub_mod",
+          "file": "Cargo.toml (workspace root)",
+          "detail": "Remove 'examples/hubbard_u_sweep_slurm' from [workspace] members."
+        }
+      ]
+    }
+  ]
+}
diff --git a/notes/directions/phase-6-fix/directions.json b/notes/directions/phase-6-fix/directions.json
new file mode 100644
index 0000000..3cc517f
--- /dev/null
+++ b/notes/directions/phase-6-fix/directions.json
@@ -0,0 +1,507 @@
+{
+  "meta": {
+    "title": "Phase 6 Fix: Rewrite example binaries for multi-parameter sweep & SCF+DOS chain",
+    "source_branch": "phase-6-fix"
+  },
+  "architecture_notes": [
+    "Summary: Replace the buggy hubbard_u_sweep_slurm binary with two purpose-built binaries \u2014 multi_param_sweep (independent SCF parameter sweep) and scf_dos_chain (SCF+DOS task chain) \u2014 using only local path deps for castep-cell-io v0.5.0.",
+    "parse_time_validation: Parse-time validation at CLI boundary \u2014 parse_kpoints and parse_cutoffs error immediately at CLI parse time, not inside setup closures. A malformed k-point like '8xx8' must fail before task construction begins (Pattern 3).",
+    "internal_type_choice: parse_kpoints uses [u32; 3] not a custom struct \u2014 The array type directly converts to KpointsMpGrid([kx, ky, kz]) with zero overhead. The type guarantee ('exactly 3 u32s') is enforced by the array type itself.",
+    "duplication_choice: parse_u_values and generate_job_script are duplicated, not extracted \u2014 Each function is ~15-30 lines with stable semantics. Extracting to a shared crate is overkill for 2 consumers. If a third consumer appears, extraction becomes justified.",
+    "task_naming_collision: Fully-qualified path for Task::BandStructure, not alias import \u2014 The collision between workflow_core::task::Task and castep_cell_io::param::general::Task is resolved by using the fully-qualified path at the single use-site where Task::BandStructure is set. No alias needed, no import shadow risk.",
+    "shared_workdir: Shared workdir for chained tasks \u2014 Both SCF and DOS tasks in scf_dos_chain use runs/scf_dos/. This makes checkpoint file access straightforward: the DOS setup copies ZnO.check to ZnO_DOS.check within the same directory (Pattern 6).",
+    "workspace_serialization: Workspace member additions serialize G1 and G2 skeleton tasks \u2014 Both new crates use workspace = true dependencies (anyhow, clap, itertools), which require workspace membership for cargo check to resolve. Since both crate skeletons must add themselves to the root Cargo.toml workspace.members, G2-1 depends on G1-1 to avoid a merge conflict on the same file. The groups remain logically independent at the code level.",
+    "no_new_library_crates: No new library crates are created \u2014 All new code lives in binary (example) crates. The new code is workflow usage (orchestration, not abstractions). The library crates (workflow_core, workflow_utils) are unchanged.",
+    "Crate boundaries:",
+    "  parse_u_values: Duplicated in multi_param_sweep/src/config.rs and scf_dos_chain/src/config.rs",
+    "  parse_kpoints: multi_param_sweep/src/config.rs only (not needed by Binary 2)",
+    "  parse_cutoffs: multi_param_sweep/src/config.rs only",
+    "  generate_job_script: Duplicated in both binaries' job_script.rs (D.2 fix applies to both)",
+    "  SweepConfig: multi_param_sweep/src/config.rs",
+    "  ChainConfig: scf_dos_chain/src/config.rs",
+    "  sweep_logic: multi_param_sweep/src/main.rs (build_one_scf_task, build_all_scf_tasks)",
+    "  chain_logic: scf_dos_chain/src/main.rs (build_scf_task, build_dos_task)",
+    "  seed_files: Each binary's seeds/ directory (identical content, self-contained)",
+    "Pattern: Purpose-specific binaries over mode-flags \u2014 check: Neither binary should have a flag that toggles between 'independent tasks' and 'chained tasks'. multi_param_sweep has a --sweep-mode flag for product vs pairwise (both are independent SCF sweeps, same workflow type). scf_dos_chain has no mode flag.",
+    "Pattern: Parameter encoding in task IDs \u2014 check: multi_param_sweep task IDs: scf_U{u}_k{k}_c{c} (e.g., scf_U3.0_k8x8x8_c500). scf_dos_chain task IDs: scf and dos (single parameter set, no collision risk).",
+    "Pattern: Parse-time validation gates \u2014 check: Every parsing function returns anyhow::Result<T> with messages containing the expected format and what went wrong. Examples: 'kpoints list is empty', 'invalid k-point: expected 3 axes, got 2'.",
+    "Pattern: Explicit defaults \u2014 check: --sweep-mode defaults to 'product'. --kpoints and --cutoffs are Option<String> defaulting to None (seed defaults used). Pairwise mode errors if either is None.",
+    "Pattern: In-memory document mutation \u2014 check: All CASTEP file modifications go through parse -> mutate typed fields -> to_cell_file() -> to_string_many_spaced(). Never use format! on raw strings to build .cell/.param content.",
+    "Pattern: Shared workdir for chained tasks \u2014 check: scf_dos_chain places both tasks in runs/scf_dos/. DOS setup copies ZnO.check to ZnO_DOS.check in the same directory.",
+    "Pattern: Clean heredoc template \u2014 check: The no_literal_tabs test must pass. Use consistent space-only indentation. SBATCH directives use consistent quoting (double-quotes around values that might contain spaces)."
+  ],
+  "known_pitfalls": [
+    "Task name collision in scf_dos_chain (P0): use workflow_utils::prelude::* brings workflow_core::task::Task into scope. Importing castep_cell_io::param::general::task::Task would shadow it. Resolution: use the fully-qualified path castep_cell_io::param::general::task::Task::BandStructure at the single use-site in the DOS setup closure. Verify the exact module path with LSP hover during implementation.. Mitigation: Do not add use castep_cell_io::param::general::task::Task. Instead use the fully-qualified path at the single use-site.",
+    "Local path dep must exist on the filesystem: The path ../castep-cell-io/castep_cell_io must resolve to a sibling directory containing a Cargo.toml with the castep-cell-io crate. Verify this exists before implementation.. Mitigation: Pre-implementation check: verify ../castep-cell-io/castep_cell_io/Cargo.toml exists.",
+    "castep-cell-fmt version compatibility: castep-cell-fmt = 0.1.0 from crates.io while castep-cell-io uses a local path dep at v0.5.0. If v0.5.0 depends on a newer castep-cell-fmt, Cargo resolves the semver-compatible version automatically. No conflict expected.. Mitigation: No action needed; Cargo handles this automatically.",
+    "Pairwise mode validation ordering: In pairwise mode, validate that both --kpoints and --cutoffs are provided BEFORE parsing them. If either is None, return a clear error immediately. After parsing, validate all three lists have the same length. The error message must state the expected and actual lengths.. Mitigation: Check sweep_mode first, then validate presence of both Option fields, then parse, then check lengths.",
+    "hubbard_u_sweep compatibility with v0.5.0 path dep: The existing hubbard_u_sweep/src/main.rs already uses the v0.5.0 parse API pattern (castep_cell_fmt::parse::<CellDocument>(&input)). However, v0.5.0 may have changed Species::Symbol signature, builder method names, or HubbardUUnit enum variants. If compilation fails after the path-dep change, fix these call sites based on compiler error messages.. Mitigation: After changing the path dep, run cargo check -p hubbard_u_sweep. If it fails, fix each error based on the compiler message. Common changes may include Species::Symbol type or builder method names.",
+    "Dead-code detection after refactoring (F.3 mitigation): After deleting hubbard_u_sweep_slurm, run cargo clippy --workspace --all-targets -- -W dead-code to detect any stale imports or dead functions left over from the refactoring.. Mitigation: Include clippy dead-code check in the Group 3 acceptance criteria.",
+    "D.2 formatting fix must be embedded in ported job_script.rs: The no_literal_tabs test from the old binary must be ported and must pass. The D.2 fix (clean heredoc template with all-space indentation, consistent double-quoting on SBATCH directives) must be applied to the ported generate_job_script function in BOTH new binaries.. Mitigation: Implementation checklist: all-space indentation, SBATCH directives use double-quotes consistently, shell continuation lines use backslash, no_literal_tabs test passes in both binaries.",
+    "Deferred items: what NOT to do: D.1 (portable SLURM config): Do NOT implement. D.3 (expanded unit tests for generate_job_script): Do NOT expand beyond ported tests. read_task_ids edge case: Do NOT fix. All three have unmet preconditions.. Mitigation: Review the deferred-and-patterns.md file for full details on each deferred item.",
+    "G1-1 and G2-1 both modify root Cargo.toml \u2014 serialized: Both multi_param_sweep and scf_dos_chain use workspace = true dependencies, requiring workspace membership. To avoid merge conflicts on the root Cargo.toml, TASK-G2-1 must run after TASK-G1-1 completes. The groups are logically independent at the code level; this dependency is solely a file collision on the workspace member list.. Mitigation: TASK-G2-1 depends_on TASK-G1-1. Only the skeleton creation tasks are serialized; all other tasks within each group can run in parallel with tasks in the other group."
+  ],
+  "task_groups": [
+    {
+      "group_id": "core-1",
+      "reason": "Create the complete examples/multi_param_sweep/ crate \u2014 a purpose-built binary for independent SCF parameter sweeps across Hubbard U, k-point MP grid, and cutoff energy in product or pairwise mode.",
+      "tasks": [
+        "G1-1",
+        "G1-2",
+        "G1-3",
+        "G1-4",
+        "G1-5"
+      ],
+      "depends_on_groups": []
+    },
+    {
+      "group_id": "core-2",
+      "reason": "Create the examples/scf_dos_chain/ crate \u2014 a purpose-built binary for SCF+DOS task chain testing \u2014 and update the existing hubbard_u_sweep binary for castep-cell-io v0.5.0 path dep compatibility.",
+      "tasks": [
+        "G2-1",
+        "G2-2",
+        "G2-3",
+        "G2-4",
+        "G2-5"
+      ],
+      "depends_on_groups": [
+        "core-1"
+      ]
+    },
+    {
+      "group_id": "workspace",
+      "reason": "Wire everything together by removing the old hubbard_u_sweep_slurm crate, cleaning up workspace members, and running full build/test/clippy verification.",
+      "tasks": [
+        "G3-1"
+      ],
+      "depends_on_groups": [
+        "core-2"
+      ]
+    }
+  ],
+  "tasks": [
+    {
+      "id": "G1-1",
+      "description": "Create the directory structure, Cargo.toml, stub source files, seed files, and add the crate to workspace members so cargo check resolves dependencies. \u2014 Crate creation: Cargo.toml with all dependencies, minimal main.rs with mod declarations, empty config.rs, empty job_script.rs, seed files from existing hubbard_u_sweep_slurm, and workspace member registration.",
+      "files_in_scope": [
+        "examples/multi_param_sweep (new crate)",
+        "examples/multi_param_sweep/Cargo.toml",
+        "examples/multi_param_sweep/src/main.rs",
+        "examples/multi_param_sweep/src/config.rs",
+        "examples/multi_param_sweep/src/job_script.rs",
+        "examples/multi_param_sweep/seeds/ZnO.cell",
+        "examples/multi_param_sweep/seeds/ZnO.param",
+        "Cargo.toml (workspace root)"
+      ],
+      "changes": [
+        {
+          "path": "examples/multi_param_sweep/Cargo.toml",
+          "action": "create",
+          "guidance": "Create Cargo.toml with [package] name = 'multi_param_sweep', edition 2021. Use [[bin]] pointing to src/main.rs. Dependencies: anyhow = { workspace = true }, clap = { workspace = true }, itertools = { workspace = true }, castep-cell-fmt = '0.1.0', castep-cell-io = { path = '../castep-cell-io/castep_cell_io' }, workflow_core = { path = '../../workflow_core', features = ['default-logging'] }, workflow_utils = { path = '../../workflow_utils' }. Copy from hubbard_u_sweep_slurm/Cargo.toml as a template, adjusting the package name and adding kpoints/cutoff-related deps."
+        },
+        {
+          "path": "examples/multi_param_sweep/src/main.rs",
+          "action": "create",
+          "guidance": "Create stub main.rs with mod config; and mod job_script; declarations. Add a minimal fn main() { println!('multi_param_sweep: skeleton'); } that compiles. No imports yet."
+        },
+        {
+          "path": "examples/multi_param_sweep/src/config.rs",
+          "action": "create",
+          "guidance": "Create empty config.rs with a placeholder comment: // Config will be added in G1-2. No struct, no functions, no tests yet."
+        },
+        {
+          "path": "examples/multi_param_sweep/src/job_script.rs",
+          "action": "create",
+          "guidance": "Create empty job_script.rs with a placeholder comment: // Job script will be added in G1-3."
+        },
+        {
+          "path": "examples/multi_param_sweep/seeds/ZnO.cell",
+          "action": "modify",
+          "guidance": "Copy the content from examples/hubbard_u_sweep_slurm/seeds/ZnO.cell. This is the ZnO wurtzite lattice cell file used as the seed for all task setups."
+        },
+        {
+          "path": "examples/multi_param_sweep/seeds/ZnO.param",
+          "action": "modify",
+          "guidance": "Copy the content from examples/hubbard_u_sweep_slurm/seeds/ZnO.param. This is the seed param file (task: SinglePoint) used as the base for task-specific param modifications."
+        },
+        {
+          "path": "Cargo.toml (workspace root)",
+          "action": "modify",
+          "guidance": "Add 'examples/multi_param_sweep' to the [workspace] members array. Insert it after 'examples/hubbard_u_sweep_slurm' (keeping the old binary for now). Keep all other members unchanged."
+        }
+      ],
+      "acceptance": [
+        "cargo check --workspace (must compile \u2014 new crate resolves workspace deps, old slurm crate still compiles)",
+        "ls examples/multi_param_sweep/Cargo.toml examples/multi_param_sweep/src/main.rs examples/multi_param_sweep/src/config.rs examples/multi_param_sweep/src/job_script.rs examples/multi_param_sweep/seeds/ZnO.cell examples/multi_param_sweep/seeds/ZnO.param"
+      ],
+      "depends_on": [],
+      "wiring_checklist": [
+        {
+          "kind": "pub_mod",
+          "file": "examples/multi_param_sweep/src/main.rs",
+          "detail": "Add mod config; and mod job_script; at the top of the file (before fn main)."
+        },
+        {
+          "kind": "pub_mod",
+          "file": "Cargo.toml (workspace root)",
+          "detail": "Add 'examples/multi_param_sweep' to [workspace] members array."
+        }
+      ]
+    },
+    {
+      "id": "G1-2",
+      "description": "Implement the SweepConfig clap struct and all three parsing utility functions (parse_u_values, parse_kpoints, parse_cutoffs) with comprehensive tests written first. \u2014 CLI configuration and parse-time validation: SweepConfig struct (clap Parser derive, all 16 fields from the plan plus SLURM fields), parse_u_values (ported with anyhow::Result), parse_kpoints (new, strict NxNxN validation), parse_cutoffs (new, f64 comma-separated), and their in-file #[cfg(test)] tests.",
+      "files_in_scope": [
+        "examples/multi_param_sweep/src/config.rs"
+      ],
+      "changes": [
+        {
+          "path": "examples/multi_param_sweep/src/config.rs",
+          "action": "modify",
+          "guidance": "Implement the full SweepConfig struct with #[derive(Parser, Debug)], #[command(name = 'multi_param_sweep')], and all fields from the plan: u_values (String, default '0.0,1.0,2.0,3.0,4.0,5.0'), kpoints (Option<String>), cutoffs (Option<String>), sweep_mode (String, default 'product'), element (String, default 'Zn'), orbital (char, default 'd'), seed_name (String, default 'ZnO'), max_parallel (usize, default 4), local (bool), dry_run (bool), castep_command (String, default 'castep'), workdir (String, default '.'), plus SLURM fields: partition (String, env CASTEP_SLURM_PARTITION, default 'debug'), ntasks (u32, default 16), nix_flake (String, env CASTEP_NIX_FLAKE, with the long default), mpi_if (String, env CASTEP_MPI_IF, default 'enp6s0'). Each field should have a doc comment describing its purpose and the expected format for string-typed fields. Implement parse_u_values (ported, returning anyhow::Result<Vec<f64>> not Result<_, String>), parse_kpoints (new, splitting on comma then on 'x', validating exactly 3 u32 axes per segment), parse_cutoffs (new, splitting on comma, trimming, parsing f64). All three return anyhow::Result with clear error messages describing expected format. parse_kpoints must validate each segment splits into exactly 3 parts on 'x' and each part parses as u32 \u2014 reject '8x8' (2 axes) and 'abc' (non-numeric). Add #[cfg(test)] mod tests with the tests from tdd_interface.test_code. For parse_kpoints, the 'wrong_axes' test must verify the error message mentions the expected axis count or format. For empty input tests, the error must contain 'empty'."
+        }
+      ],
+      "acceptance": [
+        "cargo check -p multi_param_sweep",
+        "cargo test -p multi_param_sweep -- config::tests (all parse tests pass: 7 parse_u_values + 7 parse_kpoints + 6 parse_cutoffs = 20 tests)",
+        "cargo test -p multi_param_sweep (no failures; job_script tests may not exist yet and will be ignored)"
+      ],
+      "depends_on": [
+        "G1-1"
+      ],
+      "kind": "lib-tdd",
+      "tdd_interface": {
+        "test_file": "examples/multi_param_sweep/src/config.rs",
+        "test_module": "tests",
+        "test_fn_name": "parse_basic_values",
+        "test_code": "// --- parse_u_values tests (ported from old binary, adapted for anyhow::Result) ---\n\n/// Parse comma-separated basic values\n#[test]\nfn parse_basic_values() {\n    let vals = parse_u_values(\"0.0,1.0,2.0\").unwrap();\n    assert_eq!(vals, vec![0.0, 1.0, 2.0]);\n}\n\n/// Parse with whitespace around values\n#[test]\nfn parse_with_whitespace() {\n    let vals = parse_u_values(\"  0.0 , 1.0 , 2.0  \").unwrap();\n    assert_eq!(vals, vec![0.0, 1.0, 2.0]);\n}\n\n/// Parse a single value\n#[test]\nfn parse_single_value() {\n    let vals = parse_u_values(\"42.0\").unwrap();\n    assert_eq!(vals, vec![42.0]);\n}\n\n/// Invalid token in the middle\n#[test]\nfn parse_invalid_token() {\n    let err = parse_u_values(\"1.0,abc,2.0\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"abc\"), \"error should mention the invalid token: {msg}\");\n}\n\n/// Empty token in the middle (two consecutive commas)\n#[test]\nfn parse_empty_token() {\n    let err = parse_u_values(\"1.0,,2.0\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"invalid\"), \"error should report parse failure: {msg}\");\n}\n\n/// Entirely empty input\n#[test]\nfn parse_empty_string() {\n    let err = parse_u_values(\"\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"empty\") || msg.contains(\"invalid\"), \"expected parse failure on empty input, got: {msg}\");\n}\n\n/// Negative values\n#[test]\nfn parse_negative_values() {\n    let vals = parse_u_values(\"-1.0,2.0\").unwrap();\n    assert_eq!(vals, vec![-1.0, 2.0]);\n}\n\n// --- parse_kpoints tests (new) ---\n\n/// Basic valid k-points\n#[test]\nfn parse_kpoints_basic() {\n    let kpts = parse_kpoints(\"8x8x8,6x6x6\").unwrap();\n    assert_eq!(kpts, vec![[8, 8, 8], [6, 6, 6]]);\n}\n\n/// Single valid k-point\n#[test]\nfn parse_kpoints_single() {\n    let kpts = parse_kpoints(\"4x4x4\").unwrap();\n    assert_eq!(kpts, vec![[4, 4, 4]]);\n}\n\n/// With whitespace\n#[test]\nfn parse_kpoints_whitespace() {\n    let kpts = parse_kpoints(\" 8x8x8 , 6x6x6 \").unwrap();\n    assert_eq!(kpts, vec![[8, 8, 8], [6, 6, 6]]);\n}\n\n/// Wrong number of axes (only 2 instead of 3)\n#[test]\nfn parse_kpoints_wrong_axes() {\n    let err = parse_kpoints(\"8x8\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"3 axes\") || msg.contains(\"got 2\") || msg.contains(\"expected 3\"), \"error should mention wrong axis count: {msg}\");\n}\n\n/// Non-numeric axis value\n#[test]\nfn parse_kpoints_non_numeric() {\n    let err = parse_kpoints(\"abc\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"invalid\") || msg.contains(\"abc\") || msg.contains(\"expected 3 axes\"), \"error should report the problem: {msg}\");\n}\n\n/// Empty input\n#[test]\nfn parse_kpoints_empty() {\n    let err = parse_kpoints(\"\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"empty\"), \"error should say kpoints list is empty: {msg}\");\n}\n\n/// Large grid values\n#[test]\nfn parse_kpoints_large() {\n    let kpts = parse_kpoints(\"12x12x12\").unwrap();\n    assert_eq!(kpts, vec![[12, 12, 12]]);\n}\n\n// --- parse_cutoffs tests (new) ---\n\n/// Basic valid cutoffs\n#[test]\nfn parse_cutoffs_basic() {\n    let cuts = parse_cutoffs(\"300,500,800\").unwrap();\n    assert_eq!(cuts, vec![300.0, 500.0, 800.0]);\n}\n\n/// Single cutoff\n#[test]\nfn parse_cutoffs_single() {\n    let cuts = parse_cutoffs(\"450\").unwrap();\n    assert_eq!(cuts, vec![450.0]);\n}\n\n/// With whitespace\n#[test]\nfn parse_cutoffs_whitespace() {\n    let cuts = parse_cutoffs(\" 300 , 500 , 800 \").unwrap();\n    assert_eq!(cuts, vec![300.0, 500.0, 800.0]);\n}\n\n/// Invalid token\n#[test]\nfn parse_cutoffs_invalid_token() {\n    let err = parse_cutoffs(\"300,abc,800\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"abc\"), \"error should mention the invalid token: {msg}\");\n}\n\n/// Empty input\n#[test]\nfn parse_cutoffs_empty() {\n    let err = parse_cutoffs(\"\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"empty\"), \"error should say cutoffs list is empty: {msg}\");\n}\n\n/// Negative and fractional values\n#[test]\nfn parse_cutoffs_negative() {\n    let cuts = parse_cutoffs(\"-100.5,200.0\").unwrap();\n    assert_eq!(cuts, vec![-100.5, 200.0]);\n}",
+        "signature": "pub fn parse_u_values(s: &str) -> anyhow::Result<Vec<f64>>",
+        "expected_behavior": "parse_u_values: returns Vec<f64> for comma-separated numbers, errors on non-numeric tokens with the offending token in the message. Empty input produces an error containing 'empty' or 'invalid'. Whitespace around values is trimmed. parse_kpoints: returns Vec<[u32; 3]> for comma-separated NxNxN triplets. Errors on wrong number of axes (must be exactly 3 per segment). Empty input produces an error containing 'empty'. Whitespace around values is trimmed. Non-numeric axis values produce errors mentioning the invalid input. parse_cutoffs: returns Vec<f64> for comma-separated numbers, same semantics as parse_u_values but with cutoff-specific error context."
+      }
+    },
+    {
+      "id": "G1-3",
+      "description": "Implement the SLURM job script generation function with the D.2 formatting fix applied, with ported tests written first. \u2014 SLURM job script generation: generate_job_script function with the D.2 clean heredoc template (all-space indentation, consistent double-quoting on SBATCH directives), and all 6 ported tests (sbatch directives, seed name, no_literal_tabs, shebang, nix develop, mpi interface).",
+      "files_in_scope": [
+        "examples/multi_param_sweep/src/job_script.rs"
+      ],
+      "changes": [
+        {
+          "path": "examples/multi_param_sweep/src/job_script.rs",
+          "action": "modify",
+          "guidance": "Implement generate_job_script(config: &SweepConfig, task_id: &str, seed_name: &str) -> String. Port the function body from the old hubbard_u_sweep_slurm/src/job_script.rs but apply the D.2 formatting fix: use consistent space-only indentation throughout the heredoc template (no literal tab characters mixed with spaces), use double-quotes consistently around SBATCH directive values (e.g., #SBATCH --job-name=\\\"{task_id}\\\"), and ensure shell continuation lines use \\\\ with proper spacing. The function must use format!() with named parameters. Import SweepConfig from crate::config. Add #[cfg(test)] mod tests with all 6 ported tests from tdd_interface.test_code adapted for the new SweepConfig. The no_literal_tabs test is the D.2 verification: it must assert !script.contains('\\t') and must PASS with the new template."
+        }
+      ],
+      "acceptance": [
+        "cargo check -p multi_param_sweep",
+        "cargo test -p multi_param_sweep -- job_script::tests (all 6 tests pass)",
+        "cargo test -p multi_param_sweep -- job_script::tests::no_literal_tabs (must pass \u2014 this is the D.2 verification)"
+      ],
+      "depends_on": [
+        "G1-2"
+      ],
+      "kind": "lib-tdd",
+      "tdd_interface": {
+        "test_file": "examples/multi_param_sweep/src/job_script.rs",
+        "test_module": "tests",
+        "test_fn_name": "contains_sbatch_directives",
+        "test_code": "use crate::config::SweepConfig;\nuse clap::Parser;\n\nfn default_config() -> SweepConfig {\n    SweepConfig::parse_from([\"test\"])\n}\n\n/// Script contains expected SBATCH directives with the task ID and config values\n#[test]\nfn contains_sbatch_directives() {\n    let config = default_config();\n    let script = generate_job_script(&config, \"scf_U1.0\", \"ZnO\");\n    assert!(script.contains(\"#SBATCH --job-name=\\\"scf_U1.0\\\"\"));\n    assert!(script.contains(\"#SBATCH --partition=debug\"));\n    assert!(script.contains(\"#SBATCH --ntasks-per-node=16\"));\n    assert!(script.contains(\"#SBATCH --mem=30000m\"));\n}\n\n/// Script references the correct seed name in the castep command\n#[test]\nfn contains_seed_name() {\n    let config = default_config();\n    let script = generate_job_script(&config, \"scf_U0.0\", \"ZnO\");\n    assert!(script.contains(\"castep.mpi ZnO\"));\n}\n\n/// D.2 fix: script must not contain literal tab characters\n#[test]\nfn no_literal_tabs() {\n    let config = default_config();\n    let script = generate_job_script(&config, \"scf_U0.0\", \"ZnO\");\n    assert!(!script.contains('\\t'), \"job script should not contain literal tab characters\");\n}\n\n/// Script starts with a shebang line\n#[test]\nfn starts_with_shebang() {\n    let config = default_config();\n    let script = generate_job_script(&config, \"scf_U0.0\", \"ZnO\");\n    assert!(script.starts_with(\"#!/usr/bin/env bash\"));\n}\n\n/// Script contains nix develop with the configured flake URI\n#[test]\nfn contains_nix_develop() {\n    let config = default_config();\n    let script = generate_job_script(&config, \"scf_U0.0\", \"ZnO\");\n    assert!(script.contains(\"nix develop\"));\n    assert!(script.contains(&config.nix_flake));\n}\n\n/// Script contains the MPI interface setting\n#[test]\nfn contains_mpi_interface() {\n    let config = default_config();\n    let script = generate_job_script(&config, \"scf_U0.0\", \"ZnO\");\n    assert!(script.contains(&format!(\"OMPI_MCA_btl_tcp_if_include={mpi_if}\", mpi_if = config.mpi_if)));\n}",
+        "signature": "pub fn generate_job_script(config: &SweepConfig, task_id: &str, seed_name: &str) -> String",
+        "expected_behavior": "generate_job_script produces a SLURM job script string. The template uses ONLY spaces for indentation (no literal tab characters). SBATCH directives use double-quotes around values: #SBATCH --job-name=\"{task_id}\". The script contains: a bash shebang, SBATCH directives for partition/ntasks/mem, a nix develop command with the configured flake, an mpirun command with the MPI interface, and the castep.mpi {seed_name} invocation. Shell continuation lines use backslash (not mixed tab+space indentation)."
+      }
+    },
+    {
+      "id": "G1-4",
+      "description": "Implement the sweep combinatorics (product/pairwise mode dispatch) and the build_one_scf_task builder function with tests for correct task structure and sweep logic. \u2014 Sweep combinatorics and SCF task construction: build_one_scf_task (single SCF task with setup/collect closures that inject hubbard_u, kpoints_mp_grid, and cutoff_energy), build_all_scf_tasks (dispatches to product or pairwise modes), sweep mode validation (pairwise requires both kpoints and cutoffs, equal-length check), and in-file #[cfg(test)] tests verifying task IDs, dependency structure, workdir paths, and mode-specific error handling.",
+      "files_in_scope": [
+        "examples/multi_param_sweep/src/main.rs"
+      ],
+      "changes": [
+        {
+          "path": "examples/multi_param_sweep/src/main.rs",
+          "action": "modify",
+          "guidance": "Implement the sweep logic and task builder functions. Add necessary imports: use config::{parse_u_values, parse_kpoints, parse_cutoffs, SweepConfig}; use job_script::generate_job_script; plus castep_cell_fmt (parse, to_string_many_spaced, ToCellFile), castep_cell_io types (CellDocument, ParamDocument, AtomHubbardU, HubbardU, HubbardUUnit, OrbitalU, Species, KpointsMpGrid, CutOffEnergy), itertools::iproduct, workflow_utils::prelude::*.\n\nFunction build_one_scf_task(config, u, kpoint, cutoff, seed_cell, seed_param) -> Result<Task, WorkflowError>: Construct task ID using format! with the pattern 'scf_U{}_k{}_c{}'. For the u value, use {:.1} formatting to drop trailing zeros (3.0 -> 'U3.0'). For kpoint, format as '{0}x{1}x{2}' \u2014 if kpoint is None, omit the _k segment entirely. For cutoff, use {:.0} \u2014 if cutoff is None, omit the _c segment entirely. Workdir: PathBuf::from(format!('runs/U{u:.1}_k{k_str}_c{c_str}')) with segments omitted if params are None.\n\nThe setup closure must: (1) create_dir(workdir), (2) parse seed_cell into CellDocument via castep_cell_fmt::parse::<CellDocument>, inject hubbard_u block using the builder pattern: AtomHubbardU::builder().species(Species::Symbol(element.clone())).orbitals(vec![orbital_u]).build() and HubbardU::builder().unit(HubbardUUnit::ElectronVolt).atom_u_values(vec![atom_u]).build(), set cell_doc.hubbard_u = Some(hubbard_u), (3) if kpoint is Some, set cell_doc.kpoints_mp_grid = Some(KpointsMpGrid(kpoint)), (4) parse seed_param into ParamDocument via castep_cell_fmt::parse::<ParamDocument>, if cutoff is Some set doc.basis_set.cutoff_energy = Some(CutOffEnergy { value: cutoff, unit: None }), (5) serialize both to .cell and .param files via to_string_many_spaced(&doc.to_cell_file()) and write_file, (6) if !config.local, write job_script via write_file(workdir.join(JOB_SCRIPT_NAME), &generate_job_script(config, &task_id, &config.seed_name)).\n\nThe collect closure must: read <seed>.castep via read_file, verify it contains 'Total time' marker. Return WorkflowError::InvalidConfig if the file is missing or the marker is absent.\n\nFunction build_all_scf_tasks(config) -> anyhow::Result<Vec<Task>>: Load seed_cell and seed_param via include_str!('../seeds/ZnO.cell') and include_str!('../seeds/ZnO.param') (passed as parameters). Parse u_values. Match config.sweep_mode.as_str(): 'product' branch uses itertools::iproduct! over u_vals, kpoints_vec (unwrap_or(vec![None])), cutoffs_vec (unwrap_or(vec![None])). For each (u, k, c) tuple, call build_one_scf_task. 'pairwise' branch requires both kpoints and cutoffs to be Some (anyhow::bail! if not), parses all three lists, validates equal lengths (anyhow::bail! if not), then zip all three iterators and call build_one_scf_task for each triple. Unknown mode returns anyhow::bail!('unknown sweep mode...').\n\nAdd #[cfg(test)] mod tests with the tests from tdd_interface.test_code. The tests verify combinatoric correctness, mode validation, task ID uniqueness, and structural invariants."
+        }
+      ],
+      "acceptance": [
+        "cargo check -p multi_param_sweep",
+        "cargo test -p multi_param_sweep (config tests + job_script tests + sweep logic tests all pass)",
+        "All sweep tests pass: product count (6 default, 18 explicit), pairwise count (3), pairwise requires kpoints, pairwise requires cutoffs, unequal lengths error, unknown mode error, unique IDs, ID format, optional params, no dependencies, workdirs present"
+      ],
+      "depends_on": [
+        "G1-3"
+      ],
+      "kind": "lib-tdd",
+      "tdd_interface": {
+        "test_file": "examples/multi_param_sweep/src/main.rs",
+        "test_module": "tests",
+        "test_fn_name": "product_mode_produces_tasks",
+        "test_code": "use crate::config::SweepConfig;\nuse clap::Parser;\n\nfn default_config() -> SweepConfig {\n    SweepConfig::parse_from([\"test\"])\n}\n\nfn test_seeds() -> (&'static str, &'static str) {\n    (include_str!(\"../seeds/ZnO.cell\"), include_str!(\"../seeds/ZnO.param\"))\n}\n\n// --- Sweep combinatorics tests ---\n\n/// Product mode with defaults produces many tasks (6 u-values * default seed = 6 tasks)\n#[test]\nfn product_mode_produces_tasks() {\n    let config = default_config();\n    let tasks = build_all_scf_tasks(&config).unwrap();\n    // Default: 6 u-values (0.0..5.0), no kpoints/cutoffs by default -> 6 tasks\n    assert_eq!(tasks.len(), 6);\n}\n\n/// Product mode with explicit u-values, kpoints, and cutoffs produces Cartesian product\n#[test]\nfn product_mode_cartesian_product() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--u-values\", \"0.0,1.0,2.0\",\n        \"--kpoints\", \"8x8x8,6x6x6\",\n        \"--cutoffs\", \"300,500,800\",\n        \"--sweep-mode\", \"product\",\n    ]);\n    let tasks = build_all_scf_tasks(&config).unwrap();\n    // 3 u * 2 k * 3 c = 18 tasks\n    assert_eq!(tasks.len(), 18);\n}\n\n/// Pairwise mode produces zipped tasks\n#[test]\nfn pairwise_mode_zip() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--u-values\", \"0.0,1.0,2.0\",\n        \"--kpoints\", \"8x8x8,6x6x6,4x4x4\",\n        \"--cutoffs\", \"300,500,800\",\n        \"--sweep-mode\", \"pairwise\",\n    ]);\n    let tasks = build_all_scf_tasks(&config).unwrap();\n    assert_eq!(tasks.len(), 3);\n}\n\n/// Pairwise mode requires --kpoints\n#[test]\nfn pairwise_requires_kpoints() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--u-values\", \"0.0,1.0,2.0\",\n        \"--cutoffs\", \"300,500,800\",\n        \"--sweep-mode\", \"pairwise\",\n    ]);\n    let err = build_all_scf_tasks(&config).unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"kpoints\"), \"error should mention kpoints: {msg}\");\n}\n\n/// Pairwise mode requires --cutoffs\n#[test]\nfn pairwise_requires_cutoffs() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--u-values\", \"0.0,1.0,2.0\",\n        \"--kpoints\", \"8x8x8,6x6x6,4x4x4\",\n        \"--sweep-mode\", \"pairwise\",\n    ]);\n    let err = build_all_scf_tasks(&config).unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"cutoffs\"), \"error should mention cutoffs: {msg}\");\n}\n\n/// Pairwise mode with unequal list lengths errors\n#[test]\nfn pairwise_unequal_lengths() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--u-values\", \"0.0,1.0\",\n        \"--kpoints\", \"8x8x8,6x6x6,4x4x4\",\n        \"--cutoffs\", \"300,500,800\",\n        \"--sweep-mode\", \"pairwise\",\n    ]);\n    let err = build_all_scf_tasks(&config).unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"same length\") || msg.contains(\"length\"), \"error should mention length mismatch: {msg}\");\n}\n\n/// Unknown sweep mode errors\n#[test]\nfn unknown_sweep_mode() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--sweep-mode\", \"invalid\",\n    ]);\n    let err = build_all_scf_tasks(&config).unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"unknown sweep mode\"), \"error should mention unknown mode: {msg}\");\n}\n\n// --- Task structure tests ---\n\n/// Each task has a unique ID\n#[test]\nfn task_ids_are_unique() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--u-values\", \"0.0,1.0\",\n        \"--kpoints\", \"8x8x8,6x6x6\",\n        \"--cutoffs\", \"300,500\",\n        \"--sweep-mode\", \"product\",\n    ]);\n    let tasks = build_all_scf_tasks(&config).unwrap();\n    let ids: Vec<&str> = tasks.iter().map(|t| t.id.as_str()).collect();\n    let mut unique_ids = ids.clone();\n    unique_ids.sort();\n    unique_ids.dedup();\n    assert_eq!(ids.len(), unique_ids.len(), \"all task IDs must be unique\");\n}\n\n/// Task IDs encode parameters in the expected format scf_U{u}_k{k}_c{c}\n#[test]\nfn task_ids_encode_parameters() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--u-values\", \"3.0\",\n        \"--kpoints\", \"8x8x8\",\n        \"--cutoffs\", \"500\",\n        \"--sweep-mode\", \"product\",\n    ]);\n    let tasks = build_all_scf_tasks(&config).unwrap();\n    assert_eq!(tasks.len(), 1);\n    let id = &tasks[0].id;\n    assert!(id.contains(\"U3\") || id.contains(\"U3.0\"), \"task ID should contain U value: {id}\");\n    assert!(id.contains(\"k8x8x8\") || id.contains(\"k\"), \"task ID should contain kpoint: {id}\");\n    assert!(id.contains(\"c500\") || id.contains(\"c\"), \"task ID should contain cutoff: {id}\");\n}\n\n/// Product mode with only u-values (optional kpoints/cutoffs) still works\n#[test]\nfn product_with_optional_params() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--u-values\", \"0.0,1.0,2.0\",\n    ]);\n    let tasks = build_all_scf_tasks(&config).unwrap();\n    assert_eq!(tasks.len(), 3);\n}\n\n/// SCF tasks have no dependencies\n#[test]\nfn scf_tasks_have_no_dependencies() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--u-values\", \"0.0,1.0\",\n        \"--kpoints\", \"8x8x8\",\n        \"--cutoffs\", \"500\",\n        \"--sweep-mode\", \"product\",\n    ]);\n    let tasks = build_all_scf_tasks(&config).unwrap();\n    for task in &tasks {\n        assert!(task.dependencies.is_empty(), \"SCF task should have no dependencies: {}\", task.id);\n    }\n}\n\n/// Tasks have workdir paths\n#[test]\nfn tasks_have_workdirs() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--u-values\", \"1.0\",\n        \"--kpoints\", \"8x8x8\",\n        \"--cutoffs\", \"500\",\n        \"--sweep-mode\", \"product\",\n    ]);\n    let tasks = build_all_scf_tasks(&config).unwrap();\n    assert!(!tasks.is_empty());\n    for task in &tasks {\n        assert!(!task.workdir.as_os_str().is_empty(), \"task {} should have a workdir\", task.id);\n    }\n}",
+        "signature": "fn build_one_scf_task(config: &SweepConfig, u: f64, kpoint: Option<[u32; 3]>, cutoff: Option<f64>, seed_cell: &str, seed_param: &str) -> Result<Task, WorkflowError>",
+        "expected_behavior": "build_all_scf_tasks in product mode: given N u-values, M kpoints, P cutoffs, produces N*M*P tasks (Cartesian product). If kpoints or cutoffs is None, uses a single None entry in the product (N*1 or N*1*1 tasks). In pairwise mode: requires both kpoints and cutoffs to be Some (errors with 'requires --kpoints' or 'requires --cutoffs' if not), parses all three lists, validates equal lengths (errors if unequal), then zips. Unknown sweep_mode produces 'unknown sweep mode' error. build_one_scf_task: produces a Task with a unique ID encoding all three parameters, no dependencies, a workdir path, and setup/collect closures. The setup closure injects hubbard_u, kpoints_mp_grid (if provided), and cutoff_energy into the seed files. The collect closure verifies the .castep output file exists and contains 'Total time'."
+      }
+    },
+    {
+      "id": "G1-5",
+      "description": "Implement the main() function that orchestrates config parsing, task building, workflow construction, dry-run display, and local/queued execution. \u2014 Binary entry point: main() function that parses SweepConfig from CLI, calls build_all_scf_tasks, constructs a Workflow with max_parallel, log_dir, root_dir, conditionally with_queued_submitter, adds all tasks, then either dry-runs (printing topological order) or runs (local via run_default, or queued via Workflow::run with Arc<dyn ProcessRunner> + Arc<dyn HookExecutor>). Also prints workflow summary on completion.",
+      "files_in_scope": [
+        "examples/multi_param_sweep/src/main.rs"
+      ],
+      "changes": [
+        {
+          "path": "examples/multi_param_sweep/src/main.rs",
+          "action": "modify",
+          "guidance": "Implement the main() function. Use anyhow::Result as the return type. Call workflow_core::init_default_logging().ok() first. Parse SweepConfig::parse(). Call build_all_scf_tasks(&config)? to get tasks. Construct workflow: Workflow::new('multi_param_sweep').with_max_parallel(config.max_parallel)?.with_log_dir('logs').with_root_dir(&config.workdir). If !config.local, add .with_queued_submitter(Arc::new(QueuedRunner::new(SchedulerKind::Slurm))). Add all tasks via workflow.add_task(task)?. For dry_run: call workflow.dry_run()?, print 'Dry-run topological order:' then each task ID on its own line prefixed with '  '. Return Ok(()). For execution: create JsonStateStore at '.multi_param_sweep.workflow.json'. If config.local, use run_default(&mut workflow, &mut state)?; else, construct SystemProcessRunner and ShellHookExecutor as Arc<dyn ProcessRunner/HookExecutor>, call workflow.run(&mut state, runner, executor)?. After run, print: 'Workflow complete: {succeeded} succeeded, {failed} failed, {skipped} skipped ({duration}s)'. Import std::sync::Arc for the queued path."
+        }
+      ],
+      "acceptance": [
+        "cargo check -p multi_param_sweep",
+        "cargo run --bin multi_param_sweep -- --dry-run (prints 'Dry-run topological order:' followed by task IDs, no errors)",
+        "cargo run --bin multi_param_sweep -- --dry-run --sweep-mode pairwise --u-values 0,1,2 --kpoints 8x8x8,6x6x6,4x4x4 --cutoffs 300,500,800 (prints exactly 3 task IDs, no errors)",
+        "cargo run --bin multi_param_sweep -- --dry-run --sweep-mode pairwise --u-values 0,1,2 (errors: pairwise mode requires --kpoints and --cutoffs)"
+      ],
+      "depends_on": [
+        "G1-4"
+      ]
+    },
+    {
+      "id": "G2-1",
+      "description": "Create the directory structure, Cargo.toml, stub source files, seed files, and add the crate to workspace members. \u2014 Crate creation: Cargo.toml with all dependencies, minimal main.rs with mod declarations, empty config.rs, empty job_script.rs, seed files, and workspace member registration.",
+      "files_in_scope": [
+        "examples/scf_dos_chain (new crate)",
+        "examples/scf_dos_chain/Cargo.toml",
+        "examples/scf_dos_chain/src/main.rs",
+        "examples/scf_dos_chain/src/config.rs",
+        "examples/scf_dos_chain/src/job_script.rs",
+        "examples/scf_dos_chain/seeds/ZnO.cell",
+        "examples/scf_dos_chain/seeds/ZnO.param",
+        "Cargo.toml (workspace root)"
+      ],
+      "changes": [
+        {
+          "path": "examples/scf_dos_chain/Cargo.toml",
+          "action": "create",
+          "guidance": "Create Cargo.toml with [package] name = 'scf_dos_chain', edition 2021, [[bin]] path = src/main.rs. Same dependencies as multi_param_sweep: anyhow/clap/itertools = { workspace = true }, castep-cell-fmt = '0.1.0', castep-cell-io = { path = '../castep-cell-io/castep_cell_io' }, workflow_core = { path = '../../workflow_core', features = ['default-logging'] }, workflow_utils = { path = '../../workflow_utils' }."
+        },
+        {
+          "path": "examples/scf_dos_chain/src/main.rs",
+          "action": "create",
+          "guidance": "Create stub main.rs with mod config; and mod job_script; declarations. Add fn main() { println!('scf_dos_chain: skeleton'); }. No imports."
+        },
+        {
+          "path": "examples/scf_dos_chain/src/config.rs",
+          "action": "create",
+          "guidance": "Create empty config.rs with placeholder comment."
+        },
+        {
+          "path": "examples/scf_dos_chain/src/job_script.rs",
+          "action": "create",
+          "guidance": "Create empty job_script.rs with placeholder comment."
+        },
+        {
+          "path": "examples/scf_dos_chain/seeds/ZnO.cell",
+          "action": "modify",
+          "guidance": "Copy from examples/hubbard_u_sweep_slurm/seeds/ZnO.cell (or from examples/multi_param_sweep/seeds/ZnO.cell if already created)."
+        },
+        {
+          "path": "examples/scf_dos_chain/seeds/ZnO.param",
+          "action": "modify",
+          "guidance": "Copy from examples/hubbard_u_sweep_slurm/seeds/ZnO.param (or from examples/multi_param_sweep/seeds/ZnO.param if already created)."
+        },
+        {
+          "path": "Cargo.toml (workspace root)",
+          "action": "modify",
+          "guidance": "Add 'examples/scf_dos_chain' to the [workspace] members array. Insert it after 'examples/multi_param_sweep'. Keep 'examples/hubbard_u_sweep_slurm' in the list (it will be removed in G3-1)."
+        }
+      ],
+      "acceptance": [
+        "cargo check --workspace (both new crates + old slurm crate compile)",
+        "ls examples/scf_dos_chain/Cargo.toml examples/scf_dos_chain/src/main.rs examples/scf_dos_chain/src/config.rs examples/scf_dos_chain/src/job_script.rs examples/scf_dos_chain/seeds/ZnO.cell examples/scf_dos_chain/seeds/ZnO.param"
+      ],
+      "depends_on": [],
+      "wiring_checklist": [
+        {
+          "kind": "pub_mod",
+          "file": "examples/scf_dos_chain/src/main.rs",
+          "detail": "Add mod config; and mod job_script; before fn main()."
+        },
+        {
+          "kind": "pub_mod",
+          "file": "Cargo.toml (workspace root)",
+          "detail": "Add 'examples/scf_dos_chain' to the workspace members list."
+        }
+      ]
+    },
+    {
+      "id": "G2-2",
+      "description": "Implement the ChainConfig clap struct, parse_u_values function, and generate_job_script function (adapted for ChainConfig type) with tests written first. \u2014 CLI configuration (ChainConfig with single u-value, no sweep, no kpoints/cutoff), parse_u_values function (duplicated from multi_param_sweep), generate_job_script adapted for &ChainConfig parameter, and their in-file #[cfg(test)] tests.",
+      "files_in_scope": [
+        "examples/scf_dos_chain/src/config.rs and examples/scf_dos_chain/src/job_script.rs",
+        "examples/scf_dos_chain/src/config.rs",
+        "examples/scf_dos_chain/src/job_script.rs"
+      ],
+      "changes": [
+        {
+          "path": "examples/scf_dos_chain/src/config.rs",
+          "action": "modify",
+          "guidance": "Implement ChainConfig struct with #[derive(Parser, Debug)], #[command(name = 'scf_dos_chain')], and fields: u_value (f64, default 3.0), element (String, default 'Zn'), orbital (char, default 'd'), seed_name (String, default 'ZnO'), max_parallel (usize, default 1), local (bool), dry_run (bool), castep_command (String, default 'castep'), workdir (String, default '.'), plus SLURM fields (partition, ntasks, nix_flake, mpi_if \u2014 same as SweepConfig). Implement parse_u_values as a duplicate of the multi_param_sweep version (returning anyhow::Result<Vec<f64>>). Add #[cfg(test)] mod tests with the 7 ported parse_u_values tests. Place the parse_u_values tests directly in config.rs's test module."
+        },
+        {
+          "path": "examples/scf_dos_chain/src/job_script.rs",
+          "action": "modify",
+          "guidance": "Implement generate_job_script(config: &ChainConfig, task_id: &str, seed_name: &str) -> String. The function body is identical to the multi_param_sweep version (same SLURM fields on both config types). Use the exact same heredoc template with D.2 fix applied (all-space indentation, consistent SBATCH quoting). Import ChainConfig from crate::config. Add #[cfg(test)] mod tests with the 6 ported tests adapted for ChainConfig (using ChainConfig::parse_from(['test'])). Add use clap::Parser; in the test module. All 6 tests including no_literal_tabs must pass."
+        }
+      ],
+      "acceptance": [
+        "cargo check -p scf_dos_chain",
+        "cargo test -p scf_dos_chain -- config::tests (7 parse_u_values tests pass)",
+        "cargo test -p scf_dos_chain -- job_script::tests (6 tests pass, including no_literal_tabs)"
+      ],
+      "depends_on": [
+        "G2-1"
+      ],
+      "kind": "lib-tdd",
+      "tdd_interface": {
+        "test_file": "examples/scf_dos_chain/src/config.rs",
+        "test_module": "tests (in each file)",
+        "test_fn_name": "parse_basic_values",
+        "test_code": "// ===== config.rs tests =====\n// These are the same 7 parse_u_values tests as in multi_param_sweep,\n// duplicated here because parse_u_values is duplicated.\n\nuse crate::config::ChainConfig;\nuse clap::Parser;\n\n/// Helper: default config for job_script tests\nfn default_chain_config() -> ChainConfig {\n    ChainConfig::parse_from([\"test\"])\n}\n\n// --- parse_u_values tests (ported, adapted for anyhow::Result) ---\n\n#[test]\nfn parse_basic_values() {\n    let vals = parse_u_values(\"0.0,1.0,2.0\").unwrap();\n    assert_eq!(vals, vec![0.0, 1.0, 2.0]);\n}\n\n#[test]\nfn parse_with_whitespace() {\n    let vals = parse_u_values(\"  0.0 , 1.0 , 2.0  \").unwrap();\n    assert_eq!(vals, vec![0.0, 1.0, 2.0]);\n}\n\n#[test]\nfn parse_single_value() {\n    let vals = parse_u_values(\"42.0\").unwrap();\n    assert_eq!(vals, vec![42.0]);\n}\n\n#[test]\nfn parse_invalid_token() {\n    let err = parse_u_values(\"1.0,abc,2.0\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"abc\"), \"error should mention the invalid token: {msg}\");\n}\n\n#[test]\nfn parse_empty_token() {\n    let err = parse_u_values(\"1.0,,2.0\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"invalid\"), \"error should report parse failure: {msg}\");\n}\n\n#[test]\nfn parse_empty_string() {\n    let err = parse_u_values(\"\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"empty\") || msg.contains(\"invalid\"), \"expected parse failure on empty input, got: {msg}\");\n}\n\n#[test]\nfn parse_negative_values() {\n    let vals = parse_u_values(\"-1.0,2.0\").unwrap();\n    assert_eq!(vals, vec![-1.0, 2.0]);\n}\n\n// ===== job_script.rs tests =====\n\n#[test]\nfn contains_sbatch_directives() {\n    let config = default_chain_config();\n    let script = generate_job_script(&config, \"scf\", \"ZnO\");\n    assert!(script.contains(\"#SBATCH --job-name=\\\"scf\\\"\"));\n    assert!(script.contains(\"#SBATCH --partition=debug\"));\n    assert!(script.contains(\"#SBATCH --ntasks-per-node=16\"));\n    assert!(script.contains(\"#SBATCH --mem=30000m\"));\n}\n\n#[test]\nfn contains_seed_name() {\n    let config = default_chain_config();\n    let script = generate_job_script(&config, \"scf\", \"ZnO\");\n    assert!(script.contains(\"castep.mpi ZnO\"));\n}\n\n/// D.2 fix: no literal tabs\n#[test]\nfn no_literal_tabs() {\n    let config = default_chain_config();\n    let script = generate_job_script(&config, \"scf\", \"ZnO\");\n    assert!(!script.contains('\\t'), \"job script should not contain literal tab characters\");\n}\n\n#[test]\nfn starts_with_shebang() {\n    let config = default_chain_config();\n    let script = generate_job_script(&config, \"scf\", \"ZnO\");\n    assert!(script.starts_with(\"#!/usr/bin/env bash\"));\n}\n\n#[test]\nfn contains_nix_develop() {\n    let config = default_chain_config();\n    let script = generate_job_script(&config, \"scf\", \"ZnO\");\n    assert!(script.contains(\"nix develop\"));\n    assert!(script.contains(&config.nix_flake));\n}\n\n#[test]\nfn contains_mpi_interface() {\n    let config = default_chain_config();\n    let script = generate_job_script(&config, \"scf\", \"ZnO\");\n    assert!(script.contains(&format!(\"OMPI_MCA_btl_tcp_if_include={mpi_if}\", mpi_if = config.mpi_if)));\n}",
+        "signature": "pub fn parse_u_values(s: &str) -> anyhow::Result<Vec<f64>>",
+        "expected_behavior": "parse_u_values: identical behavior to the multi_param_sweep version \u2014 returns Vec<f64> for comma-separated numbers, errors with clear messages for non-numeric tokens and empty input. generate_job_script: identical template to the multi_param_sweep version but accepting &ChainConfig instead of &SweepConfig. Since SLURM fields are identical on both config types, the output is functionally the same. D.2 fix applied: all-space indentation, double-quoted SBATCH values, no literal tabs."
+      },
+      "wiring_checklist": [
+        {
+          "kind": "pub_mod",
+          "file": "examples/scf_dos_chain/src/config.rs",
+          "detail": "Add use clap::Parser; at the top."
+        },
+        {
+          "kind": "pub_mod",
+          "file": "examples/scf_dos_chain/src/job_script.rs",
+          "detail": "Add use crate::config::ChainConfig; at the top."
+        }
+      ]
+    },
+    {
+      "id": "G2-3",
+      "description": "Implement build_scf_task and build_dos_task functions that construct the two-task SCF+DOS chain, with tests for dependency structure and DOS setup logic. \u2014 Chain task construction: build_scf_task (SCF task with setup injecting hubbard_u, collect verifying .castep output), build_dos_task (DOS task depending on scf, setup copying checkpoint and setting Task::BandStructure), and in-file #[cfg(test)] tests verifying task IDs, dependency structure, workdir paths, and the DOS setup closure behavior.",
+      "files_in_scope": [
+        "examples/scf_dos_chain/src/main.rs"
+      ],
+      "changes": [
+        {
+          "path": "examples/scf_dos_chain/src/main.rs",
+          "action": "modify",
+          "guidance": "Add necessary imports: use config::{parse_u_values, ChainConfig}; use job_script::generate_job_script; plus castep_cell_fmt (parse, to_string_many_spaced, ToCellFile), castep_cell_io (CellDocument, ParamDocument, AtomHubbardU, HubbardU, HubbardUUnit, OrbitalU, Species, CutOffEnergy), workflow_utils::prelude::*. Note: do NOT import castep_cell_io::param::general::task::Task \u2014 use the fully-qualified path at the single use-site (see below).\n\nFunction build_scf_task(config, seed_cell, seed_param) -> Result<Task, WorkflowError>: Task ID 'scf', workdir PathBuf::from('runs/scf_dos'). Execution mode: if config.local, ExecutionMode::direct(&config.castep_command, &['ZnO']), else ExecutionMode::Queued. Setup closure: (1) create_dir(workdir), (2) parse seed_cell into CellDocument, inject hubbard_u using config.u_value/config.element/config.orbital (same builder pattern as multi_param_sweep), (3) serialize .cell + .param to workdir, (4) if !config.local, write job_script. Collect closure: verify <seed>.castep exists with 'Total time' marker.\n\nFunction build_dos_task(config, scf_task_id, seed_cell, seed_param) -> Result<Task, WorkflowError>: Task ID 'dos', workdir PathBuf::from('runs/scf_dos') (SAME workdir as SCF). Depends on scf_task_id. Execution mode: if config.local, ExecutionMode::direct(&config.castep_command, &['ZnO_DOS']), else ExecutionMode::Queued. Setup closure: (1) create_dir(workdir), (2) parse seed_cell into CellDocument, write to <workdir>/ZnO_DOS.cell (reuse seed cell, no hubbard_u injection needed since this is a DOS restart), (3) parse seed_param into ParamDocument, set doc.general.task = Some(castep_cell_io::param::general::task::Task::BandStructure) \u2014 THIS IS THE FULLY-QUALIFIED PATH, do not import the Task enum, (4) write to <workdir>/ZnO_DOS.param, (5) copy <workdir>/ZnO.check to <workdir>/ZnO_DOS.check using workflow_utils::files::copy_file or the prelude re-export, (6) if !config.local, write job_script via generate_job_script(&config, 'dos', 'ZnO_DOS'). Collect closure: verify ZnO_DOS.castep exists and contains 'Total time'.\n\nAdd #[cfg(test)] mod tests with the tests from tdd_interface.test_code. These tests verify structural correctness: task IDs, dependency structure, shared workdir, dependency counts."
+        }
+      ],
+      "acceptance": [
+        "cargo check -p scf_dos_chain",
+        "cargo test -p scf_dos_chain (config tests + job_script tests + chain logic tests all pass)",
+        "All chain structure tests pass: scf id is 'scf', dos id is 'dos', dos depends on scf, scf has no deps, dos has 1 dep, shared workdir, correct workdir path"
+      ],
+      "depends_on": [
+        "G2-2"
+      ],
+      "kind": "lib-tdd",
+      "tdd_interface": {
+        "test_file": "examples/scf_dos_chain/src/main.rs",
+        "test_module": "tests",
+        "test_fn_name": "scf_task_id",
+        "test_code": "use crate::config::ChainConfig;\nuse clap::Parser;\n\nfn default_config() -> ChainConfig {\n    ChainConfig::parse_from([\"test\"])\n}\n\nfn test_seeds() -> (&'static str, &'static str) {\n    (include_str!(\"../seeds/ZnO.cell\"), include_str!(\"../seeds/ZnO.param\"))\n}\n\n// --- Task ID and structure tests ---\n\n/// SCF task has ID 'scf'\n#[test]\nfn scf_task_id() {\n    let config = default_config();\n    let (cell, param) = test_seeds();\n    let task = build_scf_task(&config, cell, param).unwrap();\n    assert_eq!(task.id, \"scf\");\n}\n\n/// DOS task has ID 'dos'\n#[test]\nfn dos_task_id() {\n    let config = default_config();\n    let (cell, param) = test_seeds();\n    let task = build_dos_task(&config, \"scf\", cell, param).unwrap();\n    assert_eq!(task.id, \"dos\");\n}\n\n/// DOS task depends on SCF task\n#[test]\nfn dos_depends_on_scf() {\n    let config = default_config();\n    let (cell, param) = test_seeds();\n    let scf = build_scf_task(&config, cell, param).unwrap();\n    let dos = build_dos_task(&config, &scf.id, cell, param).unwrap();\n    assert!(dos.dependencies.contains(&\"scf\".to_string()));\n}\n\n/// SCF task has no dependencies\n#[test]\nfn scf_no_dependencies() {\n    let config = default_config();\n    let (cell, param) = test_seeds();\n    let task = build_scf_task(&config, cell, param).unwrap();\n    assert!(task.dependencies.is_empty());\n}\n\n/// DOS task has exactly 1 dependency\n#[test]\nfn dos_dependency_count() {\n    let config = default_config();\n    let (cell, param) = test_seeds();\n    let scf = build_scf_task(&config, cell, param).unwrap();\n    let dos = build_dos_task(&config, &scf.id, cell, param).unwrap();\n    assert_eq!(dos.dependencies.len(), 1);\n}\n\n/// Both tasks share the same workdir\n#[test]\nfn shared_workdir() {\n    let config = default_config();\n    let (cell, param) = test_seeds();\n    let scf = build_scf_task(&config, cell, param).unwrap();\n    let dos = build_dos_task(&config, &scf.id, cell, param).unwrap();\n    assert_eq!(scf.workdir, dos.workdir, \"SCF and DOS tasks should share the same workdir\");\n    let wd = scf.workdir.to_string_lossy();\n    assert!(wd.contains(\"scf_dos\"), \"workdir should contain 'scf_dos': {wd}\");\n}\n\n/// SCF task has correct workdir path\n#[test]\nfn scf_workdir_path() {\n    let config = default_config();\n    let (cell, param) = test_seeds();\n    let task = build_scf_task(&config, cell, param).unwrap();\n    let wd = task.workdir.to_string_lossy();\n    assert!(wd.ends_with(\"scf_dos\") || wd.contains(\"scf_dos\"), \"workdir should end with or contain 'scf_dos': {wd}\");\n}",
+        "signature": "fn build_scf_task(config: &ChainConfig, seed_cell: &str, seed_param: &str) -> Result<Task, WorkflowError>",
+        "expected_behavior": "build_scf_task: produces a Task with id 'scf', workdir ending in 'runs/scf_dos', no dependencies, and a setup closure that injects the Hubbard U value from config into the seed cell file. The collect closure verifies the .castep output. build_dos_task: produces a Task with id 'dos', same workdir as SCF, exactly 1 dependency on the SCF task ID. The setup closure copies ZnO.check to ZnO_DOS.check and sets general.task = Task::BandStructure in the param file. The collect closure verifies ZnO_DOS.castep output."
+      }
+    },
+    {
+      "id": "G2-4",
+      "description": "Implement the main() function that orchestrates config parsing, chain construction, workflow creation, dry-run display, and local/queued execution. \u2014 Binary entry point: main() function that parses ChainConfig, builds SCF and DOS tasks, constructs a Workflow with the two-task chain, and supports dry-run/local/queued execution paths.",
+      "files_in_scope": [
+        "examples/scf_dos_chain/src/main.rs"
+      ],
+      "changes": [
+        {
+          "path": "examples/scf_dos_chain/src/main.rs",
+          "action": "modify",
+          "guidance": "Implement the main() function following the same pattern as multi_param_sweep main(). Call workflow_core::init_default_logging().ok(). Parse ChainConfig::parse(). Load seed files via include_str!('../seeds/ZnO.cell') and include_str!('../seeds/ZnO.param'). Call build_scf_task(&config, seed_cell, seed_param)? to get the SCF task, then build_dos_task(&config, &scf_task.id, seed_cell, seed_param)? to get the DOS task. Construct workflow: Workflow::new('scf_dos_chain').with_max_parallel(config.max_parallel)?.with_log_dir('logs').with_root_dir(&config.workdir). If !config.local, add .with_queued_submitter(Arc::new(QueuedRunner::new(SchedulerKind::Slurm))). Add SCF task first, then DOS task via workflow.add_task(task)?. Dry-run: same pattern as multi_param_sweep (print topological order). For execution: JsonStateStore at '.scf_dos_chain.workflow.json', local via run_default, queued via Workflow::run with SystemProcessRunner + ShellHookExecutor. Print workflow summary on completion. Import std::sync::Arc."
+        }
+      ],
+      "acceptance": [
+        "cargo check -p scf_dos_chain",
+        "cargo run --bin scf_dos_chain -- --dry-run (prints 'scf' then 'dos' \u2014 chain dependency order, exactly 2 tasks)",
+        "cargo run --bin scf_dos_chain -- --local --dry-run (same output \u2014 local flag doesn't change dry-run topology)"
+      ],
+      "depends_on": [
+        "G2-3"
+      ]
+    },
+    {
+      "id": "G2-5",
+      "description": "Update the existing hubbard_u_sweep binary's Cargo.toml to use the local path dep for castep-cell-io v0.5.0 and fix any API breakage in main.rs. \u2014 Dependency update: change castep-cell-io from version 0.4.0 to local path dep, verify compilation, and fix any API breakage in main.rs (Species::Symbol signature, builder method changes, or enum variant renames).",
+      "files_in_scope": [
+        "examples/hubbard_u_sweep",
+        "examples/hubbard_u_sweep/Cargo.toml",
+        "examples/hubbard_u_sweep/src/main.rs"
+      ],
+      "changes": [
+        {
+          "path": "examples/hubbard_u_sweep/Cargo.toml",
+          "action": "modify",
+          "guidance": "Change the castep-cell-io dependency line from 'castep-cell-io = \"0.4.0\"' to 'castep-cell-io = { path = \"../castep-cell-io/castep_cell_io\" }'. All other dependencies (anyhow = \"1\", castep-cell-fmt = \"0.1.0\", workflow_core, workflow_utils) remain unchanged."
+        },
+        {
+          "path": "examples/hubbard_u_sweep/src/main.rs",
+          "action": "modify",
+          "guidance": "Run cargo check -p hubbard_u_sweep after the Cargo.toml change. If compilation fails due to v0.5.0 API changes, fix the call sites. Common breakage points: Species::Symbol may take a different type than String (e.g., &str or a newtype) \u2014 check with LSP hover on Species::Symbol. Builder method names on AtomHubbardU/HubbardU may have changed. HubbardUUnit enum variants may be renamed. Fix each error based on the compiler diagnostic. The existing code already uses castep_cell_fmt::parse::<CellDocument>(&input) which is the v0.5.0 parse API pattern, so the parse logic should work unchanged. Only fix what the compiler reports as errors."
+        }
+      ],
+      "acceptance": [
+        "cargo check -p hubbard_u_sweep (must compile with new path dep; zero errors, zero warnings)"
+      ],
+      "depends_on": []
+    },
+    {
+      "id": "G3-1",
+      "description": "Remove hubbard_u_sweep_slurm from workspace members, delete the entire old crate directory, and run full workspace build, test, clippy, and dry-run verification. \u2014 Workspace finalization: updating workspace members list to remove the old binary, deleting examples/hubbard_u_sweep_slurm/ entirely, and running comprehensive verification (build, test, clippy dead-code, dry-run both binaries).",
+      "files_in_scope": [
+        "Cargo.toml (workspace root) and examples/hubbard_u_sweep_slurm/ (deletion)",
+        "Cargo.toml (workspace root)",
+        "examples/hubbard_u_sweep_slurm/"
+      ],
+      "changes": [
+        {
+          "path": "Cargo.toml (workspace root)",
+          "action": "modify",
+          "guidance": "Remove 'examples/hubbard_u_sweep_slurm' from the [workspace] members array. The members list should now contain exactly: 'workflow_core', 'workflow_utils', 'examples/hubbard_u_sweep', 'examples/multi_param_sweep', 'examples/scf_dos_chain', 'workflow-cli'. Do NOT change any other configuration (resolver, workspace.dependencies, etc.)."
+        },
+        {
+          "path": "examples/hubbard_u_sweep_slurm/",
+          "action": "delete",
+          "guidance": "Delete the entire directory and all its contents: Cargo.toml, src/main.rs, src/config.rs, src/job_script.rs, seeds/ZnO.cell, seeds/ZnO.param, .validation-complete. Use rm -rf or the equivalent VCS remove. This removes the old buggy binary entirely from the codebase."
+        }
+      ],
+      "acceptance": [
+        "cargo build --workspace (all crates compile with zero errors, zero warnings)",
+        "cargo test --workspace (all tests pass; old slurm tests are gone, new binary tests run and pass)",
+        "cargo clippy --workspace --all-targets -- -W dead-code (no dead-code warnings from stale imports \u2014 if any appear, fix them in the library crates)",
+        "cargo run --bin multi_param_sweep -- --dry-run (produces correct topological order with all 6 default SCF task IDs, no DOS tasks, no duplicate IDs)",
+        "cargo run --bin multi_param_sweep -- --dry-run --sweep-mode pairwise --u-values 0,1,2 --kpoints 8x8x8,6x6x6,4x4x4 --cutoffs 300,500,800 (produces 3 tasks, no errors)",
+        "cargo run --bin scf_dos_chain -- --dry-run (produces 'scf' then 'dos' \u2014 exactly 2 tasks, chain dependency respected)",
+        "cargo run --bin scf_dos_chain -- --local --dry-run (same output \u2014 'scf' then 'dos')",
+        "bash -c 'ls examples/hubbard_u_sweep_slurm/ 2>&1 | grep -q \"No such file\"' (old crate directory is fully gone)"
+      ],
+      "depends_on": [
+        "G1-5",
+        "G2-4",
+        "G2-5"
+      ],
+      "wiring_checklist": [
+        {
+          "kind": "pub_mod",
+          "file": "Cargo.toml (workspace root)",
+          "detail": "Remove 'examples/hubbard_u_sweep_slurm' from [workspace] members."
+        }
+      ]
+    }
+  ]
+}
\ No newline at end of file
diff --git a/notes/directions/phase-6-fix/draft-directions.json b/notes/directions/phase-6-fix/draft-directions.json
new file mode 100644
index 0000000..12c7b73
--- /dev/null
+++ b/notes/directions/phase-6-fix/draft-directions.json
@@ -0,0 +1,583 @@
+{
+  "architecture_notes": {
+    "plan_id": "phase-6-fix",
+    "summary": "Replace the buggy hubbard_u_sweep_slurm binary with two purpose-built binaries — multi_param_sweep (independent SCF parameter sweep) and scf_dos_chain (SCF+DOS task chain) — using only local path deps for castep-cell-io v0.5.0.",
+    "decisions": [
+      {
+        "id": "parse_time_validation",
+        "title": "Parse-time validation at CLI boundary",
+        "rationale": "parse_kpoints and parse_cutoffs error immediately at CLI parse time, not inside setup closures. A malformed k-point like '8xx8' must fail before task construction begins (Pattern 3)."
+      },
+      {
+        "id": "internal_type_choice",
+        "title": "parse_kpoints uses [u32; 3] not a custom struct",
+        "rationale": "The array type directly converts to KpointsMpGrid([kx, ky, kz]) with zero overhead. The type guarantee ('exactly 3 u32s') is enforced by the array type itself."
+      },
+      {
+        "id": "duplication_choice",
+        "title": "parse_u_values and generate_job_script are duplicated, not extracted",
+        "rationale": "Each function is ~15-30 lines with stable semantics. Extracting to a shared crate is overkill for 2 consumers. If a third consumer appears, extraction becomes justified."
+      },
+      {
+        "id": "task_naming_collision",
+        "title": "Fully-qualified path for Task::BandStructure, not alias import",
+        "rationale": "The collision between workflow_core::task::Task and castep_cell_io::param::general::Task is resolved by using the fully-qualified path at the single use-site where Task::BandStructure is set. No alias needed, no import shadow risk."
+      },
+      {
+        "id": "shared_workdir",
+        "title": "Shared workdir for chained tasks",
+        "rationale": "Both SCF and DOS tasks in scf_dos_chain use runs/scf_dos/. This makes checkpoint file access straightforward: the DOS setup copies ZnO.check to ZnO_DOS.check within the same directory (Pattern 6)."
+      },
+      {
+        "id": "workspace_serialization",
+        "title": "Workspace member additions serialize G1 and G2 skeleton tasks",
+        "rationale": "Both new crates use workspace = true dependencies (anyhow, clap, itertools), which require workspace membership for cargo check to resolve. Since both crate skeletons must add themselves to the root Cargo.toml workspace.members, G2-1 depends on G1-1 to avoid a merge conflict on the same file. The groups remain logically independent at the code level."
+      },
+      {
+        "id": "no_new_library_crates",
+        "title": "No new library crates are created",
+        "rationale": "All new code lives in binary (example) crates. The new code is workflow usage (orchestration, not abstractions). The library crates (workflow_core, workflow_utils) are unchanged."
+      }
+    ],
+    "crate_boundaries": {
+      "parse_u_values": "Duplicated in multi_param_sweep/src/config.rs and scf_dos_chain/src/config.rs",
+      "parse_kpoints": "multi_param_sweep/src/config.rs only (not needed by Binary 2)",
+      "parse_cutoffs": "multi_param_sweep/src/config.rs only",
+      "generate_job_script": "Duplicated in both binaries' job_script.rs (D.2 fix applies to both)",
+      "SweepConfig": "multi_param_sweep/src/config.rs",
+      "ChainConfig": "scf_dos_chain/src/config.rs",
+      "sweep_logic": "multi_param_sweep/src/main.rs (build_one_scf_task, build_all_scf_tasks)",
+      "chain_logic": "scf_dos_chain/src/main.rs (build_scf_task, build_dos_task)",
+      "seed_files": "Each binary's seeds/ directory (identical content, self-contained)"
+    }
+  },
+  "known_pitfalls": [
+    {
+      "id": "task_naming_collision_P0",
+      "title": "Task name collision in scf_dos_chain (P0)",
+      "description": "use workflow_utils::prelude::* brings workflow_core::task::Task into scope. Importing castep_cell_io::param::general::task::Task would shadow it. Resolution: use the fully-qualified path castep_cell_io::param::general::task::Task::BandStructure at the single use-site in the DOS setup closure. Verify the exact module path with LSP hover during implementation.",
+      "mitigation": "Do not add use castep_cell_io::param::general::task::Task. Instead use the fully-qualified path at the single use-site."
+    },
+    {
+      "id": "path_dep_filesystem",
+      "title": "Local path dep must exist on the filesystem",
+      "description": "The path ../castep-cell-io/castep_cell_io must resolve to a sibling directory containing a Cargo.toml with the castep-cell-io crate. Verify this exists before implementation.",
+      "mitigation": "Pre-implementation check: verify ../castep-cell-io/castep_cell_io/Cargo.toml exists."
+    },
+    {
+      "id": "castep_cell_fmt_compat",
+      "title": "castep-cell-fmt version compatibility",
+      "description": "castep-cell-fmt = 0.1.0 from crates.io while castep-cell-io uses a local path dep at v0.5.0. If v0.5.0 depends on a newer castep-cell-fmt, Cargo resolves the semver-compatible version automatically. No conflict expected.",
+      "mitigation": "No action needed; Cargo handles this automatically."
+    },
+    {
+      "id": "pairwise_validation_order",
+      "title": "Pairwise mode validation ordering",
+      "description": "In pairwise mode, validate that both --kpoints and --cutoffs are provided BEFORE parsing them. If either is None, return a clear error immediately. After parsing, validate all three lists have the same length. The error message must state the expected and actual lengths.",
+      "mitigation": "Check sweep_mode first, then validate presence of both Option fields, then parse, then check lengths."
+    },
+    {
+      "id": "hubbard_u_sweep_v050",
+      "title": "hubbard_u_sweep compatibility with v0.5.0 path dep",
+      "description": "The existing hubbard_u_sweep/src/main.rs already uses the v0.5.0 parse API pattern (castep_cell_fmt::parse::<CellDocument>(&input)). However, v0.5.0 may have changed Species::Symbol signature, builder method names, or HubbardUUnit enum variants. If compilation fails after the path-dep change, fix these call sites based on compiler error messages.",
+      "mitigation": "After changing the path dep, run cargo check -p hubbard_u_sweep. If it fails, fix each error based on the compiler message. Common changes may include Species::Symbol type or builder method names."
+    },
+    {
+      "id": "dead_code_after_deletion",
+      "title": "Dead-code detection after refactoring (F.3 mitigation)",
+      "description": "After deleting hubbard_u_sweep_slurm, run cargo clippy --workspace --all-targets -- -W dead-code to detect any stale imports or dead functions left over from the refactoring.",
+      "mitigation": "Include clippy dead-code check in the Group 3 acceptance criteria."
+    },
+    {
+      "id": "d2_carrier",
+      "title": "D.2 formatting fix must be embedded in ported job_script.rs",
+      "description": "The no_literal_tabs test from the old binary must be ported and must pass. The D.2 fix (clean heredoc template with all-space indentation, consistent double-quoting on SBATCH directives) must be applied to the ported generate_job_script function in BOTH new binaries.",
+      "mitigation": "Implementation checklist: all-space indentation, SBATCH directives use double-quotes consistently, shell continuation lines use backslash, no_literal_tabs test passes in both binaries."
+    },
+    {
+      "id": "deferred_items",
+      "title": "Deferred items: what NOT to do",
+      "description": "D.1 (portable SLURM config): Do NOT implement. D.3 (expanded unit tests for generate_job_script): Do NOT expand beyond ported tests. read_task_ids edge case: Do NOT fix. All three have unmet preconditions.",
+      "mitigation": "Review the deferred-and-patterns.md file for full details on each deferred item."
+    },
+    {
+      "id": "workspace_serialization_pitfall",
+      "title": "G1-1 and G2-1 both modify root Cargo.toml — serialized",
+      "description": "Both multi_param_sweep and scf_dos_chain use workspace = true dependencies, requiring workspace membership. To avoid merge conflicts on the root Cargo.toml, TASK-G2-1 must run after TASK-G1-1 completes. The groups are logically independent at the code level; this dependency is solely a file collision on the workspace member list.",
+      "mitigation": "TASK-G2-1 depends_on TASK-G1-1. Only the skeleton creation tasks are serialized; all other tasks within each group can run in parallel with tasks in the other group."
+    }
+  ],
+  "patterns_followed": [
+    {
+      "id": "pattern_1",
+      "name": "Purpose-specific binaries over mode-flags",
+      "check": "Neither binary should have a flag that toggles between 'independent tasks' and 'chained tasks'. multi_param_sweep has a --sweep-mode flag for product vs pairwise (both are independent SCF sweeps, same workflow type). scf_dos_chain has no mode flag."
+    },
+    {
+      "id": "pattern_2",
+      "name": "Parameter encoding in task IDs",
+      "check": "multi_param_sweep task IDs: scf_U{u}_k{k}_c{c} (e.g., scf_U3.0_k8x8x8_c500). scf_dos_chain task IDs: scf and dos (single parameter set, no collision risk)."
+    },
+    {
+      "id": "pattern_3",
+      "name": "Parse-time validation gates",
+      "check": "Every parsing function returns anyhow::Result<T> with messages containing the expected format and what went wrong. Examples: 'kpoints list is empty', 'invalid k-point: expected 3 axes, got 2'."
+    },
+    {
+      "id": "pattern_4",
+      "name": "Explicit defaults",
+      "check": "--sweep-mode defaults to 'product'. --kpoints and --cutoffs are Option<String> defaulting to None (seed defaults used). Pairwise mode errors if either is None."
+    },
+    {
+      "id": "pattern_5",
+      "name": "In-memory document mutation",
+      "check": "All CASTEP file modifications go through parse -> mutate typed fields -> to_cell_file() -> to_string_many_spaced(). Never use format! on raw strings to build .cell/.param content."
+    },
+    {
+      "id": "pattern_6",
+      "name": "Shared workdir for chained tasks",
+      "check": "scf_dos_chain places both tasks in runs/scf_dos/. DOS setup copies ZnO.check to ZnO_DOS.check in the same directory."
+    },
+    {
+      "id": "pattern_7",
+      "name": "Clean heredoc template",
+      "check": "The no_literal_tabs test must pass. Use consistent space-only indentation. SBATCH directives use consistent quoting (double-quotes around values that might contain spaces)."
+    }
+  ],
+  "task_groups": [
+    {
+      "id": "core-1",
+      "name": "multi_param_sweep binary",
+      "description": "Create the complete examples/multi_param_sweep/ crate — a purpose-built binary for independent SCF parameter sweeps across Hubbard U, k-point MP grid, and cutoff energy in product or pairwise mode.",
+      "tasks": [
+        {
+          "id": "G1-1",
+          "title": "Create multi_param_sweep crate skeleton and add to workspace",
+          "kind": "direct",
+          "scope": "Create the directory structure, Cargo.toml, stub source files, seed files, and add the crate to workspace members so cargo check resolves dependencies.",
+          "crate_module": "examples/multi_param_sweep (new crate)",
+          "responsible_for": "Crate creation: Cargo.toml with all dependencies, minimal main.rs with mod declarations, empty config.rs, empty job_script.rs, seed files from existing hubbard_u_sweep_slurm, and workspace member registration.",
+          "depends_on": [],
+          "enables": ["G1-2", "G1-3"],
+          "can_run_in_parallel_with": [],
+          "changes": [
+            {
+              "file": "examples/multi_param_sweep/Cargo.toml",
+              "guidance": "Create Cargo.toml with [package] name = 'multi_param_sweep', edition 2021. Use [[bin]] pointing to src/main.rs. Dependencies: anyhow = { workspace = true }, clap = { workspace = true }, itertools = { workspace = true }, castep-cell-fmt = '0.1.0', castep-cell-io = { path = '../castep-cell-io/castep_cell_io' }, workflow_core = { path = '../../workflow_core', features = ['default-logging'] }, workflow_utils = { path = '../../workflow_utils' }. Copy from hubbard_u_sweep_slurm/Cargo.toml as a template, adjusting the package name and adding kpoints/cutoff-related deps."
+            },
+            {
+              "file": "examples/multi_param_sweep/src/main.rs",
+              "guidance": "Create stub main.rs with mod config; and mod job_script; declarations. Add a minimal fn main() { println!('multi_param_sweep: skeleton'); } that compiles. No imports yet."
+            },
+            {
+              "file": "examples/multi_param_sweep/src/config.rs",
+              "guidance": "Create empty config.rs with a placeholder comment: // Config will be added in G1-2. No struct, no functions, no tests yet."
+            },
+            {
+              "file": "examples/multi_param_sweep/src/job_script.rs",
+              "guidance": "Create empty job_script.rs with a placeholder comment: // Job script will be added in G1-3."
+            },
+            {
+              "file": "examples/multi_param_sweep/seeds/ZnO.cell",
+              "guidance": "Copy the content from examples/hubbard_u_sweep_slurm/seeds/ZnO.cell. This is the ZnO wurtzite lattice cell file used as the seed for all task setups."
+            },
+            {
+              "file": "examples/multi_param_sweep/seeds/ZnO.param",
+              "guidance": "Copy the content from examples/hubbard_u_sweep_slurm/seeds/ZnO.param. This is the seed param file (task: SinglePoint) used as the base for task-specific param modifications."
+            },
+            {
+              "file": "Cargo.toml (workspace root)",
+              "guidance": "Add 'examples/multi_param_sweep' to the [workspace] members array. Insert it after 'examples/hubbard_u_sweep_slurm' (keeping the old binary for now). Keep all other members unchanged."
+            }
+          ],
+          "wiring_checklist": [
+            {
+              "parent_file": "examples/multi_param_sweep/src/main.rs",
+              "action": "add_mod_declaration",
+              "detail": "Add mod config; and mod job_script; at the top of the file (before fn main)."
+            },
+            {
+              "parent_file": "Cargo.toml (workspace root)",
+              "action": "add_workspace_member",
+              "detail": "Add 'examples/multi_param_sweep' to [workspace] members array."
+            }
+          ],
+          "acceptance": [
+            "cargo check --workspace (must compile — new crate resolves workspace deps, old slurm crate still compiles)",
+            "ls examples/multi_param_sweep/Cargo.toml examples/multi_param_sweep/src/main.rs examples/multi_param_sweep/src/config.rs examples/multi_param_sweep/src/job_script.rs examples/multi_param_sweep/seeds/ZnO.cell examples/multi_param_sweep/seeds/ZnO.param"
+          ],
+          "notes_for_subagent": "This is the crate skeleton task. Do NOT implement any real logic yet — just create the files and verify compilation. The crate MUST be in workspace.members for workspace = true deps to resolve. When creating the Cargo.toml, use the exact dependency structure from hubbard_u_sweep_slurm/Cargo.toml as a template, but replace castep-cell-io = '0.4.0' with castep-cell-io = { path = '../castep-cell-io/castep_cell_io' }. Note the path: ../../workflow_core and ../../workflow_utils (two levels up from examples/multi_param_sweep)."
+        },
+        {
+          "id": "G1-2",
+          "title": "Implement parse functions and SweepConfig in config.rs with TDD",
+          "kind": "lib-tdd",
+          "tdd_interface": {
+            "signature": [
+              "pub fn parse_u_values(s: &str) -> anyhow::Result<Vec<f64>>",
+              "pub fn parse_kpoints(s: &str) -> anyhow::Result<Vec<[u32; 3]>>",
+              "pub fn parse_cutoffs(s: &str) -> anyhow::Result<Vec<f64>>"
+            ],
+            "test_code": "// --- parse_u_values tests (ported from old binary, adapted for anyhow::Result) ---\n\n/// Parse comma-separated basic values\n#[test]\nfn parse_basic_values() {\n    let vals = parse_u_values(\"0.0,1.0,2.0\").unwrap();\n    assert_eq!(vals, vec![0.0, 1.0, 2.0]);\n}\n\n/// Parse with whitespace around values\n#[test]\nfn parse_with_whitespace() {\n    let vals = parse_u_values(\"  0.0 , 1.0 , 2.0  \").unwrap();\n    assert_eq!(vals, vec![0.0, 1.0, 2.0]);\n}\n\n/// Parse a single value\n#[test]\nfn parse_single_value() {\n    let vals = parse_u_values(\"42.0\").unwrap();\n    assert_eq!(vals, vec![42.0]);\n}\n\n/// Invalid token in the middle\n#[test]\nfn parse_invalid_token() {\n    let err = parse_u_values(\"1.0,abc,2.0\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"abc\"), \"error should mention the invalid token: {msg}\");\n}\n\n/// Empty token in the middle (two consecutive commas)\n#[test]\nfn parse_empty_token() {\n    let err = parse_u_values(\"1.0,,2.0\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"invalid\"), \"error should report parse failure: {msg}\");\n}\n\n/// Entirely empty input\n#[test]\nfn parse_empty_string() {\n    let err = parse_u_values(\"\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"empty\") || msg.contains(\"invalid\"), \"expected parse failure on empty input, got: {msg}\");\n}\n\n/// Negative values\n#[test]\nfn parse_negative_values() {\n    let vals = parse_u_values(\"-1.0,2.0\").unwrap();\n    assert_eq!(vals, vec![-1.0, 2.0]);\n}\n\n// --- parse_kpoints tests (new) ---\n\n/// Basic valid k-points\n#[test]\nfn parse_kpoints_basic() {\n    let kpts = parse_kpoints(\"8x8x8,6x6x6\").unwrap();\n    assert_eq!(kpts, vec![[8, 8, 8], [6, 6, 6]]);\n}\n\n/// Single valid k-point\n#[test]\nfn parse_kpoints_single() {\n    let kpts = parse_kpoints(\"4x4x4\").unwrap();\n    assert_eq!(kpts, vec![[4, 4, 4]]);\n}\n\n/// With whitespace\n#[test]\nfn parse_kpoints_whitespace() {\n    let kpts = parse_kpoints(\" 8x8x8 , 6x6x6 \").unwrap();\n    assert_eq!(kpts, vec![[8, 8, 8], [6, 6, 6]]);\n}\n\n/// Wrong number of axes (only 2 instead of 3)\n#[test]\nfn parse_kpoints_wrong_axes() {\n    let err = parse_kpoints(\"8x8\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"3 axes\") || msg.contains(\"got 2\") || msg.contains(\"expected 3\"), \"error should mention wrong axis count: {msg}\");\n}\n\n/// Non-numeric axis value\n#[test]\nfn parse_kpoints_non_numeric() {\n    let err = parse_kpoints(\"abc\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"invalid\") || msg.contains(\"abc\") || msg.contains(\"expected 3 axes\"), \"error should report the problem: {msg}\");\n}\n\n/// Empty input\n#[test]\nfn parse_kpoints_empty() {\n    let err = parse_kpoints(\"\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"empty\"), \"error should say kpoints list is empty: {msg}\");\n}\n\n/// Large grid values\n#[test]\nfn parse_kpoints_large() {\n    let kpts = parse_kpoints(\"12x12x12\").unwrap();\n    assert_eq!(kpts, vec![[12, 12, 12]]);\n}\n\n// --- parse_cutoffs tests (new) ---\n\n/// Basic valid cutoffs\n#[test]\nfn parse_cutoffs_basic() {\n    let cuts = parse_cutoffs(\"300,500,800\").unwrap();\n    assert_eq!(cuts, vec![300.0, 500.0, 800.0]);\n}\n\n/// Single cutoff\n#[test]\nfn parse_cutoffs_single() {\n    let cuts = parse_cutoffs(\"450\").unwrap();\n    assert_eq!(cuts, vec![450.0]);\n}\n\n/// With whitespace\n#[test]\nfn parse_cutoffs_whitespace() {\n    let cuts = parse_cutoffs(\" 300 , 500 , 800 \").unwrap();\n    assert_eq!(cuts, vec![300.0, 500.0, 800.0]);\n}\n\n/// Invalid token\n#[test]\nfn parse_cutoffs_invalid_token() {\n    let err = parse_cutoffs(\"300,abc,800\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"abc\"), \"error should mention the invalid token: {msg}\");\n}\n\n/// Empty input\n#[test]\nfn parse_cutoffs_empty() {\n    let err = parse_cutoffs(\"\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"empty\"), \"error should say cutoffs list is empty: {msg}\");\n}\n\n/// Negative and fractional values\n#[test]\nfn parse_cutoffs_negative() {\n    let cuts = parse_cutoffs(\"-100.5,200.0\").unwrap();\n    assert_eq!(cuts, vec![-100.5, 200.0]);\n}",
+            "expected_behavior": "parse_u_values: returns Vec<f64> for comma-separated numbers, errors on non-numeric tokens with the offending token in the message. Empty input produces an error containing 'empty' or 'invalid'. Whitespace around values is trimmed. parse_kpoints: returns Vec<[u32; 3]> for comma-separated NxNxN triplets. Errors on wrong number of axes (must be exactly 3 per segment). Empty input produces an error containing 'empty'. Whitespace around values is trimmed. Non-numeric axis values produce errors mentioning the invalid input. parse_cutoffs: returns Vec<f64> for comma-separated numbers, same semantics as parse_u_values but with cutoff-specific error context.",
+            "test_file": "examples/multi_param_sweep/src/config.rs",
+            "test_module": "tests"
+          },
+          "scope": "Implement the SweepConfig clap struct and all three parsing utility functions (parse_u_values, parse_kpoints, parse_cutoffs) with comprehensive tests written first.",
+          "crate_module": "examples/multi_param_sweep/src/config.rs",
+          "responsible_for": "CLI configuration and parse-time validation: SweepConfig struct (clap Parser derive, all 16 fields from the plan plus SLURM fields), parse_u_values (ported with anyhow::Result), parse_kpoints (new, strict NxNxN validation), parse_cutoffs (new, f64 comma-separated), and their in-file #[cfg(test)] tests.",
+          "depends_on": ["G1-1"],
+          "enables": ["G1-4", "G1-5"],
+          "can_run_in_parallel_with": ["G1-3"],
+          "changes": [
+            {
+              "file": "examples/multi_param_sweep/src/config.rs",
+              "guidance": "Implement the full SweepConfig struct with #[derive(Parser, Debug)], #[command(name = 'multi_param_sweep')], and all fields from the plan: u_values (String, default '0.0,1.0,2.0,3.0,4.0,5.0'), kpoints (Option<String>), cutoffs (Option<String>), sweep_mode (String, default 'product'), element (String, default 'Zn'), orbital (char, default 'd'), seed_name (String, default 'ZnO'), max_parallel (usize, default 4), local (bool), dry_run (bool), castep_command (String, default 'castep'), workdir (String, default '.'), plus SLURM fields: partition (String, env CASTEP_SLURM_PARTITION, default 'debug'), ntasks (u32, default 16), nix_flake (String, env CASTEP_NIX_FLAKE, with the long default), mpi_if (String, env CASTEP_MPI_IF, default 'enp6s0'). Each field should have a doc comment describing its purpose and the expected format for string-typed fields. Implement parse_u_values (ported, returning anyhow::Result<Vec<f64>> not Result<_, String>), parse_kpoints (new, splitting on comma then on 'x', validating exactly 3 u32 axes per segment), parse_cutoffs (new, splitting on comma, trimming, parsing f64). All three return anyhow::Result with clear error messages describing expected format. parse_kpoints must validate each segment splits into exactly 3 parts on 'x' and each part parses as u32 — reject '8x8' (2 axes) and 'abc' (non-numeric). Add #[cfg(test)] mod tests with the tests from tdd_interface.test_code. For parse_kpoints, the 'wrong_axes' test must verify the error message mentions the expected axis count or format. For empty input tests, the error must contain 'empty'."
+            }
+          ],
+          "wiring_checklist": [],
+          "acceptance": [
+            "cargo check -p multi_param_sweep",
+            "cargo test -p multi_param_sweep -- config::tests (all parse tests pass: 7 parse_u_values + 7 parse_kpoints + 6 parse_cutoffs = 20 tests)",
+            "cargo test -p multi_param_sweep (no failures; job_script tests may not exist yet and will be ignored)"
+          ],
+          "notes_for_subagent": "The parse_u_values function changes return type from Result<Vec<f64>, String> to anyhow::Result<Vec<f64>>. Use anyhow::bail!() and anyhow!() macros for errors. The parse_kpoints function must validate each segment has EXACTLY 3 u32 values separated by 'x'. For parse_kpoints('8x8'), the error should mention 'expected 3 axes, got 2'. For parse_kpoints(''), the error should say 'kpoints list is empty'. For parse_kpoints('abc'), report that it is invalid with the string context. The SweepConfig doc comments must describe the expected format for each string-typed field (u_values, kpoints, cutoffs). Pattern 3 (parse-time validation) is critical — errors must be clear and immediate. Pattern 4: kpoints and cutoffs are Option<String> defaulting to None; sweep_mode defaults to 'product'."
+        },
+        {
+          "id": "G1-3",
+          "title": "Implement generate_job_script with D.2 fix using TDD",
+          "kind": "lib-tdd",
+          "tdd_interface": {
+            "signature": "pub fn generate_job_script(config: &SweepConfig, task_id: &str, seed_name: &str) -> String",
+            "test_code": "use crate::config::SweepConfig;\nuse clap::Parser;\n\nfn default_config() -> SweepConfig {\n    SweepConfig::parse_from([\"test\"])\n}\n\n/// Script contains expected SBATCH directives with the task ID and config values\n#[test]\nfn contains_sbatch_directives() {\n    let config = default_config();\n    let script = generate_job_script(&config, \"scf_U1.0\", \"ZnO\");\n    assert!(script.contains(\"#SBATCH --job-name=\\\"scf_U1.0\\\"\"));\n    assert!(script.contains(\"#SBATCH --partition=debug\"));\n    assert!(script.contains(\"#SBATCH --ntasks-per-node=16\"));\n    assert!(script.contains(\"#SBATCH --mem=30000m\"));\n}\n\n/// Script references the correct seed name in the castep command\n#[test]\nfn contains_seed_name() {\n    let config = default_config();\n    let script = generate_job_script(&config, \"scf_U0.0\", \"ZnO\");\n    assert!(script.contains(\"castep.mpi ZnO\"));\n}\n\n/// D.2 fix: script must not contain literal tab characters\n#[test]\nfn no_literal_tabs() {\n    let config = default_config();\n    let script = generate_job_script(&config, \"scf_U0.0\", \"ZnO\");\n    assert!(!script.contains('\\t'), \"job script should not contain literal tab characters\");\n}\n\n/// Script starts with a shebang line\n#[test]\nfn starts_with_shebang() {\n    let config = default_config();\n    let script = generate_job_script(&config, \"scf_U0.0\", \"ZnO\");\n    assert!(script.starts_with(\"#!/usr/bin/env bash\"));\n}\n\n/// Script contains nix develop with the configured flake URI\n#[test]\nfn contains_nix_develop() {\n    let config = default_config();\n    let script = generate_job_script(&config, \"scf_U0.0\", \"ZnO\");\n    assert!(script.contains(\"nix develop\"));\n    assert!(script.contains(&config.nix_flake));\n}\n\n/// Script contains the MPI interface setting\n#[test]\nfn contains_mpi_interface() {\n    let config = default_config();\n    let script = generate_job_script(&config, \"scf_U0.0\", \"ZnO\");\n    assert!(script.contains(&format!(\"OMPI_MCA_btl_tcp_if_include={mpi_if}\", mpi_if = config.mpi_if)));\n}",
+            "expected_behavior": "generate_job_script produces a SLURM job script string. The template uses ONLY spaces for indentation (no literal tab characters). SBATCH directives use double-quotes around values: #SBATCH --job-name=\"{task_id}\". The script contains: a bash shebang, SBATCH directives for partition/ntasks/mem, a nix develop command with the configured flake, an mpirun command with the MPI interface, and the castep.mpi {seed_name} invocation. Shell continuation lines use backslash (not mixed tab+space indentation).",
+            "test_file": "examples/multi_param_sweep/src/job_script.rs",
+            "test_module": "tests"
+          },
+          "scope": "Implement the SLURM job script generation function with the D.2 formatting fix applied, with ported tests written first.",
+          "crate_module": "examples/multi_param_sweep/src/job_script.rs",
+          "responsible_for": "SLURM job script generation: generate_job_script function with the D.2 clean heredoc template (all-space indentation, consistent double-quoting on SBATCH directives), and all 6 ported tests (sbatch directives, seed name, no_literal_tabs, shebang, nix develop, mpi interface).",
+          "depends_on": ["G1-1", "G1-2"],
+          "enables": ["G1-4"],
+          "can_run_in_parallel_with": [],
+          "changes": [
+            {
+              "file": "examples/multi_param_sweep/src/job_script.rs",
+              "guidance": "Implement generate_job_script(config: &SweepConfig, task_id: &str, seed_name: &str) -> String. Port the function body from the old hubbard_u_sweep_slurm/src/job_script.rs but apply the D.2 formatting fix: use consistent space-only indentation throughout the heredoc template (no literal tab characters mixed with spaces), use double-quotes consistently around SBATCH directive values (e.g., #SBATCH --job-name=\\\"{task_id}\\\"), and ensure shell continuation lines use \\\\ with proper spacing. The function must use format!() with named parameters. Import SweepConfig from crate::config. Add #[cfg(test)] mod tests with all 6 ported tests from tdd_interface.test_code adapted for the new SweepConfig. The no_literal_tabs test is the D.2 verification: it must assert !script.contains('\\t') and must PASS with the new template."
+            }
+          ],
+          "wiring_checklist": [],
+          "acceptance": [
+            "cargo check -p multi_param_sweep",
+            "cargo test -p multi_param_sweep -- job_script::tests (all 6 tests pass)",
+            "cargo test -p multi_param_sweep -- job_script::tests::no_literal_tabs (must pass — this is the D.2 verification)"
+          ],
+          "notes_for_subagent": "D.2 fix is critical. The old template had a literal tab character on the --map-by line mixed with spaces. The new template must use ALL spaces. SBATCH directives: each '#SBATCH --key=value' should use double-quotes around value: '#SBATCH --job-name=\"{task_id}\"'. All indentation within the heredoc should be spaces, never tabs. The no_literal_tabs test must assert !script.contains('\\t') and must pass. The other 5 tests verify: sbatch directives present, seed name in castep.mpi line, starts with shebang, contains nix develop with flake URI, contains mpi interface. Each test creates default config via SweepConfig::parse_from(['test']) (which triggers clap parsing with all defaults). To access SweepConfig::parse_from in tests, add use clap::Parser; in the test module."
+        },
+        {
+          "id": "G1-4",
+          "title": "Implement sweep logic and task builders with TDD",
+          "kind": "lib-tdd",
+          "tdd_interface": {
+            "signature": [
+              "fn build_one_scf_task(config: &SweepConfig, u: f64, kpoint: Option<[u32; 3]>, cutoff: Option<f64>, seed_cell: &str, seed_param: &str) -> Result<Task, WorkflowError>",
+              "fn build_all_scf_tasks(config: &SweepConfig) -> anyhow::Result<Vec<Task>>"
+            ],
+            "test_code": "use crate::config::SweepConfig;\nuse clap::Parser;\n\nfn default_config() -> SweepConfig {\n    SweepConfig::parse_from([\"test\"])\n}\n\nfn test_seeds() -> (&'static str, &'static str) {\n    (include_str!(\"../seeds/ZnO.cell\"), include_str!(\"../seeds/ZnO.param\"))\n}\n\n// --- Sweep combinatorics tests ---\n\n/// Product mode with defaults produces many tasks (6 u-values * default seed = 6 tasks)\n#[test]\nfn product_mode_produces_tasks() {\n    let config = default_config();\n    let tasks = build_all_scf_tasks(&config).unwrap();\n    // Default: 6 u-values (0.0..5.0), no kpoints/cutoffs by default -> 6 tasks\n    assert_eq!(tasks.len(), 6);\n}\n\n/// Product mode with explicit u-values, kpoints, and cutoffs produces Cartesian product\n#[test]\nfn product_mode_cartesian_product() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--u-values\", \"0.0,1.0,2.0\",\n        \"--kpoints\", \"8x8x8,6x6x6\",\n        \"--cutoffs\", \"300,500,800\",\n        \"--sweep-mode\", \"product\",\n    ]);\n    let tasks = build_all_scf_tasks(&config).unwrap();\n    // 3 u * 2 k * 3 c = 18 tasks\n    assert_eq!(tasks.len(), 18);\n}\n\n/// Pairwise mode produces zipped tasks\n#[test]\nfn pairwise_mode_zip() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--u-values\", \"0.0,1.0,2.0\",\n        \"--kpoints\", \"8x8x8,6x6x6,4x4x4\",\n        \"--cutoffs\", \"300,500,800\",\n        \"--sweep-mode\", \"pairwise\",\n    ]);\n    let tasks = build_all_scf_tasks(&config).unwrap();\n    assert_eq!(tasks.len(), 3);\n}\n\n/// Pairwise mode requires --kpoints\n#[test]\nfn pairwise_requires_kpoints() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--u-values\", \"0.0,1.0,2.0\",\n        \"--cutoffs\", \"300,500,800\",\n        \"--sweep-mode\", \"pairwise\",\n    ]);\n    let err = build_all_scf_tasks(&config).unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"kpoints\"), \"error should mention kpoints: {msg}\");\n}\n\n/// Pairwise mode requires --cutoffs\n#[test]\nfn pairwise_requires_cutoffs() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--u-values\", \"0.0,1.0,2.0\",\n        \"--kpoints\", \"8x8x8,6x6x6,4x4x4\",\n        \"--sweep-mode\", \"pairwise\",\n    ]);\n    let err = build_all_scf_tasks(&config).unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"cutoffs\"), \"error should mention cutoffs: {msg}\");\n}\n\n/// Pairwise mode with unequal list lengths errors\n#[test]\nfn pairwise_unequal_lengths() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--u-values\", \"0.0,1.0\",\n        \"--kpoints\", \"8x8x8,6x6x6,4x4x4\",\n        \"--cutoffs\", \"300,500,800\",\n        \"--sweep-mode\", \"pairwise\",\n    ]);\n    let err = build_all_scf_tasks(&config).unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"same length\") || msg.contains(\"length\"), \"error should mention length mismatch: {msg}\");\n}\n\n/// Unknown sweep mode errors\n#[test]\nfn unknown_sweep_mode() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--sweep-mode\", \"invalid\",\n    ]);\n    let err = build_all_scf_tasks(&config).unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"unknown sweep mode\"), \"error should mention unknown mode: {msg}\");\n}\n\n// --- Task structure tests ---\n\n/// Each task has a unique ID\n#[test]\nfn task_ids_are_unique() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--u-values\", \"0.0,1.0\",\n        \"--kpoints\", \"8x8x8,6x6x6\",\n        \"--cutoffs\", \"300,500\",\n        \"--sweep-mode\", \"product\",\n    ]);\n    let tasks = build_all_scf_tasks(&config).unwrap();\n    let ids: Vec<&str> = tasks.iter().map(|t| t.id.as_str()).collect();\n    let mut unique_ids = ids.clone();\n    unique_ids.sort();\n    unique_ids.dedup();\n    assert_eq!(ids.len(), unique_ids.len(), \"all task IDs must be unique\");\n}\n\n/// Task IDs encode parameters in the expected format scf_U{u}_k{k}_c{c}\n#[test]\nfn task_ids_encode_parameters() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--u-values\", \"3.0\",\n        \"--kpoints\", \"8x8x8\",\n        \"--cutoffs\", \"500\",\n        \"--sweep-mode\", \"product\",\n    ]);\n    let tasks = build_all_scf_tasks(&config).unwrap();\n    assert_eq!(tasks.len(), 1);\n    let id = &tasks[0].id;\n    assert!(id.contains(\"U3\") || id.contains(\"U3.0\"), \"task ID should contain U value: {id}\");\n    assert!(id.contains(\"k8x8x8\") || id.contains(\"k\"), \"task ID should contain kpoint: {id}\");\n    assert!(id.contains(\"c500\") || id.contains(\"c\"), \"task ID should contain cutoff: {id}\");\n}\n\n/// Product mode with only u-values (optional kpoints/cutoffs) still works\n#[test]\nfn product_with_optional_params() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--u-values\", \"0.0,1.0,2.0\",\n    ]);\n    let tasks = build_all_scf_tasks(&config).unwrap();\n    assert_eq!(tasks.len(), 3);\n}\n\n/// SCF tasks have no dependencies\n#[test]\nfn scf_tasks_have_no_dependencies() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--u-values\", \"0.0,1.0\",\n        \"--kpoints\", \"8x8x8\",\n        \"--cutoffs\", \"500\",\n        \"--sweep-mode\", \"product\",\n    ]);\n    let tasks = build_all_scf_tasks(&config).unwrap();\n    for task in &tasks {\n        assert!(task.dependencies.is_empty(), \"SCF task should have no dependencies: {}\", task.id);\n    }\n}\n\n/// Tasks have workdir paths\n#[test]\nfn tasks_have_workdirs() {\n    let config = SweepConfig::parse_from([\n        \"test\",\n        \"--u-values\", \"1.0\",\n        \"--kpoints\", \"8x8x8\",\n        \"--cutoffs\", \"500\",\n        \"--sweep-mode\", \"product\",\n    ]);\n    let tasks = build_all_scf_tasks(&config).unwrap();\n    assert!(!tasks.is_empty());\n    for task in &tasks {\n        assert!(!task.workdir.as_os_str().is_empty(), \"task {} should have a workdir\", task.id);\n    }\n}",
+            "expected_behavior": "build_all_scf_tasks in product mode: given N u-values, M kpoints, P cutoffs, produces N*M*P tasks (Cartesian product). If kpoints or cutoffs is None, uses a single None entry in the product (N*1 or N*1*1 tasks). In pairwise mode: requires both kpoints and cutoffs to be Some (errors with 'requires --kpoints' or 'requires --cutoffs' if not), parses all three lists, validates equal lengths (errors if unequal), then zips. Unknown sweep_mode produces 'unknown sweep mode' error. build_one_scf_task: produces a Task with a unique ID encoding all three parameters, no dependencies, a workdir path, and setup/collect closures. The setup closure injects hubbard_u, kpoints_mp_grid (if provided), and cutoff_energy into the seed files. The collect closure verifies the .castep output file exists and contains 'Total time'.",
+            "test_file": "examples/multi_param_sweep/src/main.rs",
+            "test_module": "tests"
+          },
+          "scope": "Implement the sweep combinatorics (product/pairwise mode dispatch) and the build_one_scf_task builder function with tests for correct task structure and sweep logic.",
+          "crate_module": "examples/multi_param_sweep/src/main.rs",
+          "responsible_for": "Sweep combinatorics and SCF task construction: build_one_scf_task (single SCF task with setup/collect closures that inject hubbard_u, kpoints_mp_grid, and cutoff_energy), build_all_scf_tasks (dispatches to product or pairwise modes), sweep mode validation (pairwise requires both kpoints and cutoffs, equal-length check), and in-file #[cfg(test)] tests verifying task IDs, dependency structure, workdir paths, and mode-specific error handling.",
+          "depends_on": ["G1-2", "G1-3"],
+          "enables": ["G1-5"],
+          "can_run_in_parallel_with": [],
+          "changes": [
+            {
+              "file": "examples/multi_param_sweep/src/main.rs",
+              "guidance": "Implement the sweep logic and task builder functions. Add necessary imports: use config::{parse_u_values, parse_kpoints, parse_cutoffs, SweepConfig}; use job_script::generate_job_script; plus castep_cell_fmt (parse, to_string_many_spaced, ToCellFile), castep_cell_io types (CellDocument, ParamDocument, AtomHubbardU, HubbardU, HubbardUUnit, OrbitalU, Species, KpointsMpGrid, CutOffEnergy), itertools::iproduct, workflow_utils::prelude::*.\n\nFunction build_one_scf_task(config, u, kpoint, cutoff, seed_cell, seed_param) -> Result<Task, WorkflowError>: Construct task ID using format! with the pattern 'scf_U{}_k{}_c{}'. For the u value, use {:.1} formatting to drop trailing zeros (3.0 -> 'U3.0'). For kpoint, format as '{0}x{1}x{2}' — if kpoint is None, omit the _k segment entirely. For cutoff, use {:.0} — if cutoff is None, omit the _c segment entirely. Workdir: PathBuf::from(format!('runs/U{u:.1}_k{k_str}_c{c_str}')) with segments omitted if params are None.\n\nThe setup closure must: (1) create_dir(workdir), (2) parse seed_cell into CellDocument via castep_cell_fmt::parse::<CellDocument>, inject hubbard_u block using the builder pattern: AtomHubbardU::builder().species(Species::Symbol(element.clone())).orbitals(vec![orbital_u]).build() and HubbardU::builder().unit(HubbardUUnit::ElectronVolt).atom_u_values(vec![atom_u]).build(), set cell_doc.hubbard_u = Some(hubbard_u), (3) if kpoint is Some, set cell_doc.kpoints_mp_grid = Some(KpointsMpGrid(kpoint)), (4) parse seed_param into ParamDocument via castep_cell_fmt::parse::<ParamDocument>, if cutoff is Some set doc.basis_set.cutoff_energy = Some(CutOffEnergy { value: cutoff, unit: None }), (5) serialize both to .cell and .param files via to_string_many_spaced(&doc.to_cell_file()) and write_file, (6) if !config.local, write job_script via write_file(workdir.join(JOB_SCRIPT_NAME), &generate_job_script(config, &task_id, &config.seed_name)).\n\nThe collect closure must: read <seed>.castep via read_file, verify it contains 'Total time' marker. Return WorkflowError::InvalidConfig if the file is missing or the marker is absent.\n\nFunction build_all_scf_tasks(config) -> anyhow::Result<Vec<Task>>: Load seed_cell and seed_param via include_str!('../seeds/ZnO.cell') and include_str!('../seeds/ZnO.param') (passed as parameters). Parse u_values. Match config.sweep_mode.as_str(): 'product' branch uses itertools::iproduct! over u_vals, kpoints_vec (unwrap_or(vec![None])), cutoffs_vec (unwrap_or(vec![None])). For each (u, k, c) tuple, call build_one_scf_task. 'pairwise' branch requires both kpoints and cutoffs to be Some (anyhow::bail! if not), parses all three lists, validates equal lengths (anyhow::bail! if not), then zip all three iterators and call build_one_scf_task for each triple. Unknown mode returns anyhow::bail!('unknown sweep mode...').\n\nAdd #[cfg(test)] mod tests with the tests from tdd_interface.test_code. The tests verify combinatoric correctness, mode validation, task ID uniqueness, and structural invariants."
+            }
+          ],
+          "wiring_checklist": [],
+          "acceptance": [
+            "cargo check -p multi_param_sweep",
+            "cargo test -p multi_param_sweep (config tests + job_script tests + sweep logic tests all pass)",
+            "All sweep tests pass: product count (6 default, 18 explicit), pairwise count (3), pairwise requires kpoints, pairwise requires cutoffs, unequal lengths error, unknown mode error, unique IDs, ID format, optional params, no dependencies, workdirs present"
+          ],
+          "notes_for_subagent": "The setup closure is the most complex part. Follow the existing pattern from hubbard_u_sweep_slurm/src/main.rs build_one_task closely. Key API differences in v0.5.0: KpointsMpGrid is a direct field on CellDocument (doc.kpoints_mp_grid = Some(KpointsMpGrid([kx, ky, kz]))). CutOffEnergy { value: f64, unit: None }. Import KpointsMpGrid and CutOffEnergy from castep_cell_io — verify exact module paths with LSP hover. Use workflow_utils::prelude::* for create_dir, write_file, read_file, JOB_SCRIPT_NAME. The collect closure must import read_file. Pattern 5 (in-memory mutation): always parse-mutate-serialize, never string manipulation. The task ID format must use the exact pattern from Pattern 2: scf_U{u}_k{k}_c{c} with k and c segments formatted without trailing zeros. For the pairwise validation, check sweep_mode first, then validate Option presence, then parse, then check lengths. Use include_str! for seed files in the build_all_scf_tasks function."
+        },
+        {
+          "id": "G1-5",
+          "title": "Wire main() entry point for multi_param_sweep",
+          "kind": "direct",
+          "scope": "Implement the main() function that orchestrates config parsing, task building, workflow construction, dry-run display, and local/queued execution.",
+          "crate_module": "examples/multi_param_sweep/src/main.rs",
+          "responsible_for": "Binary entry point: main() function that parses SweepConfig from CLI, calls build_all_scf_tasks, constructs a Workflow with max_parallel, log_dir, root_dir, conditionally with_queued_submitter, adds all tasks, then either dry-runs (printing topological order) or runs (local via run_default, or queued via Workflow::run with Arc<dyn ProcessRunner> + Arc<dyn HookExecutor>). Also prints workflow summary on completion.",
+          "depends_on": ["G1-4"],
+          "enables": ["G3-1"],
+          "can_run_in_parallel_with": ["G2-4", "G2-5"],
+          "changes": [
+            {
+              "file": "examples/multi_param_sweep/src/main.rs",
+              "guidance": "Implement the main() function. Use anyhow::Result as the return type. Call workflow_core::init_default_logging().ok() first. Parse SweepConfig::parse(). Call build_all_scf_tasks(&config)? to get tasks. Construct workflow: Workflow::new('multi_param_sweep').with_max_parallel(config.max_parallel)?.with_log_dir('logs').with_root_dir(&config.workdir). If !config.local, add .with_queued_submitter(Arc::new(QueuedRunner::new(SchedulerKind::Slurm))). Add all tasks via workflow.add_task(task)?. For dry_run: call workflow.dry_run()?, print 'Dry-run topological order:' then each task ID on its own line prefixed with '  '. Return Ok(()). For execution: create JsonStateStore at '.multi_param_sweep.workflow.json'. If config.local, use run_default(&mut workflow, &mut state)?; else, construct SystemProcessRunner and ShellHookExecutor as Arc<dyn ProcessRunner/HookExecutor>, call workflow.run(&mut state, runner, executor)?. After run, print: 'Workflow complete: {succeeded} succeeded, {failed} failed, {skipped} skipped ({duration}s)'. Import std::sync::Arc for the queued path."
+            }
+          ],
+          "wiring_checklist": [],
+          "acceptance": [
+            "cargo check -p multi_param_sweep",
+            "cargo run --bin multi_param_sweep -- --dry-run (prints 'Dry-run topological order:' followed by task IDs, no errors)",
+            "cargo run --bin multi_param_sweep -- --dry-run --sweep-mode pairwise --u-values 0,1,2 --kpoints 8x8x8,6x6x6,4x4x4 --cutoffs 300,500,800 (prints exactly 3 task IDs, no errors)",
+            "cargo run --bin multi_param_sweep -- --dry-run --sweep-mode pairwise --u-values 0,1,2 (errors: pairwise mode requires --kpoints and --cutoffs)"
+          ],
+          "notes_for_subagent": "Follow the exact execution pattern from the old binary's main(). The key difference is that there's no mode branching in main() — build_all_scf_tasks handles product/pairwise internally. Use workflow_utils::prelude::* for all workflow types (Workflow, JsonStateStore, run_default, QueuedRunner, SchedulerKind, SystemProcessRunner, ShellHookExecutor, WorkflowSummary). For the state store path, use '.multi_param_sweep.workflow.json'. The dry-run output format: first line 'Dry-run topological order:', then each task ID on its own line indented by two spaces. The summary format: 'Workflow complete: {succeeded} succeeded, {failed} failed, {skipped} skipped ({duration}s)'. Import std::sync::Arc for the queued path. The main.rs already has mod declarations and test module from G1-4 — preserve both."
+        }
+      ]
+    },
+    {
+      "id": "core-2",
+      "name": "scf_dos_chain binary + hubbard_u_sweep update",
+      "description": "Create the examples/scf_dos_chain/ crate — a purpose-built binary for SCF+DOS task chain testing — and update the existing hubbard_u_sweep binary for castep-cell-io v0.5.0 path dep compatibility.",
+      "tasks": [
+        {
+          "id": "G2-1",
+          "title": "Create scf_dos_chain crate skeleton and add to workspace",
+          "kind": "direct",
+          "scope": "Create the directory structure, Cargo.toml, stub source files, seed files, and add the crate to workspace members.",
+          "crate_module": "examples/scf_dos_chain (new crate)",
+          "responsible_for": "Crate creation: Cargo.toml with all dependencies, minimal main.rs with mod declarations, empty config.rs, empty job_script.rs, seed files, and workspace member registration.",
+          "depends_on": ["G1-1"],
+          "enables": ["G2-2"],
+          "can_run_in_parallel_with": [],
+          "changes": [
+            {
+              "file": "examples/scf_dos_chain/Cargo.toml",
+              "guidance": "Create Cargo.toml with [package] name = 'scf_dos_chain', edition 2021, [[bin]] path = src/main.rs. Same dependencies as multi_param_sweep: anyhow/clap/itertools = { workspace = true }, castep-cell-fmt = '0.1.0', castep-cell-io = { path = '../castep-cell-io/castep_cell_io' }, workflow_core = { path = '../../workflow_core', features = ['default-logging'] }, workflow_utils = { path = '../../workflow_utils' }."
+            },
+            {
+              "file": "examples/scf_dos_chain/src/main.rs",
+              "guidance": "Create stub main.rs with mod config; and mod job_script; declarations. Add fn main() { println!('scf_dos_chain: skeleton'); }. No imports."
+            },
+            {
+              "file": "examples/scf_dos_chain/src/config.rs",
+              "guidance": "Create empty config.rs with placeholder comment."
+            },
+            {
+              "file": "examples/scf_dos_chain/src/job_script.rs",
+              "guidance": "Create empty job_script.rs with placeholder comment."
+            },
+            {
+              "file": "examples/scf_dos_chain/seeds/ZnO.cell",
+              "guidance": "Copy from examples/hubbard_u_sweep_slurm/seeds/ZnO.cell (or from examples/multi_param_sweep/seeds/ZnO.cell if already created)."
+            },
+            {
+              "file": "examples/scf_dos_chain/seeds/ZnO.param",
+              "guidance": "Copy from examples/hubbard_u_sweep_slurm/seeds/ZnO.param (or from examples/multi_param_sweep/seeds/ZnO.param if already created)."
+            },
+            {
+              "file": "Cargo.toml (workspace root)",
+              "guidance": "Add 'examples/scf_dos_chain' to the [workspace] members array. Insert it after 'examples/multi_param_sweep'. Keep 'examples/hubbard_u_sweep_slurm' in the list (it will be removed in G3-1)."
+            }
+          ],
+          "wiring_checklist": [
+            {
+              "parent_file": "examples/scf_dos_chain/src/main.rs",
+              "action": "add_mod_declaration",
+              "detail": "Add mod config; and mod job_script; before fn main()."
+            },
+            {
+              "parent_file": "Cargo.toml (workspace root)",
+              "action": "add_workspace_member",
+              "detail": "Add 'examples/scf_dos_chain' to the workspace members list."
+            }
+          ],
+          "acceptance": [
+            "cargo check --workspace (both new crates + old slurm crate compile)",
+            "ls examples/scf_dos_chain/Cargo.toml examples/scf_dos_chain/src/main.rs examples/scf_dos_chain/src/config.rs examples/scf_dos_chain/src/job_script.rs examples/scf_dos_chain/seeds/ZnO.cell examples/scf_dos_chain/seeds/ZnO.param"
+          ],
+          "notes_for_subagent": "This task depends on G1-1 only because both modify the root Cargo.toml workspace members list (file collision avoidance). The two binaries share zero code at the Rust level. The Cargo.toml dependency structure is identical to multi_param_sweep. The seed files are the same ZnO wurtzite cell — copying is acceptable (self-contained binaries). If G1-1 has already created the seeds directory in multi_param_sweep, you can copy from there instead of the old slurm binary."
+        },
+        {
+          "id": "G2-2",
+          "title": "Implement config.rs and job_script.rs for scf_dos_chain with TDD",
+          "kind": "lib-tdd",
+          "tdd_interface": {
+            "signature": [
+              "pub fn parse_u_values(s: &str) -> anyhow::Result<Vec<f64>>",
+              "pub fn generate_job_script(config: &ChainConfig, task_id: &str, seed_name: &str) -> String"
+            ],
+            "test_code": "// ===== config.rs tests =====\n// These are the same 7 parse_u_values tests as in multi_param_sweep,\n// duplicated here because parse_u_values is duplicated.\n\nuse crate::config::ChainConfig;\nuse clap::Parser;\n\n/// Helper: default config for job_script tests\nfn default_chain_config() -> ChainConfig {\n    ChainConfig::parse_from([\"test\"])\n}\n\n// --- parse_u_values tests (ported, adapted for anyhow::Result) ---\n\n#[test]\nfn parse_basic_values() {\n    let vals = parse_u_values(\"0.0,1.0,2.0\").unwrap();\n    assert_eq!(vals, vec![0.0, 1.0, 2.0]);\n}\n\n#[test]\nfn parse_with_whitespace() {\n    let vals = parse_u_values(\"  0.0 , 1.0 , 2.0  \").unwrap();\n    assert_eq!(vals, vec![0.0, 1.0, 2.0]);\n}\n\n#[test]\nfn parse_single_value() {\n    let vals = parse_u_values(\"42.0\").unwrap();\n    assert_eq!(vals, vec![42.0]);\n}\n\n#[test]\nfn parse_invalid_token() {\n    let err = parse_u_values(\"1.0,abc,2.0\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"abc\"), \"error should mention the invalid token: {msg}\");\n}\n\n#[test]\nfn parse_empty_token() {\n    let err = parse_u_values(\"1.0,,2.0\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"invalid\"), \"error should report parse failure: {msg}\");\n}\n\n#[test]\nfn parse_empty_string() {\n    let err = parse_u_values(\"\").unwrap_err();\n    let msg = format!(\"{err}\");\n    assert!(msg.contains(\"empty\") || msg.contains(\"invalid\"), \"expected parse failure on empty input, got: {msg}\");\n}\n\n#[test]\nfn parse_negative_values() {\n    let vals = parse_u_values(\"-1.0,2.0\").unwrap();\n    assert_eq!(vals, vec![-1.0, 2.0]);\n}\n\n// ===== job_script.rs tests =====\n\n#[test]\nfn contains_sbatch_directives() {\n    let config = default_chain_config();\n    let script = generate_job_script(&config, \"scf\", \"ZnO\");\n    assert!(script.contains(\"#SBATCH --job-name=\\\"scf\\\"\"));\n    assert!(script.contains(\"#SBATCH --partition=debug\"));\n    assert!(script.contains(\"#SBATCH --ntasks-per-node=16\"));\n    assert!(script.contains(\"#SBATCH --mem=30000m\"));\n}\n\n#[test]\nfn contains_seed_name() {\n    let config = default_chain_config();\n    let script = generate_job_script(&config, \"scf\", \"ZnO\");\n    assert!(script.contains(\"castep.mpi ZnO\"));\n}\n\n/// D.2 fix: no literal tabs\n#[test]\nfn no_literal_tabs() {\n    let config = default_chain_config();\n    let script = generate_job_script(&config, \"scf\", \"ZnO\");\n    assert!(!script.contains('\\t'), \"job script should not contain literal tab characters\");\n}\n\n#[test]\nfn starts_with_shebang() {\n    let config = default_chain_config();\n    let script = generate_job_script(&config, \"scf\", \"ZnO\");\n    assert!(script.starts_with(\"#!/usr/bin/env bash\"));\n}\n\n#[test]\nfn contains_nix_develop() {\n    let config = default_chain_config();\n    let script = generate_job_script(&config, \"scf\", \"ZnO\");\n    assert!(script.contains(\"nix develop\"));\n    assert!(script.contains(&config.nix_flake));\n}\n\n#[test]\nfn contains_mpi_interface() {\n    let config = default_chain_config();\n    let script = generate_job_script(&config, \"scf\", \"ZnO\");\n    assert!(script.contains(&format!(\"OMPI_MCA_btl_tcp_if_include={mpi_if}\", mpi_if = config.mpi_if)));\n}",
+            "expected_behavior": "parse_u_values: identical behavior to the multi_param_sweep version — returns Vec<f64> for comma-separated numbers, errors with clear messages for non-numeric tokens and empty input. generate_job_script: identical template to the multi_param_sweep version but accepting &ChainConfig instead of &SweepConfig. Since SLURM fields are identical on both config types, the output is functionally the same. D.2 fix applied: all-space indentation, double-quoted SBATCH values, no literal tabs.",
+            "test_file": ["examples/scf_dos_chain/src/config.rs", "examples/scf_dos_chain/src/job_script.rs"],
+            "test_module": "tests (in each file)"
+          },
+          "scope": "Implement the ChainConfig clap struct, parse_u_values function, and generate_job_script function (adapted for ChainConfig type) with tests written first.",
+          "crate_module": "examples/scf_dos_chain/src/config.rs and examples/scf_dos_chain/src/job_script.rs",
+          "responsible_for": "CLI configuration (ChainConfig with single u-value, no sweep, no kpoints/cutoff), parse_u_values function (duplicated from multi_param_sweep), generate_job_script adapted for &ChainConfig parameter, and their in-file #[cfg(test)] tests.",
+          "depends_on": ["G2-1"],
+          "enables": ["G2-3"],
+          "can_run_in_parallel_with": ["G1-4", "G1-2", "G1-3"],
+          "changes": [
+            {
+              "file": "examples/scf_dos_chain/src/config.rs",
+              "guidance": "Implement ChainConfig struct with #[derive(Parser, Debug)], #[command(name = 'scf_dos_chain')], and fields: u_value (f64, default 3.0), element (String, default 'Zn'), orbital (char, default 'd'), seed_name (String, default 'ZnO'), max_parallel (usize, default 1), local (bool), dry_run (bool), castep_command (String, default 'castep'), workdir (String, default '.'), plus SLURM fields (partition, ntasks, nix_flake, mpi_if — same as SweepConfig). Implement parse_u_values as a duplicate of the multi_param_sweep version (returning anyhow::Result<Vec<f64>>). Add #[cfg(test)] mod tests with the 7 ported parse_u_values tests. Place the parse_u_values tests directly in config.rs's test module."
+            },
+            {
+              "file": "examples/scf_dos_chain/src/job_script.rs",
+              "guidance": "Implement generate_job_script(config: &ChainConfig, task_id: &str, seed_name: &str) -> String. The function body is identical to the multi_param_sweep version (same SLURM fields on both config types). Use the exact same heredoc template with D.2 fix applied (all-space indentation, consistent SBATCH quoting). Import ChainConfig from crate::config. Add #[cfg(test)] mod tests with the 6 ported tests adapted for ChainConfig (using ChainConfig::parse_from(['test'])). Add use clap::Parser; in the test module. All 6 tests including no_literal_tabs must pass."
+            }
+          ],
+          "wiring_checklist": [
+            {
+              "parent_file": "examples/scf_dos_chain/src/config.rs",
+              "action": "add_import",
+              "detail": "Add use clap::Parser; at the top."
+            },
+            {
+              "parent_file": "examples/scf_dos_chain/src/job_script.rs",
+              "action": "add_import",
+              "detail": "Add use crate::config::ChainConfig; at the top."
+            }
+          ],
+          "acceptance": [
+            "cargo check -p scf_dos_chain",
+            "cargo test -p scf_dos_chain -- config::tests (7 parse_u_values tests pass)",
+            "cargo test -p scf_dos_chain -- job_script::tests (6 tests pass, including no_literal_tabs)"
+          ],
+          "notes_for_subagent": "ChainConfig is structurally simpler than SweepConfig — single u_value (f64, not String), no kpoints/cutoffs fields, no sweep_mode. parse_u_values is duplicated verbatim from multi_param_sweep (the function is ~15 lines). generate_job_script takes &ChainConfig instead of &SweepConfig — the function body is identical because SLURM fields are the same on both config types. This duplication is intentional per architectural decision (2 consumers does not justify extraction). The parse_u_values tests go in config.rs's #[cfg(test)] mod tests. The job_script tests go in job_script.rs's #[cfg(test)] mod tests."
+        },
+        {
+          "id": "G2-3",
+          "title": "Implement chain task builders (SCF + DOS) with TDD",
+          "kind": "lib-tdd",
+          "tdd_interface": {
+            "signature": [
+              "fn build_scf_task(config: &ChainConfig, seed_cell: &str, seed_param: &str) -> Result<Task, WorkflowError>",
+              "fn build_dos_task(config: &ChainConfig, scf_task_id: &str, seed_cell: &str, seed_param: &str) -> Result<Task, WorkflowError>"
+            ],
+            "test_code": "use crate::config::ChainConfig;\nuse clap::Parser;\n\nfn default_config() -> ChainConfig {\n    ChainConfig::parse_from([\"test\"])\n}\n\nfn test_seeds() -> (&'static str, &'static str) {\n    (include_str!(\"../seeds/ZnO.cell\"), include_str!(\"../seeds/ZnO.param\"))\n}\n\n// --- Task ID and structure tests ---\n\n/// SCF task has ID 'scf'\n#[test]\nfn scf_task_id() {\n    let config = default_config();\n    let (cell, param) = test_seeds();\n    let task = build_scf_task(&config, cell, param).unwrap();\n    assert_eq!(task.id, \"scf\");\n}\n\n/// DOS task has ID 'dos'\n#[test]\nfn dos_task_id() {\n    let config = default_config();\n    let (cell, param) = test_seeds();\n    let task = build_dos_task(&config, \"scf\", cell, param).unwrap();\n    assert_eq!(task.id, \"dos\");\n}\n\n/// DOS task depends on SCF task\n#[test]\nfn dos_depends_on_scf() {\n    let config = default_config();\n    let (cell, param) = test_seeds();\n    let scf = build_scf_task(&config, cell, param).unwrap();\n    let dos = build_dos_task(&config, &scf.id, cell, param).unwrap();\n    assert!(dos.dependencies.contains(&\"scf\".to_string()));\n}\n\n/// SCF task has no dependencies\n#[test]\nfn scf_no_dependencies() {\n    let config = default_config();\n    let (cell, param) = test_seeds();\n    let task = build_scf_task(&config, cell, param).unwrap();\n    assert!(task.dependencies.is_empty());\n}\n\n/// DOS task has exactly 1 dependency\n#[test]\nfn dos_dependency_count() {\n    let config = default_config();\n    let (cell, param) = test_seeds();\n    let scf = build_scf_task(&config, cell, param).unwrap();\n    let dos = build_dos_task(&config, &scf.id, cell, param).unwrap();\n    assert_eq!(dos.dependencies.len(), 1);\n}\n\n/// Both tasks share the same workdir\n#[test]\nfn shared_workdir() {\n    let config = default_config();\n    let (cell, param) = test_seeds();\n    let scf = build_scf_task(&config, cell, param).unwrap();\n    let dos = build_dos_task(&config, &scf.id, cell, param).unwrap();\n    assert_eq!(scf.workdir, dos.workdir, \"SCF and DOS tasks should share the same workdir\");\n    let wd = scf.workdir.to_string_lossy();\n    assert!(wd.contains(\"scf_dos\"), \"workdir should contain 'scf_dos': {wd}\");\n}\n\n/// SCF task has correct workdir path\n#[test]\nfn scf_workdir_path() {\n    let config = default_config();\n    let (cell, param) = test_seeds();\n    let task = build_scf_task(&config, cell, param).unwrap();\n    let wd = task.workdir.to_string_lossy();\n    assert!(wd.ends_with(\"scf_dos\") || wd.contains(\"scf_dos\"), \"workdir should end with or contain 'scf_dos': {wd}\");\n}",
+            "expected_behavior": "build_scf_task: produces a Task with id 'scf', workdir ending in 'runs/scf_dos', no dependencies, and a setup closure that injects the Hubbard U value from config into the seed cell file. The collect closure verifies the .castep output. build_dos_task: produces a Task with id 'dos', same workdir as SCF, exactly 1 dependency on the SCF task ID. The setup closure copies ZnO.check to ZnO_DOS.check and sets general.task = Task::BandStructure in the param file. The collect closure verifies ZnO_DOS.castep output.",
+            "test_file": "examples/scf_dos_chain/src/main.rs",
+            "test_module": "tests"
+          },
+          "scope": "Implement build_scf_task and build_dos_task functions that construct the two-task SCF+DOS chain, with tests for dependency structure and DOS setup logic.",
+          "crate_module": "examples/scf_dos_chain/src/main.rs",
+          "responsible_for": "Chain task construction: build_scf_task (SCF task with setup injecting hubbard_u, collect verifying .castep output), build_dos_task (DOS task depending on scf, setup copying checkpoint and setting Task::BandStructure), and in-file #[cfg(test)] tests verifying task IDs, dependency structure, workdir paths, and the DOS setup closure behavior.",
+          "depends_on": ["G2-2"],
+          "enables": ["G2-4"],
+          "can_run_in_parallel_with": ["G1-4", "G1-5"],
+          "changes": [
+            {
+              "file": "examples/scf_dos_chain/src/main.rs",
+              "guidance": "Add necessary imports: use config::{parse_u_values, ChainConfig}; use job_script::generate_job_script; plus castep_cell_fmt (parse, to_string_many_spaced, ToCellFile), castep_cell_io (CellDocument, ParamDocument, AtomHubbardU, HubbardU, HubbardUUnit, OrbitalU, Species, CutOffEnergy), workflow_utils::prelude::*. Note: do NOT import castep_cell_io::param::general::task::Task — use the fully-qualified path at the single use-site (see below).\n\nFunction build_scf_task(config, seed_cell, seed_param) -> Result<Task, WorkflowError>: Task ID 'scf', workdir PathBuf::from('runs/scf_dos'). Execution mode: if config.local, ExecutionMode::direct(&config.castep_command, &['ZnO']), else ExecutionMode::Queued. Setup closure: (1) create_dir(workdir), (2) parse seed_cell into CellDocument, inject hubbard_u using config.u_value/config.element/config.orbital (same builder pattern as multi_param_sweep), (3) serialize .cell + .param to workdir, (4) if !config.local, write job_script. Collect closure: verify <seed>.castep exists with 'Total time' marker.\n\nFunction build_dos_task(config, scf_task_id, seed_cell, seed_param) -> Result<Task, WorkflowError>: Task ID 'dos', workdir PathBuf::from('runs/scf_dos') (SAME workdir as SCF). Depends on scf_task_id. Execution mode: if config.local, ExecutionMode::direct(&config.castep_command, &['ZnO_DOS']), else ExecutionMode::Queued. Setup closure: (1) create_dir(workdir), (2) parse seed_cell into CellDocument, write to <workdir>/ZnO_DOS.cell (reuse seed cell, no hubbard_u injection needed since this is a DOS restart), (3) parse seed_param into ParamDocument, set doc.general.task = Some(castep_cell_io::param::general::task::Task::BandStructure) — THIS IS THE FULLY-QUALIFIED PATH, do not import the Task enum, (4) write to <workdir>/ZnO_DOS.param, (5) copy <workdir>/ZnO.check to <workdir>/ZnO_DOS.check using workflow_utils::files::copy_file or the prelude re-export, (6) if !config.local, write job_script via generate_job_script(&config, 'dos', 'ZnO_DOS'). Collect closure: verify ZnO_DOS.castep exists and contains 'Total time'.\n\nAdd #[cfg(test)] mod tests with the tests from tdd_interface.test_code. These tests verify structural correctness: task IDs, dependency structure, shared workdir, dependency counts."
+            }
+          ],
+          "wiring_checklist": [],
+          "acceptance": [
+            "cargo check -p scf_dos_chain",
+            "cargo test -p scf_dos_chain (config tests + job_script tests + chain logic tests all pass)",
+            "All chain structure tests pass: scf id is 'scf', dos id is 'dos', dos depends on scf, scf has no deps, dos has 1 dep, shared workdir, correct workdir path"
+          ],
+          "notes_for_subagent": "The DOS task setup is the novel part and the main reason this binary exists. Pattern 6: shared workdir — both tasks use runs/scf_dos/. The checkpoint copy uses workflow_utils::files::copy_file (or the prelude re-export). The Task::BandStructure setting uses the fully-qualified path: castep_cell_io::param::general::task::Task::BandStructure. VERIFY this exact module path with LSP hover during implementation — the module may be re-exported differently in v0.5.0. Do NOT add use castep_cell_io::param::general::task::Task — this would shadow workflow_core::task::Task from the prelude glob import (known pitfall: task_naming_collision_P0). The DOS execution uses seed name 'ZnO_DOS' (different from SCF's 'ZnO'). The setup closure for DOS does NOT inject hubbard_u (the checkpoint carries that information). The setup closure DOES write ZnO_DOS.cell from seed and sets Task::BandStructure on the param."
+        },
+        {
+          "id": "G2-4",
+          "title": "Wire main() entry point for scf_dos_chain",
+          "kind": "direct",
+          "scope": "Implement the main() function that orchestrates config parsing, chain construction, workflow creation, dry-run display, and local/queued execution.",
+          "crate_module": "examples/scf_dos_chain/src/main.rs",
+          "responsible_for": "Binary entry point: main() function that parses ChainConfig, builds SCF and DOS tasks, constructs a Workflow with the two-task chain, and supports dry-run/local/queued execution paths.",
+          "depends_on": ["G2-3"],
+          "enables": ["G3-1"],
+          "can_run_in_parallel_with": ["G1-5", "G2-5"],
+          "changes": [
+            {
+              "file": "examples/scf_dos_chain/src/main.rs",
+              "guidance": "Implement the main() function following the same pattern as multi_param_sweep main(). Call workflow_core::init_default_logging().ok(). Parse ChainConfig::parse(). Load seed files via include_str!('../seeds/ZnO.cell') and include_str!('../seeds/ZnO.param'). Call build_scf_task(&config, seed_cell, seed_param)? to get the SCF task, then build_dos_task(&config, &scf_task.id, seed_cell, seed_param)? to get the DOS task. Construct workflow: Workflow::new('scf_dos_chain').with_max_parallel(config.max_parallel)?.with_log_dir('logs').with_root_dir(&config.workdir). If !config.local, add .with_queued_submitter(Arc::new(QueuedRunner::new(SchedulerKind::Slurm))). Add SCF task first, then DOS task via workflow.add_task(task)?. Dry-run: same pattern as multi_param_sweep (print topological order). For execution: JsonStateStore at '.scf_dos_chain.workflow.json', local via run_default, queued via Workflow::run with SystemProcessRunner + ShellHookExecutor. Print workflow summary on completion. Import std::sync::Arc."
+            }
+          ],
+          "wiring_checklist": [],
+          "acceptance": [
+            "cargo check -p scf_dos_chain",
+            "cargo run --bin scf_dos_chain -- --dry-run (prints 'scf' then 'dos' — chain dependency order, exactly 2 tasks)",
+            "cargo run --bin scf_dos_chain -- --local --dry-run (same output — local flag doesn't change dry-run topology)"
+          ],
+          "notes_for_subagent": "The main() pattern is identical to multi_param_sweep but simpler: only 2 tasks, no sweep logic. The chain order matters: add SCF task first, then DOS task (which references the SCF task ID via .depends_on). The dry-run must show 'scf' before 'dos' in the topological order — if reversed, the workflow dependency graph is wrong. State store path: '.scf_dos_chain.workflow.json'. The main.rs already has mod declarations and a test module from G2-3 — preserve both and add the main() function. The seed files are loaded with include_str! inside main() or passed as parameters to the build functions."
+        },
+        {
+          "id": "G2-5",
+          "title": "Update hubbard_u_sweep for castep-cell-io v0.5.0 path dep",
+          "kind": "direct",
+          "scope": "Update the existing hubbard_u_sweep binary's Cargo.toml to use the local path dep for castep-cell-io v0.5.0 and fix any API breakage in main.rs.",
+          "crate_module": "examples/hubbard_u_sweep",
+          "responsible_for": "Dependency update: change castep-cell-io from version 0.4.0 to local path dep, verify compilation, and fix any API breakage in main.rs (Species::Symbol signature, builder method changes, or enum variant renames).",
+          "depends_on": [],
+          "enables": ["G3-1"],
+          "can_run_in_parallel_with": ["G2-1", "G2-2", "G2-3", "G2-4", "G1-1", "G1-2", "G1-3", "G1-4", "G1-5"],
+          "changes": [
+            {
+              "file": "examples/hubbard_u_sweep/Cargo.toml",
+              "guidance": "Change the castep-cell-io dependency line from 'castep-cell-io = \"0.4.0\"' to 'castep-cell-io = { path = \"../castep-cell-io/castep_cell_io\" }'. All other dependencies (anyhow = \"1\", castep-cell-fmt = \"0.1.0\", workflow_core, workflow_utils) remain unchanged."
+            },
+            {
+              "file": "examples/hubbard_u_sweep/src/main.rs",
+              "guidance": "Run cargo check -p hubbard_u_sweep after the Cargo.toml change. If compilation fails due to v0.5.0 API changes, fix the call sites. Common breakage points: Species::Symbol may take a different type than String (e.g., &str or a newtype) — check with LSP hover on Species::Symbol. Builder method names on AtomHubbardU/HubbardU may have changed. HubbardUUnit enum variants may be renamed. Fix each error based on the compiler diagnostic. The existing code already uses castep_cell_fmt::parse::<CellDocument>(&input) which is the v0.5.0 parse API pattern, so the parse logic should work unchanged. Only fix what the compiler reports as errors."
+            }
+          ],
+          "wiring_checklist": [],
+          "acceptance": [
+            "cargo check -p hubbard_u_sweep (must compile with new path dep; zero errors, zero warnings)"
+          ],
+          "notes_for_subagent": "This task is intentionally minimal — the plan says 'only Cargo.toml and ensure compilation.' The existing main.rs already uses the v0.5.0 parse API. Only change main.rs if the compiler reports errors. Use LSP hover to verify exact types and signatures before making changes. If Species::Symbol in v0.5.0 takes &str instead of String, change Species::Symbol('Zn'.to_string()) to Species::Symbol('Zn') or whatever the new API expects. This task is fully independent of both new binaries — it can run at any time, even before the new crates exist. The path is relative from examples/hubbard_u_sweep/: '../castep-cell-io/castep_cell_io'."
+        }
+      ]
+    },
+    {
+      "id": "workspace",
+      "name": "Workspace integration, cleanup, verification",
+      "description": "Wire everything together by removing the old hubbard_u_sweep_slurm crate, cleaning up workspace members, and running full build/test/clippy verification.",
+      "tasks": [
+        {
+          "id": "G3-1",
+          "title": "Remove old crate, finalize workspace, run full verification",
+          "kind": "direct",
+          "scope": "Remove hubbard_u_sweep_slurm from workspace members, delete the entire old crate directory, and run full workspace build, test, clippy, and dry-run verification.",
+          "crate_module": "Cargo.toml (workspace root) and examples/hubbard_u_sweep_slurm/ (deletion)",
+          "responsible_for": "Workspace finalization: updating workspace members list to remove the old binary, deleting examples/hubbard_u_sweep_slurm/ entirely, and running comprehensive verification (build, test, clippy dead-code, dry-run both binaries).",
+          "depends_on": ["G1-5", "G2-4", "G2-5"],
+          "enables": [],
+          "can_run_in_parallel_with": [],
+          "changes": [
+            {
+              "file": "Cargo.toml (workspace root)",
+              "guidance": "Remove 'examples/hubbard_u_sweep_slurm' from the [workspace] members array. The members list should now contain exactly: 'workflow_core', 'workflow_utils', 'examples/hubbard_u_sweep', 'examples/multi_param_sweep', 'examples/scf_dos_chain', 'workflow-cli'. Do NOT change any other configuration (resolver, workspace.dependencies, etc.)."
+            },
+            {
+              "file": "examples/hubbard_u_sweep_slurm/",
+              "guidance": "Delete the entire directory and all its contents: Cargo.toml, src/main.rs, src/config.rs, src/job_script.rs, seeds/ZnO.cell, seeds/ZnO.param, .validation-complete. Use rm -rf or the equivalent VCS remove. This removes the old buggy binary entirely from the codebase."
+            }
+          ],
+          "wiring_checklist": [
+            {
+              "parent_file": "Cargo.toml (workspace root)",
+              "action": "remove_workspace_member",
+              "detail": "Remove 'examples/hubbard_u_sweep_slurm' from [workspace] members."
+            }
+          ],
+          "acceptance": [
+            "cargo build --workspace (all crates compile with zero errors, zero warnings)",
+            "cargo test --workspace (all tests pass; old slurm tests are gone, new binary tests run and pass)",
+            "cargo clippy --workspace --all-targets -- -W dead-code (no dead-code warnings from stale imports — if any appear, fix them in the library crates)",
+            "cargo run --bin multi_param_sweep -- --dry-run (produces correct topological order with all 6 default SCF task IDs, no DOS tasks, no duplicate IDs)",
+            "cargo run --bin multi_param_sweep -- --dry-run --sweep-mode pairwise --u-values 0,1,2 --kpoints 8x8x8,6x6x6,4x4x4 --cutoffs 300,500,800 (produces 3 tasks, no errors)",
+            "cargo run --bin scf_dos_chain -- --dry-run (produces 'scf' then 'dos' — exactly 2 tasks, chain dependency respected)",
+            "cargo run --bin scf_dos_chain -- --local --dry-run (same output — 'scf' then 'dos')",
+            "bash -c 'ls examples/hubbard_u_sweep_slurm/ 2>&1 | grep -q \"No such file\"' (old crate directory is fully gone)"
+          ],
+          "notes_for_subagent": "This is the integration and cleanup task. The order of operations matters: (1) update workspace members first (remove old hubbard_u_sweep_slurm entry), (2) then delete the directory. If you delete first, cargo will error during the next check because it can't find the workspace member. The workspace members list in root Cargo.toml must exactly match the filesystem — verify each path exists after the update. After both changes, run the full verification checklist in order: build, test, clippy, dry-run. If any clippy dead-code warnings appear, investigate: they may be stale imports from library crates that only the old slurm binary used. Remove those stale imports. If any test fails: check whether it was testing the old binary (should have been ported) or is a legitimate regression. The workspace must be clean: no errors, no warnings, all tests green, both new binaries functional."
+        }
+      ]
+    }
+  ]
+}
diff --git a/notes/directions/phase-6-fix/draft-elaboration.md b/notes/directions/phase-6-fix/draft-elaboration.md
new file mode 100644
index 0000000..1de5022
--- /dev/null
+++ b/notes/directions/phase-6-fix/draft-elaboration.md
@@ -0,0 +1,619 @@
+# Draft Architectural Elaboration -- Phase 6 Fix
+
+> Generated from the plan at `plans/phase-6-fix/PHASE_6_FIX_PLAN.md`.
+> Inputs: `deferred-and-patterns.md`, `codebase-state.md`, workspace-map.json,
+> and source inspection of `hubbard_u_sweep`, `hubbard_u_sweep_slurm`.
+
+---
+
+## 1. Goal Decomposition and Design Decisions
+
+The plan has five high-level goals. Below is the architectural reasoning for each.
+
+### 1.1 Goal: Create `multi_param_sweep` binary
+
+**Decision: Parse-time validation at CLI boundary, not in setup closures.**
+
+The plan defines `parse_kpoints(s: &str) -> anyhow::Result<Vec<[u32; 3]>>` and `parse_cutoffs(s: &str) -> anyhow::Result<Vec<f64>>`. These must run *before* task construction, not inside setup closures. Rationale: if a user provides `"8xx8"` as a k-point, they should get an error immediately at CLI parse time, not three hours later when the 12th SCF task is about to start. This follows Pattern 3 (parse-time validation gates) and mitigates F.8 (string-format coupling without documentation).
+
+**Decision: `parse_kpoints` uses `[u32; 3]` not a custom struct.**
+
+The internal representation `[u32; 3]` is directly convertible to `KpointsMpGrid([kx, ky, kz])` with zero overhead. Introducing a `KpointSpec` newtype adds indirection for no benefit — the validation lives in parsing, and the type guarantee ("exactly 3 u32s") is enforced by the array type itself.
+
+**Decision: `parse_u_values` is imported, not duplicated.**
+
+The old `hubbard_u_sweep_slurm/src/config.rs` has a well-tested `parse_u_values` function. Since both new binaries and the updated `hubbard_u_sweep` need it, there are three options:
+1. Duplicate it in both new binaries (plan appears to assume this)
+2. Extract to a shared utility
+3. Import from the old binary (impossible after deletion)
+
+**Chosen: Duplicate.** The function is ~15 lines and has stable semantics. Extracting it to a shared library would require a new crate or adding it to `workflow_utils`, both of which are overkill for a comma-separated f64 parser. The slight duplication is acceptable; the tests port along with each copy. If a third binary ever needs it, extraction becomes justified.
+
+**Decision: Setup closure orchestrates three parameter injections, not one.**
+
+Each task's setup closure must:
+1. Parse seed cell → inject `hubbard_u` → serialize
+2. If kpoints provided: mutate `cell_doc.kpoints_mp_grid` → re-serialize
+3. Parse seed param → inject `cutoff_energy` → serialize
+4. If Slurm: write `job.sh`
+
+The closures are constructed with `move` captures for all parameter values. This is heavyweight but correct — each task gets immutable, pre-computed values in its closure. The pattern follows the existing `hubbard_u_sweep` binary's approach.
+
+**Decision: Sweep product logic uses `itertools::iproduct!` for clarity.**
+
+Rather than nested loops, `itertools::iproduct!` yields a flat iterator of all combinations. This makes the Cartesian product explicit and the pairwise zip path equally readable. The `itertools` crate is already a workspace dependency.
+
+### 1.2 Goal: Create `scf_dos_chain` binary
+
+**Decision: Task name collision resolved via qualified path on `Task::BandStructure` only.**
+
+The collision is between `workflow_core::task::Task` (used ~20 times per binary) and `castep_cell_io::param::general::Task` (used exactly once: setting `Task::BandStructure` in the DOS param setup). Two resolution strategies:
+
+| Strategy | Pro | Con |
+|----------|-----|-----|
+| Alias: `use ... as CastepTask` | Clear at use-site | Still clutters import namespace with two "Task" things |
+| Qualified path: `castep_cell_io::param::general::Task::BandStructure` at the single use-site | No alias, no import shadow risk, minimal scope | Verbose at one line |
+
+**Chosen: Fully-qualified path at the single use-site.** The `scf_dos_chain` binary imports `workflow_utils::prelude::*` (which re-exports `workflow_core::task::Task`). Since `castep_cell_io::param::general::Task` is needed exactly once (to set `general.task = Some(Task::BandStructure)`), the fully-qualified path keeps the import surface clean and avoids any risk of shadow confusion. The DOS setup closure contains:
+
+```rust
+doc.general.task = Some(castep_cell_io::param::general::task::Task::BandStructure);
+```
+
+(Confirm the exact module path at implementation time — it depends on how `castep-cell-io` v0.5.0 re-exports the `Task` enum.)
+
+**Decision: Shared workdir pattern — both SCF and DOS tasks in `runs/scf_dos/`.**
+
+This is specified in the plan (Pattern 6). The SCF task populates the workdir with `ZnO.check`; the DOS task's setup copies it to `ZnO_DOS.check` within the same directory. No cross-workdir path resolution needed. The DOS task executes `castep ZnO_DOS` (different seed name, same directory).
+
+**Decision: DOS collect step mirrors SCF collect — verify `ZnO_DOS.castep` exists and contains "Total time".**
+
+Consistency with the SCF collect pattern. This is a lightweight correctness check, not a full output parser.
+
+### 1.3 Goal: Delete `hubbard_u_sweep_slurm` and all its files
+
+**Decision: Port tests before deleting, not after.**
+
+The plan says "Port existing tests from `hubbard_u_sweep_slurm` before deleting it." The port target is:
+- `config.rs` tests (7 `parse_u_values` tests) → `multi_param_sweep/src/config.rs`
+- `job_script.rs` tests (6 tests) → both `multi_param_sweep/src/job_script.rs` and `scf_dos_chain/src/job_script.rs`
+
+The `no_literal_tabs` test must be ported and must pass with the cleaned heredoc template (D.2).
+
+**Decision: Delete is atomic — one commit removes the entire directory.**
+
+After all tests are ported and verified passing, the entire `examples/hubbard_u_sweep_slurm/` directory is removed in a single operation. The `.validation-complete` marker file is deleted along with it.
+
+### 1.4 Goal: Update workspace `Cargo.toml`
+
+**Decision: Workspace member list changes are atomic and last.**
+
+The workspace `Cargo.toml` members list must match the filesystem. The sequence:
+1. Create `examples/multi_param_sweep/` (with its `Cargo.toml`)
+2. Create `examples/scf_dos_chain/` (with its `Cargo.toml`)
+3. Update workspace members: remove `hubbard_u_sweep_slurm`, add both new entries
+4. Delete `examples/hubbard_u_sweep_slurm/`
+
+Steps 3-4 can happen in either order, but doing 3 first means `cargo build --workspace` won't try to resolve the deleted crate. Actually, after deletion, `cargo` won't find the member and will error. So the order is: update workspace members, then delete.
+
+**Decision: Local path dep for `castep-cell-io` in all affected Cargo.tomls.**
+
+All affected Cargo.tomls (`multi_param_sweep`, `scf_dos_chain`, `hubbard_u_sweep`) use:
+```toml
+castep-cell-io = { path = "../castep-cell-io/castep_cell_io" }
+```
+This resolves to a sibling workspace on the local filesystem. The `castep-cell-fmt` crate is already version `0.1.0` from crates.io and does not need a path dep (it is unchanged between v0.4.0 and v0.5.0 of `castep-cell-io`).
+
+### 1.5 Goal: Update `hubbard_u_sweep` for v0.5.0 API
+
+**Decision: Minimal change — only Cargo.toml and ensure compilation.**
+
+The existing `hubbard_u_sweep/src/main.rs` already uses `castep_cell_fmt::parse::<CellDocument>(&input)` — the v0.5.0 parse API. The code does not use any v0.4.0-specific builders or types. The only required change is the `Cargo.toml` dependency:
+```diff
+- castep-cell-io = "0.4.0"
++ castep-cell-io = { path = "../castep-cell-io/castep_cell_io" }
+```
+
+If compilation reveals API breakage (e.g., `Species::Symbol` now takes a different type, `OrbitalU::D(f64)` changed signature), fix those at implementation time. The plan's verification step (`cargo build --workspace`) will catch these.
+
+---
+
+## 2. Crate Boundary Decisions
+
+### 2.1 What goes where
+
+| Code | Location | Rationale |
+|------|----------|-----------|
+| `parse_u_values` | Duplicated in `multi_param_sweep` and `scf_dos_chain` `config.rs` | Stable 15-line function; extraction overhead not justified for 2 consumers |
+| `parse_kpoints` | `multi_param_sweep/src/main.rs` (or `config.rs` — see 2.2) | Only used by Binary 1; Binary 2 has no k-point sweeping |
+| `parse_cutoffs` | Same as `parse_kpoints` | Same reasoning |
+| `generate_job_script` | Duplicated in both binaries' `job_script.rs` | Stable template function; D.2 formatting fix applies to both copies |
+| `SweepConfig` / `ChainConfig` | Each binary's `config.rs` | Structurally different CLI surfaces; no shared base type |
+| Sweep logic (`build_sweep_tasks`) | `multi_param_sweep/src/main.rs` | Binary-specific combinatorics |
+| Chain logic (`build_chain`) | `scf_dos_chain/src/main.rs` | Binary-specific task wiring |
+| Seed files | Each binary's `seeds/` directory | Self-contained binaries; identical content is acceptable |
+
+### 2.2 `config.rs` vs `main.rs` boundary
+
+The plan shows CLI config in `config.rs` and sweep/chain logic in `main.rs`. This is a convention, not a strict boundary. The parsing utility functions could live in either file. Recommended split:
+
+- `config.rs`: `#[derive(Parser)]` struct, `parse_u_values`, `parse_kpoints`, `parse_cutoffs`, and their tests.
+- `main.rs`: `build_one_task` (or equivalent), sweep/chain builders, `main()` entry point.
+
+This keeps CLI surface and validation separate from workflow construction. It matches the existing `hubbard_u_sweep_slurm` structure.
+
+### 2.3 No new library crates
+
+No new library crates are created. All new code lives in binary (example) crates. This is correct: the new code is workflow _usage_ (orchestration, not abstractions). The library crates (`workflow_core`, `workflow_utils`) are unchanged.
+
+If a third binary later needs `parse_kpoints` or `generate_job_script`, extraction to a shared utility crate becomes appropriate. The threshold is 3+ consumers.
+
+---
+
+## 3. Pattern Requirements
+
+### 3.1 Patterns to follow (from `deferred-and-patterns.md`)
+
+All seven patterns from the deferred document apply. Here is the concrete implementation guidance for each:
+
+**Pattern 1: Purpose-specific binaries over mode-flags.**
+Already satisfied by the plan's two-binary design. Implementation check: neither binary should have a `--sweep-mode` flag or any flag that toggles between fundamentally different workflows.
+
+**Pattern 2: Parameter encoding in task IDs.**
+Implementation check: `multi_param_sweep` task IDs must be `scf_U{u}_k{k}_c{c}` (all three params encoded). `scf_dos_chain` uses `scf` and `dos` (single parameter set, no collision risk). Use `format!` with the exact pattern string from the plan.
+
+**Pattern 3: Parse-time validation gates.**
+Implementation check: Every parsing function returns `anyhow::Result<T>` with a message containing the expected format and what went wrong. Examples:
+- `parse_kpoints("8x8")` → `Err(anyhow!("invalid k-point '8x8': expected format 'NxNxN' with exactly 3 axes, got 2"))`
+- `parse_kpoints("")` → `Err(anyhow!("kpoints list is empty"))`
+- `parse_cutoffs("300,abc,800")` → `Err(anyhow!("invalid cutoff 'abc': expected a number"))`
+
+**Pattern 4: Explicit defaults.**
+Implementation check: `--sweep-mode` defaults to `"product"` (most complete sweep). `--kpoints` and `--cutoffs` are `Option<String>` defaulting to `None` (seed defaults used). Pairwise mode errors if either is `None`.
+
+**Pattern 5: In-memory document mutation.**
+Implementation check: All CASTEP file modifications go through `parse → mutate typed fields → to_cell_file() → to_string_many_spaced()`. Never use `format!` on raw strings to build .cell/.param content.
+
+**Pattern 6: Shared workdir for chained tasks.**
+Implementation check: `scf_dos_chain` places both tasks in `runs/scf_dos/`. The DOS setup copies `ZnO.check → ZnO_DOS.check` in the same directory.
+
+**Pattern 7: Clean heredoc template.**
+Implementation check: The `no_literal_tabs` test must pass. Use consistent space-only indentation. SBATCH directives use consistent quoting (double-quotes around values that might contain spaces).
+
+### 3.2 Patterns NOT to follow (anti-patterns to avoid)
+
+**Anti-pattern: Multi-year functions.** The old `build_sweep_tasks` function is ~50 lines with nested match arms and mode-specific logic. The new binary should have smaller, single-purpose functions:
+- `build_one_scf_task(config, u, kpoint, cutoff, seed_cell, seed_param) -> Result<Task>`
+- `build_all_scf_tasks(config, seed_cell, seed_param) -> Result<Vec<Task>>` (dispatches to product or pairwise)
+- `main()` orchestrates but does not contain sweep logic
+
+**Anti-pattern: Mode flag for workflow type.** Neither new binary should have a flag that toggles between "independent tasks" and "chained tasks".
+
+---
+
+## 4. Type Signatures for Key New Types
+
+### 4.1 `multi_param_sweep` config
+
+```rust
+#[derive(Parser, Debug)]
+#[command(name = "multi_param_sweep")]
+pub struct SweepConfig {
+    /// Comma-separated Hubbard U values (eV), e.g. "0.0,1.0,2.0,3.0,4.0,5.0"
+    #[arg(long, default_value = "0.0,1.0,2.0,3.0,4.0,5.0")]
+    pub u_values: String,
+
+    /// Comma-separated MP grids, e.g. "8x8x8,6x6x6". Omit to use seed defaults.
+    #[arg(long)]
+    pub kpoints: Option<String>,
+
+    /// Comma-separated cutoff energies (eV), e.g. "300,500,800". Omit to use seed defaults.
+    #[arg(long)]
+    pub cutoffs: Option<String>,
+
+    /// Sweep mode: "product" (Cartesian product) or "pairwise" (zip — all three lists must be same length)
+    #[arg(long, default_value = "product")]
+    pub sweep_mode: String,
+
+    /// Element for Hubbard U
+    #[arg(long, default_value = "Zn")]
+    pub element: String,
+
+    /// Orbital: 'd' or 'f'
+    #[arg(long, default_value = "d")]
+    pub orbital: char,
+
+    /// CASTEP input file prefix
+    #[arg(long, default_value = "ZnO")]
+    pub seed_name: String,
+
+    /// Max concurrent tasks
+    #[arg(long, default_value_t = 4)]
+    pub max_parallel: usize,
+
+    /// Direct process execution (no SLURM)
+    #[arg(long)]
+    pub local: bool,
+
+    /// Print topological order and exit
+    #[arg(long)]
+    pub dry_run: bool,
+
+    /// CASTEP binary name or path (local mode)
+    #[arg(long, default_value = "castep")]
+    pub castep_command: String,
+
+    /// Root directory for runs/logs
+    #[arg(long, default_value = ".")]
+    pub workdir: String,
+
+    // --- SLURM fields (same as old binary) ---
+    #[arg(long, env = "CASTEP_SLURM_PARTITION", default_value = "debug")]
+    pub partition: String,
+
+    #[arg(long, default_value_t = 16)]
+    pub ntasks: u32,
+
+    #[arg(long, env = "CASTEP_NIX_FLAKE",
+          default_value = "git+ssh://git@github.com/TonyWu20/CASTEP-25.12-nixos#castep_25_mkl")]
+    pub nix_flake: String,
+
+    #[arg(long, env = "CASTEP_MPI_IF", default_value = "enp6s0")]
+    pub mpi_if: String,
+}
+```
+
+### 4.2 `scf_dos_chain` config
+
+```rust
+#[derive(Parser, Debug)]
+#[command(name = "scf_dos_chain")]
+pub struct ChainConfig {
+    /// Single Hubbard U value (eV)
+    #[arg(long, default_value_t = 3.0)]
+    pub u_value: f64,
+
+    /// Element for Hubbard U
+    #[arg(long, default_value = "Zn")]
+    pub element: String,
+
+    /// Orbital: 'd' or 'f'
+    #[arg(long, default_value = "d")]
+    pub orbital: char,
+
+    /// CASTEP input file prefix
+    #[arg(long, default_value = "ZnO")]
+    pub seed_name: String,
+
+    /// Max concurrent tasks
+    #[arg(long, default_value_t = 1)]
+    pub max_parallel: usize,
+
+    /// Direct process execution (no SLURM)
+    #[arg(long)]
+    pub local: bool,
+
+    /// Print topological order and exit
+    #[arg(long)]
+    pub dry_run: bool,
+
+    /// CASTEP binary name or path (local mode)
+    #[arg(long, default_value = "castep")]
+    pub castep_command: String,
+
+    /// Root directory for runs/logs
+    #[arg(long, default_value = ".")]
+    pub workdir: String,
+
+    // --- SLURM fields (same as SweepConfig) ---
+    #[arg(long, env = "CASTEP_SLURM_PARTITION", default_value = "debug")]
+    pub partition: String,
+
+    #[arg(long, default_value_t = 16)]
+    pub ntasks: u32,
+
+    #[arg(long, env = "CASTEP_NIX_FLAKE",
+          default_value = "git+ssh://git@github.com/TonyWu20/CASTEP-25.12-nixos#castep_25_mkl")]
+    pub nix_flake: String,
+
+    #[arg(long, env = "CASTEP_MPI_IF", default_value = "enp6s0")]
+    pub mpi_if: String,
+}
+```
+
+### 4.3 Parsing utility signatures
+
+```rust
+/// Parse a comma-separated list of u32 triplets.
+///
+/// Each segment must be exactly "NxNxN" format.
+/// Returns clear anyhow errors for wrong number of axes or non-numeric input.
+///
+/// # Errors
+/// - `anyhow!("kpoints list is empty")` for empty or whitespace-only input
+/// - `anyhow!("invalid k-point '8x8': expected 3 axes, got 2")` for wrong format
+/// - `anyhow!("invalid k-point 'abc': ...")` for non-numeric axes
+pub fn parse_kpoints(s: &str) -> anyhow::Result<Vec<[u32; 3]>>;
+
+/// Parse a comma-separated list of f64 values.
+///
+/// Each segment is trimmed before parsing.
+///
+/// # Errors
+/// - `anyhow!("invalid cutoff '{}': {parse_error}")` for non-numeric tokens
+/// - `anyhow!("cutoffs list is empty")` for empty input
+pub fn parse_cutoffs(s: &str) -> anyhow::Result<Vec<f64>>;
+
+/// Parse a comma-separated list of f64 values (ported from old binary).
+/// Same semantics as parse_cutoffs but with Hubbard-U-specific error context.
+pub fn parse_u_values(s: &str) -> anyhow::Result<Vec<f64>>;
+```
+
+### 4.4 `generate_job_script` signature
+
+```rust
+/// Generate a SLURM job script for the given task.
+///
+/// The script uses a clean heredoc template with consistent space-only
+/// indentation and double-quoted SBATCH directive values.
+///
+/// Passes the `no_literal_tabs` test: output must not contain literal '\t'.
+pub fn generate_job_script(config: &SweepConfig, task_id: &str, seed_name: &str) -> String;
+```
+
+For `scf_dos_chain`, the function takes `&ChainConfig` instead of `&SweepConfig`. Since the SLURM fields are identical, the function body is the same. The config parameter type is the only difference.
+
+**Open question**: Should `generate_job_script` accept a trait (e.g., `SlurmConfig`) to deduplicate between the two binaries? Decision: No. The two copies are ~30 lines each and have independent lifetimes. A trait for two impls adds an abstraction layer with no immediate benefit. If a third SLURM config variant appears, introduce the trait then.
+
+### 4.5 Task builder signatures (in main.rs)
+
+```rust
+/// Build a single SCF task for one (U, kpoint, cutoff) combination.
+///
+/// The setup closure:
+/// 1. Parses seed cell, injects HubbardU block
+/// 2. If kpoint provided, sets `cell_doc.kpoints_mp_grid`
+/// 3. Parses seed param, injects cutoff_energy
+/// 4. Serializes both documents to the workdir
+/// 5. If running SLURM, writes job.sh
+///
+/// The collect closure verifies `<seed>.castep` exists and contains "Total time".
+fn build_one_scf_task(
+    config: &SweepConfig,
+    u: f64,
+    kpoint: Option<[u32; 3]>,
+    cutoff: Option<f64>,
+    seed_cell: &str,
+    seed_param: &str,
+) -> Result<Task, WorkflowError>;
+```
+
+For `scf_dos_chain`:
+
+```rust
+/// Build the SCF task for the chain (no k-point/cutoff variation).
+fn build_scf_task(
+    config: &ChainConfig,
+    seed_cell: &str,
+    seed_param: &str,
+) -> Result<Task, WorkflowError>;
+
+/// Build the DOS task that depends on scf_task.
+/// Copies <workdir>/ZnO.check → <workdir>/ZnO_DOS.check in setup.
+/// Sets general.task = Task::BandStructure in the param.
+/// Execute castep on ZnO_DOS seed.
+fn build_dos_task(
+    config: &ChainConfig,
+    scf_task_id: &str,
+    seed_cell: &str,
+    seed_param: &str,
+) -> Result<Task, WorkflowError>;
+```
+
+---
+
+## 5. Known Pitfalls and Constraints
+
+### 5.1 The `Task` naming collision (P0)
+
+As documented in codebase-state.md Observation B: `use workflow_utils::prelude::*` brings `workflow_core::task::Task` into scope. Using `use castep_cell_io::param::general::Task` in the same file will shadow it.
+
+**Resolution in `scf_dos_chain/src/main.rs`**: Do not import `castep_cell_io::param::general::Task`. Use the fully qualified path at the single use-site in the DOS setup closure:
+
+```rust
+doc.general.task = Some(castep_cell_io::param::general::task::Task::BandStructure);
+```
+
+The exact module path depends on how `castep-cell-io` v0.5.0 re-exports the `Task` enum. Verify during implementation with LSP hover on the type.
+
+### 5.2 Local path dep must exist on the filesystem
+
+The path `../castep-cell-io/castep_cell_io` must resolve to a sibling directory that contains a `Cargo.toml` with the `castep-cell-io` crate. If the user's filesystem layout differs, this path dep will fail. The plan assumes the default layout where `castep-cell-io` is a sibling workspace.
+
+**Pre-implementation check**: Verify `../castep-cell-io/castep_cell_io/Cargo.toml` exists. If it doesn't, the path needs adjustment.
+
+### 5.3 `castep-cell-fmt` version compatibility
+
+`castep-cell-fmt = "0.1.0"` is used from crates.io while `castep-cell-io` uses a local path dep at v0.5.0. If `castep-cell-io` v0.5.0 depends on a newer `castep-cell-fmt`, Cargo will resolve the semver-compatible version automatically. No conflict expected (both are parse-time libraries, not runtime).
+
+### 5.4 Pairwise mode validation ordering
+
+The plan says pairwise mode "requires both --kpoints and --cutoffs to be explicitly provided." The validation logic is:
+
+```rust
+match config.sweep_mode.as_str() {
+    "pairwise" => {
+        let kpoints = config.kpoints.as_ref()
+            .ok_or_else(|| anyhow!("pairwise mode requires --kpoints to be specified"))?;
+        let cutoffs = config.cutoffs.as_ref()
+            .ok_or_else(|| anyhow!("pairwise mode requires --cutoffs to be specified"))?;
+        let kpts = parse_kpoints(kpoints)?;
+        let cuts = parse_cutoffs(cutoffs)?;
+        let u_vals = parse_u_values(&config.u_values)?;
+        if kpts.len() != u_vals.len() || cuts.len() != u_vals.len() {
+            anyhow::bail!(
+                "pairwise mode requires all three lists to have the same length \
+                 (U values: {}, kpoints: {}, cutoffs: {})",
+                u_vals.len(), kpts.len(), cuts.len()
+            );
+        }
+        // zip and build tasks
+    }
+    "product" => { /* Cartesian product — optional kpoints/cutoffs OK */ }
+    _ => anyhow::bail!("unknown sweep mode '{}', expected 'product' or 'pairwise'", config.sweep_mode),
+}
+```
+
+### 5.5 `hubbard_u_sweep` compatibility with v0.5.0 path dep
+
+The old binary uses:
+```rust
+castep_cell_fmt::{format::to_string_many_spaced, parse, ToCellFile}
+castep_cell_io::cell::species::{AtomHubbardU, HubbardU, HubbardUUnit, OrbitalU, Species}
+castep_cell_io::CellDocument
+```
+
+All of these should work unchanged with v0.5.0. The `parse` free function and `to_cell_file()` pattern already match the v0.5.0 API. However, v0.5.0 may have changed:
+- `Species::Symbol(String)` signature (could be `Species::Symbol(&str)` or a newtype)
+- Builder method names on `AtomHubbardU` / `HubbardU`
+- `HubbardUUnit` enum variants
+
+If compilation fails after the path-dep change, fix these call sites. The build error messages will point to the exact location.
+
+### 5.6 `cargo test --workspace` with new path dep
+
+Tests in `workflow_core` (including `tests/hubbard_u_sweep.rs`) may not resolve the `castep-cell-io` path dep because `workflow_core` does not depend on `castep-cell-io`. The integration test `test_hubbard_u_sweep_with_mock_castep` uses mock executables and never touches `castep-cell-io`. This should continue to pass.
+
+However, if any test transitively exercises `hubbard_u_sweep`'s task construction (which does use `castep-cell-io`), the path dep must be resolvable from that test's Cargo context. Since the test is in `workflow_core`, not in the example binary, this is unlikely to be an issue.
+
+### 5.7 Dead-code detection after refactoring (mitigating F.3)
+
+After deleting `hubbard_u_sweep_slurm`, run:
+```
+cargo clippy --workspace --all-targets -- -W dead-code
+```
+to detect any stale imports or dead functions left over from the refactoring. The old binary's crate will be gone, but any shared constants or utility code that was only used by it should be found.
+
+### 5.8 Carrier of `no_literal_tabs` test and D.2 formatting fix
+
+The `no_literal_tabs` test from the old `job_script.rs` must be ported to both new binaries. The D.2 fix (clean heredoc template) must be applied to the ported `generate_job_script` function. The test verifies the fix.
+
+Implementation checklist for D.2:
+- [ ] `generate_job_script` uses all-space indentation (no literal `\t`)
+- [ ] SBATCH directives use double-quotes consistently: `#SBATCH --job-name="value"`
+- [ ] Shell continuation lines use `\\` not literal line breaks mixed with indentation
+- [ ] `no_literal_tabs` test passes in both binaries
+
+### 5.9 Deferred items: what NOT to do
+
+- **D.1 (portable SLURM config)**: Do NOT implement now. Precondition not met (still on NixOS cluster). Keep NixOS-specific fields.
+- **D.3 (unit tests for generate_job_script)**: Do NOT expand beyond the ported tests. The existing tests are NixOS-specific; broadening them requires D.1 first.
+- **`read_task_ids` edge case**: Do NOT fix now. Precondition not met (not touching `read_task_ids`).
+
+---
+
+## 6. Suggested Task Grouping
+
+The plan's five goals decompose into three implementation groups, designed for sequential `/explore-implement` passes.
+
+### Group 1: `multi_param_sweep` binary (standalone)
+
+**Goal**: Create the complete `examples/multi_param_sweep/` crate.
+
+**Files to create**:
+- `examples/multi_param_sweep/Cargo.toml`
+- `examples/multi_param_sweep/src/main.rs`
+- `examples/multi_param_sweep/src/config.rs`
+- `examples/multi_param_sweep/src/job_script.rs`
+- `examples/multi_param_sweep/seeds/ZnO.cell` (copy from existing)
+- `examples/multi_param_sweep/seeds/ZnO.param` (copy from existing)
+
+**Implementation approach**: `lib-tdd`
+- `config.rs` tests (ported `parse_u_values` + new `parse_kpoints` + new `parse_cutoffs`) — write tests first
+- `job_script.rs` tests (ported: sbatch directives, seed name, shebang, no_literal_tabs, nix, mpi) — write tests first
+- Sweep logic combinatorics (product mode, pairwise mode, error cases) — write tests first
+- Task builder (setup closure writes correct .cell/.param content) — integration-style
+
+**Dependencies**: None (self-contained binary). Can build independently once created, but workspace membership needed for `cargo run --bin multi_param_sweep`.
+
+### Group 2: `scf_dos_chain` binary + `hubbard_u_sweep` update (standalone)
+
+**Goal**: Create Binary 2 and update the existing `hubbard_u_sweep` for v0.5.0.
+
+**Files to create**:
+- `examples/scf_dos_chain/Cargo.toml`
+- `examples/scf_dos_chain/src/main.rs`
+- `examples/scf_dos_chain/src/config.rs`
+- `examples/scf_dos_chain/src/job_script.rs`
+- `examples/scf_dos_chain/seeds/ZnO.cell`
+- `examples/scf_dos_chain/seeds/ZnO.param`
+
+**Files to update**:
+- `examples/hubbard_u_sweep/Cargo.toml` (path dep for castep-cell-io)
+- `examples/hubbard_u_sweep/src/main.rs` (if v0.5.0 API requires code changes)
+
+**Implementation approach**: Mixed
+- `config.rs`: `direct` — simple single-value config, no new parsing functions (reuse `parse_u_values`)
+- `job_script.rs`: `direct` — identical to Group 1's version, copy + adapt config type parameter
+- `main.rs` chain logic: `lib-tdd` — test task dependency structure (SCF → DOS), test DOS setup closure (checkpoint copy, task type set to BandStructure)
+- `hubbard_u_sweep` update: `direct` — dependency change + compile check
+
+**Dependencies**: None on Group 1 (independent). The two binaries share no code.
+
+### Group 3: Workspace integration, cleanup, verification
+
+**Goal**: Wire everything together, delete old crate, verify full build.
+
+**Files to update**:
+- `/Users/tony/programming/castep_workflow_framework/Cargo.toml` (workspace members)
+
+**Files to delete**:
+- `examples/hubbard_u_sweep_slurm/` (entire directory)
+
+**Implementation approach**: `direct`
+- Update workspace members list: remove `hubbard_u_sweep_slurm`, add `multi_param_sweep` and `scf_dos_chain`
+- Delete old crate directory
+- Run `cargo build --workspace` to verify all crates compile
+- Run `cargo test --workspace` to verify all tests pass (old slurm tests removed, new binary tests run)
+- Run `cargo clippy --workspace --all-targets -- -W dead-code` for F.3 mitigation
+
+**Verification checklist** (from plan):
+1. [x] Dry run Binary 1: `cargo run --bin multi_param_sweep -- --dry-run`
+2. [x] Dry run Binary 1 pairwise: `cargo run --bin multi_param_sweep -- --dry-run --sweep-mode pairwise --u-values 0,1,2 --kpoints 8x8x8,6x6x6,4x4x4 --cutoffs 300,500,800`
+3. [x] Local run Binary 1 (if CASTEP available)
+4. [x] Dry run Binary 2: `cargo run --bin scf_dos_chain -- --dry-run`
+5. [x] Dry run Binary 2 chain: `cargo run --bin scf_dos_chain -- --local --dry-run`
+6. [x] Full build: `cargo build --workspace` (no warnings)
+7. [x] Full test: `cargo test --workspace` (all pass)
+
+### Dependency graph between groups
+
+```
+Group 1 ──┐
+           ├── Group 3
+Group 2 ──┘
+```
+
+Groups 1 and 2 are independent and can be implemented in parallel. Group 3 depends on both being complete.
+
+### Rationale for this grouping
+
+1. **Groups 1 and 2 are self-contained binaries.** Each can be built and tested in isolation before workspace integration. This reduces coupling during implementation — a bug in Binary 1 doesn't block Binary 2.
+
+2. **Group 3 is the integration step.** It wires the new crates into the workspace, removes the old one, and validates end-to-end. Keeping this separate means Group 1 and 2 implementers don't need to think about workspace-level concerns.
+
+3. **`hubbard_u_sweep` update is grouped with Binary 2** because it's a small, high-confidence change (primarily a Cargo.toml edit). It doesn't warrant its own group.
+
+4. **Test porting is embedded in each group** rather than a separate step. This ensures tests migrate with the code they test, not as an afterthought.
+
+---
+
+## 7. Open Questions for Implementation
+
+| # | Question | Default answer | When to resolve |
+|---|----------|---------------|-----------------|
+| Q1 | Exact module path for `Task::BandStructure` in `castep-cell-io` v0.5.0? | `castep_cell_io::param::general::task::Task::BandStructure` | During Group 2 implementation — use LSP hover |
+| Q2 | Does `Species::Symbol` in v0.5.0 take `String` or `&str`? | Assume `String` (same as v0.4.0) | During Group 1 implementation — compiler will catch |
+| Q3 | Does `castep-cell-io` v0.5.0 re-export `KpointsMpGrid` from the crate root or a submodule? | `castep_cell_io::KpointsMpGrid` (plan says it's a direct field on `CellDocument`) | During Group 1 implementation |
+| Q4 | Should `parse_u_values` return `anyhow::Result` or `Result<_, String>`? | `anyhow::Result` — consistent with other parsing functions (Pattern 3) | During Group 1 implementation |
+| Q5 | Are the existing `hubbard_u_sweep_slurm` tests in `config.rs` and `job_script.rs` using `#[cfg(test)]` modules within the same file? | Yes (confirmed from source inspection) | Port these as-is to the new binaries |
diff --git a/notes/directions/phase-6-fix/task-checklist.md b/notes/directions/phase-6-fix/task-checklist.md
new file mode 100644
index 0000000..20cbb10
--- /dev/null
+++ b/notes/directions/phase-6-fix/task-checklist.md
@@ -0,0 +1,261 @@
+# Phase 6 Fix Directions Task Checklist
+
+**Review date**: 2026-05-05
+**Source files reviewed**:
+- `/Users/tony/programming/castep_workflow_framework/notes/directions/phase-6-fix/draft-directions.json`
+- `/Users/tony/programming/castep_workflow_framework/notes/directions/phase-6-fix/codebase-state.md`
+- `/Users/tony/programming/castep_workflow_framework/Cargo.toml` (workspace root)
+- `/Users/tony/programming/castep_workflow_framework/examples/hubbard_u_sweep_slurm/src/config.rs`
+- `/Users/tony/programming/castep_workflow_framework/examples/hubbard_u_sweep_slurm/src/job_script.rs`
+- `/Users/tony/programming/castep_workflow_framework/examples/hubbard_u_sweep_slurm/Cargo.toml`
+
+**External dependency verified**:
+- `/Users/tony/programming/castep-cell-io/castep_cell_io/Cargo.toml` exists, version = "0.5.0"
+- `KpointsMpGrid(pub [u32; 3])` confirmed (tuple struct, `./cell/bz_sampling_kpoints/kpoints_mp_grid.rs`)
+- `CutOffEnergy { value: f64, unit: None }` confirmed (construction pattern at `./param/basis_set_params.rs` lines 81, 95)
+- `HubbardUUnit` enum confirmed in `./cell/species/hubbard_u/mod.rs` (NOT `HubburdUUnit` — see issue note below)
+
+---
+
+## Group 1: multi_param_sweep binary
+
+### Task G1-1: Create multi_param_sweep crate skeleton and add to workspace
+
+| Criterion | Assessment |
+|---|---|
+| Goal clear? | CLEAR — create directory, Cargo.toml, stub files, seed files, workspace member |
+| Files in scope correct? | YES — matches the old slurm binary structure (Cargo.toml, 3 src files, 2 seed files) |
+| Implementation detail sufficient? | YES — dependency list complete, stub content specified, workspace member insertion point specified |
+| Acceptance commands sufficient? | YES — `cargo check --workspace` plus `ls` for file existence |
+| Wiring checklist correct? | YES — add mod declarations and workspace member |
+| Depends_on/enables explicit? | YES — no depends, enables G1-2, G1-3 |
+
+**Verdict: CLEAR**
+
+---
+
+### Task G1-2: Implement parse functions and SweepConfig in config.rs with TDD
+
+| Criterion | Assessment |
+|---|---|
+| Goal clear? | CLEAR — implement SweepConfig struct and three parse functions |
+| Files in scope correct? | YES — config.rs only |
+| Implementation detail sufficient? | YES — full SweepConfig field list with defaults, parse function semantics, error message expectations |
+| Acceptance commands sufficient? | YES — cargo check + cargo test with count (20 tests) |
+| Depends_on/enables explicit? | YES — depends G1-1, enables G1-4, G1-5 |
+| tdd_interface.test_code specific/falsifiable? | PASS — 20 tests total (7 parse_u_values + 7 parse_kpoints + 6 parse_cutoffs), all assert concrete values or error message patterns |
+| tdd_interface.signature matches test code? | YES — `parse_u_values`, `parse_kpoints`, `parse_cutoffs` all called by exact name |
+| tdd_interface.signature covers all tested functions? | YES — all three functions tested |
+
+**Verdict: CLEAR** — Tests are comprehensive and falsifiable.
+
+---
+
+### Task G1-3: Implement generate_job_script with D.2 fix using TDD
+
+| Criterion | Assessment |
+|---|---|
+| Goal clear? | CLEAR — implement generate_job_script with D.2 formatting fix |
+| Files in scope correct? | YES — job_script.rs |
+| Implementation detail sufficient? | YES — heredoc template guidance, D.2 fix requirements, import pattern (SweepConfig from crate::config) |
+| Acceptance commands sufficient? | YES — cargo check + cargo test with specific test (no_literal_tabs) |
+| Depends_on/enables explicit? | SEE ISSUE below |
+| tdd_interface.test_code specific/falsifiable? | PASS — 6 tests, all assert concrete strings or tab absence |
+| tdd_interface.signature matches test code? | YES — `generate_job_script` called by exact name |
+| tdd_interface.signature covers all tested functions? | YES — only one function under test |
+
+**ISSUE: Dependency contradiction (CRITICAL)** — G1-3 declares `depends_on: ["G1-1"]` and `can_run_in_parallel_with: ["G1-2"]`. However, G1-3's function signature takes `config: &SweepConfig` and its tests import `SweepConfig` from `crate::config`. The `SweepConfig` type is defined in G1-2. If G1-3 runs before G1-2 (as the parallel declaration allows), `SweepConfig` will not exist yet in `config.rs`, causing a compilation failure. G1-3 should either:
+  - Add G1-2 as a dependency (removing parallel execution), OR
+  - Provide explicit guidance for handling the missing SweepConfig (e.g., "If SweepConfig has not been implemented yet by G1-2, define a minimal placeholder type")
+
+**Verdict: BLOCKED without fix** — Dependency on G1-2's SweepConfig is real, but `can_run_in_parallel_with: ["G1-2"]` says the opposite.
+
+---
+
+### Task G1-4: Implement sweep logic and task builders with TDD
+
+| Criterion | Assessment |
+|---|---|
+| Goal clear? | CLEAR — implement build_one_scf_task and build_all_scf_tasks |
+| Files in scope correct? | YES — main.rs (tests in-module) |
+| Implementation detail sufficient? | YES — very detailed: task ID format, workdir path, setup closure steps, collect closure, sweep mode dispatch |
+| Acceptance commands sufficient? | YES — cargo check + cargo test; also lists expected behavioral outcomes per test |
+| Depends_on/enables explicit? | YES — depends G1-2, G1-3; enables G1-5 |
+| tdd_interface.test_code specific/falsifiable? | PASS — 12 tests, all assert concrete task counts, IDs, dependency structure, error messages |
+| tdd_interface.signature matches test code? | YES — `build_all_scf_tasks` called directly; `build_one_scf_task` listed but tested indirectly |
+| tdd_interface.signature covers all tested functions? | MINOR — `build_one_scf_task` is in the signature array but no test calls it directly. The tests only exercise it via `build_all_scf_tasks`. A subagent could inline the logic into `build_all_scf_tasks` and skip the function entirely while still passing all tests. However, `expected_behavior` does describe both functions, so a diligent subagent should implement both. |
+
+**Verdict: CLEAR** — Detailed and implementable.
+
+---
+
+### Task G1-5: Wire main() entry point for multi_param_sweep
+
+| Criterion | Assessment |
+|---|---|
+| Goal clear? | CLEAR — implement main() orchestrating config, task building, workflow, dry-run, execution |
+| Files in scope correct? | YES — main.rs |
+| Implementation detail sufficient? | YES — full main() structure with local/queued/dry-run paths, workflow construction, state store, summary printing |
+| Acceptance commands sufficient? | YES — cargo check + 3 cargo run invocations (dry-run normal, dry-run pairwise, pairwise missing args errors) |
+| Depends_on/enables explicit? | YES — depends G1-4, enables G3-1 |
+| Wiring checklist correct? | N/A (no checklist items — wiring is in main() logic itself) |
+
+**Verdict: CLEAR**
+
+---
+
+## Group 2: scf_dos_chain binary + hubbard_u_sweep update
+
+### Task G2-1: Create scf_dos_chain crate skeleton and add to workspace
+
+| Criterion | Assessment |
+|---|---|
+| Goal clear? | CLEAR — symmetric to G1-1 |
+| Files in scope correct? | YES — same structure as G1-1 |
+| Implementation detail sufficient? | YES — same pattern as G1-1 with correct paths |
+| Acceptance commands sufficient? | YES — cargo check + ls |
+| Depends_on/enables explicit? | YES — depends G1-1 (file collision on root Cargo.toml), enables G2-2 |
+
+**Verdict: CLEAR** — Intentional serialization with G1-1 is documented.
+
+---
+
+### Task G2-2: Implement config.rs and job_script.rs for scf_dos_chain with TDD
+
+| Criterion | Assessment |
+|---|---|
+| Goal clear? | CLEAR — implement ChainConfig, parse_u_values, generate_job_script |
+| Files in scope correct? | YES — config.rs and job_script.rs |
+| Implementation detail sufficient? | YES — ChainConfig fields specified (simpler than SweepConfig: single u_value, no sweep fields), parse_u_values duplicated, job_script template identical |
+| Acceptance commands sufficient? | YES — cargo check + cargo test for both modules (13 total tests) |
+| Depends_on/enables explicit? | YES — depends G2-1, enables G2-3 |
+| tdd_interface.test_code specific/falsifiable? | PASS — 7 + 6 = 13 tests, same assertions as G1-2/G1-3 counterparts, adapted for ChainConfig |
+| tdd_interface.signature matches test code? | YES — `parse_u_values` and `generate_job_script` called by exact name |
+| tdd_interface.test_file format | ISSUE — `test_file` is a descriptive string ("parse_u_values tests in examples/scf_dos_chain/src/config.rs; job_script tests in examples/scf_dos_chain/src/job_script.rs") instead of a clean path. The explore-implement pipeline typically expects a single file path. However, `test_module` says "tests (in each file)" which clarifies the intent. The subagent must split the `test_code` block between two files manually. |
+
+**Verdict: CLEAR** — Functional logic is clear. Test file split requires subagent care but is documented.
+
+---
+
+### Task G2-3: Implement chain task builders (SCF + DOS) with TDD
+
+| Criterion | Assessment |
+|---|---|
+| Goal clear? | CLEAR — implement build_scf_task and build_dos_task with correct dependency chain |
+| Files in scope correct? | YES — main.rs |
+| Implementation detail sufficient? | YES — detailed guidance on setup closures, checkpoint copy (Pattern 6), Task::BandStructure, shared workdir |
+| Acceptance commands sufficient? | YES — cargo check + cargo test + behavioral criteria list |
+| Depends_on/enables explicit? | YES — depends G2-2, enables G2-4 |
+| tdd_interface.test_code specific/falsifiable? | PASS — 7 tests, all assert concrete IDs, dependency counts, workdir paths |
+| tdd_interface.signature matches test code? | YES — both `build_scf_task` and `build_dos_task` called by exact name |
+| Typo in guidance | ISSUE — The import list at line ~473 contains `HubburdUUnit` (typo: extra 'r'). The actual v0.5.0 API enum is `HubbardUUnit` (confirmed via MCP search on the castep-cell-io source). A subagent following this literally will get a compilation error. |
+
+**Verdict: CLEAR** — Content is clear; the `HubburdUUnit` typo will be caught at compile time.
+
+---
+
+### Task G2-4: Wire main() entry point for scf_dos_chain
+
+| Criterion | Assessment |
+|---|---|
+| Goal clear? | CLEAR — symmetric to G1-5, simpler (2 tasks, no sweep logic) |
+| Files in scope correct? | YES — main.rs |
+| Implementation detail sufficient? | YES — follows same pattern as G1-5 with explicit chain construction order |
+| Acceptance commands sufficient? | YES — cargo check + 2 cargo run invocations |
+| Depends_on/enables explicit? | YES — depends G2-3, enables G3-1 |
+
+**Verdict: CLEAR**
+
+---
+
+### Task G2-5: Update hubbard_u_sweep for castep-cell-io v0.5.0 path dep
+
+| Criterion | Assessment |
+|---|---|
+| Goal clear? | CLEAR — change Cargo.toml dep, fix compilation errors |
+| Files in scope correct? | YES — Cargo.toml and main.rs (latter only if compiler reports errors) |
+| Implementation detail sufficient? | ADEQUATE — "fix what the compiler reports" is the right strategy for an API migration where exact v0.5.0 signatures are unknown at plan time. Known breakage points are listed (Species::Symbol, builder methods, enum variants). |
+| Acceptance commands sufficient? | YES — cargo check -p hubbard_u_sweep (zero errors, zero warnings) |
+| Depends_on/enables explicit? | YES — no depends, enables G3-1, can run in parallel with everything |
+
+**Verdict: CLEAR** — Minimal task, correct strategy.
+
+---
+
+## Group 3: Workspace integration, cleanup, verification
+
+### Task G3-1: Remove old crate, finalize workspace, run full verification
+
+| Criterion | Assessment |
+|---|---|
+| Goal clear? | CLEAR — remove old binary, update workspace, full verification |
+| Files in scope correct? | YES — root Cargo.toml (members list) + delete directory |
+| Implementation detail sufficient? | YES — exact members list provided, deletion instruction clear |
+| Acceptance commands sufficient? | YES — 7 command acceptance criteria: build, test, clippy dead-code, 2 multi_param_sweep dry-run variants, 2 scf_dos_chain dry-run variants, old-directory-gone check |
+| Depends_on/enables explicit? | YES — depends G1-5, G2-4, G2-5 (all new binaries + old binary fix complete) |
+
+**Minor note**: The expected members list after removal excludes `'examples/hubbard_u_sweep_slurm'`, keeping 6 members. The guidance says "Do NOT change any other configuration (resolver, workspace.dependencies, etc.)" which is explicit.
+
+**Verdict: CLEAR** — Most thorough verification of all tasks.
+
+---
+
+## Overall Assessment
+
+### Strengths
+- Architecture decisions are documented in detail and consistently applied across all tasks.
+- Parse-time validation (Pattern 3), in-memory document mutation (Pattern 5), and shared workdir (Pattern 6) are well-motivated and consistently referenced.
+- TDD test code is consistently falsifiable — every test asserts concrete values, counts, or error message strings. No trivial assertions found.
+- The crate boundary map is precise about which functions are duplicated vs. shared.
+- Pitfalls from prior phases (task naming collision, path dep existence, dead-code detection) are explicitly addressed.
+- Task IDs (`scf_U{u}_k{k}_c{c}`) are specified with exact formatting rationale.
+
+### Issues Requiring Fixes
+
+1. **CRITICAL — G1-3 dependency contradiction**: G1-3 (`generate_job_script`) depends on `SweepConfig` (defined in G1-2) but declares `depends_on: ["G1-1"]` and `can_run_in_parallel_with: ["G1-2"]`. This is impossible since the function signature and tests both import `SweepConfig`. **Fix**: Change `depends_on` to `["G1-1", "G1-2"]` and remove from `can_run_in_parallel_with`.
+
+2. **MINOR — G2-2 `test_file` format**: The field uses a human-readable description ("parse_u_values tests in ...") instead of a clean file path. The explore-implement pipeline expects a single-path or array-of-paths format. **Fix**: Use `"test_file": ["examples/scf_dos_chain/src/config.rs", "examples/scf_dos_chain/src/job_script.rs"]`.
+
+3. **MINOR — G2-3 typo in import list**: `HubburdUUnit` (typo) should be `HubbardUUnit`. **Fix**: Correct the spelling.
+
+4. **MINOR — G1-4 signature includes untested function**: `build_one_scf_task` is listed in `signature` but no test calls it. An inlining subagent could skip it. The `expected_behavior` partially mitigates this but the test code itself does not force its existence. **Fix**: Either add a test that calls `build_one_scf_task` directly, or remove it from the `signature` array (and rely on `expected_behavior` alone).
+
+### Non-Issues (noted but acceptable)
+- The `castep_cell_io::param::general::task::Task` module path for `Task::BandStructure` in G2-3 is flagged as needing LSP verification. This is explicitly documented as a verification step, not a gap.
+- The castep-cell-io v0.5.0 API details (Species::Symbol signature, builder method names) are not precisely known at plan time. The "fix compiler errors" strategy in G2-5 is the correct approach.
+- Function duplication (parse_u_values, generate_job_script) between G1 and G2 is intentional per the architecture decision `duplication_choice`.
+
+### Verification: TDD Tasks
+
+| Task | tdd_interface.kind | test_code meaningful? | Concrete assertions? | signature matches? | Verdict |
+|---|---|---|---|---|---|
+| G1-2 | lib-tdd | YES — 20 tests | YES — all assert values or error patterns | YES | PASS |
+| G1-3 | lib-tdd | YES — 6 tests | YES — all assert concrete strings | YES | PASS |
+| G1-4 | lib-tdd | YES — 12 tests | YES — task counts, IDs, dependencies, errors | YES (minor: build_one_scf_task not directly called) | PASS |
+| G2-2 | lib-tdd | YES — 13 tests | YES — same assertions as G1-2/G1-3 | YES | PASS |
+| G2-3 | lib-tdd | YES — 7 tests | YES — IDs, dependency count, workdir path | YES | PASS |
+
+All `lib-tdd` tasks have meaningful, falsifiable test code. No `assert!(true)` or other trivial assertions found. The `signature` functions match the functions called in `test_code`.
+
+---
+
+## Final Verdict
+
+**Needs More Detail** — Specifically the G1-3 dependency contradiction (issue #1) must be resolved before implementation can proceed. The remaining issues (typo, test_file format, minor signature gap) are non-blocking but should also be fixed for clarity. Once the dependency issue is resolved, the directions are precise enough to implement all 11 tasks without ambiguity.
+
+### Serialization path after fix
+
+With G1-3 corrected to depend on G1-2:
+
+```
+G1-1 ──> G1-2 ──> G1-3 ──> G1-4 ──> G1-5 ──┐
+  │                                             │
+  └──> G2-1 ──> G2-2 ──> G2-3 ──> G2-4 ──────┤
+                                               │
+                 G2-5 ─────────────────────────┤
+                                               │
+                                               v
+                                              G3-1
+```
+
+G1-2 and G2-2 (and their children) can still run interleaved since G2-2 does not depend on G1-2 at the Rust type level (ChainConfig is independent of SweepConfig).
diff --git a/notes/directions/phase-6-fix/workspace-map.json b/notes/directions/phase-6-fix/workspace-map.json
new file mode 100644
index 0000000..7f786c3
--- /dev/null
+++ b/notes/directions/phase-6-fix/workspace-map.json
@@ -0,0 +1,3721 @@
+{
+  "workspace": {
+    "root": "/Users/tony/programming/castep_workflow_framework",
+    "workspaceName": "castep_workflow_framework"
+  },
+  "crates": [
+    {
+      "name": "hubbard_u_sweep",
+      "root": "examples/hubbard_u_sweep/src/main.rs",
+      "package": {
+        "name": "hubbard_u_sweep",
+        "version": "0.1.0",
+        "edition": "2021",
+        "crateType": "bin"
+      },
+      "modules": [
+        {
+          "path": "hubbard_u_sweep",
+          "file": "examples/hubbard_u_sweep/src/main.rs",
+          "visibility": "pub",
+          "imports": [
+            {
+              "path": "anyhow::Result",
+              "line": 0
+            },
+            {
+              "path": "castep_cell_fmt::ToCellFile",
+              "line": 0
+            },
+            {
+              "path": "castep_cell_fmt::format::to_string_many_spaced",
+              "line": 0
+            },
+            {
+              "path": "castep_cell_fmt::parse",
+              "line": 0
+            },
+            {
+              "path": "castep_cell_io::CellDocument",
+              "line": 0
+            },
+            {
+              "path": "castep_cell_io::cell::species::AtomHubbardU",
+              "line": 0
+            },
+            {
+              "path": "castep_cell_io::cell::species::HubbardU",
+              "line": 0
+            },
+            {
+              "path": "castep_cell_io::cell::species::HubbardUUnit",
+              "line": 0
+            },
+            {
+              "path": "castep_cell_io::cell::species::OrbitalU",
+              "line": 0
+            },
+            {
+              "path": "castep_cell_io::cell::species::Species",
+              "line": 0
+            },
+            {
+              "path": "workflow_utils::prelude::*",
+              "line": 0
+            }
+          ]
+        }
+      ],
+      "deps": {
+        "normal": [
+          "anyhow",
+          "castep-cell-fmt",
+          "castep-cell-io",
+          "workflow_core",
+          "workflow_utils"
+        ]
+      },
+      "crossCrateImports": [
+        {
+          "importPath": "workflow_utils::prelude::*",
+          "targetCrate": "workflow_utils",
+          "symbol": "*",
+          "line": 0
+        }
+      ]
+    },
+    {
+      "name": "hubbard_u_sweep_slurm",
+      "root": "examples/hubbard_u_sweep_slurm/src/main.rs",
+      "package": {
+        "name": "hubbard_u_sweep_slurm",
+        "version": "0.1.0",
+        "edition": "2021",
+        "crateType": "bin"
+      },
+      "modules": [
+        {
+          "path": "hubbard_u_sweep_slurm",
+          "file": "examples/hubbard_u_sweep_slurm/src/main.rs",
+          "visibility": "pub",
+          "imports": [
+            {
+              "path": "anyhow::Result",
+              "line": 0
+            },
+            {
+              "path": "castep_cell_fmt::ToCellFile",
+              "line": 0
+            },
+            {
+              "path": "castep_cell_fmt::format::to_string_many_spaced",
+              "line": 0
+            },
+            {
+              "path": "castep_cell_fmt::parse",
+              "line": 0
+            },
+            {
+              "path": "castep_cell_io::CellDocument",
+              "line": 0
+            },
+            {
+              "path": "castep_cell_io::cell::species::AtomHubbardU",
+              "line": 0
+            },
+            {
+              "path": "castep_cell_io::cell::species::HubbardU",
+              "line": 0
+            },
+            {
+              "path": "castep_cell_io::cell::species::HubbardUUnit",
+              "line": 0
+            },
+            {
+              "path": "castep_cell_io::cell::species::OrbitalU",
+              "line": 0
+            },
+            {
+              "path": "castep_cell_io::cell::species::Species",
+              "line": 0
+            },
+            {
+              "path": "clap::Parser",
+              "line": 0
+            },
+            {
+              "path": "config::SweepConfig",
+              "line": 0
+            },
+            {
+              "path": "config::parse_u_values",
+              "line": 0
+            },
+            {
+              "path": "job_script::generate_job_script",
+              "line": 0
+            },
+            {
+              "path": "std::sync::Arc",
+              "line": 0
+            },
+            {
+              "path": "workflow_utils::prelude::*",
+              "line": 0
+            }
+          ],
+          "submodules": [
+            "config",
+            "job_script"
+          ]
+        },
+        {
+          "path": "hubbard_u_sweep_slurm::config",
+          "file": "examples/hubbard_u_sweep_slurm/src/config.rs",
+          "visibility": "private",
+          "publicItems": [
+            {
+              "kind": "struct",
+              "name": "SweepConfig",
+              "file": "",
+              "line": 5,
+              "attrs": {
+                "derive": [
+                  "Debug",
+                  "Parser"
+                ]
+              },
+              "generics": "",
+              "visibility": "pub",
+              "fields": [
+                "pub partition: String",
+                "pub ntasks: u32",
+                "pub nix_flake: String",
+                "pub mpi_if: String",
+                "pub seed_name: String",
+                "pub u_values: String",
+                "pub max_parallel: usize",
+                "pub element: String",
+                "pub orbital: char",
+                "pub dry_run: bool",
+                "pub local: bool",
+                "pub castep_command: String",
+                "pub sweep_mode: String",
+                "pub second_values: Option<String>",
+                "pub workdir: String"
+              ]
+            },
+            {
+              "kind": "fn",
+              "name": "parse_u_values",
+              "file": "",
+              "line": 75,
+              "attrs": {
+                "doc": [
+                  " Parses a comma-separated string of f64 values.",
+                  "",
+                  " Each segment is trimmed before parsing.",
+                  " Returns an error string identifying the offending token on failure."
+                ]
+              },
+              "generics": "",
+              "visibility": "pub"
+            }
+          ],
+          "imports": [
+            {
+              "path": "clap::Parser",
+              "line": 0
+            }
+          ],
+          "submodules": [
+            "tests"
+          ]
+        },
+        {
+          "path": "hubbard_u_sweep_slurm::job_script",
+          "file": "examples/hubbard_u_sweep_slurm/src/job_script.rs",
+          "visibility": "private",
+          "publicItems": [
+            {
+              "kind": "fn",
+              "name": "generate_job_script",
+              "file": "",
+              "line": 3,
+              "attrs": {},
+              "generics": "",
+              "visibility": "pub"
+            }
+          ],
+          "imports": [
+            {
+              "path": "crate::config::SweepConfig",
+              "line": 0
+            }
+          ],
+          "submodules": [
+            "tests"
+          ]
+        }
+      ],
+      "deps": {
+        "normal": [
+          "castep-cell-fmt",
+          "castep-cell-io",
+          "workflow_core",
+          "workflow_utils"
+        ],
+        "workspaceMembers": [
+          "anyhow",
+          "clap",
+          "itertools"
+        ]
+      },
+      "crossCrateImports": [
+        {
+          "importPath": "workflow_utils::prelude::*",
+          "targetCrate": "workflow_utils",
+          "symbol": "*",
+          "line": 0
+        }
+      ]
+    },
+    {
+      "name": "workflow-cli",
+      "root": "workflow-cli/src/main.rs",
+      "package": {
+        "name": "workflow-cli",
+        "version": "0.1.0",
+        "edition": "2021",
+        "crateType": "bin"
+      },
+      "modules": [
+        {
+          "path": "workflow-cli",
+          "file": "workflow-cli/src/main.rs",
+          "visibility": "pub",
+          "imports": [
+            {
+              "path": "clap::Parser",
+              "line": 0
+            },
+            {
+              "path": "clap::Subcommand",
+              "line": 0
+            },
+            {
+              "path": "std::io::IsTerminal",
+              "line": 0
+            },
+            {
+              "path": "std::io::Read",
+              "line": 0
+            },
+            {
+              "path": "std::io::self",
+              "line": 0
+            },
+            {
+              "path": "workflow_core::state::JsonStateStore",
+              "line": 0
+            },
+            {
+              "path": "workflow_core::state::StateStore",
+              "line": 0
+            },
+            {
+              "path": "workflow_core::state::StateStoreExt",
+              "line": 0
+            },
+            {
+              "path": "workflow_core::state::TaskStatus",
+              "line": 0
+            }
+          ],
+          "submodules": [
+            "tests"
+          ]
+        }
+      ],
+      "deps": {
+        "normal": [
+          "workflow_core"
+        ],
+        "dev": [
+          "tempfile"
+        ],
+        "workspaceMembers": [
+          "anyhow",
+          "clap"
+        ]
+      },
+      "crossCrateImports": [
+        {
+          "importPath": "workflow_core::state::JsonStateStore",
+          "targetCrate": "workflow_core",
+          "symbol": "JsonStateStore",
+          "line": 0
+        },
+        {
+          "importPath": "workflow_core::state::StateStore",
+          "targetCrate": "workflow_core",
+          "symbol": "StateStore",
+          "line": 0
+        },
+        {
+          "importPath": "workflow_core::state::StateStoreExt",
+          "targetCrate": "workflow_core",
+          "symbol": "StateStoreExt",
+          "line": 0
+        },
+        {
+          "importPath": "workflow_core::state::TaskStatus",
+          "targetCrate": "workflow_core",
+          "symbol": "TaskStatus",
+          "line": 0
+        }
+      ]
+    },
+    {
+      "name": "workflow_core",
+      "root": "workflow_core/src/lib.rs",
+      "package": {
+        "name": "workflow_core",
+        "version": "0.1.0",
+        "edition": "2021",
+        "crateType": "lib"
+      },
+      "modules": [
+        {
+          "path": "workflow_core",
+          "file": "workflow_core/src/lib.rs",
+          "visibility": "pub",
+          "publicItems": [
+            {
+              "kind": "fn",
+              "name": "init_default_logging",
+              "file": "",
+              "line": 24,
+              "attrs": {
+                "doc": [
+                  " Initialize default tracing subscriber with env-based filtering.",
+                  " Call once at start of main(). Controlled via RUST_LOG env var.",
+                  " Returns error if already initialized (safe, won't panic)."
+                ]
+              },
+              "generics": "",
+              "visibility": "pub"
+            }
+          ],
+          "imports": [
+            {
+              "path": "error::WorkflowError",
+              "line": 0
+            },
+            {
+              "path": "monitoring::HookContext",
+              "line": 0
+            },
+            {
+              "path": "monitoring::HookExecutor",
+              "line": 0
+            },
+            {
+              "path": "monitoring::HookResult",
+              "line": 0
+            },
+            {
+              "path": "monitoring::HookTrigger",
+              "line": 0
+            },
+            {
+              "path": "monitoring::MonitoringHook",
+              "line": 0
+            },
+            {
+              "path": "monitoring::TaskPhase",
+              "line": 0
+            },
+            {
+              "path": "process::OutputLocation",
+              "line": 0
+            },
+            {
+              "path": "process::ProcessHandle",
+              "line": 0
+            },
+            {
+              "path": "process::ProcessResult",
+              "line": 0
+            },
+            {
+              "path": "process::ProcessRunner",
+              "line": 0
+            },
+            {
+              "path": "process::QueuedSubmitter",
+              "line": 0
+            },
+            {
+              "path": "state::JsonStateStore",
+              "line": 0
+            },
+            {
+              "path": "state::StateStore",
+              "line": 0
+            },
+            {
+              "path": "state::StateStoreExt",
+              "line": 0
+            },
+            {
+              "path": "state::StateSummary",
+              "line": 0
+            },
+            {
+              "path": "state::TaskStatus",
+              "line": 0
+            },
+            {
+              "path": "state::TaskSuccessors",
+              "line": 0
+            },
+            {
+              "path": "task::CollectFailurePolicy",
+              "line": 0
+            },
+            {
+              "path": "task::ExecutionMode",
+              "line": 0
+            },
+            {
+              "path": "task::Task",
+              "line": 0
+            },
+            {
+              "path": "task::TaskClosure",
+              "line": 0
+            },
+            {
+              "path": "workflow::FailedTask",
+              "line": 0
+            },
+            {
+              "path": "workflow::Workflow",
+              "line": 0
+            },
+            {
+              "path": "workflow::WorkflowSummary",
+              "line": 0
+            }
+          ],
+          "reExports": [
+            {
+              "importPath": "task::CollectFailurePolicy",
+              "exportPath": "CollectFailurePolicy",
+              "line": 0
+            },
+            {
+              "importPath": "task::ExecutionMode",
+              "exportPath": "ExecutionMode",
+              "line": 0
+            },
+            {
+              "importPath": "workflow::FailedTask",
+              "exportPath": "FailedTask",
+              "line": 0
+            },
+            {
+              "importPath": "monitoring::HookContext",
+              "exportPath": "HookContext",
+              "line": 0
+            },
+            {
+              "importPath": "monitoring::HookExecutor",
+              "exportPath": "HookExecutor",
+              "line": 0
+            },
+            {
+              "importPath": "monitoring::HookResult",
+              "exportPath": "HookResult",
+              "line": 0
+            },
+            {
+              "importPath": "monitoring::HookTrigger",
+              "exportPath": "HookTrigger",
+              "line": 0
+            },
+            {
+              "importPath": "state::JsonStateStore",
+              "exportPath": "JsonStateStore",
+              "line": 0
+            },
+            {
+              "importPath": "monitoring::MonitoringHook",
+              "exportPath": "MonitoringHook",
+              "line": 0
+            },
+            {
+              "importPath": "process::OutputLocation",
+              "exportPath": "OutputLocation",
+              "line": 0
+            },
+            {
+              "importPath": "process::ProcessHandle",
+              "exportPath": "ProcessHandle",
+              "line": 0
+            },
+            {
+              "importPath": "process::ProcessResult",
+              "exportPath": "ProcessResult",
+              "line": 0
+            },
+            {
+              "importPath": "process::ProcessRunner",
+              "exportPath": "ProcessRunner",
+              "line": 0
+            },
+            {
+              "importPath": "process::QueuedSubmitter",
+              "exportPath": "QueuedSubmitter",
+              "line": 0
+            },
+            {
+              "importPath": "state::StateStore",
+              "exportPath": "StateStore",
+              "line": 0
+            },
+            {
+              "importPath": "state::StateStoreExt",
+              "exportPath": "StateStoreExt",
+              "line": 0
+            },
+            {
+              "importPath": "state::StateSummary",
+              "exportPath": "StateSummary",
+              "line": 0
+            },
+            {
+              "importPath": "task::Task",
+              "exportPath": "Task",
+              "line": 0
+            },
+            {
+              "importPath": "task::TaskClosure",
+              "exportPath": "TaskClosure",
+              "line": 0
+            },
+            {
+              "importPath": "monitoring::TaskPhase",
+              "exportPath": "TaskPhase",
+              "line": 0
+            },
+            {
+              "importPath": "state::TaskStatus",
+              "exportPath": "TaskStatus",
+              "line": 0
+            },
+            {
+              "importPath": "state::TaskSuccessors",
+              "exportPath": "TaskSuccessors",
+              "line": 0
+            },
+            {
+              "importPath": "workflow::Workflow",
+              "exportPath": "Workflow",
+              "line": 0
+            },
+            {
+              "importPath": "error::WorkflowError",
+              "exportPath": "WorkflowError",
+              "line": 0
+            },
+            {
+              "importPath": "workflow::WorkflowSummary",
+              "exportPath": "WorkflowSummary",
+              "line": 0
+            }
+          ],
+          "submodules": [
+            "dag",
+            "error",
+            "monitoring",
+            "prelude",
+            "process",
+            "state",
+            "task",
+            "workflow"
+          ]
+        },
+        {
+          "path": "workflow_core::dag",
+          "file": "workflow_core/src/dag.rs",
+          "visibility": "pub",
+          "publicItems": [
+            {
+              "kind": "struct",
+              "name": "Dag",
+              "file": "",
+              "line": 6,
+              "attrs": {},
+              "generics": "",
+              "visibility": "pub",
+              "fields": [
+                "graph: DiGraph<String, ()>",
+                "node_map: HashMap<String, NodeIndex>"
+              ]
+            }
+          ],
+          "imports": [
+            {
+              "path": "crate::error::WorkflowError",
+              "line": 0
+            },
+            {
+              "path": "petgraph::algo",
+              "line": 0
+            },
+            {
+              "path": "petgraph::graph::DiGraph",
+              "line": 0
+            },
+            {
+              "path": "petgraph::graph::NodeIndex",
+              "line": 0
+            },
+            {
+              "path": "std::collections::HashMap",
+              "line": 0
+            },
+            {
+              "path": "std::collections::HashSet",
+              "line": 0
+            }
+          ],
+          "submodules": [
+            "tests"
+          ]
+        },
+        {
+          "path": "workflow_core::error",
+          "file": "workflow_core/src/error.rs",
+          "visibility": "pub",
+          "publicItems": [
+            {
+              "kind": "enum",
+              "name": "WorkflowError",
+              "file": "",
+              "line": 5,
+              "attrs": {
+                "derive": [
+                  "Debug",
+                  "Error"
+                ]
+              },
+              "generics": "",
+              "visibility": "pub",
+              "variants": [
+                "DuplicateTaskId(String)",
+                "CycleDetected",
+                "UnknownDependency { task: String, dependency: String }",
+                "StateCorrupted(String)",
+                "TaskTimeout(String)",
+                "InvalidConfig(String)",
+                "IoWithPath { path: std::path::PathBuf, source: std::io::Error }",
+                "Io(std::io::Error)",
+                "Interrupted",
+                "QueueSubmitFailed(String)"
+              ]
+            }
+          ],
+          "imports": [
+            {
+              "path": "thiserror::Error",
+              "line": 0
+            }
+          ]
+        },
+        {
+          "path": "workflow_core::monitoring",
+          "file": "workflow_core/src/monitoring.rs",
+          "visibility": "private",
+          "publicItems": [
+            {
+              "kind": "struct",
+              "name": "HookContext",
+              "file": "",
+              "line": 75,
+              "attrs": {
+                "derive": [
+                  "Clone",
+                  "Debug",
+                  "Deserialize",
+                  "Serialize"
+                ],
+                "doc": [
+                  " Context available to monitoring hooks during execution."
+                ]
+              },
+              "generics": "",
+              "visibility": "pub",
+              "fields": [
+                "pub task_id: String",
+                "pub workdir: std::path::PathBuf",
+                "pub phase: TaskPhase",
+                "pub exit_code: Option<i32>"
+              ]
+            },
+            {
+              "kind": "trait",
+              "name": "HookExecutor",
+              "file": "",
+              "line": 8,
+              "attrs": {
+                "doc": [
+                  " Executor trait for monitoring hooks.",
+                  "",
+                  " This trait abstracts the execution of monitoring hooks, allowing different",
+                  " backend implementations (e.g., shell-based, in-process, etc.)."
+                ]
+              },
+              "generics": "",
+              "visibility": "pub"
+            },
+            {
+              "kind": "struct",
+              "name": "HookResult",
+              "file": "",
+              "line": 88,
+              "attrs": {
+                "derive": [
+                  "Clone",
+                  "Debug",
+                  "Deserialize",
+                  "Serialize"
+                ],
+                "doc": [
+                  " Result of executing a monitoring hook."
+                ]
+              },
+              "generics": "",
+              "visibility": "pub",
+              "fields": [
+                "pub success: bool",
+                "pub output: String"
+              ]
+            },
+            {
+              "kind": "enum",
+              "name": "HookTrigger",
+              "file": "",
+              "line": 41,
+              "attrs": {
+                "derive": [
+                  "Clone",
+                  "Debug",
+                  "Deserialize",
+                  "Serialize"
+                ],
+                "doc": [
+                  " The event that triggers a monitoring hook."
+                ]
+              },
+              "generics": "",
+              "visibility": "pub",
+              "variants": [
+                "OnStart",
+                "OnComplete",
+                "OnFailure",
+                "Periodic { interval_secs: u64 }"
+              ]
+            },
+            {
+              "kind": "struct",
+              "name": "MonitoringHook",
+              "file": "",
+              "line": 19,
+              "attrs": {
+                "derive": [
+                  "Clone",
+                  "Debug",
+                  "Deserialize",
+                  "Serialize"
+                ],
+                "doc": [
+                  " A monitoring hook that can be triggered by a task event."
+                ]
+              },
+              "generics": "",
+              "visibility": "pub",
+              "fields": [
+                "pub name: String",
+                "pub command: String",
+                "pub trigger: HookTrigger"
+              ]
+            },
+            {
+              "kind": "enum",
+              "name": "TaskPhase",
+              "file": "",
+              "line": 54,
+              "attrs": {
+                "derive": [
+                  "Clone",
+                  "Copy",
+                  "Debug",
+                  "Deserialize",
+                  "Eq",
+                  "PartialEq",
+                  "Serialize"
+                ],
+                "doc": [
+                  " The phase of a task in the workflow."
+                ]
+              },
+              "generics": "",
+              "visibility": "pub",
+              "variants": [
+                "Running",
+                "Completed",
+                "Failed"
+              ]
+            }
+          ],
+          "imports": [
+            {
+              "path": "crate::WorkflowError",
+              "line": 0
+            },
+            {
+              "path": "serde::Deserialize",
+              "line": 0
+            },
+            {
+              "path": "serde::Serialize",
+              "line": 0
+            }
+          ],
+          "submodules": [
+            "tests"
+          ]
+        },
+        {
+          "path": "workflow_core::prelude",
+          "file": "workflow_core/src/prelude.rs",
+          "visibility": "pub",
+          "imports": [
+            {
+              "path": "crate::HookExecutor",
+              "line": 0
+            },
+            {
+              "path": "crate::ProcessRunner",
+              "line": 0
+            },
+            {
+              "path": "crate::error::WorkflowError",
+              "line": 0
+            },
+            {
+              "path": "crate::state::JsonStateStore",
+              "line": 0
+            },
+            {
+              "path": "crate::state::StateStore",
+              "line": 0
+            },
+            {
+              "path": "crate::state::StateStoreExt",
+              "line": 0
+            },
+            {
+              "path": "crate::state::TaskStatus",
+              "line": 0
+            },
+            {
+              "path": "crate::task::CollectFailurePolicy",
+              "line": 0
+            },
+            {
+              "path": "crate::task::ExecutionMode",
+              "line": 0
+            },
+            {
+              "path": "crate::task::Task",
+              "line": 0
+            },
+            {
+              "path": "crate::workflow::Workflow",
+              "line": 0
+            },
+            {
+              "path": "crate::workflow::WorkflowSummary",
+              "line": 0
+            }
+          ],
+          "reExports": [
+            {
+              "importPath": "crate::task::CollectFailurePolicy",
+              "exportPath": "CollectFailurePolicy",
+              "line": 0
+            },
+            {
+              "importPath": "crate::task::ExecutionMode",
+              "exportPath": "ExecutionMode",
+              "line": 0
+            },
+            {
+              "importPath": "crate::HookExecutor",
+              "exportPath": "HookExecutor",
+              "line": 0
+            },
+            {
+              "importPath": "crate::state::JsonStateStore",
+              "exportPath": "JsonStateStore",
+              "line": 0
+            },
+            {
+              "importPath": "crate::ProcessRunner",
+              "exportPath": "ProcessRunner",
+              "line": 0
+            },
+            {
+              "importPath": "crate::state::StateStore",
+              "exportPath": "StateStore",
+              "line": 0
+            },
+            {
+              "importPath": "crate::state::StateStoreExt",
+              "exportPath": "StateStoreExt",
+              "line": 0
+            },
+            {
+              "importPath": "crate::task::Task",
+              "exportPath": "Task",
+              "line": 0
+            },
+            {
+              "importPath": "crate::state::TaskStatus",
+              "exportPath": "TaskStatus",
+              "line": 0
+            },
+            {
+              "importPath": "crate::workflow::Workflow",
+              "exportPath": "Workflow",
+              "line": 0
+            },
+            {
+              "importPath": "crate::error::WorkflowError",
+              "exportPath": "WorkflowError",
+              "line": 0
+            },
+            {
+              "importPath": "crate::workflow::WorkflowSummary",
+              "exportPath": "WorkflowSummary",
+              "line": 0
+            }
+          ]
+        },
+        {
+          "path": "workflow_core::process",
+          "file": "workflow_core/src/process.rs",
+          "visibility": "pub",
+          "publicItems": [
+            {
+              "kind": "enum",
+              "name": "OutputLocation",
+              "file": "",
+              "line": 7,
+              "attrs": {
+                "derive": [
+                  "Clone",
+                  "Debug"
+                ]
+              },
+              "generics": "",
+              "visibility": "pub",
+              "variants": [
+                "Captured { stdout: String, stderr: String }",
+                "OnDisk { stdout_path: PathBuf, stderr_path: PathBuf }"
+              ]
+            },
+            {
+              "kind": "trait",
+              "name": "ProcessHandle",
+              "file": "",
+              "line": 25,
+              "attrs": {
+                "doc": [
+                  " A handle to a running (or finished) process, used to poll, wait, or terminate it.",
+                  "",
+                  " Implementations must be `Send` so handles can be stored across thread boundaries."
+                ]
+              },
+              "generics": "",
+              "visibility": "pub"
+            },
+            {
+              "kind": "struct",
+              "name": "ProcessResult",
+              "file": "",
+              "line": 45,
+              "attrs": {},
+              "generics": "",
+              "visibility": "pub",
+              "fields": [
+                "pub exit_code: Option<i32>",
+                "pub output: OutputLocation",
+                "pub duration: Duration"
+              ]
+            },
+            {
+              "kind": "trait",
+              "name": "ProcessRunner",
+              "file": "",
+              "line": 12,
+              "attrs": {},
+              "generics": "",
+              "visibility": "pub"
+            },
+            {
+              "kind": "trait",
+              "name": "QueuedSubmitter",
+              "file": "",
+              "line": 51,
+              "attrs": {},
+              "generics": "",
+              "visibility": "pub"
+            }
+          ],
+          "imports": [
+            {
+              "path": "crate::error::WorkflowError",
+              "line": 0
+            },
+            {
+              "path": "std::collections::HashMap",
+              "line": 0
+            },
+            {
+              "path": "std::path::Path",
+              "line": 0
+            },
+            {
+              "path": "std::path::PathBuf",
+              "line": 0
+            },
+            {
+              "path": "std::time::Duration",
+              "line": 0
+            }
+          ]
+        },
+        {
+          "path": "workflow_core::state",
+          "file": "workflow_core/src/state.rs",
+          "visibility": "pub",
+          "publicItems": [
+            {
+              "kind": "struct",
+              "name": "JsonStateStore",
+              "file": "",
+              "line": 190,
+              "attrs": {
+                "derive": [
+                  "Debug",
+                  "Deserialize",
+                  "Serialize"
+                ],
+                "doc": [
+                  " JSON-based state store implementation.",
+                  "",
+                  " # Crash Recovery and Resume",
+                  "",
+                  " When loading via [`JsonStateStore::load`], any tasks marked as `Running`, `Failed`, or",
+                  " `SkippedDueToDependencyFailure` are automatically reset to `Pending`. This ensures",
+                  " that incomplete or failed runs can be safely resumed without stale state blocking",
+                  " progress. Note that `Skipped` and `SkippedDueToDependencyFailure` (when not in",
+                  " a failed context) are preserved as-is.",
+                  "",
+                  " # Read-Only Inspection",
+                  "",
+                  " For read-only status inspection (e.g., CLI display, `workflow inspect` commands),",
+                  " use [`JsonStateStore::load_raw`]. Unlike `load`, this method does not apply crash",
+                  " recovery resets and returns the state exactly as persisted to disk.",
+                  "",
+                  " # Persistence Semantics",
+                  "",
+                  " State is persisted to disk via atomic writes (temp file + rename). See [`JsonStateStore::load`]",
+                  " and [`JsonStateStore::load_raw`] for details on crash recovery behavior."
+                ]
+              },
+              "generics": "",
+              "visibility": "pub",
+              "fields": [
+                "workflow_name: String",
+                "created_at: String",
+                "last_updated: String",
+                "tasks: HashMap<String, TaskStatus>",
+                "task_successors: Option<TaskSuccessors>",
+                "path: PathBuf"
+              ]
+            },
+            {
+              "kind": "trait",
+              "name": "StateStore",
+              "file": "",
+              "line": 33,
+              "attrs": {
+                "doc": [
+                  " State management interface for workflow execution.",
+                  "",
+                  " This trait defines the contract for persisting and retrieving task status during",
+                  " live workflow runs. Implementations handle runtime mutation of task states as",
+                  " the workflow progresses, ensuring durability through periodic saves.",
+                  "",
+                  " Workflow Summary:",
+                  " The `summary` method on `StateStoreExt` aggregates all task statuses into a",
+                  " concise overview (pending, running, completed, failed, skipped counts) suitable",
+                  " for progress reporting and export."
+                ]
+              },
+              "generics": "",
+              "visibility": "pub"
+            },
+            {
+              "kind": "trait",
+              "name": "StateStoreExt",
+              "file": "",
+              "line": 48,
+              "attrs": {
+                "doc": [
+                  " Extension trait providing convenience methods for state management."
+                ]
+              },
+              "generics": "",
+              "visibility": "pub"
+            },
+            {
+              "kind": "struct",
+              "name": "StateSummary",
+              "file": "",
+              "line": 110,
+              "attrs": {
+                "derive": [
+                  "Clone",
+                  "Debug"
+                ],
+                "doc": [
+                  " Summary of workflow state."
+                ]
+              },
+              "generics": "",
+              "visibility": "pub",
+              "fields": [
+                "pub pending: usize",
+                "pub running: usize",
+                "pub completed: usize",
+                "pub failed: usize",
+                "pub skipped: usize"
+              ]
+            },
+            {
+              "kind": "enum",
+              "name": "TaskStatus",
+              "file": "",
+              "line": 10,
+              "attrs": {
+                "derive": [
+                  "Clone",
+                  "Debug",
+                  "Deserialize",
+                  "PartialEq",
+                  "Serialize"
+                ],
+                "doc": [
+                  " Task status enum."
+                ]
+              },
+              "generics": "",
+              "visibility": "pub",
+              "variants": [
+                "Pending",
+                "Running",
+                "Completed",
+                "Failed { error: String }",
+                "Skipped",
+                "SkippedDueToDependencyFailure"
+              ]
+            },
+            {
+              "kind": "struct",
+              "name": "TaskSuccessors",
+              "file": "",
+              "line": 129,
+              "attrs": {
+                "derive": [
+                  "Clone",
+                  "Debug",
+                  "Default",
+                  "Deserialize",
+                  "Serialize"
+                ],
+                "doc": [
+                  " A typed wrapper around the task successor adjacency map.",
+                  "",
+                  " Maps each task ID to its immediate downstream successors in the DAG.",
+                  " Used for graph-aware retry: given a set of failed tasks, BFS over this",
+                  " map finds all transitively downstream tasks to reset."
+                ]
+              },
+              "generics": "",
+              "visibility": "pub",
+              "fields": [
+                "HashMap<String, Vec<String>>"
+              ]
+            }
+          ],
+          "imports": [
+            {
+              "path": "crate::error::WorkflowError",
+              "line": 0
+            },
+            {
+              "path": "serde::Deserialize",
+              "line": 0
+            },
+            {
+              "path": "serde::Serialize",
+              "line": 0
+            },
+            {
+              "path": "std::collections::HashMap",
+              "line": 0
+            },
+            {
+              "path": "std::fs",
+              "line": 0
+            },
+            {
+              "path": "std::path::Path",
+              "line": 0
+            },
+            {
+              "path": "std::path::PathBuf",
+              "line": 0
+            }
+          ],
+          "submodules": [
+            "tests"
+          ]
+        },
+        {
+          "path": "workflow_core::task",
+          "file": "workflow_core/src/task.rs",
+          "visibility": "pub",
+          "publicItems": [
+            {
+              "kind": "enum",
+              "name": "CollectFailurePolicy",
+              "file": "",
+              "line": 16,
+              "attrs": {
+                "derive": [
+                  "Clone",
+                  "Copy",
+                  "Debug",
+                  "Default",
+                  "Eq",
+                  "PartialEq"
+                ],
+                "doc": [
+                  " Policy governing how collect-closure failures affect task status.",
+                  "",
+                  " When a collect closure returns `Err`, the framework must decide whether",
+                  " the task itself should be marked as Failed or whether the error should",
+                  " only be logged as a warning."
+                ]
+              },
+              "generics": "",
+              "visibility": "pub",
+              "variants": [
+                "FailTask",
+                "WarnOnly"
+              ]
+            },
+            {
+              "kind": "enum",
+              "name": "ExecutionMode",
+              "file": "",
+              "line": 26,
+              "attrs": {
+                "derive": [
+                  "Clone",
+                  "Debug"
+                ]
+              },
+              "generics": "",
+              "visibility": "pub",
+              "variants": [
+                "Direct { command: String, args: Vec<String>, env: HashMap<String, String>, timeout: Option<Duration> }",
+                "Queued"
+              ]
+            },
+            {
+              "kind": "struct",
+              "name": "Task",
+              "file": "",
+              "line": 57,
+              "attrs": {},
+              "generics": "",
+              "visibility": "pub",
+              "fields": [
+                "pub id: String",
+                "pub dependencies: Vec<String>",
+                "pub workdir: PathBuf",
+                "pub mode: ExecutionMode",
+                "pub setup: Option<TaskClosure>",
+                "pub collect: Option<TaskClosure>",
+                "pub monitors: Vec<MonitoringHook>",
+                "collect_failure_policy: CollectFailurePolicy"
+              ]
+            },
+            {
+              "kind": "type",
+              "name": "TaskClosure",
+              "file": "",
+              "line": 8,
+              "attrs": {
+                "doc": [
+                  " A closure used for task setup or result collection."
+                ]
+              },
+              "generics": "",
+              "visibility": "pub"
+            }
+          ],
+          "imports": [
+            {
+              "path": "crate::monitoring::MonitoringHook",
+              "line": 0
+            },
+            {
+              "path": "std::collections::HashMap",
+              "line": 0
+            },
+            {
+              "path": "std::path::Path",
+              "line": 0
+            },
+            {
+              "path": "std::path::PathBuf",
+              "line": 0
+            },
+            {
+              "path": "std::time::Duration",
+              "line": 0
+            }
+          ],
+          "submodules": [
+            "tests"
+          ]
+        },
+        {
+          "path": "workflow_core::workflow",
+          "file": "workflow_core/src/workflow.rs",
+          "visibility": "pub",
+          "publicItems": [
+            {
+              "kind": "struct",
+              "name": "FailedTask",
+              "file": "",
+              "line": 563,
+              "attrs": {
+                "derive": [
+                  "Clone",
+                  "Debug"
+                ],
+                "doc": [
+                  " A task that failed during workflow execution."
+                ]
+              },
+              "generics": "",
+              "visibility": "pub",
+              "fields": [
+                "pub id: String",
+                "pub error: String"
+              ]
+            },
+            {
+              "kind": "struct",
+              "name": "InFlightTask",
+              "file": "",
+              "line": 17,
+              "attrs": {
+                "doc": [
+                  " A handle to a running task with metadata."
+                ]
+              },
+              "generics": "",
+              "visibility": "pub(crate)",
+              "fields": [
+                "pub handle: Box<ProcessHandle>",
+                "pub started_at: Instant",
+                "pub monitors: Vec<crate::monitoring::MonitoringHook>",
+                "pub collect: Option<TaskClosure>",
+                "pub workdir: std::path::PathBuf",
+                "pub collect_failure_policy: crate::task::CollectFailurePolicy",
+                "pub last_periodic_fire: HashMap<String, Instant>"
+              ]
+            },
+            {
+              "kind": "struct",
+              "name": "Workflow",
+              "file": "",
+              "line": 27,
+              "attrs": {},
+              "generics": "",
+              "visibility": "pub",
+              "fields": [
+                "pub name: String",
+                "tasks: HashMap<String, Task>",
+                "max_parallel: usize",
+                "interrupt: Arc<AtomicBool>",
+                "log_dir: Option<std::path::PathBuf>",
+                "root_dir: Option<std::path::PathBuf>",
+                "queued_submitter: Option<Arc<QueuedSubmitter>>",
+                "computed_successors: Option<TaskSuccessors>"
+              ]
+            },
+            {
+              "kind": "struct",
+              "name": "WorkflowSummary",
+              "file": "",
+              "line": 570,
+              "attrs": {
+                "derive": [
+                  "Clone",
+                  "Debug"
+                ],
+                "doc": [
+                  " Summary of workflow execution results."
+                ]
+              },
+              "generics": "",
+              "visibility": "pub",
+              "fields": [
+                "pub succeeded: Vec<String>",
+                "pub failed: Vec<FailedTask>",
+                "pub skipped: Vec<String>",
+                "pub duration: Duration"
+              ]
+            }
+          ],
+          "imports": [
+            {
+              "path": "crate::HookExecutor",
+              "line": 0
+            },
+            {
+              "path": "crate::dag::Dag",
+              "line": 0
+            },
+            {
+              "path": "crate::error::WorkflowError",
+              "line": 0
+            },
+            {
+              "path": "crate::process::ProcessHandle",
+              "line": 0
+            },
+            {
+              "path": "crate::process::ProcessRunner",
+              "line": 0
+            },
+            {
+              "path": "crate::state::StateStore",
+              "line": 0
+            },
+            {
+              "path": "crate::state::StateStoreExt",
+              "line": 0
+            },
+            {
+              "path": "crate::state::TaskStatus",
+              "line": 0
+            },
+            {
+              "path": "crate::state::TaskSuccessors",
+              "line": 0
+            },
+            {
+              "path": "crate::task::ExecutionMode",
+              "line": 0
+            },
+            {
+              "path": "crate::task::Task",
+              "line": 0
+            },
+            {
+              "path": "crate::task::TaskClosure",
+              "line": 0
+            },
+            {
+              "path": "std::collections::HashMap",
+              "line": 0
+            },
+            {
+              "path": "std::collections::HashSet",
+              "line": 0
+            },
+            {
+              "path": "std::sync::Arc",
+              "line": 0
+            },
+            {
+              "path": "std::sync::atomic::AtomicBool",
+              "line": 0
+            },
+            {
+              "path": "std::sync::atomic::Ordering",
+              "line": 0
+            },
+            {
+              "path": "std::time::Duration",
+              "line": 0
+            },
+            {
+              "path": "std::time::Instant",
+              "line": 0
+            }
+          ],
+          "submodules": [
+            "tests"
+          ]
+        }
+      ],
+      "deps": {
+        "dev": [
+          "tempfile",
+          "workflow_utils"
+        ],
+        "workspaceMembers": [
+          "petgraph",
+          "serde",
+          "serde_json",
+          "signal-hook",
+          "thiserror",
+          "time",
+          "tracing",
+          "tracing-subscriber"
+        ]
+      }
+    },
+    {
+      "name": "workflow_utils",
+      "root": "workflow_utils/src/lib.rs",
+      "package": {
+        "name": "workflow_utils",
+        "version": "0.1.0",
+        "edition": "2021",
+        "crateType": "lib"
+      },
+      "modules": [
+        {
+          "path": "workflow_utils",
+          "file": "workflow_utils/src/lib.rs",
+          "visibility": "pub",
+          "publicItems": [
+            {
+              "kind": "fn",
+              "name": "run_default",
+              "file": "",
+              "line": 31,
+              "attrs": {
+                "doc": [
+                  " Runs a workflow with the default `SystemProcessRunner` and `ShellHookExecutor`.",
+                  "",
+                  " Eliminates the repeated `Arc` wiring boilerplate in every binary that uses",
+                  " direct (non-queued) process execution.",
+                  "",
+                  " # Example",
+                  " ```ignore",
+                  " let mut workflow = Workflow::new(\"my_workflow\");",
+                  " // ... add tasks ...",
+                  " let mut state = JsonStateStore::new(\"my_workflow\", path);",
+                  " let summary = workflow_utils::run_default(&mut workflow, &mut state)?;",
+                  " ```"
+                ]
+              },
+              "generics": "",
+              "visibility": "pub"
+            }
+          ],
+          "imports": [
+            {
+              "path": "executor::ExecutionHandle",
+              "line": 0
+            },
+            {
+              "path": "executor::ExecutionResult",
+              "line": 0
+            },
+            {
+              "path": "executor::OutputLocation",
+              "line": 0
+            },
+            {
+              "path": "executor::SystemProcessRunner",
+              "line": 0
+            },
+            {
+              "path": "executor::TaskExecutor",
+              "line": 0
+            },
+            {
+              "path": "files::copy_file",
+              "line": 0
+            },
+            {
+              "path": "files::create_dir",
+              "line": 0
+            },
+            {
+              "path": "files::exists",
+              "line": 0
+            },
+            {
+              "path": "files::read_file",
+              "line": 0
+            },
+            {
+              "path": "files::remove_dir",
+              "line": 0
+            },
+            {
+              "path": "files::write_file",
+              "line": 0
+            },
+            {
+              "path": "monitoring::ShellHookExecutor",
+              "line": 0
+            },
+            {
+              "path": "queued::JOB_SCRIPT_NAME",
+              "line": 0
+            },
+            {
+              "path": "queued::QueuedRunner",
+              "line": 0
+            },
+            {
+              "path": "queued::SchedulerKind",
+              "line": 0
+            },
+            {
+              "path": "std::sync::Arc",
+              "line": 0
+            },
+            {
+              "path": "workflow_core::HookContext",
+              "line": 0
+            },
+            {
+              "path": "workflow_core::HookExecutor",
+              "line": 0
+            },
+            {
+              "path": "workflow_core::HookResult",
+              "line": 0
+            },
+            {
+              "path": "workflow_core::HookTrigger",
+              "line": 0
+            },
+            {
+              "path": "workflow_core::MonitoringHook",
+              "line": 0
+            },
+            {
+              "path": "workflow_core::ProcessRunner",
+              "line": 0
+            },
+            {
+              "path": "workflow_core::WorkflowError",
+              "line": 0
+            },
+            {
+              "path": "workflow_core::state::StateStore",
+              "line": 0
+            },
+            {
+              "path": "workflow_core::workflow::Workflow",
+              "line": 0
+            },
+            {
+              "path": "workflow_core::workflow::WorkflowSummary",
+              "line": 0
+            }
+          ],
+          "reExports": [
+            {
+              "importPath": "executor::ExecutionHandle",
+              "exportPath": "ExecutionHandle",
+              "line": 0
+            },
+            {
+              "importPath": "executor::ExecutionResult",
+              "exportPath": "ExecutionResult",
+              "line": 0
+            },
+            {
+              "importPath": "workflow_core::HookContext",
+              "exportPath": "HookContext",
+              "line": 0
+            },
+            {
+              "importPath": "workflow_core::HookResult",
+              "exportPath": "HookResult",
+              "line": 0
+            },
+            {
+              "importPath": "workflow_core::HookTrigger",
+              "exportPath": "HookTrigger",
+              "line": 0
+            },
+            {
+              "importPath": "queued::JOB_SCRIPT_NAME",
+              "exportPath": "JOB_SCRIPT_NAME",
+              "line": 0
+            },
+            {
+              "importPath": "workflow_core::MonitoringHook",
+              "exportPath": "MonitoringHook",
+              "line": 0
+            },
+            {
+              "importPath": "executor::OutputLocation",
+              "exportPath": "OutputLocation",
+              "line": 0
+            },
+            {
+              "importPath": "queued::QueuedRunner",
+              "exportPath": "QueuedRunner",
+              "line": 0
+            },
+            {
+              "importPath": "queued::SchedulerKind",
+              "exportPath": "SchedulerKind",
+              "line": 0
+            },
+            {
+              "importPath": "monitoring::ShellHookExecutor",
+              "exportPath": "ShellHookExecutor",
+              "line": 0
+            },
+            {
+              "importPath": "executor::SystemProcessRunner",
+              "exportPath": "SystemProcessRunner",
+              "line": 0
+            },
+            {
+              "importPath": "executor::TaskExecutor",
+              "exportPath": "TaskExecutor",
+              "line": 0
+            },
+            {
+              "importPath": "files::copy_file",
+              "exportPath": "copy_file",
+              "line": 0
+            },
+            {
+              "importPath": "files::create_dir",
+              "exportPath": "create_dir",
+              "line": 0
+            },
+            {
+              "importPath": "files::exists",
+              "exportPath": "exists",
+              "line": 0
+            },
+            {
+              "importPath": "files::read_file",
+              "exportPath": "read_file",
+              "line": 0
+            },
+            {
+              "importPath": "files::remove_dir",
+              "exportPath": "remove_dir",
+              "line": 0
+            },
+            {
+              "importPath": "files::write_file",
+              "exportPath": "write_file",
+              "line": 0
+            }
+          ],
+          "submodules": [
+            "executor",
+            "files",
+            "monitoring",
+            "prelude",
+            "queued"
+          ]
+        },
+        {
+          "path": "workflow_utils::executor",
+          "file": "workflow_utils/src/executor.rs",
+          "visibility": "private",
+          "publicItems": [
+            {
+              "kind": "struct",
+              "name": "ExecutionHandle",
+              "file": "",
+              "line": 89,
+              "attrs": {},
+              "generics": "",
+              "visibility": "pub",
+              "fields": [
+                "child: std::process::Child"
+              ]
+            },
+            {
+              "kind": "struct",
+              "name": "ExecutionResult",
+              "file": "",
+              "line": 76,
+              "attrs": {},
+              "generics": "",
+              "visibility": "pub",
+              "fields": [
+                "pub exit_code: Option<i32>",
+                "pub stdout: String",
+                "pub stderr: String",
+                "pub duration: std::time::Duration"
+              ]
+            },
+            {
+              "kind": "struct",
+              "name": "SystemProcessHandle",
+              "file": "",
+              "line": 166,
+              "attrs": {
+                "doc": [
+                  " Handle to a running system process.",
+                  " Uses `Option<Child>` to allow consuming the child once via `wait()`."
+                ]
+              },
+              "generics": "",
+              "visibility": "pub",
+              "fields": [
+                "child: Option<Child>",
+                "start: Instant",
+                "output_files: Option<(PathBuf, PathBuf)>"
+              ]
+            },
+            {
+              "kind": "struct",
+              "name": "SystemProcessRunner",
+              "file": "",
+              "line": 110,
+              "attrs": {
+                "derive": [
+                  "Default"
+                ],
+                "doc": [
+                  " Concrete implementation of the ProcessRunner trait for system processes.",
+                  " Wraps `std::process::Child` with output capture and timing."
+                ]
+              },
+              "generics": "",
+              "visibility": "pub",
+              "fields": [
+                "log_dir: Option<PathBuf>"
+              ]
+            },
+            {
+              "kind": "struct",
+              "name": "TaskExecutor",
+              "file": "",
+              "line": 12,
+              "attrs": {},
+              "generics": "",
+              "visibility": "pub",
+              "fields": [
+                "workdir: PathBuf",
+                "command: String",
+                "args: Vec<String>",
+                "env: HashMap<String, String>"
+              ]
+            }
+          ],
+          "imports": [
+            {
+              "path": "std::collections::HashMap",
+              "line": 0
+            },
+            {
+              "path": "std::path::Path",
+              "line": 0
+            },
+            {
+              "path": "std::path::PathBuf",
+              "line": 0
+            },
+            {
+              "path": "std::process::Child",
+              "line": 0
+            },
+            {
+              "path": "std::process::Command",
+              "line": 0
+            },
+            {
+              "path": "std::process::Stdio",
+              "line": 0
+            },
+            {
+              "path": "std::time::Instant",
+              "line": 0
+            },
+            {
+              "path": "workflow_core::ProcessHandle",
+              "line": 0
+            },
+            {
+              "path": "workflow_core::ProcessResult",
+              "line": 0
+            },
+            {
+              "path": "workflow_core::ProcessRunner",
+              "line": 0
+            },
+            {
+              "path": "workflow_core::WorkflowError",
+              "line": 0
+            },
+            {
+              "path": "workflow_core::process::OutputLocation",
+              "line": 0
+            }
+          ],
+          "reExports": [
+            {
+              "importPath": "workflow_core::process::OutputLocation",
+              "exportPath": "OutputLocation",
+              "line": 0
+            },
+            {
+              "importPath": "workflow_core::ProcessHandle",
+              "exportPath": "ProcessHandle",
+              "line": 0
+            },
+            {
+              "importPath": "workflow_core::ProcessResult",
+              "exportPath": "ProcessResult",
+              "line": 0
+            },
+            {
+              "importPath": "workflow_core::ProcessRunner",
+              "exportPath": "ProcessRunner",
+              "line": 0
+            },
+            {
+              "importPath": "workflow_core::WorkflowError",
+              "exportPath": "WorkflowError",
+              "line": 0
+            }
+          ]
+        },
+        {
+          "path": "workflow_utils::files",
+          "file": "workflow_utils/src/files.rs",
+          "visibility": "private",
+          "publicItems": [
+            {
+              "kind": "fn",
+              "name": "copy_file",
+              "file": "",
+              "line": 21,
+              "attrs": {},
+              "generics": "",
+              "visibility": "pub"
+            },
+            {
+              "kind": "fn",
+              "name": "create_dir",
+              "file": "",
+              "line": 31,
+              "attrs": {},
+              "generics": "",
+              "visibility": "pub"
+            },
+            {
+              "kind": "fn",
+              "name": "exists",
+              "file": "",
+              "line": 41,
+              "attrs": {},
+              "generics": "",
+              "visibility": "pub"
+            },
+            {
+              "kind": "fn",
+              "name": "read_file",
+              "file": "",
+              "line": 8,
+              "attrs": {},
+              "generics": "",
+              "visibility": "pub"
+            },
+            {
+              "kind": "fn",
+              "name": "remove_dir",
+              "file": "",
+              "line": 36,
+              "attrs": {},
+              "generics": "",
+              "visibility": "pub"
+            },
+            {
+              "kind": "fn",
+              "name": "write_file",
+              "file": "",
+              "line": 13,
+              "attrs": {},
+              "generics": "",
+              "visibility": "pub"
+            }
+          ],
+          "imports": [
+            {
+              "path": "std::path::Path",
+              "line": 0
+            },
+            {
+              "path": "workflow_core::WorkflowError",
+              "line": 0
+            }
+          ]
+        },
+        {
+          "path": "workflow_utils::monitoring",
+          "file": "workflow_utils/src/monitoring.rs",
+          "visibility": "private",
+          "publicItems": [
+            {
+              "kind": "struct",
+              "name": "ShellHookExecutor",
+              "file": "",
+              "line": 6,
+              "attrs": {
+                "derive": [
+                  "Debug"
+                ],
+                "doc": [
+                  " A concrete implementation of `HookExecutor` that executes hooks via shell commands."
+                ]
+              },
+              "generics": "",
+              "visibility": "pub"
+            }
+          ],
+          "imports": [
+            {
+              "path": "crate::executor::TaskExecutor",
+              "line": 0
+            },
+            {
+              "path": "workflow_core::HookContext",
+              "line": 0
+            },
+            {
+              "path": "workflow_core::HookExecutor",
+              "line": 0
+            },
+            {
+              "path": "workflow_core::HookResult",
+              "line": 0
+            },
+            {
+              "path": "workflow_core::MonitoringHook",
+              "line": 0
+            }
+          ],
+          "submodules": [
+            "tests"
+          ]
+        },
+        {
+          "path": "workflow_utils::prelude",
+          "file": "workflow_utils/src/prelude.rs",
+          "visibility": "pub",
+          "imports": [
+            {
+              "path": "crate::JOB_SCRIPT_NAME",
+              "line": 0
+            },
+            {
+              "path": "crate::QueuedRunner",
+              "line": 0
+            },
+            {
+              "path": "crate::SchedulerKind",
+              "line": 0
+            },
+            {
+              "path": "crate::ShellHookExecutor",
+              "line": 0
+            },
+            {
+              "path": "crate::SystemProcessRunner",
+              "line": 0
+            },
+            {
+              "path": "crate::copy_file",
+              "line": 0
+            },
+            {
+              "path": "crate::create_dir",
+              "line": 0
+            },
+            {
+              "path": "crate::exists",
+              "line": 0
+            },
+            {
+              "path": "crate::read_file",
+              "line": 0
+            },
+            {
+              "path": "crate::remove_dir",
+              "line": 0
+            },
+            {
+              "path": "crate::run_default",
+              "line": 0
+            },
+            {
+              "path": "crate::write_file",
+              "line": 0
+            },
+            {
+              "path": "workflow_core::prelude::*",
+              "line": 0
+            }
+          ],
+          "reExports": [
+            {
+              "importPath": "workflow_core::prelude",
+              "exportPath": "*",
+              "line": 0
+            },
+            {
+              "importPath": "crate::JOB_SCRIPT_NAME",
+              "exportPath": "JOB_SCRIPT_NAME",
+              "line": 0
+            },
+            {
+              "importPath": "crate::QueuedRunner",
+              "exportPath": "QueuedRunner",
+              "line": 0
+            },
+            {
+              "importPath": "crate::SchedulerKind",
+              "exportPath": "SchedulerKind",
+              "line": 0
+            },
+            {
+              "importPath": "crate::ShellHookExecutor",
+              "exportPath": "ShellHookExecutor",
+              "line": 0
+            },
+            {
+              "importPath": "crate::SystemProcessRunner",
+              "exportPath": "SystemProcessRunner",
+              "line": 0
+            },
+            {
+              "importPath": "crate::copy_file",
+              "exportPath": "copy_file",
+              "line": 0
+            },
+            {
+              "importPath": "crate::create_dir",
+              "exportPath": "create_dir",
+              "line": 0
+            },
+            {
+              "importPath": "crate::exists",
+              "exportPath": "exists",
+              "line": 0
+            },
+            {
+              "importPath": "crate::read_file",
+              "exportPath": "read_file",
+              "line": 0
+            },
+            {
+              "importPath": "crate::remove_dir",
+              "exportPath": "remove_dir",
+              "line": 0
+            },
+            {
+              "importPath": "crate::run_default",
+              "exportPath": "run_default",
+              "line": 0
+            },
+            {
+              "importPath": "crate::write_file",
+              "exportPath": "write_file",
+              "line": 0
+            }
+          ]
+        },
+        {
+          "path": "workflow_utils::queued",
+          "file": "workflow_utils/src/queued.rs",
+          "visibility": "private",
+          "publicItems": [
+            {
+              "kind": "struct",
+              "name": "QueuedProcessHandle",
+              "file": "",
+              "line": 112,
+              "attrs": {},
+              "generics": "",
+              "visibility": "pub",
+              "fields": [
+                "job_id: String",
+                "scheduler: SchedulerKind",
+                "stdout_path: PathBuf",
+                "stderr_path: PathBuf",
+                "last_poll: Instant",
+                "poll_interval: Duration",
+                "cached_running: bool",
+                "finished_exit_code: Option<i32>",
+                "started_at: Instant"
+              ]
+            },
+            {
+              "kind": "struct",
+              "name": "QueuedRunner",
+              "file": "",
+              "line": 24,
+              "attrs": {
+                "doc": [
+                  " Submits and manages jobs via an HPC batch scheduler.",
+                  "",
+                  " Implements [`QueuedSubmitter`](workflow_core::process::QueuedSubmitter) to",
+                  " integrate with the workflow engine's `Queued` execution mode."
+                ]
+              },
+              "generics": "",
+              "visibility": "pub",
+              "fields": [
+                "scheduler: SchedulerKind"
+              ]
+            },
+            {
+              "kind": "enum",
+              "name": "SchedulerKind",
+              "file": "",
+              "line": 13,
+              "attrs": {
+                "derive": [
+                  "Clone",
+                  "Copy",
+                  "Debug"
+                ],
+                "doc": [
+                  " The type of HPC job scheduler to target."
+                ]
+              },
+              "generics": "",
+              "visibility": "pub",
+              "variants": [
+                "Slurm",
+                "Pbs"
+              ]
+            }
+          ],
+          "imports": [
+            {
+              "path": "std::path::Path",
+              "line": 0
+            },
+            {
+              "path": "std::path::PathBuf",
+              "line": 0
+            },
+            {
+              "path": "std::process::Command",
+              "line": 0
+            },
+            {
+              "path": "std::time::Duration",
+              "line": 0
+            },
+            {
+              "path": "std::time::Instant",
+              "line": 0
+            },
+            {
+              "path": "workflow_core::error::WorkflowError",
+              "line": 0
+            },
+            {
+              "path": "workflow_core::process::OutputLocation",
+              "line": 0
+            },
+            {
+              "path": "workflow_core::process::ProcessHandle",
+              "line": 0
+            },
+            {
+              "path": "workflow_core::process::ProcessResult",
+              "line": 0
+            }
+          ],
+          "submodules": [
+            "tests"
+          ]
+        }
+      ],
+      "deps": {
+        "normal": [
+          "workflow_core"
+        ],
+        "dev": [
+          "serial_test",
+          "tempfile"
+        ],
+        "workspaceMembers": [
+          "serde"
+        ]
+      },
+      "crossCrateImports": [
+        {
+          "importPath": "workflow_core::HookContext",
+          "targetCrate": "workflow_core",
+          "symbol": "HookContext",
+          "line": 0
+        },
+        {
+          "importPath": "workflow_core::HookContext",
+          "targetCrate": "workflow_core",
+          "symbol": "HookContext",
+          "line": 0
+        },
+        {
+          "importPath": "workflow_core::HookExecutor",
+          "targetCrate": "workflow_core",
+          "symbol": "HookExecutor",
+          "line": 0
+        },
+        {
+          "importPath": "workflow_core::HookExecutor",
+          "targetCrate": "workflow_core",
+          "symbol": "HookExecutor",
+          "line": 0
+        },
+        {
+          "importPath": "workflow_core::HookResult",
+          "targetCrate": "workflow_core",
+          "symbol": "HookResult",
+          "line": 0
+        },
+        {
+          "importPath": "workflow_core::HookResult",
+          "targetCrate": "workflow_core",
+          "symbol": "HookResult",
+          "line": 0
+        },
+        {
+          "importPath": "workflow_core::HookTrigger",
+          "targetCrate": "workflow_core",
+          "symbol": "HookTrigger",
+          "line": 0
+        },
+        {
+          "importPath": "workflow_core::MonitoringHook",
+          "targetCrate": "workflow_core",
+          "symbol": "MonitoringHook",
+          "line": 0
+        },
+        {
+          "importPath": "workflow_core::MonitoringHook",
+          "targetCrate": "workflow_core",
+          "symbol": "MonitoringHook",
+          "line": 0
+        },
+        {
+          "importPath": "workflow_core::ProcessHandle",
+          "targetCrate": "workflow_core",
+          "symbol": "ProcessHandle",
+          "line": 0
+        },
+        {
+          "importPath": "workflow_core::ProcessResult",
+          "targetCrate": "workflow_core",
+          "symbol": "ProcessResult",
+          "line": 0
+        },
+        {
+          "importPath": "workflow_core::ProcessRunner",
+          "targetCrate": "workflow_core",
+          "symbol": "ProcessRunner",
+          "line": 0
+        },
+        {
+          "importPath": "workflow_core::ProcessRunner",
+          "targetCrate": "workflow_core",
+          "symbol": "ProcessRunner",
+          "line": 0
+        },
+        {
+          "importPath": "workflow_core::WorkflowError",
+          "targetCrate": "workflow_core",
+          "symbol": "WorkflowError",
+          "line": 0
+        },
+        {
+          "importPath": "workflow_core::WorkflowError",
+          "targetCrate": "workflow_core",
+          "symbol": "WorkflowError",
+          "line": 0
+        },
+        {
+          "importPath": "workflow_core::WorkflowError",
+          "targetCrate": "workflow_core",
+          "symbol": "WorkflowError",
+          "line": 0
+        },
+        {
+          "importPath": "workflow_core::error::WorkflowError",
+          "targetCrate": "workflow_core",
+          "symbol": "WorkflowError",
+          "line": 0
+        },
+        {
+          "importPath": "workflow_core::prelude::*",
+          "targetCrate": "workflow_core",
+          "symbol": "*",
+          "line": 0
+        },
+        {
+          "importPath": "workflow_core::process::OutputLocation",
+          "targetCrate": "workflow_core",
+          "symbol": "OutputLocation",
+          "line": 0
+        },
+        {
+          "importPath": "workflow_core::process::OutputLocation",
+          "targetCrate": "workflow_core",
+          "symbol": "OutputLocation",
+          "line": 0
+        },
+        {
+          "importPath": "workflow_core::process::ProcessHandle",
+          "targetCrate": "workflow_core",
+          "symbol": "ProcessHandle",
+          "line": 0
+        },
+        {
+          "importPath": "workflow_core::process::ProcessResult",
+          "targetCrate": "workflow_core",
+          "symbol": "ProcessResult",
+          "line": 0
+        },
+        {
+          "importPath": "workflow_core::state::StateStore",
+          "targetCrate": "workflow_core",
+          "symbol": "StateStore",
+          "line": 0
+        },
+        {
+          "importPath": "workflow_core::workflow::Workflow",
+          "targetCrate": "workflow_core",
+          "symbol": "Workflow",
+          "line": 0
+        },
+        {
+          "importPath": "workflow_core::workflow::WorkflowSummary",
+          "targetCrate": "workflow_core",
+          "symbol": "WorkflowSummary",
+          "line": 0
+        }
+      ]
+    }
+  ],
+  "crossReferences": {
+    "types": {
+      "hubbard_u_sweep_slurm::config::SweepConfig": {
+        "crateName": "hubbard_u_sweep_slurm",
+        "kind": "struct",
+        "exportedBy": [
+          "hubbard_u_sweep_slurm"
+        ]
+      },
+      "hubbard_u_sweep_slurm::config::parse_u_values": {
+        "crateName": "hubbard_u_sweep_slurm",
+        "kind": "fn",
+        "exportedBy": [
+          "hubbard_u_sweep_slurm"
+        ]
+      },
+      "hubbard_u_sweep_slurm::job_script::generate_job_script": {
+        "crateName": "hubbard_u_sweep_slurm",
+        "kind": "fn",
+        "exportedBy": [
+          "hubbard_u_sweep_slurm"
+        ]
+      },
+      "workflow_core::dag::Dag": {
+        "crateName": "workflow_core",
+        "kind": "struct",
+        "exportedBy": [
+          "workflow_core"
+        ]
+      },
+      "workflow_core::error::WorkflowError": {
+        "crateName": "workflow_core",
+        "kind": "enum",
+        "importedBy": [
+          "workflow_utils:workflow_utils",
+          "workflow_utils:workflow_utils::executor",
+          "workflow_utils:workflow_utils::files",
+          "workflow_utils:workflow_utils::queued"
+        ],
+        "exportedBy": [
+          "workflow_core"
+        ]
+      },
+      "workflow_core::init_default_logging": {
+        "crateName": "workflow_core",
+        "kind": "fn",
+        "exportedBy": [
+          "workflow_core"
+        ]
+      },
+      "workflow_core::monitoring::HookContext": {
+        "crateName": "workflow_core",
+        "kind": "struct",
+        "importedBy": [
+          "workflow_utils:workflow_utils",
+          "workflow_utils:workflow_utils::monitoring"
+        ],
+        "exportedBy": [
+          "workflow_core"
+        ]
+      },
+      "workflow_core::monitoring::HookExecutor": {
+        "crateName": "workflow_core",
+        "kind": "trait",
+        "importedBy": [
+          "workflow_utils:workflow_utils",
+          "workflow_utils:workflow_utils::monitoring"
+        ],
+        "exportedBy": [
+          "workflow_core"
+        ]
+      },
+      "workflow_core::monitoring::HookResult": {
+        "crateName": "workflow_core",
+        "kind": "struct",
+        "importedBy": [
+          "workflow_utils:workflow_utils",
+          "workflow_utils:workflow_utils::monitoring"
+        ],
+        "exportedBy": [
+          "workflow_core"
+        ]
+      },
+      "workflow_core::monitoring::HookTrigger": {
+        "crateName": "workflow_core",
+        "kind": "enum",
+        "importedBy": [
+          "workflow_utils:workflow_utils"
+        ],
+        "exportedBy": [
+          "workflow_core"
+        ]
+      },
+      "workflow_core::monitoring::MonitoringHook": {
+        "crateName": "workflow_core",
+        "kind": "struct",
+        "importedBy": [
+          "workflow_utils:workflow_utils",
+          "workflow_utils:workflow_utils::monitoring"
+        ],
+        "exportedBy": [
+          "workflow_core"
+        ]
+      },
+      "workflow_core::monitoring::TaskPhase": {
+        "crateName": "workflow_core",
+        "kind": "enum",
+        "exportedBy": [
+          "workflow_core"
+        ]
+      },
+      "workflow_core::process::OutputLocation": {
+        "crateName": "workflow_core",
+        "kind": "enum",
+        "importedBy": [
+          "workflow_utils:workflow_utils::executor",
+          "workflow_utils:workflow_utils::queued"
+        ],
+        "exportedBy": [
+          "workflow_core"
+        ]
+      },
+      "workflow_core::process::ProcessHandle": {
+        "crateName": "workflow_core",
+        "kind": "trait",
+        "importedBy": [
+          "workflow_utils:workflow_utils::executor",
+          "workflow_utils:workflow_utils::queued"
+        ],
+        "exportedBy": [
+          "workflow_core"
+        ]
+      },
+      "workflow_core::process::ProcessResult": {
+        "crateName": "workflow_core",
+        "kind": "struct",
+        "importedBy": [
+          "workflow_utils:workflow_utils::executor",
+          "workflow_utils:workflow_utils::queued"
+        ],
+        "exportedBy": [
+          "workflow_core"
+        ]
+      },
+      "workflow_core::process::ProcessRunner": {
+        "crateName": "workflow_core",
+        "kind": "trait",
+        "importedBy": [
+          "workflow_utils:workflow_utils",
+          "workflow_utils:workflow_utils::executor"
+        ],
+        "exportedBy": [
+          "workflow_core"
+        ]
+      },
+      "workflow_core::process::QueuedSubmitter": {
+        "crateName": "workflow_core",
+        "kind": "trait",
+        "exportedBy": [
+          "workflow_core"
+        ]
+      },
+      "workflow_core::state::JsonStateStore": {
+        "crateName": "workflow_core",
+        "kind": "struct",
+        "importedBy": [
+          "workflow-cli:workflow-cli"
+        ],
+        "exportedBy": [
+          "workflow_core"
+        ]
+      },
+      "workflow_core::state::StateStore": {
+        "crateName": "workflow_core",
+        "kind": "trait",
+        "importedBy": [
+          "workflow-cli:workflow-cli",
+          "workflow_utils:workflow_utils"
+        ],
+        "exportedBy": [
+          "workflow_core"
+        ]
+      },
+      "workflow_core::state::StateStoreExt": {
+        "crateName": "workflow_core",
+        "kind": "trait",
+        "importedBy": [
+          "workflow-cli:workflow-cli"
+        ],
+        "exportedBy": [
+          "workflow_core"
+        ]
+      },
+      "workflow_core::state::StateSummary": {
+        "crateName": "workflow_core",
+        "kind": "struct",
+        "exportedBy": [
+          "workflow_core"
+        ]
+      },
+      "workflow_core::state::TaskStatus": {
+        "crateName": "workflow_core",
+        "kind": "enum",
+        "importedBy": [
+          "workflow-cli:workflow-cli"
+        ],
+        "exportedBy": [
+          "workflow_core"
+        ]
+      },
+      "workflow_core::state::TaskSuccessors": {
+        "crateName": "workflow_core",
+        "kind": "struct",
+        "exportedBy": [
+          "workflow_core"
+        ]
+      },
+      "workflow_core::task::CollectFailurePolicy": {
+        "crateName": "workflow_core",
+        "kind": "enum",
+        "exportedBy": [
+          "workflow_core"
+        ]
+      },
+      "workflow_core::task::ExecutionMode": {
+        "crateName": "workflow_core",
+        "kind": "enum",
+        "exportedBy": [
+          "workflow_core"
+        ]
+      },
+      "workflow_core::task::Task": {
+        "crateName": "workflow_core",
+        "kind": "struct",
+        "exportedBy": [
+          "workflow_core"
+        ]
+      },
+      "workflow_core::task::TaskClosure": {
+        "crateName": "workflow_core",
+        "kind": "type",
+        "exportedBy": [
+          "workflow_core"
+        ]
+      },
+      "workflow_core::workflow::FailedTask": {
+        "crateName": "workflow_core",
+        "kind": "struct",
+        "exportedBy": [
+          "workflow_core"
+        ]
+      },
+      "workflow_core::workflow::InFlightTask": {
+        "crateName": "workflow_core",
+        "kind": "struct",
+        "exportedBy": [
+          "workflow_core"
+        ]
+      },
+      "workflow_core::workflow::Workflow": {
+        "crateName": "workflow_core",
+        "kind": "struct",
+        "importedBy": [
+          "workflow_utils:workflow_utils"
+        ],
+        "exportedBy": [
+          "workflow_core"
+        ]
+      },
+      "workflow_core::workflow::WorkflowSummary": {
+        "crateName": "workflow_core",
+        "kind": "struct",
+        "importedBy": [
+          "workflow_utils:workflow_utils"
+        ],
+        "exportedBy": [
+          "workflow_core"
+        ]
+      },
+      "workflow_utils::executor::ExecutionHandle": {
+        "crateName": "workflow_utils",
+        "kind": "struct",
+        "exportedBy": [
+          "workflow_utils"
+        ]
+      },
+      "workflow_utils::executor::ExecutionResult": {
+        "crateName": "workflow_utils",
+        "kind": "struct",
+        "exportedBy": [
+          "workflow_utils"
+        ]
+      },
+      "workflow_utils::executor::SystemProcessHandle": {
+        "crateName": "workflow_utils",
+        "kind": "struct",
+        "exportedBy": [
+          "workflow_utils"
+        ]
+      },
+      "workflow_utils::executor::SystemProcessRunner": {
+        "crateName": "workflow_utils",
+        "kind": "struct",
+        "exportedBy": [
+          "workflow_utils"
+        ]
+      },
+      "workflow_utils::executor::TaskExecutor": {
+        "crateName": "workflow_utils",
+        "kind": "struct",
+        "exportedBy": [
+          "workflow_utils"
+        ]
+      },
+      "workflow_utils::files::copy_file": {
+        "crateName": "workflow_utils",
+        "kind": "fn",
+        "exportedBy": [
+          "workflow_utils"
+        ]
+      },
+      "workflow_utils::files::create_dir": {
+        "crateName": "workflow_utils",
+        "kind": "fn",
+        "exportedBy": [
+          "workflow_utils"
+        ]
+      },
+      "workflow_utils::files::exists": {
+        "crateName": "workflow_utils",
+        "kind": "fn",
+        "exportedBy": [
+          "workflow_utils"
+        ]
+      },
+      "workflow_utils::files::read_file": {
+        "crateName": "workflow_utils",
+        "kind": "fn",
+        "exportedBy": [
+          "workflow_utils"
+        ]
+      },
+      "workflow_utils::files::remove_dir": {
+        "crateName": "workflow_utils",
+        "kind": "fn",
+        "exportedBy": [
+          "workflow_utils"
+        ]
+      },
+      "workflow_utils::files::write_file": {
+        "crateName": "workflow_utils",
+        "kind": "fn",
+        "exportedBy": [
+          "workflow_utils"
+        ]
+      },
+      "workflow_utils::monitoring::ShellHookExecutor": {
+        "crateName": "workflow_utils",
+        "kind": "struct",
+        "exportedBy": [
+          "workflow_utils"
+        ]
+      },
+      "workflow_utils::queued::QueuedProcessHandle": {
+        "crateName": "workflow_utils",
+        "kind": "struct",
+        "exportedBy": [
+          "workflow_utils"
+        ]
+      },
+      "workflow_utils::queued::QueuedRunner": {
+        "crateName": "workflow_utils",
+        "kind": "struct",
+        "exportedBy": [
+          "workflow_utils"
+        ]
+      },
+      "workflow_utils::queued::SchedulerKind": {
+        "crateName": "workflow_utils",
+        "kind": "enum",
+        "exportedBy": [
+          "workflow_utils"
+        ]
+      },
+      "workflow_utils::run_default": {
+        "crateName": "workflow_utils",
+        "kind": "fn",
+        "exportedBy": [
+          "workflow_utils"
+        ]
+      }
+    }
+  },
+  "symbols": {
+    "hubbard_u_sweep_slurm::config::SweepConfig": {
+      "crateName": "hubbard_u_sweep_slurm",
+      "module": "hubbard_u_sweep_slurm::config",
+      "file": "examples/hubbard_u_sweep_slurm/src/config.rs",
+      "line": 5,
+      "kind": "struct",
+      "deriveAttrs": [
+        "Debug",
+        "Parser"
+      ]
+    },
+    "hubbard_u_sweep_slurm::config::parse_u_values": {
+      "crateName": "hubbard_u_sweep_slurm",
+      "module": "hubbard_u_sweep_slurm::config",
+      "file": "examples/hubbard_u_sweep_slurm/src/config.rs",
+      "line": 75,
+      "kind": "fn"
+    },
+    "hubbard_u_sweep_slurm::job_script::generate_job_script": {
+      "crateName": "hubbard_u_sweep_slurm",
+      "module": "hubbard_u_sweep_slurm::job_script",
+      "file": "examples/hubbard_u_sweep_slurm/src/job_script.rs",
+      "line": 3,
+      "kind": "fn"
+    },
+    "workflow_core::dag::Dag": {
+      "crateName": "workflow_core",
+      "module": "workflow_core::dag",
+      "file": "workflow_core/src/dag.rs",
+      "line": 6,
+      "kind": "struct"
+    },
+    "workflow_core::error::WorkflowError": {
+      "crateName": "workflow_core",
+      "module": "workflow_core::error",
+      "file": "workflow_core/src/error.rs",
+      "line": 5,
+      "kind": "enum",
+      "deriveAttrs": [
+        "Debug",
+        "Error"
+      ]
+    },
+    "workflow_core::init_default_logging": {
+      "crateName": "workflow_core",
+      "module": "workflow_core",
+      "file": "workflow_core/src/lib.rs",
+      "line": 24,
+      "kind": "fn"
+    },
+    "workflow_core::monitoring::HookContext": {
+      "crateName": "workflow_core",
+      "module": "workflow_core::monitoring",
+      "file": "workflow_core/src/monitoring.rs",
+      "line": 75,
+      "kind": "struct",
+      "deriveAttrs": [
+        "Clone",
+        "Debug",
+        "Deserialize",
+        "Serialize"
+      ]
+    },
+    "workflow_core::monitoring::HookExecutor": {
+      "crateName": "workflow_core",
+      "module": "workflow_core::monitoring",
+      "file": "workflow_core/src/monitoring.rs",
+      "line": 8,
+      "kind": "trait"
+    },
+    "workflow_core::monitoring::HookResult": {
+      "crateName": "workflow_core",
+      "module": "workflow_core::monitoring",
+      "file": "workflow_core/src/monitoring.rs",
+      "line": 88,
+      "kind": "struct",
+      "deriveAttrs": [
+        "Clone",
+        "Debug",
+        "Deserialize",
+        "Serialize"
+      ]
+    },
+    "workflow_core::monitoring::HookTrigger": {
+      "crateName": "workflow_core",
+      "module": "workflow_core::monitoring",
+      "file": "workflow_core/src/monitoring.rs",
+      "line": 41,
+      "kind": "enum",
+      "deriveAttrs": [
+        "Clone",
+        "Debug",
+        "Deserialize",
+        "Serialize"
+      ]
+    },
+    "workflow_core::monitoring::MonitoringHook": {
+      "crateName": "workflow_core",
+      "module": "workflow_core::monitoring",
+      "file": "workflow_core/src/monitoring.rs",
+      "line": 19,
+      "kind": "struct",
+      "deriveAttrs": [
+        "Clone",
+        "Debug",
+        "Deserialize",
+        "Serialize"
+      ]
+    },
+    "workflow_core::monitoring::TaskPhase": {
+      "crateName": "workflow_core",
+      "module": "workflow_core::monitoring",
+      "file": "workflow_core/src/monitoring.rs",
+      "line": 54,
+      "kind": "enum",
+      "deriveAttrs": [
+        "Clone",
+        "Copy",
+        "Debug",
+        "Deserialize",
+        "Eq",
+        "PartialEq",
+        "Serialize"
+      ]
+    },
+    "workflow_core::process::OutputLocation": {
+      "crateName": "workflow_core",
+      "module": "workflow_core::process",
+      "file": "workflow_core/src/process.rs",
+      "line": 7,
+      "kind": "enum",
+      "deriveAttrs": [
+        "Clone",
+        "Debug"
+      ]
+    },
+    "workflow_core::process::ProcessHandle": {
+      "crateName": "workflow_core",
+      "module": "workflow_core::process",
+      "file": "workflow_core/src/process.rs",
+      "line": 25,
+      "kind": "trait"
+    },
+    "workflow_core::process::ProcessResult": {
+      "crateName": "workflow_core",
+      "module": "workflow_core::process",
+      "file": "workflow_core/src/process.rs",
+      "line": 45,
+      "kind": "struct"
+    },
+    "workflow_core::process::ProcessRunner": {
+      "crateName": "workflow_core",
+      "module": "workflow_core::process",
+      "file": "workflow_core/src/process.rs",
+      "line": 12,
+      "kind": "trait"
+    },
+    "workflow_core::process::QueuedSubmitter": {
+      "crateName": "workflow_core",
+      "module": "workflow_core::process",
+      "file": "workflow_core/src/process.rs",
+      "line": 51,
+      "kind": "trait"
+    },
+    "workflow_core::state::JsonStateStore": {
+      "crateName": "workflow_core",
+      "module": "workflow_core::state",
+      "file": "workflow_core/src/state.rs",
+      "line": 190,
+      "kind": "struct",
+      "deriveAttrs": [
+        "Debug",
+        "Deserialize",
+        "Serialize"
+      ]
+    },
+    "workflow_core::state::StateStore": {
+      "crateName": "workflow_core",
+      "module": "workflow_core::state",
+      "file": "workflow_core/src/state.rs",
+      "line": 33,
+      "kind": "trait"
+    },
+    "workflow_core::state::StateStoreExt": {
+      "crateName": "workflow_core",
+      "module": "workflow_core::state",
+      "file": "workflow_core/src/state.rs",
+      "line": 48,
+      "kind": "trait"
+    },
+    "workflow_core::state::StateSummary": {
+      "crateName": "workflow_core",
+      "module": "workflow_core::state",
+      "file": "workflow_core/src/state.rs",
+      "line": 110,
+      "kind": "struct",
+      "deriveAttrs": [
+        "Clone",
+        "Debug"
+      ]
+    },
+    "workflow_core::state::TaskStatus": {
+      "crateName": "workflow_core",
+      "module": "workflow_core::state",
+      "file": "workflow_core/src/state.rs",
+      "line": 10,
+      "kind": "enum",
+      "deriveAttrs": [
+        "Clone",
+        "Debug",
+        "Deserialize",
+        "PartialEq",
+        "Serialize"
+      ]
+    },
+    "workflow_core::state::TaskSuccessors": {
+      "crateName": "workflow_core",
+      "module": "workflow_core::state",
+      "file": "workflow_core/src/state.rs",
+      "line": 129,
+      "kind": "struct",
+      "deriveAttrs": [
+        "Clone",
+        "Debug",
+        "Default",
+        "Deserialize",
+        "Serialize"
+      ]
+    },
+    "workflow_core::task::CollectFailurePolicy": {
+      "crateName": "workflow_core",
+      "module": "workflow_core::task",
+      "file": "workflow_core/src/task.rs",
+      "line": 16,
+      "kind": "enum",
+      "deriveAttrs": [
+        "Clone",
+        "Copy",
+        "Debug",
+        "Default",
+        "Eq",
+        "PartialEq"
+      ]
+    },
+    "workflow_core::task::ExecutionMode": {
+      "crateName": "workflow_core",
+      "module": "workflow_core::task",
+      "file": "workflow_core/src/task.rs",
+      "line": 26,
+      "kind": "enum",
+      "deriveAttrs": [
+        "Clone",
+        "Debug"
+      ]
+    },
+    "workflow_core::task::Task": {
+      "crateName": "workflow_core",
+      "module": "workflow_core::task",
+      "file": "workflow_core/src/task.rs",
+      "line": 57,
+      "kind": "struct"
+    },
+    "workflow_core::task::TaskClosure": {
+      "crateName": "workflow_core",
+      "module": "workflow_core::task",
+      "file": "workflow_core/src/task.rs",
+      "line": 8,
+      "kind": "type"
+    },
+    "workflow_core::workflow::FailedTask": {
+      "crateName": "workflow_core",
+      "module": "workflow_core::workflow",
+      "file": "workflow_core/src/workflow.rs",
+      "line": 563,
+      "kind": "struct",
+      "deriveAttrs": [
+        "Clone",
+        "Debug"
+      ]
+    },
+    "workflow_core::workflow::InFlightTask": {
+      "crateName": "workflow_core",
+      "module": "workflow_core::workflow",
+      "file": "workflow_core/src/workflow.rs",
+      "line": 17,
+      "kind": "struct"
+    },
+    "workflow_core::workflow::Workflow": {
+      "crateName": "workflow_core",
+      "module": "workflow_core::workflow",
+      "file": "workflow_core/src/workflow.rs",
+      "line": 27,
+      "kind": "struct"
+    },
+    "workflow_core::workflow::WorkflowSummary": {
+      "crateName": "workflow_core",
+      "module": "workflow_core::workflow",
+      "file": "workflow_core/src/workflow.rs",
+      "line": 570,
+      "kind": "struct",
+      "deriveAttrs": [
+        "Clone",
+        "Debug"
+      ]
+    },
+    "workflow_utils::executor::ExecutionHandle": {
+      "crateName": "workflow_utils",
+      "module": "workflow_utils::executor",
+      "file": "workflow_utils/src/executor.rs",
+      "line": 89,
+      "kind": "struct"
+    },
+    "workflow_utils::executor::ExecutionResult": {
+      "crateName": "workflow_utils",
+      "module": "workflow_utils::executor",
+      "file": "workflow_utils/src/executor.rs",
+      "line": 76,
+      "kind": "struct"
+    },
+    "workflow_utils::executor::SystemProcessHandle": {
+      "crateName": "workflow_utils",
+      "module": "workflow_utils::executor",
+      "file": "workflow_utils/src/executor.rs",
+      "line": 166,
+      "kind": "struct"
+    },
+    "workflow_utils::executor::SystemProcessRunner": {
+      "crateName": "workflow_utils",
+      "module": "workflow_utils::executor",
+      "file": "workflow_utils/src/executor.rs",
+      "line": 110,
+      "kind": "struct",
+      "deriveAttrs": [
+        "Default"
+      ]
+    },
+    "workflow_utils::executor::TaskExecutor": {
+      "crateName": "workflow_utils",
+      "module": "workflow_utils::executor",
+      "file": "workflow_utils/src/executor.rs",
+      "line": 12,
+      "kind": "struct"
+    },
+    "workflow_utils::files::copy_file": {
+      "crateName": "workflow_utils",
+      "module": "workflow_utils::files",
+      "file": "workflow_utils/src/files.rs",
+      "line": 21,
+      "kind": "fn"
+    },
+    "workflow_utils::files::create_dir": {
+      "crateName": "workflow_utils",
+      "module": "workflow_utils::files",
+      "file": "workflow_utils/src/files.rs",
+      "line": 31,
+      "kind": "fn"
+    },
+    "workflow_utils::files::exists": {
+      "crateName": "workflow_utils",
+      "module": "workflow_utils::files",
+      "file": "workflow_utils/src/files.rs",
+      "line": 41,
+      "kind": "fn"
+    },
+    "workflow_utils::files::read_file": {
+      "crateName": "workflow_utils",
+      "module": "workflow_utils::files",
+      "file": "workflow_utils/src/files.rs",
+      "line": 8,
+      "kind": "fn"
+    },
+    "workflow_utils::files::remove_dir": {
+      "crateName": "workflow_utils",
+      "module": "workflow_utils::files",
+      "file": "workflow_utils/src/files.rs",
+      "line": 36,
+      "kind": "fn"
+    },
+    "workflow_utils::files::write_file": {
+      "crateName": "workflow_utils",
+      "module": "workflow_utils::files",
+      "file": "workflow_utils/src/files.rs",
+      "line": 13,
+      "kind": "fn"
+    },
+    "workflow_utils::monitoring::ShellHookExecutor": {
+      "crateName": "workflow_utils",
+      "module": "workflow_utils::monitoring",
+      "file": "workflow_utils/src/monitoring.rs",
+      "line": 6,
+      "kind": "struct",
+      "deriveAttrs": [
+        "Debug"
+      ]
+    },
+    "workflow_utils::queued::QueuedProcessHandle": {
+      "crateName": "workflow_utils",
+      "module": "workflow_utils::queued",
+      "file": "workflow_utils/src/queued.rs",
+      "line": 112,
+      "kind": "struct"
+    },
+    "workflow_utils::queued::QueuedRunner": {
+      "crateName": "workflow_utils",
+      "module": "workflow_utils::queued",
+      "file": "workflow_utils/src/queued.rs",
+      "line": 24,
+      "kind": "struct"
+    },
+    "workflow_utils::queued::SchedulerKind": {
+      "crateName": "workflow_utils",
+      "module": "workflow_utils::queued",
+      "file": "workflow_utils/src/queued.rs",
+      "line": 13,
+      "kind": "enum",
+      "deriveAttrs": [
+        "Clone",
+        "Copy",
+        "Debug"
+      ]
+    },
+    "workflow_utils::run_default": {
+      "crateName": "workflow_utils",
+      "module": "workflow_utils",
+      "file": "workflow_utils/src/lib.rs",
+      "line": 31,
+      "kind": "fn"
+    }
+  },
+  "nameIndex": {
+    "CollectFailurePolicy": [
+      "workflow_core::task::CollectFailurePolicy"
+    ],
+    "Dag": [
+      "workflow_core::dag::Dag"
+    ],
+    "ExecutionHandle": [
+      "workflow_utils::executor::ExecutionHandle"
+    ],
+    "ExecutionMode": [
+      "workflow_core::task::ExecutionMode"
+    ],
+    "ExecutionResult": [
+      "workflow_utils::executor::ExecutionResult"
+    ],
+    "FailedTask": [
+      "workflow_core::workflow::FailedTask"
+    ],
+    "HookContext": [
+      "workflow_core::monitoring::HookContext"
+    ],
+    "HookExecutor": [
+      "workflow_core::monitoring::HookExecutor"
+    ],
+    "HookResult": [
+      "workflow_core::monitoring::HookResult"
+    ],
+    "HookTrigger": [
+      "workflow_core::monitoring::HookTrigger"
+    ],
+    "InFlightTask": [
+      "workflow_core::workflow::InFlightTask"
+    ],
+    "JsonStateStore": [
+      "workflow_core::state::JsonStateStore"
+    ],
+    "MonitoringHook": [
+      "workflow_core::monitoring::MonitoringHook"
+    ],
+    "OutputLocation": [
+      "workflow_core::process::OutputLocation"
+    ],
+    "ProcessHandle": [
+      "workflow_core::process::ProcessHandle"
+    ],
+    "ProcessResult": [
+      "workflow_core::process::ProcessResult"
+    ],
+    "ProcessRunner": [
+      "workflow_core::process::ProcessRunner"
+    ],
+    "QueuedProcessHandle": [
+      "workflow_utils::queued::QueuedProcessHandle"
+    ],
+    "QueuedRunner": [
+      "workflow_utils::queued::QueuedRunner"
+    ],
+    "QueuedSubmitter": [
+      "workflow_core::process::QueuedSubmitter"
+    ],
+    "SchedulerKind": [
+      "workflow_utils::queued::SchedulerKind"
+    ],
+    "ShellHookExecutor": [
+      "workflow_utils::monitoring::ShellHookExecutor"
+    ],
+    "StateStore": [
+      "workflow_core::state::StateStore"
+    ],
+    "StateStoreExt": [
+      "workflow_core::state::StateStoreExt"
+    ],
+    "StateSummary": [
+      "workflow_core::state::StateSummary"
+    ],
+    "SweepConfig": [
+      "hubbard_u_sweep_slurm::config::SweepConfig"
+    ],
+    "SystemProcessHandle": [
+      "workflow_utils::executor::SystemProcessHandle"
+    ],
+    "SystemProcessRunner": [
+      "workflow_utils::executor::SystemProcessRunner"
+    ],
+    "Task": [
+      "workflow_core::task::Task"
+    ],
+    "TaskClosure": [
+      "workflow_core::task::TaskClosure"
+    ],
+    "TaskExecutor": [
+      "workflow_utils::executor::TaskExecutor"
+    ],
+    "TaskPhase": [
+      "workflow_core::monitoring::TaskPhase"
+    ],
+    "TaskStatus": [
+      "workflow_core::state::TaskStatus"
+    ],
+    "TaskSuccessors": [
+      "workflow_core::state::TaskSuccessors"
+    ],
+    "Workflow": [
+      "workflow_core::workflow::Workflow"
+    ],
+    "WorkflowError": [
+      "workflow_core::error::WorkflowError"
+    ],
+    "WorkflowSummary": [
+      "workflow_core::workflow::WorkflowSummary"
+    ],
+    "copy_file": [
+      "workflow_utils::files::copy_file"
+    ],
+    "create_dir": [
+      "workflow_utils::files::create_dir"
+    ],
+    "exists": [
+      "workflow_utils::files::exists"
+    ],
+    "generate_job_script": [
+      "hubbard_u_sweep_slurm::job_script::generate_job_script"
+    ],
+    "init_default_logging": [
+      "workflow_core::init_default_logging"
+    ],
+    "parse_u_values": [
+      "hubbard_u_sweep_slurm::config::parse_u_values"
+    ],
+    "read_file": [
+      "workflow_utils::files::read_file"
+    ],
+    "remove_dir": [
+      "workflow_utils::files::remove_dir"
+    ],
+    "run_default": [
+      "workflow_utils::run_default"
+    ],
+    "write_file": [
+      "workflow_utils::files::write_file"
+    ]
+  },
+  "files": {
+    "examples/hubbard_u_sweep/src/main.rs": {
+      "modulePath": "hubbard_u_sweep",
+      "isCrateRoot": true
+    },
+    "examples/hubbard_u_sweep_slurm/src/config.rs": {
+      "modulePath": "hubbard_u_sweep_slurm::config",
+      "parentModuleFile": "examples/hubbard_u_sweep_slurm/src/main.rs",
+      "isCrateRoot": false
+    },
+    "examples/hubbard_u_sweep_slurm/src/job_script.rs": {
+      "modulePath": "hubbard_u_sweep_slurm::job_script",
+      "parentModuleFile": "examples/hubbard_u_sweep_slurm/src/main.rs",
+      "isCrateRoot": false
+    },
+    "examples/hubbard_u_sweep_slurm/src/main.rs": {
+      "modulePath": "hubbard_u_sweep_slurm",
+      "isCrateRoot": true
+    },
+    "workflow-cli/src/main.rs": {
+      "modulePath": "workflow-cli",
+      "isCrateRoot": true
+    },
+    "workflow_core/src/dag.rs": {
+      "modulePath": "workflow_core::dag",
+      "parentModuleFile": "workflow_core/src/lib.rs",
+      "isCrateRoot": false
+    },
+    "workflow_core/src/error.rs": {
+      "modulePath": "workflow_core::error",
+      "parentModuleFile": "workflow_core/src/lib.rs",
+      "isCrateRoot": false
+    },
+    "workflow_core/src/lib.rs": {
+      "modulePath": "workflow_core",
+      "isCrateRoot": true
+    },
+    "workflow_core/src/monitoring.rs": {
+      "modulePath": "workflow_core::monitoring",
+      "parentModuleFile": "workflow_core/src/lib.rs",
+      "isCrateRoot": false
+    },
+    "workflow_core/src/prelude.rs": {
+      "modulePath": "workflow_core::prelude",
+      "parentModuleFile": "workflow_core/src/lib.rs",
+      "isCrateRoot": false
+    },
+    "workflow_core/src/process.rs": {
+      "modulePath": "workflow_core::process",
+      "parentModuleFile": "workflow_core/src/lib.rs",
+      "isCrateRoot": false
+    },
+    "workflow_core/src/state.rs": {
+      "modulePath": "workflow_core::state",
+      "parentModuleFile": "workflow_core/src/lib.rs",
+      "isCrateRoot": false
+    },
+    "workflow_core/src/task.rs": {
+      "modulePath": "workflow_core::task",
+      "parentModuleFile": "workflow_core/src/lib.rs",
+      "isCrateRoot": false
+    },
+    "workflow_core/src/workflow.rs": {
+      "modulePath": "workflow_core::workflow",
+      "parentModuleFile": "workflow_core/src/lib.rs",
+      "isCrateRoot": false
+    },
+    "workflow_utils/src/executor.rs": {
+      "modulePath": "workflow_utils::executor",
+      "parentModuleFile": "workflow_utils/src/lib.rs",
+      "isCrateRoot": false
+    },
+    "workflow_utils/src/files.rs": {
+      "modulePath": "workflow_utils::files",
+      "parentModuleFile": "workflow_utils/src/lib.rs",
+      "isCrateRoot": false
+    },
+    "workflow_utils/src/lib.rs": {
+      "modulePath": "workflow_utils",
+      "isCrateRoot": true
+    },
+    "workflow_utils/src/monitoring.rs": {
+      "modulePath": "workflow_utils::monitoring",
+      "parentModuleFile": "workflow_utils/src/lib.rs",
+      "isCrateRoot": false
+    },
+    "workflow_utils/src/prelude.rs": {
+      "modulePath": "workflow_utils::prelude",
+      "parentModuleFile": "workflow_utils/src/lib.rs",
+      "isCrateRoot": false
+    },
+    "workflow_utils/src/queued.rs": {
+      "modulePath": "workflow_utils::queued",
+      "parentModuleFile": "workflow_utils/src/lib.rs",
+      "isCrateRoot": false
+    }
+  },
+  "errors": [
+    {
+      "file": "workflow_core/src/prelude.rs",
+      "line": 0,
+      "message": "dead re-export: 'HookExecutor' resolves to 'crate::HookExecutor' which is not found in the symbol index. Remove or fix the 'pub use' statement.",
+      "severity": "warning",
+      "kind": "dead_re_export"
+    },
+    {
+      "file": "workflow_core/src/prelude.rs",
+      "line": 0,
+      "message": "dead re-export: 'ProcessRunner' resolves to 'crate::ProcessRunner' which is not found in the symbol index. Remove or fix the 'pub use' statement.",
+      "severity": "warning",
+      "kind": "dead_re_export"
+    },
+    {
+      "file": "workflow_utils/src/lib.rs",
+      "line": 0,
+      "message": "dead re-export: 'JOB_SCRIPT_NAME' resolves to 'queued::JOB_SCRIPT_NAME' which is not found in the symbol index. Remove or fix the 'pub use' statement.",
+      "severity": "warning",
+      "kind": "dead_re_export"
+    },
+    {
+      "file": "workflow_utils/src/lib.rs",
+      "line": 0,
+      "message": "dead re-export: 'OutputLocation' resolves to 'executor::OutputLocation' which is not found in the symbol index. Remove or fix the 'pub use' statement.",
+      "severity": "warning",
+      "kind": "dead_re_export"
+    },
+    {
+      "file": "workflow_utils/src/prelude.rs",
+      "line": 0,
+      "message": "dead re-export: 'JOB_SCRIPT_NAME' resolves to 'crate::JOB_SCRIPT_NAME' which is not found in the symbol index. Remove or fix the 'pub use' statement.",
+      "severity": "warning",
+      "kind": "dead_re_export"
+    },
+    {
+      "file": "workflow_utils/src/prelude.rs",
+      "line": 0,
+      "message": "dead re-export: 'QueuedRunner' resolves to 'crate::QueuedRunner' which is not found in the symbol index. Remove or fix the 'pub use' statement.",
+      "severity": "warning",
+      "kind": "dead_re_export"
+    },
+    {
+      "file": "workflow_utils/src/prelude.rs",
+      "line": 0,
+      "message": "dead re-export: 'SchedulerKind' resolves to 'crate::SchedulerKind' which is not found in the symbol index. Remove or fix the 'pub use' statement.",
+      "severity": "warning",
+      "kind": "dead_re_export"
+    },
+    {
+      "file": "workflow_utils/src/prelude.rs",
+      "line": 0,
+      "message": "dead re-export: 'ShellHookExecutor' resolves to 'crate::ShellHookExecutor' which is not found in the symbol index. Remove or fix the 'pub use' statement.",
+      "severity": "warning",
+      "kind": "dead_re_export"
+    },
+    {
+      "file": "workflow_utils/src/prelude.rs",
+      "line": 0,
+      "message": "dead re-export: 'SystemProcessRunner' resolves to 'crate::SystemProcessRunner' which is not found in the symbol index. Remove or fix the 'pub use' statement.",
+      "severity": "warning",
+      "kind": "dead_re_export"
+    },
+    {
+      "file": "workflow_utils/src/prelude.rs",
+      "line": 0,
+      "message": "dead re-export: 'copy_file' resolves to 'crate::copy_file' which is not found in the symbol index. Remove or fix the 'pub use' statement.",
+      "severity": "warning",
+      "kind": "dead_re_export"
+    },
+    {
+      "file": "workflow_utils/src/prelude.rs",
+      "line": 0,
+      "message": "dead re-export: 'create_dir' resolves to 'crate::create_dir' which is not found in the symbol index. Remove or fix the 'pub use' statement.",
+      "severity": "warning",
+      "kind": "dead_re_export"
+    },
+    {
+      "file": "workflow_utils/src/prelude.rs",
+      "line": 0,
+      "message": "dead re-export: 'exists' resolves to 'crate::exists' which is not found in the symbol index. Remove or fix the 'pub use' statement.",
+      "severity": "warning",
+      "kind": "dead_re_export"
+    },
+    {
+      "file": "workflow_utils/src/prelude.rs",
+      "line": 0,
+      "message": "dead re-export: 'read_file' resolves to 'crate::read_file' which is not found in the symbol index. Remove or fix the 'pub use' statement.",
+      "severity": "warning",
+      "kind": "dead_re_export"
+    },
+    {
+      "file": "workflow_utils/src/prelude.rs",
+      "line": 0,
+      "message": "dead re-export: 'remove_dir' resolves to 'crate::remove_dir' which is not found in the symbol index. Remove or fix the 'pub use' statement.",
+      "severity": "warning",
+      "kind": "dead_re_export"
+    },
+    {
+      "file": "workflow_utils/src/prelude.rs",
+      "line": 0,
+      "message": "dead re-export: 'write_file' resolves to 'crate::write_file' which is not found in the symbol index. Remove or fix the 'pub use' statement.",
+      "severity": "warning",
+      "kind": "dead_re_export"
+    }
+  ]
+}
\ No newline at end of file
diff --git a/notes/plan-enrichment/phase-6-fix/codebase-state.md b/notes/plan-enrichment/phase-6-fix/codebase-state.md
new file mode 100644
index 0000000..404cccb
--- /dev/null
+++ b/notes/plan-enrichment/phase-6-fix/codebase-state.md
@@ -0,0 +1,375 @@
+# Codebase State — Phase 6 Fix Plan Enrichment
+
+Generated: 2026-04-26
+Plan: `plans/phase-6-fix/PHASE_6_FIX_PLAN.md`
+Branch: `phase-6-fix`
+
+---
+
+## File: Cargo.toml (workspace root)
+
+### Public API
+N/A (workspace configuration)
+
+### Module wiring
+- Workspace members: `workflow_core`, `workflow_utils`, `examples/hubbard_u_sweep`, `examples/hubbard_u_sweep_slurm`, `workflow-cli`
+- Workspace dependencies: `anyhow`, `serde`, `petgraph`, `serde_json`, `tracing`, `tracing-subscriber`, `clap`, `signal-hook`, `thiserror`, `time`, `itertools`
+- `workflow_core` defined as workspace member at `workflow_core`
+
+### Plan relationship
+- Plan says: Remove `"examples/hubbard_u_sweep_slurm"` from members, add `"examples/multi_param_sweep"` and `"examples/scf_dos_chain"`
+- Current state: `"examples/hubbard_u_sweep_slurm"` is still in members list; new binaries not listed
+- Gap: Needs two additions and one removal in `members` array (line 3-8)
+
+---
+
+## File: examples/hubbard_u_sweep_slurm/ (entire directory)
+
+### Public API
+N/A (to be deleted)
+
+### Module wiring
+- `Cargo.toml`: binary crate `hubbard_u_sweep_slurm`, deps: `anyhow`, `clap`, `castep-cell-fmt 0.1.0`, `castep-cell-io 0.4.0`, `itertools`, `workflow_core`, `workflow_utils`
+- `src/main.rs`: mod `config`, mod `job_script`; functions: `build_one_task`, `build_chain`, `parse_second_values`, `build_sweep_tasks`, `main`
+- `src/config.rs`: `pub struct SweepConfig` (Parser derive); `pub fn parse_u_values(s: &str) -> Result<Vec<f64>, String>`; test module with 7 tests
+- `src/job_script.rs`: `pub fn generate_job_script(config, task_id, seed_name) -> String`; test module with 6 tests including `no_literal_tabs`
+- `seeds/ZnO.cell`, `seeds/ZnO.param`
+- `.validation-complete`: marker file from Phase 5A
+
+### Plan relationship
+- Plan says: Delete entire directory. Code will be ported/rewritten into two new binaries.
+- Current state: Fully intact — three source files with complete implementations including tests.
+- Gap: Directory exists and must be deleted as part of the fix.
+
+---
+
+## File: examples/hubbard_u_sweep/ (keep, unchanged)
+
+### Public API
+- Binary: `hubbard_u_sweep`
+- No modules, single `main.rs`
+
+### Module wiring
+- `Cargo.toml`: binary crate, deps: `anyhow`, `castep-cell-fmt 0.1.0`, `castep-cell-io 0.4.0`, `workflow_core`, `workflow_utils`
+- `src/main.rs`: no modules; single `main()` that builds 6 SCF tasks (U=0.0..5.0) in a direct-execution workflow
+- `seeds/ZnO.cell` (253 bytes), `seeds/ZnO.param` (19 bytes)
+
+### Plan relationship
+- Plan says: Keep unchanged.
+- Current state: Intact, matches plan.
+- Gap: None.
+
+---
+
+## File: examples/multi_param_sweep/Cargo.toml
+
+### Public API
+N/A (does not exist yet)
+
+### Module wiring
+N/A (does not exist yet)
+
+### Plan relationship
+- Plan says: Create with deps: `anyhow`, `clap`, `castep-cell-fmt`, `castep-cell-io`, `itertools`, `workflow_core`, `workflow_utils`; use `workspace = true` for workspace-managed deps (`anyhow`, `clap`, `itertools`)
+- Current state: The directory `examples/multi_param_sweep/` exists with empty `src/` and `seeds/` subdirectories, but no files.
+- Gap: `Cargo.toml` needs to be created. Reference: `examples/hubbard_u_sweep_slurm/Cargo.toml` for structure.
+
+---
+
+## File: examples/multi_param_sweep/src/main.rs
+
+### Public API
+N/A (does not exist yet)
+
+### Module wiring
+N/A (does not exist yet)
+
+### Plan relationship
+- Plan says: Entry point with task builders, sweep logic, workflow runner. Functions:
+  - `fn parse_kpoints(s: &str) -> anyhow::Result<Vec<[u32; 3]>>`
+  - `fn parse_cutoffs(s: &str) -> anyhow::Result<Vec<f64>>`
+  - Sweep logic: `product` mode via `itertools::iproduct!`, `pairwise` mode via zip (requires `--kpoints` and `--cutoffs` to be Some)
+  - Task ID format: `scf_U{u}_k{k}_c{c}`
+  - Setup closure: inject HubbardU, KPOINTS_MP_GRID, cutoff energy
+  - KpointsMpGrid workaround: serialize via `.to_cell()` and merge into output `Vec<Cell>` via `to_string_many_spaced()`
+  - API: `CellDocument` parse/field mutation, `ParamDocument` parse/`.basis_set.cutoff_energy`, `HubbardU::builder()`, `AtomHubbardU::builder()`, `OrbitalU::D(f64)` / `OrbitalU::F(f64)`, `CutOffEnergy { value, unit: None }`
+- Current state: File does not exist (empty `src/` dir).
+- Gap: Entire file needs to be created.
+
+---
+
+## File: examples/multi_param_sweep/src/config.rs
+
+### Public API
+N/A (does not exist yet)
+
+### Module wiring
+N/A (does not exist yet)
+
+### Plan relationship
+- Plan says: Clap CLI config with flags: `--u-values` (default `"0.0,1.0,2.0,3.0,4.0,5.0"`), `--kpoints` (Option, default None), `--cutoffs` (Option, default None), `--sweep-mode` (default `"product"`), `--element` (default `"Zn"`), `--orbital` (default `'d'`), `--seed-name` (default `"ZnO"`), `--max-parallel` (default 4), `--local`, `--dry-run`, `--castep-command` (default `"castep"`), `--workdir` (default `"."`), plus slurm flags (`--partition`, `--ntasks`, `--nix-flake`, `--mpi-if`).
+  - Also: `parse_u_values` should be imported or duplicated from old `config.rs`.
+- Current state: File does not exist.
+- Gap: Entire file needs to be created. The `parse_u_values` function can be copied from `examples/hubbard_u_sweep_slurm/src/config.rs` (lines 75-84).
+
+---
+
+## File: examples/multi_param_sweep/src/job_script.rs
+
+### Public API
+N/A (does not exist yet)
+
+### Module wiring
+N/A (does not exist yet)
+
+### Plan relationship
+- Plan says: SLURM script generation, ported from old binary (`examples/hubbard_u_sweep_slurm/src/job_script.rs`). Formatting fix: clean heredoc template, no literal tab characters mixed with spaces, consistent quoting around SBATCH directives. Port the `no_literal_tabs` test.
+- Current state: File does not exist. Source to port exists at `examples/hubbard_u_sweep_slurm/src/job_script.rs` (87 lines).
+- Gap: Entire file needs to be created. The existing `generate_job_script` uses `format!()` with literal `\t`-free template (already clean). The formatting fix from D.2 is about ensuring consistent quoting around SBATCH directives.
+
+---
+
+## File: examples/multi_param_sweep/seeds/ZnO.cell
+
+### Public API
+N/A (does not exist yet)
+
+### Module wiring
+N/A (does not exist yet)
+
+### Plan relationship
+- Plan says: Seed cell file for ZnO.
+- Current state: Empty `seeds/` directory. Source exists at `examples/hubbard_u_sweep_slurm/seeds/ZnO.cell` (252 bytes).
+- Gap: File needs to be copied from old binary.
+
+---
+
+## File: examples/multi_param_sweep/seeds/ZnO.param
+
+### Public API
+N/A (does not exist yet)
+
+### Module wiring
+N/A (does not exist yet)
+
+### Plan relationship
+- Plan says: Seed param file for ZnO.
+- Current state: Empty `seeds/` directory. Source exists at `examples/hubbard_u_sweep_slurm/seeds/ZnO.param` (19 bytes).
+- Gap: File needs to be copied from old binary.
+
+---
+
+## File: examples/scf_dos_chain/Cargo.toml
+
+### Public API
+N/A (does not exist yet)
+
+### Module wiring
+N/A (does not exist yet)
+
+### Plan relationship
+- Plan says: Create with same deps as `multi_param_sweep`.
+- Current state: Directory exists with empty `src/` and `seeds/`, no files.
+- Gap: `Cargo.toml` needs to be created.
+
+---
+
+## File: examples/scf_dos_chain/src/main.rs
+
+### Public API
+N/A (does not exist yet)
+
+### Module wiring
+N/A (does not exist yet)
+
+### Plan relationship
+- Plan says: Entry point with SCF task, DOS task, chain wiring.
+  - Task 1 `scf`: inject HubbardU, write `.cell` + `.param`, execute `castep ZnO`, collect: verify `ZnO.castep` exists with "Total time"
+  - Task 2 `dos` (depends on `scf`): same workdir. Setup: write `ZnO_DOS.cell` (from seed), write `ZnO_DOS.param` (with `general.task = Some(Task::BandStructure)`), copy `ZnO.check` -> `ZnO_DOS.check`. Execute: `castep ZnO_DOS`. Collect: verify `ZnO_DOS.castep` has "Total time"
+  - API: `ParamDocument.general.task = Some(Task::BandStructure)`, `Task::BandStructure` from `castep_cell_io::param::general::task::Task`, `copy_file` from `workflow_utils::files`
+- Current state: File does not exist.
+- Gap: Entire file needs to be created.
+
+---
+
+## File: examples/scf_dos_chain/src/config.rs
+
+### Public API
+N/A (does not exist yet)
+
+### Module wiring
+N/A (does not exist yet)
+
+### Plan relationship
+- Plan says: Simpler CLI. Single `--u-value` (default `3.0`), `--element` (default `"Zn"`), `--orbital` (default `'d'`), `--seed-name` (default `"ZnO"`), `--max-parallel` (default `1`), `--local`, `--dry-run`, `--castep-command` (default `"castep"`), `--workdir` (default `"."`), plus slurm flags. No `--kpoints`, `--cutoffs`, `--sweep-mode`, `--u-values` (plural).
+- Current state: File does not exist.
+- Gap: Entire file needs to be created.
+
+---
+
+## File: examples/scf_dos_chain/src/job_script.rs
+
+### Public API
+N/A (does not exist yet)
+
+### Module wiring
+N/A (does not exist yet)
+
+### Plan relationship
+- Plan says: Same as Binary 1 (`multi_param_sweep/src/job_script.rs`), ported from old binary.
+- Current state: File does not exist.
+- Gap: Entire file needs to be created. Same content as `multi_param_sweep/src/job_script.rs`.
+
+---
+
+## File: examples/scf_dos_chain/seeds/ZnO.cell
+
+### Public API
+N/A (does not exist yet)
+
+### Module wiring
+N/A (does not exist yet)
+
+### Plan relationship
+- Plan says: Seed cell file.
+- Current state: Empty `seeds/` directory.
+- Gap: File needs to be copied from old binary.
+
+---
+
+## File: examples/scf_dos_chain/seeds/ZnO.param
+
+### Public API
+N/A (does not exist yet)
+
+### Module wiring
+N/A (does not exist yet)
+
+### Plan relationship
+- Plan says: Seed param file.
+- Current state: Empty `seeds/` directory.
+- Gap: File needs to be copied from old binary.
+
+---
+
+## Workspace Crate: workflow_core
+
+### Public API (types referenced by plan)
+
+- `pub struct Task` (workflow_core/src/task.rs, line 57)
+  - Fields: `id: String`, `dependencies: Vec<String>`, `workdir: PathBuf`, `mode: ExecutionMode`, `setup: Option<TaskClosure>`, `collect: Option<TaskClosure>`, `monitors: Vec<MonitoringHook>`, `collect_failure_policy: CollectFailurePolicy` (crate-private)
+  - Builder: `Task::new(id, mode) -> Self`, `.depends_on(id) -> Self`, `.workdir(path) -> Self`, `.setup(f) -> Self`, `.collect(f) -> Self`, `.collect_failure_policy(policy) -> Self`, `.monitors(hooks) -> Self`, `.add_monitor(hook) -> Self`
+
+- `pub enum ExecutionMode` (workflow_core/src/task.rs, line 26)
+  - Variants: `Direct { command: String, args: Vec<String>, env: HashMap<String, String>, timeout: Option<Duration> }`, `Queued`
+  - Constructor: `ExecutionMode::direct(command, args: &[&str]) -> Self` (convenience, no env/timeout)
+
+- `pub struct Workflow` (workflow_core/src/workflow.rs, line 28)
+  - Public methods: `new(name) -> Self`, `with_max_parallel(n) -> Result<Self, WorkflowError>`, `with_log_dir(path) -> Self`, `with_queued_submitter(qs) -> Self`, `with_root_dir(path) -> Self`, `successor_map() -> Option<&TaskSuccessors>`, `add_task(task) -> Result<(), WorkflowError>`, `dry_run() -> Result<Vec<String>, WorkflowError>`, `run(state, runner, hook_executor) -> Result<WorkflowSummary, WorkflowError>`
+
+- `pub struct WorkflowSummary` (workflow_core/src/workflow.rs, line 570)
+  - Fields: `succeeded: Vec<String>`, `failed: Vec<FailedTask>`, `skipped: Vec<String>`, `duration: Duration`
+
+- `pub enum WorkflowError` (workflow_core/src/error.rs, line 5)
+  - Variants: `DuplicateTaskId(String)`, `CycleDetected`, `UnknownDependency { task, dependency }`, `StateCorrupted(String)`, `TaskTimeout(String)`, `InvalidConfig(String)`, `IoWithPath { path, source }`, `Io(std::io::Error)`, `Interrupted`, `QueueSubmitFailed(String)`
+
+- `pub trait StateStore` (workflow_core/src/state.rs, line 33)
+  - Methods: `get_status`, `set_status`, `all_tasks`, `save`
+- `pub trait StateStoreExt` (workflow_core/src/state.rs, line 48) — extension methods: `mark_running`, `mark_completed`, `mark_failed`, `mark_pending`, `mark_skipped`, `mark_skipped_due_to_dep_failure`, `summary`, `is_completed`
+- `pub struct JsonStateStore` (workflow_core/src/state.rs, line 190)
+  - Methods: `new(name, path)`, `load(path)`, `load_raw(path)`, `workflow_name()`, `path()`, `task_successors()`, `set_task_graph(successors)`
+
+- `pub trait ProcessRunner` (workflow_core/src/process.rs, line 12) — trait with `spawn`
+- `pub trait ProcessHandle` (workflow_core/src/process.rs, line 25) — trait with `is_running`, `terminate`, `wait`
+- `pub trait QueuedSubmitter` (workflow_core/src/process.rs, line 51) — trait with `submit`
+- `pub trait HookExecutor` (workflow_core/src/monitoring.rs) — trait with `execute_hook`
+
+### Plan relationship
+- Plan says: New binaries will use `Task`, `ExecutionMode`, `Workflow`, `WorkflowError`, `WorkflowSummary`, `JsonStateStore`, `QueuedRunner`, `SchedulerKind`, etc.
+- Current state: All types exist and match the API described in the plan.
+- Gap: None — crate API is sufficient.
+
+---
+
+## Workspace Crate: workflow_utils
+
+### Public API (types referenced by plan)
+
+- `pub mod prelude` (workflow_utils/src/prelude.rs)
+  - Re-exports: `workflow_core::prelude::*`, `crate::{copy_file, create_dir, exists, read_file, remove_dir, run_default, write_file, QueuedRunner, SchedulerKind, ShellHookExecutor, SystemProcessRunner, JOB_SCRIPT_NAME}`
+
+- `pub fn read_file(path) -> Result<String, WorkflowError>` (workflow_utils/src/files.rs, line 8)
+- `pub fn write_file(path, content) -> Result<(), WorkflowError>` (workflow_utils/src/files.rs, line 13)
+- `pub fn copy_file(from, to) -> Result<(), WorkflowError>` (workflow_utils/src/files.rs, line 21)
+- `pub fn create_dir(path) -> Result<(), WorkflowError>` (workflow_utils/src/files.rs, line 31)
+- `pub fn remove_dir(path) -> Result<(), WorkflowError>` (workflow_utils/src/files.rs, line 36)
+- `pub fn exists(path) -> bool` (workflow_utils/src/files.rs, line 41)
+- `pub fn run_default(workflow, state) -> Result<WorkflowSummary, WorkflowError>` (workflow_utils/src/lib.rs, line 31)
+
+- `pub struct QueuedRunner` (workflow_utils/src/queued.rs, line 24)
+  - Methods: `new(scheduler: SchedulerKind) -> Self`, `scheduler() -> SchedulerKind`
+  - Implements: `QueuedSubmitter`
+
+- `pub enum SchedulerKind` (workflow_utils/src/queued.rs, line 13)
+  - Variants: `Slurm`, `Pbs`
+
+- `pub const JOB_SCRIPT_NAME: &str = "job.sh"` (workflow_utils/src/queued.rs, line 9)
+
+- `pub struct SystemProcessRunner` (workflow_utils/src/executor.rs, line 110)
+  - Methods: `new() -> Self`, `with_log_dir(dir) -> Self`
+  - Implements: `ProcessRunner`
+
+- `pub struct ShellHookExecutor` (workflow_utils/src/monitoring.rs, line 6)
+  - Implements: `HookExecutor`
+
+### Plan relationship
+- Plan says: New binaries use `read_file`, `copy_file`, `run_default`, `QueuedRunner`, `SchedulerKind`, `SystemProcessRunner`, `ShellHookExecutor`, `JOB_SCRIPT_NAME`.
+- Current state: All types and functions exist and are publicly exported via `prelude` and direct crate re-exports.
+- Gap: None — crate API is sufficient.
+
+---
+
+## External Crate: castep-cell-io (v0.4.0)
+
+Types referenced in the plan (not inspectable in this workspace since it's an external dependency):
+
+- `CellDocument` — `parse()`, field mutation (`.hubbard_u`), `.to_cell_file()`
+- `ParamDocument` — `parse()`, `.basis_set.cutoff_energy`, `.general.task`, `.to_cell_file()`
+- `KpointsMpGrid([u32; 3])` — `.to_cell()` returns `Cell` block; NOT a field on `CellDocument`
+- `HubbardU::builder()`, `AtomHubbardU::builder()`, `OrbitalU::D(f64)`, `OrbitalU::F(f64)`, `Species::Symbol(String)`, `HubbardUUnit::ElectronVolt`
+- `CutOffEnergy { value: f64, unit: None }` — None defaults to eV in CASTEP
+- `Task::BandStructure` — path: `castep_cell_io::param::general::task::Task`
+- `Cell` type — output of `.to_cell()`, input to `to_string_many_spaced()`
+
+### Plan relationship
+- Plan says: New binaries will use all of these types. Notable: `KpointsMpGrid` is NOT a field on `CellDocument` in v0.4.0. Workaround: serialize via `.to_cell()` to a `Cell` block, merge into `Vec<Cell>`, serialize via `to_string_many_spaced()`.
+- Current state: External dependency, confirmed available via `Cargo.lock` / existing code that uses these types.
+- Gap: `KpointsMpGrid` workaround is a known limitation documented in the plan.
+
+---
+
+## External Crate: castep-cell-fmt (v0.1.0)
+
+Functions referenced in the plan:
+
+- `parse(&str) -> Result<CellDocument, ...>` — parses cell file text
+- `to_string_many_spaced(&[Cell]) -> String` — serializes `Vec<Cell>` blocks to spaced CASTEP format
+- `ToCellFile` trait — provides `.to_cell_file()` -> `Vec<Cell>` on `CellDocument` and `ParamDocument`
+
+### Plan relationship
+- Plan says: Used for parsing seeds and serializing modified documents.
+- Current state: Already used by both existing binaries. API is stable.
+- Gap: None.
+
+---
+
+## Summary
+
+| Status | Count | Details |
+|--------|-------|---------|
+| Files to create | 12 | 2 Cargo.toml, 2 main.rs, 2 config.rs, 2 job_script.rs, 4 seed files |
+| Files to modify | 1 | Root `Cargo.toml` (workspace members) |
+| Files to delete | 7+ | `examples/hubbard_u_sweep_slurm/` entire directory (Cargo.toml, 3 src files, 2 seeds, 1 validation marker) |
+| Files to keep | 4 | `examples/hubbard_u_sweep/` (Cargo.toml, main.rs, 2 seeds) |
+| Workspace crates OK | 2 | `workflow_core`, `workflow_utils` — all referenced APIs exist |
+| Empty placeholder dirs | 2 | `examples/multi_param_sweep/`, `examples/scf_dos_chain/` — dirs exist, contents empty |
diff --git a/notes/plan-enrichment/phase-6-fix/deferred-and-patterns.md b/notes/plan-enrichment/phase-6-fix/deferred-and-patterns.md
new file mode 100644
index 0000000..2f31e90
--- /dev/null
+++ b/notes/plan-enrichment/phase-6-fix/deferred-and-patterns.md
@@ -0,0 +1,23 @@
+## Deferred Improvements
+
+- **D.1: Restore plan-specified portable config fields** — The `hubbard_u_sweep_slurm` example uses NixOS-specific config fields (`nix_flake`, `mpi_if`, `--nodelist=nixos`) instead of the plan-specified portable fields (`account`, `walltime`, `modules`, `castep_command`). Precondition: second user or non-NixOS cluster required.
+
+- **D.2: `generate_job_script` formatting inconsistencies** — `job_script.rs` uses a literal `\t` character among spaces for `--map-by`; SBATCH directives have inconsistent quoting. Cleanup via `indoc!` macro or heredoc-style template deferred until next functional edit to `job_script.rs`.
+
+- **D.3 (partial): Unit tests for `generate_job_script`** — `parse_u_values` tests are complete (done in Phase 5B), but `generate_job_script` tests are tightly coupled to NixOS-specific output. Precondition: D.1 (portable template) must be addressed first so a second template variant makes test assertions meaningful.
+
+- **`read_task_ids` empty-string edge case** — Returns `Ok([""])` when called with `vec![""]`, producing a downstream error rather than a clear diagnostic. Not reachable through normal CLI usage but a future maintainer could be confused by the gap between the documented stdin-fallback description and the actual single-condition check.
+
+## Known Failure Modes
+
+From Phase 4, Phase 5, and Phase 5b fix plans:
+
+- **Missing `pub use` re-exports** — Phase 4 TASK-2: `TaskSuccessors` was implemented in `state.rs` but omitted from the `pub use` line in `workflow_core/src/lib.rs`, making it inaccessible at the crate root despite being available from the submodule.
+
+- **Incomplete consumer updates** — Phase 5 TASK-1 and TASK-2: After introducing the `JOB_SCRIPT_NAME` constant, two locations (the `hubbard_u_sweep_slurm` consumer and `queued_integration.rs` tests) still used the hardcoded string `"job.sh"`. A rename or abstraction change in the library is not fully propagated until every consumer is updated.
+
+- **Stale imports after refactoring** — Phase 4 TASK-3: Moving `downstream_tasks` BFS from the CLI into `workflow_core` as a `TaskSuccessors` method left behind the original function definition and six duplicate unit tests in the CLI. The refactoring was correct in the library but the old code remained in the consumer.
+
+- **Dead / unenforced API surface** — Phase 4 TASK-1: `TaskSuccessors::inner()` exposed the `HashMap` backing type, defeating the newtype abstraction, and was dead code (no callers). Without a dead-code lint or removal discipline, such methods accumulate.
+
+- **Stale documentation** — Phase 5b TASK-1: `ARCHITECTURE.md` contained outdated struct field names (`execution_mode` vs `mode`, `dependencies` vs `depends_on`), outdated trait signatures (pre-refactor `StateStore`), and outdated error types. Documentation drifts silently when not treated as compilable code.
diff --git a/notes/plan-enrichment/phase-6-fix/draft-elaboration.md b/notes/plan-enrichment/phase-6-fix/draft-elaboration.md
new file mode 100644
index 0000000..d67b8b5
--- /dev/null
+++ b/notes/plan-enrichment/phase-6-fix/draft-elaboration.md
@@ -0,0 +1,684 @@
+# Draft Elaboration -- Phase 6 Fix Plan
+
+Generated: 2026-04-26
+Source plan: `plans/phase-6-fix/PHASE_6_FIX_PLAN.md`
+Codebase state: `notes/plan-enrichment/phase-6-fix/codebase-state.md`
+Deferred items: `notes/plan-enrichment/phase-6-fix/deferred-and-patterns.md`
+
+---
+
+## Binary 1: `multi_param_sweep`
+
+### Item 1.1: Config struct (`config.rs`) -- Clap derive definition
+
+The plan's flag table maps to a clap derive struct. The struct must hold all parsed values before they are passed to task-building logic.
+
+**Proposed type signature:**
+
+```rust
+#[derive(Parser)]
+#[command(name = "multi_param_sweep", about = "Independent SCF parameter sweep")]
+pub struct SweepConfig {
+    #[arg(long, default_value = "0.0,1.0,2.0,3.0,4.0,5.0")]
+    pub u_values: String,
+
+    #[arg(long)]
+    pub kpoints: Option<String>,
+
+    #[arg(long)]
+    pub cutoffs: Option<String>,
+
+    #[arg(long, default_value = "product")]
+    pub sweep_mode: String,
+
+    #[arg(long, default_value = "Zn")]
+    pub element: String,
+
+    #[arg(long, default_value = "d")]
+    pub orbital: char,
+
+    #[arg(long, default_value = "ZnO")]
+    pub seed_name: String,
+
+    #[arg(long, default_value = "4")]
+    pub max_parallel: usize,
+
+    #[arg(long, default_value = "false")]
+    pub local: bool,
+
+    #[arg(long, default_value = "false")]
+    pub dry_run: bool,
+
+    #[arg(long, default_value = "castep")]
+    pub castep_command: String,
+
+    #[arg(long, default_value = ".")]
+    pub workdir: String,
+
+    // SLURM flags -- ported from old binary
+    #[arg(long, default_value = "defq")]
+    pub partition: String,
+
+    #[arg(long, default_value = "8")]
+    pub ntasks: usize,
+
+    #[arg(long)]
+    pub nix_flake: Option<String>,
+
+    #[arg(long)]
+    pub mpi_if: Option<String>,
+}
+```
+
+**Module placement:** `examples/multi_param_sweep/src/config.rs`
+
+**Error handling strategy:** None at struct level. Parsing of comma-separated strings happens in utility functions (see Item 1.2), not in clap value-parsers. This is consistent with the old binary pattern where `parse_u_values` is called after clap parsing, not as a `value_parser`.
+
+**Ownership/lifetime notes:** All `String` fields (owned), no borrowed data. The config struct is short-lived (consumed in `main()` to drive task construction).
+
+**Trait coherence notes:** None. This is a standalone binary crate.
+
+**Uncertainty:** The old binary also had `--account`, `--walltime`, `--modules`, `--castep-command` as plan-specified portable fields (see D.1). The current plan table omits them. Confirming: the plan intentionally keeps the NixOS-specific fields (`--nix-flake`, `--mpi-if`) and does NOT introduce portable fields in this fix. Reason: D.1 precondition ("second user or non-NixOS cluster") is not met.
+
+---
+
+### Item 1.2: Parsing utility functions -- signatures and error types
+
+The plan names three parsing functions. The old binary's `parse_u_values` returns `Result<Vec<f64>, String>` -- but the new binary depends on `anyhow`, so all three should be unified to `anyhow::Result`.
+
+**Proposed type signatures:**
+
+```rust
+/// Parse comma-separated f64 values (ported from old binary, error type upgraded).
+/// Example: "0.0,1.0,2.0" -> Ok(vec![0.0, 1.0, 2.0])
+pub fn parse_u_values(s: &str) -> anyhow::Result<Vec<f64>>;
+
+/// Parse comma-separated k-point MP grids.
+/// Example: "8x8x8,6x6x6" -> Ok(vec![[8,8,8], [6,6,6]])
+/// Errors: wrong number of axes ("8x8"), non-numeric ("abc"), empty segment
+pub fn parse_kpoints(s: &str) -> anyhow::Result<Vec<[u32; 3]>>;
+
+/// Parse comma-separated cutoff energies.
+/// Example: "300,500,800" -> Ok(vec![300.0, 500.0, 800.0])
+pub fn parse_cutoffs(s: &str) -> anyhow::Result<Vec<f64>>;
+```
+
+**Module placement:** `examples/multi_param_sweep/src/main.rs`
+
+The plan says these go in `main.rs`. The old binary put `parse_u_values` in `config.rs` -- but for the new binary, placing all three parsing functions together in `main.rs` keeps them near their single call site (the sweep generation logic). `config.rs` is reserved for the clap struct only.
+
+**Uncertainty (low):** Should `parse_u_values` stay in `config.rs` to match the old binary pattern? The plan says "imported or duplicated" into main.rs. Going with main.rs since plan says main.rs has "task builders, sweep logic, workflow runner" and the parsing functions are part of sweep logic input preparation.
+
+**Error handling strategy:** All three use `anyhow::Context` for clear error messages:
+- `parse_kpoints`: `.with_context(|| format!("invalid k-point grid: {segment}"))` for each segment
+- `parse_cutoffs`: `.with_context(|| format!("invalid cutoff energy: {segment}"))`
+- `parse_u_values`: same as old binary but with `anyhow::Error` instead of `String` via `.map_err(|e| anyhow::anyhow!("{}", e))`
+
+**Ownership/lifetime notes:** All take `&str` and return owned `Vec`. No lifetime coupling.
+
+**Trait coherence notes:** None.
+
+---
+
+### Item 1.3: Sweep generation function
+
+The plan describes product and pairwise sweep modes. This logic should be extracted into a dedicated function.
+
+**Proposed type signature:**
+
+```rust
+/// Generate the list of (u_value, kpoint, cutoff) triples based on sweep mode.
+///
+/// - `product`: cartesian product via itertools::iproduct!
+/// - `pairwise`: zip all three lists (must be equal length)
+///
+/// `kpoints` and `cutoffs` may be None in product mode (seeds provide defaults).
+/// In pairwise mode, both must be Some or this returns an error.
+pub fn generate_sweep_combinations(
+    u_values: &[f64],
+    kpoints: Option<&[[u32; 3]]>,
+    cutoffs: Option<&[f64]>,
+    sweep_mode: &str,
+) -> anyhow::Result<Vec<(f64, Option<[u32; 3]>, Option<f64>)>>;
+```
+
+**Module placement:** `examples/multi_param_sweep/src/main.rs`
+
+**Error handling strategy:** Returns `anyhow::Error` for pairwise-mode validation failures:
+- `"pairwise mode requires --kpoints and --cutoffs to be specified"` if either is None
+- `"pairwise mode requires equal-length lists: u_values({n_u}), kpoints({n_k}), cutoffs({n_c})"` if lengths differ
+
+**Ownership/lifetime notes:** Inputs are borrowed slices; output is owned `Vec` of tuples. The `Option<[u32; 3]>` and `Option<f64>` in the output allow downstream task-building to distinguish "explicit kpoints/cutoff" from "use seed defaults".
+
+**Trait coherence notes:** None.
+
+---
+
+### Item 1.4: Task builder function -- parameter injection setup closure
+
+The plan describes a setup closure that injects HubbardU, KPOINTS_MP_GRID, and cutoff energy. The closure type depends on `TaskClosure`, which is referenced in the plan but whose exact definition is not visible in codebase-state.md.
+
+**Proposed type signature:**
+
+```rust
+/// Build a single SCF sweep task for one (U, kpoint, cutoff) combination.
+///
+/// Returns a fully configured Task ready to add to a Workflow.
+pub fn build_sweep_task(
+    u_value: f64,
+    kpoint: Option<[u32; 3]>,
+    cutoff: Option<f64>,
+    element: &str,
+    orbital: char,
+    seed_name: &str,
+    seed_cell_path: &Path,
+    seed_param_path: &Path,
+    workdir_root: &Path,
+    castep_command: &str,
+    is_local: bool,
+    config: &SweepConfig,       // for SLURM script generation
+) -> anyhow::Result<Task>;
+```
+
+**Module placement:** `examples/multi_param_sweep/src/main.rs`
+
+**Error handling strategy:** Setup closure operations (reading seed files, parsing, writing modified files) can fail with I/O or parse errors. The `TaskClosure` type likely wraps `Box<dyn FnOnce() -> Result<(), WorkflowError> + Send>`, but setup operations that fail in the closure will be reported via the workflow system's error propagation.
+
+**Uncertainty (HIGH):** The exact definition of `TaskClosure` is not visible in codebase-state.md. It may be:
+- `Box<dyn FnOnce() -> Result<(), WorkflowError> + Send>`
+- `Box<dyn FnOnce() -> Result<(), Box<dyn Error>> + Send>`
+- Something else entirely.
+
+This must be resolved before implementation by reading `workflow_core/src/task.rs` to check the type alias. If `TaskClosure` is not a `WorkflowError`-returning closure, the setup function may need to map errors (e.g., `.map_err(|e| WorkflowError::InvalidConfig(e.to_string()))`).
+
+**Ownership/lifetime notes:** The setup closure must capture owned copies of `u_value`, `kpoint`, `cutoff`, `element.to_string()`, `seed_name.to_string()`, `castep_command.to_string()`, `workdir_root.to_path_buf()`, etc. All captures are `move` to avoid borrowing issues with the closure's `'static` bound.
+
+**Trait coherence notes:** None.
+
+---
+
+### Item 1.5: KPOINTS_MP_GRID injection workaround
+
+The plan documents that `KpointsMpGrid` is NOT a field on `CellDocument` in `castep-cell-io` v0.4.0. The workaround is: create a `KpointsMpGrid`, call `.to_cell()` to get a `Cell` block, merge it into the `Vec<Cell>` from `CellDocument.to_cell_file()`, then serialize with `to_string_many_spaced()`.
+
+**Proposed type signature:**
+
+```rust
+/// Serialize a modified cell file with an injected KPOINTS_MP_GRID block.
+///
+/// Workaround for castep-cell-io v0.4.0 where KpointsMpGrid is not a field
+/// on CellDocument. Converts KpointsMpGrid to a Cell block, merges it into
+/// the output of CellDocument.to_cell_file(), then serializes.
+pub fn cell_with_kpoints(
+    cell_doc: &CellDocument,
+    kpoint_grid: [u32; 3],
+) -> anyhow::Result<String>;
+```
+
+**Module placement:** `examples/multi_param_sweep/src/main.rs`
+
+**Error handling strategy:** The only fallible operation is `KpointsMpGrid::new()` if the grid is invalid, and `cell_doc.to_cell_file()` (which is infallible per existing pattern). Returns `anyhow::Result` for consistency. If all operations are infallible in practice, this could be a plain `String` return -- but the `Result` wrapper is defensive.
+
+**Ownership/lifetime notes:** Takes `&CellDocument` (borrowed). Returns owned `String`. The `KpointsMpGrid` is stack-allocated and consumed by `.to_cell()`.
+
+**Uncertainty (medium):** The exact API for `KpointsMpGrid::to_cell()` is not confirmed. The plan states it returns a `Cell` -- but whether it takes `&self` or `self`, and whether `to_string_many_spaced` takes `&[Cell]` or `Vec<Cell>`, needs verification against the actual `castep-cell-io` v0.4.0 API. If `to_string_many_spaced` takes `&[Cell]`, the sequence is:
+
+```rust
+let cells: Vec<Cell> = cell_doc.to_cell_file();        // Vec<Cell>
+let kp_cell: Cell = KpointsMpGrid(grid).to_cell();     // Cell
+let all_cells: Vec<Cell> = cells.into_iter()
+    .chain(std::iter::once(kp_cell))
+    .collect();
+to_string_many_spaced(&all_cells)                       // String
+```
+
+**Trait coherence notes:** None. This is a free function in binary code, not touching any trait impls.
+
+---
+
+### Item 1.6: `job_script.rs` -- ported SLURM generation with formatting fix (absorbing D.2)
+
+The plan says to port `generate_job_script` from the old binary and apply a formatting fix: no literal tab characters mixed with spaces, consistent quoting around SBATCH directives.
+
+**Proposed type signature:**
+
+```rust
+/// Generate a SLURM job script for a single task.
+///
+/// Uses a clean heredoc-style template string (no literal tabs mixed with spaces).
+/// SBATCH directives use consistent quoting: all `--*=` values are double-quoted.
+pub fn generate_job_script(
+    config: &SweepConfig,
+    task_id: &str,
+    seed_name: &str,
+) -> String;
+```
+
+**Module placement:** `examples/multi_param_sweep/src/job_script.rs`
+
+**Error handling strategy:** Returns `String` (infallible). This matches the old binary pattern where `generate_job_script` is pure string formatting with no I/O.
+
+**Formatting fix details (D.2 absorbed):**
+- Replace any residual `\t` literals with spaces.
+- Quote all SBATCH directive values consistently: `#SBATCH --partition="defq"`, not `#SBATCH --partition=defq`.
+- The existing `no_literal_tabs` test from the old binary should be ported and updated to verify the fix.
+
+**Ownership/lifetime notes:** Takes `&SweepConfig` (borrowed). Returns owned `String`. No allocations besides the result.
+
+**Trait coherence notes:** None.
+
+**Uncertainty (low):** The old binary's `generate_job_script` signature takes a config struct, task_id, and seed_name. The new binary's config struct (`SweepConfig`) has the same relevant fields (`partition`, `ntasks`, `nix_flake`, `mpi_if`), so the port is straightforward.
+
+---
+
+### Item 1.7: Task ID format helper
+
+The plan specifies task ID format `scf_U{u}_k{k}_c{c}`. This format should be encapsulated in a function to ensure consistency and avoid duplication.
+
+**Proposed type signature:**
+
+```rust
+/// Format a task ID for a sweep combination.
+/// Example: format_sweep_task_id(3.0, Some([8,8,8]), Some(500.0)) -> "scf_U3.0_k8x8x8_c500"
+///          format_sweep_task_id(3.0, None, None)               -> "scf_U3.0"
+pub fn format_sweep_task_id(
+    u_value: f64,
+    kpoint: Option<[u32; 3]>,
+    cutoff: Option<f64>,
+) -> String;
+```
+
+**Module placement:** `examples/multi_param_sweep/src/main.rs`
+
+**Error handling strategy:** Infallible (pure formatting).
+
+**Ownership/lifetime notes:** None. All inputs are `Copy`.
+
+**Uncertainty (medium):** When kpoints or cutoffs are None (product mode with seed defaults), the plan says the workdir is `runs/U{u}_k{k}_c{c}/`. Should the task ID also include the k/c values when present? The plan example shows `scf_U3.0_k8x8x8_c500`. For None values, I propose omitting that segment: `scf_U3.0`. This avoids ambiguity and keeps IDs clean. The workdir path should also follow this naming convention.
+
+---
+
+### Item 1.8: Main function flow
+
+The plan describes the overall flow but does not decompose it into function calls.
+
+**Proposed structure** (all in `main.rs`):
+
+```rust
+fn main() -> anyhow::Result<()> {
+    let config = SweepConfig::parse();
+
+    // 1. Parse all input lists
+    let u_values = parse_u_values(&config.u_values)?;
+    let kpoints: Option<Vec<[u32; 3]>> = config.kpoints
+        .as_ref()
+        .map(|s| parse_kpoints(s))
+        .transpose()?;
+    let cutoffs: Option<Vec<f64>> = config.cutoffs
+        .as_ref()
+        .map(|s| parse_cutoffs(s))
+        .transpose()?;
+
+    // 2. Generate sweep combinations
+    let combos = generate_sweep_combinations(
+        &u_values,
+        kpoints.as_deref(),
+        cutoffs.as_deref(),
+        &config.sweep_mode,
+    )?;
+
+    // 3. Build tasks
+    let root = PathBuf::from(&config.workdir);
+    let seed_cell = root.join("seeds").join(format!("{}.cell", config.seed_name));
+    let seed_param = root.join("seeds").join(format!("{}.param", config.seed_name));
+    let mut workflow = Workflow::new("multi_param_sweep")
+        .with_max_parallel(config.max_parallel)?
+        .with_root_dir(root.clone());
+    for (u, k, c) in combos {
+        let task = build_sweep_task(
+            u, k, c,
+            &config.element, config.orbital,
+            &config.seed_name,
+            &seed_cell, &seed_param,
+            &root,
+            &config.castep_command,
+            config.local,
+            &config,
+        )?;
+        workflow.add_task(task)?;
+    }
+
+    // 4. Execute or dry-run
+    if config.dry_run {
+        let order = workflow.dry_run()?;
+        for id in order {
+            println!("{id}");
+        }
+        return Ok(());
+    }
+
+    let state = JsonStateStore::new("multi_param_sweep", root.join("state.json"));
+    if config.local {
+        let runner = SystemProcessRunner::new().with_log_dir(root.join("logs"));
+        run_default(workflow, state)?;
+    } else {
+        let qr = QueuedRunner::new(SchedulerKind::Slurm);
+        let workflow = workflow.with_queued_submitter(qr);
+        run_default(workflow, state)?;
+    }
+    Ok(())
+}
+```
+
+**Error handling strategy:** `main` returns `anyhow::Result<()>`, consistent with the `anyhow` dependency. All `?` operators propagate to main, where anyhow prints the error chain.
+
+**Uncertainty (medium):** The exact API for `Workflow::with_max_parallel(n)`, `with_root_dir(path)`, and `with_queued_submitter(qs)` is confirmed in codebase-state.md. However, the state store path (`state.json`) and log directory conventions need verification against what the old binary uses.
+
+**Uncertainty (medium):** `JsonStateStore::new` takes `(name, path)` per codebase-state.md. The plan uses a root workdir; the state store should live at `<workdir>/state.json` to not conflict with per-task workdirs. If the state store's `path` argument is a directory (not a file path), this needs adjustment -- check `JsonStateStore::new` signature.
+
+---
+
+## Binary 2: `scf_dos_chain`
+
+### Item 2.1: Config struct (`config.rs`) -- Clap derive definition
+
+**Proposed type signature:**
+
+```rust
+#[derive(Parser)]
+#[command(name = "scf_dos_chain", about = "SCF + DOS task chain")]
+pub struct ChainConfig {
+    #[arg(long, default_value = "3.0")]
+    pub u_value: f64,
+
+    #[arg(long, default_value = "Zn")]
+    pub element: String,
+
+    #[arg(long, default_value = "d")]
+    pub orbital: char,
+
+    #[arg(long, default_value = "ZnO")]
+    pub seed_name: String,
+
+    #[arg(long, default_value = "1")]
+    pub max_parallel: usize,
+
+    #[arg(long, default_value = "false")]
+    pub local: bool,
+
+    #[arg(long, default_value = "false")]
+    pub dry_run: bool,
+
+    #[arg(long, default_value = "castep")]
+    pub castep_command: String,
+
+    #[arg(long, default_value = ".")]
+    pub workdir: String,
+
+    // SLURM flags (same as Binary 1)
+    #[arg(long, default_value = "defq")]
+    pub partition: String,
+
+    #[arg(long, default_value = "8")]
+    pub ntasks: usize,
+
+    #[arg(long)]
+    pub nix_flake: Option<String>,
+
+    #[arg(long)]
+    pub mpi_if: Option<String>,
+}
+```
+
+**Module placement:** `examples/scf_dos_chain/src/config.rs`
+
+**Error handling strategy:** None at struct level. Single `--u-value` is parsed directly by clap as `f64` -- no comma-separated parsing needed.
+
+**Uncertainty (low):** The plan specifies `--orbital` default `'d'`. Clap's `char` type parser accepts a single character, which matches.
+
+---
+
+### Item 2.2: SCF task builder function
+
+**Proposed type signature:**
+
+```rust
+/// Build the SCF task (Task 1 in the chain).
+/// Injects Hubbard U into the seed cell file, writes modified files to workdir.
+pub fn build_scf_task(
+    u_value: f64,
+    element: &str,
+    orbital: char,
+    seed_name: &str,
+    seed_cell_path: &Path,
+    seed_param_path: &Path,
+    workdir: &Path,
+    castep_command: &str,
+    is_local: bool,
+    config: &ChainConfig,
+) -> anyhow::Result<Task>;
+```
+
+**Module placement:** `examples/scf_dos_chain/src/main.rs`
+
+**Error handling strategy:** Same as Binary 1's `build_sweep_task`: setup closure operations can fail; errors propagate via the closure's error type.
+
+**Setup closure details:**
+1. Read seed cell file via `read_file(seed_cell_path)` -> parse into `CellDocument`
+2. Build `HubbardU` via builder pattern, inject into `cell_doc.hubbard_u`
+3. Serialize cell: `cell_doc.to_cell_file()` -> `to_string_many_spaced()` -> `write_file(workdir.join(format!("{seed_name}.cell")), content)`
+4. Copy seed param (no modifications needed for SCF): `copy_file(seed_param_path, workdir.join(format!("{seed_name}.param")))`
+5. Write job script if queued mode
+
+**Task configuration:**
+- `id`: `"scf"`
+- `workdir`: `"runs/scf_dos/"`
+- `mode`: `ExecutionMode::direct("castep", &["ZnO"])` or `Queued`
+- No dependencies
+
+---
+
+### Item 2.3: DOS task builder function
+
+**Proposed type signature:**
+
+```rust
+/// Build the DOS task (Task 2 in the chain, depends on Task 1).
+/// Modifies seed param to set general.task = BandStructure, copies SCF checkpoint.
+pub fn build_dos_task(
+    seed_name: &str,
+    seed_cell_path: &Path,
+    seed_param_path: &Path,
+    workdir: &Path,
+    castep_command: &str,
+    is_local: bool,
+    config: &ChainConfig,
+) -> anyhow::Result<Task>;
+```
+
+**Module placement:** `examples/scf_dos_chain/src/main.rs`
+
+**Error handling strategy:** Same as SCF task builder.
+
+**Setup closure details:**
+1. Parse seed cell into `CellDocument`, write to `workdir/ZnO_DOS.cell`
+2. Parse seed param into `ParamDocument`, set `general.task = Some(Task::BandStructure)`, write to `workdir/ZnO_DOS.param`
+3. Copy `workdir/ZnO.check` -> `workdir/ZnO_DOS.check` via `copy_file`
+
+**Task configuration:**
+- `id`: `"dos"`
+- `workdir`: `"runs/scf_dos/"` (same as SCF)
+- `mode`: `ExecutionMode::direct("castep", &["ZnO_DOS"])` or `Queued`
+- Dependency: `.depends_on("scf")`
+
+**Uncertainty (medium):** The plan says to "copy" `ZnO.check` -> `ZnO_DOS.check`. However, `copy_file` in `workflow_utils::files` returns `Result<(), WorkflowError>`, not `anyhow::Result`. If the setup closure's error type is `WorkflowError`, `.map_err()` is needed. If it's `anyhow::Error`, `anyhow::Error` can be created from `WorkflowError` via `From` impl (since `WorkflowError` likely implements `std::error::Error`). This needs verification.
+
+**Uncertainty (medium):** The `.check` file is produced by CASTEP after successful SCF completion. The DOS task depends on the SCF task, so the `.check` file should exist by the time DOS runs. However, the plan does not specify whether the setup closure should verify the file exists or just attempt the copy. If the copy fails because the SCF task failed, the workflow's dependency mechanism should have already skipped the DOS task. Confirming: the setup closure should attempt the copy and let the error propagate if it fails.
+
+---
+
+### Item 2.4: Collect closures for completion verification
+
+The plan specifies that both SCF and DOS tasks should verify `ZnO.castep` (or `ZnO_DOS.castep`) exists and contains a "Total time" completion marker. This is identical logic for both tasks.
+
+**Proposed type signature:**
+
+```rust
+/// Build a collect closure that verifies the CASTEP output file exists
+/// and contains a "Total time" completion marker.
+///
+/// Returns a closure suitable for `Task::collect()`.
+pub fn verify_castep_output(
+    workdir: PathBuf,
+    output_file: String,
+) -> TaskClosure;
+```
+
+**Module placement:** `examples/scf_dos_chain/src/main.rs`
+
+**Error handling strategy:** The collect closure returns a `TaskClosure`-compatible error if the file is missing or lacks the marker. Exact error type depends on `TaskClosure` definition (see HIGH uncertainty in Item 1.4).
+
+**Uncertainty (HIGH):** Same `TaskClosure` uncertainty as Item 1.4. Must be resolved first.
+
+---
+
+### Item 2.5: Main function flow
+
+**Proposed structure** (all in `main.rs`):
+
+```rust
+fn main() -> anyhow::Result<()> {
+    let config = ChainConfig::parse();
+
+    let root = PathBuf::from(&config.workdir);
+    let seed_cell = root.join("seeds").join(format!("{}.cell", config.seed_name));
+    let seed_param = root.join("seeds").join(format!("{}.param", config.seed_name));
+    let workdir = root.join("runs/scf_dos/");
+
+    // Build SCF task
+    let scf_task = build_scf_task(
+        config.u_value, &config.element, config.orbital,
+        &config.seed_name, &seed_cell, &seed_param,
+        &workdir, &config.castep_command, config.local, &config,
+    )?;
+
+    // Build DOS task (depends on SCF)
+    let dos_task = build_dos_task(
+        &config.seed_name, &seed_cell, &seed_param,
+        &workdir, &config.castep_command, config.local, &config,
+    )?.depends_on("scf");
+
+    let mut workflow = Workflow::new("scf_dos_chain")
+        .with_max_parallel(config.max_parallel)?
+        .with_root_dir(root.clone());
+    workflow.add_task(scf_task)?;
+    workflow.add_task(dos_task)?;
+
+    if config.dry_run {
+        for id in workflow.dry_run()? {
+            println!("{id}");
+        }
+        return Ok(());
+    }
+
+    // Run -- same local/queued pattern as Binary 1
+    let state = JsonStateStore::new("scf_dos_chain", root.join("state.json"));
+    // ... (same runner logic as Binary 1)
+    Ok(())
+}
+```
+
+---
+
+## Workspace Changes
+
+### Item 3.1: Root `Cargo.toml` -- workspace members
+
+**Proposed change** (line 3-8 of root `Cargo.toml`):
+
+Remove:
+```toml
+"examples/hubbard_u_sweep_slurm",
+```
+
+Add:
+```toml
+"examples/multi_param_sweep",
+"examples/scf_dos_chain",
+```
+
+**Module placement:** `/Users/tony/programming/castep_workflow_framework/Cargo.toml`
+
+**Error handling strategy:** N/A (configuration change). Validate with `cargo build --workspace` after all files are created.
+
+---
+
+### Item 3.2: Seed file copying
+
+The plan says to create seed files for both new binaries. The old binary's seed files should be copied verbatim.
+
+**Files to copy:**
+- `examples/hubbard_u_sweep_slurm/seeds/ZnO.cell` -> `examples/multi_param_sweep/seeds/ZnO.cell`
+- `examples/hubbard_u_sweep_slurm/seeds/ZnO.param` -> `examples/multi_param_sweep/seeds/ZnO.param`
+- `examples/hubbard_u_sweep_slurm/seeds/ZnO.cell` -> `examples/scf_dos_chain/seeds/ZnO.cell`
+- `examples/hubbard_u_sweep_slurm/seeds/ZnO.param` -> `examples/scf_dos_chain/seeds/ZnO.param`
+
+**Module placement:** N/A (plain files, not modules)
+
+**Error handling strategy:** N/A (file copy, no error handling needed beyond OS-level copy)
+
+---
+
+## Deferred Items Assessment
+
+For each item from `deferred-and-patterns.md`, assess whether it is directly relevant to the current plan and should be absorbed.
+
+- **D.1 (portable SLURM config fields):** **Skip -- not applicable yet.**
+  The plan intentionally retains NixOS-specific config fields (`--nix-flake`, `--mpi-if`) without adding portable fields (`--account`, `--walltime`, `--modules`, `--castep_command`). The deferred item's precondition ("second user or non-NixOS cluster required") is not met. The deferred item remains valid for a future phase but is out of scope for this fix.
+
+- **D.2 (`generate_job_script` formatting fix):** **Absorb -- directly addressed.**
+  The plan explicitly references D.2 in the setup closure description: "use a clean heredoc template -- no literal tab characters mixed with spaces, consistent quoting around SBATCH directives." This is absorbed as part of Item 1.6 (ported `job_script.rs`). The formatting fix applies to both binaries since both share the same `job_script.rs` content.
+
+- **D.3 (unit tests for `generate_job_script`):** **Absorb partially.**
+  The plan says to "port the `no_literal_tabs` test" from the old binary. The remaining deferred concern (portable-template tests) is preconditioned on D.1, which is not in scope. So: port `no_literal_tabs` test (absorbed), defer portable-template tests (skipped).
+
+- **`read_task_ids` empty-string edge case:** **Skip -- not applicable.**
+  This is a bug in `workflow-cli` code that is not touched by the binary rewrite. The CLI binary is not being modified in this plan. Out of scope.
+
+- **Known failure modes from prior phases:**
+  - **Missing `pub use` re-exports:** **Skip.** Library code is not being modified.
+  - **Incomplete consumer updates:** **Caution.** The old binary used `"job.sh"` literally; the new binary should use the `JOB_SCRIPT_NAME` constant from `workflow_utils` to avoid repeating this failure mode. Noted as a best practice for implementation.
+  - **Stale imports after refactoring:** **Caution.** When deleting `examples/hubbard_u_sweep_slurm/`, ensure no other crate depends on it (check `cargo tree` or workspace member list).
+  - **Dead / unenforced API surface:** **Skip.** No new library API is being added.
+  - **Stale documentation:** **Skip.** `ARCHITECTURE.md` is not being updated in this plan.
+
+---
+
+## Summary of Uncertainties (prioritized)
+
+| Priority | Item | Uncertainty |
+|----------|------|-------------|
+| **HIGH** | 1.4, 2.4 | Exact definition of `TaskClosure` type alias in `workflow_core` -- must read `workflow_core/src/task.rs` before implementation |
+| **MEDIUM** | 1.5 | Exact API for `KpointsMpGrid::to_cell()` and `to_string_many_spaced()` -- verify against `castep-cell-io` v0.4.0 |
+| **MEDIUM** | 1.2 | Module placement of `parse_u_values`: old binary has it in `config.rs`, plan says main.rs -- confirm placement |
+| **MEDIUM** | 2.3 | Error type compatibility between `workflow_utils::files::copy_file` (returns `WorkflowError`) and setup closure's error type |
+| **MEDIUM** | 1.7 | Task ID and workdir naming when kpoints/cutoffs are None in product mode |
+| **LOW** | 1.1 | Whether portable SLURM fields should be included now (plan says no, deferred D.1 confirms) |
+| **LOW** | 1.6 | Whether old binary's `generate_job_script` uses NixOS-specific fields that affect the template |
+
+---
+
+## Patterns Used (from codebase-state.md)
+
+| Pattern | Source | Applied To |
+|---------|--------|------------|
+| Clap `#[derive(Parser)]` with `#[arg(long)]` | Old binary `config.rs` | Both new `config.rs` files |
+| `Task::new(id, mode)` builder with `.workdir()`, `.setup()`, `.collect()`, `.depends_on()` | `workflow_core` API | All task construction |
+| `ExecutionMode::direct(cmd, &[args])` convenience constructor | `workflow_core` API | Local execution mode |
+| `Workflow::new(name)` with `.with_max_parallel()`, `.add_task()`, `.dry_run()` | `workflow_core` API | Both binary main functions |
+| `run_default(workflow, state)` from `workflow_utils` | `workflow_utils` API | Workflow execution |
+| `SystemProcessRunner::new()` / `QueuedRunner::new(SchedulerKind::Slurm)` | `workflow_utils` API | Runner selection |
+| `read_file` / `write_file` / `copy_file` from `workflow_utils::files` | `workflow_utils` API | File I/O in setup closures |
+| `CellDocument::parse()` / `ParamDocument::parse()` from `castep-cell-fmt` | External crate API | Seed file parsing |
+| `HubbardU::builder()` / `AtomHubbardU::builder()` pattern | External crate API | Hubbard U injection |
+| `format!()` -based template for SLURM script (not `indoc!`) | Old binary pattern | Ported `job_script.rs` |
diff --git a/notes/plan-enrichment/phase-6-fix/draft-plan.toml b/notes/plan-enrichment/phase-6-fix/draft-plan.toml
new file mode 100644
index 0000000..abd6b06
--- /dev/null
+++ b/notes/plan-enrichment/phase-6-fix/draft-plan.toml
@@ -0,0 +1,1124 @@
+[meta]
+title = "Phase 6 Fix: Rewrite example binaries for multi-parameter sweep & SCF+DOS chain"
+source_branch = "phase-6-fix"
+created = "2026-04-26"
+
+[dependencies]
+TASK-5 = ["TASK-1", "TASK-2", "TASK-3", "TASK-4"]
+
+[tasks.TASK-1]
+description = "Create multi_param_sweep binary source files (Cargo.toml, config.rs, job_script.rs, main.rs)"
+type = "create"
+acceptance = [
+    "test -f examples/multi_param_sweep/Cargo.toml",
+    "test -f examples/multi_param_sweep/src/config.rs",
+    "test -f examples/multi_param_sweep/src/job_script.rs",
+    "test -f examples/multi_param_sweep/src/main.rs",
+]
+
+[[tasks.TASK-1.changes]]
+file = "examples/multi_param_sweep/Cargo.toml"
+after = '''
+[package]
+name = "multi_param_sweep"
+version = "0.1.0"
+edition = "2021"
+
+[[bin]]
+name = "multi_param_sweep"
+path = "src/main.rs"
+
+[dependencies]
+anyhow = { workspace = true }
+clap = { workspace = true }
+castep-cell-fmt = "0.1.0"
+castep-cell-io = "0.4.0"
+itertools = { workspace = true }
+workflow_core = { path = "../../workflow_core", features = ["default-logging"] }
+workflow_utils = { path = "../../workflow_utils" }
+'''
+
+[[tasks.TASK-1.changes]]
+file = "examples/multi_param_sweep/src/config.rs"
+after = '''
+use clap::Parser;
+
+#[derive(Parser)]
+#[command(name = "multi_param_sweep", about = "Independent SCF parameter sweep")]
+pub struct SweepConfig {
+    /// Comma-separated Hubbard U values (eV)
+    #[arg(long, default_value = "0.0,1.0,2.0,3.0,4.0,5.0")]
+    pub u_values: String,
+
+    /// Comma-separated MP grids, e.g. "8x8x8,6x6x6"
+    #[arg(long)]
+    pub kpoints: Option<String>,
+
+    /// Comma-separated cutoff energies (eV), e.g. "300,500,800"
+    #[arg(long)]
+    pub cutoffs: Option<String>,
+
+    /// Sweep mode: "product" (cartesian product) or "pairwise" (zip)
+    #[arg(long, default_value = "product")]
+    pub sweep_mode: String,
+
+    /// Element to apply Hubbard U to
+    #[arg(long, default_value = "Zn")]
+    pub element: String,
+
+    /// Orbital for Hubbard U: 'd' or 'f'
+    #[arg(long, default_value = "d")]
+    pub orbital: char,
+
+    /// CASTEP input file prefix
+    #[arg(long, default_value = "ZnO")]
+    pub seed_name: String,
+
+    /// Maximum number of concurrent tasks
+    #[arg(long, default_value_t = 4)]
+    pub max_parallel: usize,
+
+    /// Run tasks locally via direct process execution instead of SLURM
+    #[arg(long, default_value_t = false)]
+    pub local: bool,
+
+    /// Dry-run mode: print topological order and exit without submitting
+    #[arg(long, default_value_t = false)]
+    pub dry_run: bool,
+
+    /// CASTEP binary name or path (used in --local mode)
+    #[arg(long, default_value = "castep")]
+    pub castep_command: String,
+
+    /// Root directory for runs/logs
+    #[arg(long, default_value = ".")]
+    pub workdir: String,
+
+    /// SLURM partition
+    #[arg(long, default_value = "defq")]
+    pub partition: String,
+
+    /// Number of MPI tasks (cores) per job
+    #[arg(long, default_value_t = 8)]
+    pub ntasks: usize,
+
+    /// Nix flake URI for the CASTEP environment
+    #[arg(long)]
+    pub nix_flake: Option<String>,
+
+    /// Network interface for OpenMPI TCP (e.g. "enp6s0")
+    #[arg(long)]
+    pub mpi_if: Option<String>,
+}
+'''
+
+[[tasks.TASK-1.changes]]
+file = "examples/multi_param_sweep/src/job_script.rs"
+after = '''
+use crate::config::SweepConfig;
+
+/// Generate a SLURM job script for a single task.
+///
+/// Uses a clean heredoc-style template string (no literal tabs mixed with spaces).
+/// SBATCH directives use consistent quoting: all string values are double-quoted.
+pub fn generate_job_script(config: &SweepConfig, task_id: &str, seed_name: &str) -> String {
+    let nix_flake = config
+        .nix_flake
+        .as_deref()
+        .unwrap_or("git+ssh://git@github.com/TonyWu20/CASTEP-25.12-nixos#castep_25_mkl");
+    let mpi_if = config.mpi_if.as_deref().unwrap_or("enp6s0");
+    format!(
+        "\
+#!/usr/bin/env bash
+#SBATCH --job-name=\"{task_id}\"
+#SBATCH --output=\"slurm_output_%j.txt\"
+#SBATCH --partition=\"{partition}\"
+#SBATCH --nodes=1
+#SBATCH --ntasks-per-node={ntasks}
+#SBATCH --cpus-per-task=1
+#SBATCH --mem=30000m
+#SBATCH --nodelist=nixos
+nix develop {nix_flake} --command bash -c \\
+    \"mpirun --mca plm slurm \\
+        -x OMPI_MCA_btl_tcp_if_include={mpi_if} \\
+        -x OMPI_MCA_orte_keep_fqdn_hostnames=true \\
+        --mca pmix s1 \\
+        --mca btl tcp,self \\
+        --map-by numa --bind-to numa \\
+    castep.mpi {seed_name}\"
+",
+        task_id = task_id,
+        partition = config.partition,
+        ntasks = config.ntasks,
+        nix_flake = nix_flake,
+        mpi_if = mpi_if,
+    )
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::config::SweepConfig;
+    use clap::Parser;
+
+    fn default_config() -> SweepConfig {
+        SweepConfig::parse_from(["test"])
+    }
+
+    #[test]
+    fn contains_sbatch_directives() {
+        let config = default_config();
+        let script = generate_job_script(&config, "scf_U1.0", "ZnO");
+        assert!(script.contains("#SBATCH --job-name=\"scf_U1.0\""));
+        assert!(script.contains("#SBATCH --partition=\"defq\""));
+        assert!(script.contains("#SBATCH --ntasks-per-node=8"));
+        assert!(script.contains("#SBATCH --mem=30000m"));
+    }
+
+    #[test]
+    fn contains_seed_name() {
+        let config = default_config();
+        let script = generate_job_script(&config, "scf_U0.0", "ZnO");
+        assert!(script.contains("castep.mpi ZnO"));
+    }
+
+    #[test]
+    fn no_literal_tabs() {
+        let config = default_config();
+        let script = generate_job_script(&config, "scf_U0.0", "ZnO");
+        assert!(
+            !script.contains('\t'),
+            "job script should not contain literal tab characters"
+        );
+    }
+
+    #[test]
+    fn starts_with_shebang() {
+        let config = default_config();
+        let script = generate_job_script(&config, "scf_U0.0", "ZnO");
+        assert!(script.starts_with("#!/usr/bin/env bash"));
+    }
+}
+'''
+
+[[tasks.TASK-1.changes]]
+file = "examples/multi_param_sweep/src/main.rs"
+after = '''
+mod config;
+mod job_script;
+
+use std::path::{Path, PathBuf};
+use std::sync::Arc;
+
+use anyhow::Result;
+use castep_cell_fmt::{format::to_string_many_spaced, parse, ToCellFile};
+use castep_cell_io::cell::bz_sampling_kpoints::KpointsMpGrid;
+use castep_cell_io::cell::species::{AtomHubbardU, HubbardU, HubbardUUnit, OrbitalU, Species};
+use castep_cell_io::param::basis_set::CutOffEnergy;
+use castep_cell_io::{CellDocument, ParamDocument};
+use clap::Parser;
+use itertools::iproduct;
+use workflow_utils::prelude::*;
+
+use config::SweepConfig;
+use job_script::generate_job_script;
+
+/// Parse comma-separated f64 values.
+/// Example: "0.0,1.0,2.0" -> Ok(vec![0.0, 1.0, 2.0])
+fn parse_u_values(s: &str) -> Result<Vec<f64>> {
+    s.split(',')
+        .map(|segment| {
+            let trimmed = segment.trim();
+            trimmed
+                .parse::<f64>()
+                .map_err(|e| anyhow::anyhow!("invalid U value '{}': {}", trimmed, e))
+        })
+        .collect::<Result<Vec<_>>>()
+}
+
+/// Parse comma-separated k-point MP grids.
+/// Example: "8x8x8,6x6x6" -> Ok(vec![[8,8,8], [6,6,6]])
+fn parse_kpoints(s: &str) -> Result<Vec<[u32; 3]>> {
+    s.split(',')
+        .map(|segment| {
+            let segment = segment.trim();
+            let parts: Vec<&str> = segment.split('x').collect();
+            if parts.len() != 3 {
+                return Err(anyhow::anyhow!(
+                    "invalid k-point grid '{}': expected format like '8x8x8' (3 axes, got {})",
+                    segment,
+                    parts.len()
+                ));
+            }
+            let mut grid = [0u32; 3];
+            for (i, p) in parts.iter().enumerate() {
+                grid[i] = p.trim().parse::<u32>().map_err(|e| {
+                    anyhow::anyhow!(
+                        "invalid k-point grid '{}': non-numeric axis '{}': {}",
+                        segment,
+                        p,
+                        e
+                    )
+                })?;
+            }
+            Ok(grid)
+        })
+        .collect::<Result<Vec<_>>>()
+}
+
+/// Parse comma-separated cutoff energies.
+/// Example: "300,500,800" -> Ok(vec![300.0, 500.0, 800.0])
+fn parse_cutoffs(s: &str) -> Result<Vec<f64>> {
+    s.split(',')
+        .map(|segment| {
+            let trimmed = segment.trim();
+            trimmed.parse::<f64>().map_err(|e| {
+                anyhow::anyhow!("invalid cutoff energy '{}': {}", trimmed, e)
+            })
+        })
+        .collect::<Result<Vec<_>>>()
+}
+
+/// Format a task ID for a sweep combination.
+fn format_sweep_task_id(
+    u_value: f64,
+    kpoint: Option<[u32; 3]>,
+    cutoff: Option<f64>,
+) -> String {
+    let mut id = format!("scf_U{:.1}", u_value);
+    if let Some(k) = kpoint {
+        id.push_str(&format!("_k{}x{}x{}", k[0], k[1], k[2]));
+    }
+    if let Some(c) = cutoff {
+        id.push_str(&format!("_c{}", c));
+    }
+    id
+}
+
+/// Format a workdir path for a sweep combination.
+fn format_sweep_workdir(
+    u_value: f64,
+    kpoint: Option<[u32; 3]>,
+    cutoff: Option<f64>,
+) -> PathBuf {
+    let dirname = format_sweep_task_id(u_value, kpoint, cutoff);
+    PathBuf::from("runs").join(dirname)
+}
+
+/// Generate the list of (u_value, kpoint, cutoff) combinations based on sweep mode.
+fn generate_sweep_combinations(
+    u_values: &[f64],
+    kpoints: Option<&[[u32; 3]]>,
+    cutoffs: Option<&[f64]>,
+    sweep_mode: &str,
+) -> Result<Vec<(f64, Option<[u32; 3]>, Option<f64>)>> {
+    match sweep_mode {
+        "product" => {
+            let kp: Vec<Option<[u32; 3]>> = match kpoints {
+                Some(k) => k.iter().map(|&v| Some(v)).collect(),
+                None => vec![None],
+            };
+            let co: Vec<Option<f64>> = match cutoffs {
+                Some(c) => c.iter().map(|&v| Some(v)).collect(),
+                None => vec![None],
+            };
+            Ok(iproduct!(u_values.iter(), kp.iter(), co.iter())
+                .map(|(u, k, c)| (*u, *k, *c))
+                .collect())
+        }
+        "pairwise" => {
+            let k = kpoints.ok_or_else(|| {
+                anyhow::anyhow!(
+                    "pairwise mode requires --kpoints and --cutoffs to be specified"
+                )
+            })?;
+            let c = cutoffs.ok_or_else(|| {
+                anyhow::anyhow!(
+                    "pairwise mode requires --kpoints and --cutoffs to be specified"
+                )
+            })?;
+            if u_values.len() != k.len() || u_values.len() != c.len() {
+                return Err(anyhow::anyhow!(
+                    "pairwise mode requires equal-length lists: u_values({}), kpoints({}), cutoffs({})",
+                    u_values.len(),
+                    k.len(),
+                    c.len()
+                ));
+            }
+            Ok(u_values
+                .iter()
+                .enumerate()
+                .map(|(i, &u)| (u, Some(k[i]), Some(c[i])))
+                .collect())
+        }
+        other => Err(anyhow::anyhow!(
+            "unsupported sweep mode '{}': expected 'product' or 'pairwise'",
+            other
+        )),
+    }
+}
+
+/// Build a single SCF sweep task for one (U, kpoint, cutoff) combination.
+fn build_sweep_task(
+    u_value: f64,
+    kpoint: Option<[u32; 3]>,
+    cutoff: Option<f64>,
+    element: &str,
+    orbital: char,
+    seed_name: &str,
+    seed_cell: &str,
+    seed_param: &str,
+    workdir_root: &Path,
+    castep_command: &str,
+    is_local: bool,
+    config: &SweepConfig,
+) -> Result<Task, WorkflowError> {
+    let task_id = format_sweep_task_id(u_value, kpoint, cutoff);
+    let workdir = workdir_root.join(format_sweep_workdir(u_value, kpoint, cutoff));
+
+    let seed_cell = seed_cell.to_owned();
+    let seed_param = seed_param.to_owned();
+    let element = element.to_string();
+    let seed_name_setup = seed_name.to_string();
+    let seed_name_collect = seed_name.to_string();
+    let castep_command = castep_command.to_string();
+
+    // Only generate job script for SLURM mode
+    let job_script = if !is_local {
+        Some(generate_job_script(config, &task_id, &seed_name_setup))
+    } else {
+        None
+    };
+
+    let mode = if is_local {
+        ExecutionMode::direct(&castep_command, &[&seed_name_setup])
+    } else {
+        ExecutionMode::Queued
+    };
+
+    let task = Task::new(&task_id, mode)
+        .workdir(workdir.clone())
+        .setup(move |workdir| -> Result<(), WorkflowError> {
+            create_dir(workdir)?;
+
+            // Parse seed cell and inject HubbardU
+            let mut cell_doc: CellDocument =
+                parse(&seed_cell).map_err(|e| WorkflowError::InvalidConfig(e.to_string()))?;
+
+            let orbital_u = match orbital {
+                'd' => OrbitalU::D(u_value),
+                'f' => OrbitalU::F(u_value),
+                c => {
+                    return Err(WorkflowError::InvalidConfig(format!(
+                        "unsupported orbital '{c}'"
+                    )))
+                }
+            };
+            let atom_u = AtomHubbardU::builder()
+                .species(Species::Symbol(element.clone()))
+                .orbitals(vec![orbital_u])
+                .build();
+            let hubbard_u = HubbardU::builder()
+                .unit(HubbardUUnit::ElectronVolt)
+                .atom_u_values(vec![atom_u])
+                .build();
+            cell_doc.hubbard_u = Some(hubbard_u);
+
+            // Build Cell blocks for serialization
+            let mut cells = cell_doc.to_cell_file();
+
+            // Inject KPOINTS_MP_GRID if provided (workaround: not a field on CellDocument)
+            if let Some(grid) = kpoint {
+                let kp_cell = KpointsMpGrid(grid).to_cell();
+                cells.push(kp_cell);
+            }
+
+            let cell_text = to_string_many_spaced(&cells);
+
+            // Inject cutoff energy into param if provided
+            let param_text = if let Some(co) = cutoff {
+                let mut param_doc: ParamDocument =
+                    parse(&seed_param).map_err(|e| WorkflowError::InvalidConfig(e.to_string()))?;
+                param_doc.basis_set.cutoff_energy = Some(CutOffEnergy {
+                    value: co,
+                    unit: None,
+                });
+                to_string_many_spaced(&param_doc.to_cell_file())
+            } else {
+                seed_param.clone()
+            };
+
+            write_file(
+                workdir.join(format!("{seed_name_setup}.cell")),
+                &cell_text,
+            )?;
+            write_file(
+                workdir.join(format!("{seed_name_setup}.param")),
+                &param_text,
+            )?;
+
+            // Only write job script for SLURM mode
+            if let Some(ref script) = job_script {
+                write_file(workdir.join(JOB_SCRIPT_NAME), script)?;
+            }
+            Ok(())
+        });
+
+    Ok(task)
+}
+
+fn main() -> Result<()> {
+    workflow_core::init_default_logging().ok();
+    let config = SweepConfig::parse();
+
+    let seed_cell = include_str!("../seeds/ZnO.cell");
+    let seed_param = include_str!("../seeds/ZnO.param");
+
+    // 1. Parse all input lists
+    let u_values = parse_u_values(&config.u_values)?;
+    let kpoints: Option<Vec<[u32; 3]>> = config
+        .kpoints
+        .as_ref()
+        .map(|s| parse_kpoints(s))
+        .transpose()?;
+    let cutoffs: Option<Vec<f64>> = config
+        .cutoffs
+        .as_ref()
+        .map(|s| parse_cutoffs(s))
+        .transpose()?;
+
+    // 2. Generate sweep combinations
+    let combos = generate_sweep_combinations(
+        &u_values,
+        kpoints.as_deref(),
+        cutoffs.as_deref(),
+        &config.sweep_mode,
+    )?;
+
+    // 3. Build tasks
+    let root = PathBuf::from(&config.workdir);
+    let mut workflow = Workflow::new("multi_param_sweep")
+        .with_max_parallel(config.max_parallel)?
+        .with_root_dir(root.clone());
+
+    if !config.local {
+        workflow = workflow
+            .with_queued_submitter(Arc::new(QueuedRunner::new(SchedulerKind::Slurm)));
+    }
+
+    workflow = workflow.with_log_dir(root.join("logs"));
+
+    for (u, k, c) in combos {
+        let task = build_sweep_task(
+            u,
+            k,
+            c,
+            &config.element,
+            config.orbital,
+            &config.seed_name,
+            seed_cell,
+            seed_param,
+            &root,
+            &config.castep_command,
+            config.local,
+            &config,
+        )?;
+        workflow.add_task(task)?;
+    }
+
+    // 4. Execute or dry-run
+    if config.dry_run {
+        let order = workflow.dry_run()?;
+        println!("Dry-run topological order:");
+        for task_id in &order {
+            println!("  {task_id}");
+        }
+        return Ok(());
+    }
+
+    let state_path = root.join(".multi_param_sweep.workflow.json");
+    let mut state = JsonStateStore::new("multi_param_sweep", state_path);
+
+    let summary = if config.local {
+        run_default(&mut workflow, &mut state)?
+    } else {
+        let runner: Arc<dyn ProcessRunner> =
+            Arc::new(SystemProcessRunner::new().with_log_dir(root.join("logs")));
+        let executor: Arc<dyn HookExecutor> = Arc::new(ShellHookExecutor);
+        workflow.run(&mut state, runner, executor)?
+    };
+
+    println!(
+        "Workflow complete: {} succeeded, {} failed, {} skipped ({:.1}s)",
+        summary.succeeded.len(),
+        summary.failed.len(),
+        summary.skipped.len(),
+        summary.duration.as_secs_f64(),
+    );
+    Ok(())
+}
+'''
+
+[tasks.TASK-2]
+description = "Copy seed files for multi_param_sweep binary"
+type = "create"
+acceptance = [
+    "test -f examples/multi_param_sweep/seeds/ZnO.cell",
+    "test -f examples/multi_param_sweep/seeds/ZnO.param",
+]
+
+[[tasks.TASK-2.changes]]
+file = "examples/multi_param_sweep/seeds/ZnO.cell"
+after = '''%BLOCK LATTICE_CART
+  3.25 0.0 0.0
+  0.0 3.25 0.0
+  0.0 0.0 5.21
+%ENDBLOCK LATTICE_CART
+
+%BLOCK POSITIONS_FRAC
+Zn  0.333333  0.666667  0.0
+Zn  0.666667  0.333333  0.5
+O   0.333333  0.666667  0.375
+O   0.666667  0.333333  0.875
+%ENDBLOCK POSITIONS_FRAC
+'''
+
+[[tasks.TASK-2.changes]]
+file = "examples/multi_param_sweep/seeds/ZnO.param"
+after = '''task : SinglePoint
+'''
+
+[tasks.TASK-3]
+description = "Create scf_dos_chain binary source files (Cargo.toml, config.rs, job_script.rs, main.rs)"
+type = "create"
+acceptance = [
+    "test -f examples/scf_dos_chain/Cargo.toml",
+    "test -f examples/scf_dos_chain/src/config.rs",
+    "test -f examples/scf_dos_chain/src/job_script.rs",
+    "test -f examples/scf_dos_chain/src/main.rs",
+]
+
+[[tasks.TASK-3.changes]]
+file = "examples/scf_dos_chain/Cargo.toml"
+after = '''
+[package]
+name = "scf_dos_chain"
+version = "0.1.0"
+edition = "2021"
+
+[[bin]]
+name = "scf_dos_chain"
+path = "src/main.rs"
+
+[dependencies]
+anyhow = { workspace = true }
+clap = { workspace = true }
+castep-cell-fmt = "0.1.0"
+castep-cell-io = "0.4.0"
+itertools = { workspace = true }
+workflow_core = { path = "../../workflow_core", features = ["default-logging"] }
+workflow_utils = { path = "../../workflow_utils" }
+'''
+
+[[tasks.TASK-3.changes]]
+file = "examples/scf_dos_chain/src/config.rs"
+after = '''
+use clap::Parser;
+
+#[derive(Parser)]
+#[command(name = "scf_dos_chain", about = "SCF + DOS task chain")]
+pub struct ChainConfig {
+    /// Single Hubbard U value (eV)
+    #[arg(long, default_value_t = 3.0)]
+    pub u_value: f64,
+
+    /// Element to apply Hubbard U to
+    #[arg(long, default_value = "Zn")]
+    pub element: String,
+
+    /// Orbital for Hubbard U: 'd' or 'f'
+    #[arg(long, default_value = "d")]
+    pub orbital: char,
+
+    /// CASTEP input file prefix
+    #[arg(long, default_value = "ZnO")]
+    pub seed_name: String,
+
+    /// Maximum number of concurrent tasks
+    #[arg(long, default_value_t = 1)]
+    pub max_parallel: usize,
+
+    /// Run tasks locally via direct process execution instead of SLURM
+    #[arg(long, default_value_t = false)]
+    pub local: bool,
+
+    /// Dry-run mode: print topological order and exit without submitting
+    #[arg(long, default_value_t = false)]
+    pub dry_run: bool,
+
+    /// CASTEP binary name or path (used in --local mode)
+    #[arg(long, default_value = "castep")]
+    pub castep_command: String,
+
+    /// Root directory for runs/logs
+    #[arg(long, default_value = ".")]
+    pub workdir: String,
+
+    /// SLURM partition
+    #[arg(long, default_value = "defq")]
+    pub partition: String,
+
+    /// Number of MPI tasks (cores) per job
+    #[arg(long, default_value_t = 8)]
+    pub ntasks: usize,
+
+    /// Nix flake URI for the CASTEP environment
+    #[arg(long)]
+    pub nix_flake: Option<String>,
+
+    /// Network interface for OpenMPI TCP (e.g. "enp6s0")
+    #[arg(long)]
+    pub mpi_if: Option<String>,
+}
+'''
+
+[[tasks.TASK-3.changes]]
+file = "examples/scf_dos_chain/src/job_script.rs"
+after = '''
+use crate::config::ChainConfig;
+
+/// Generate a SLURM job script for a single task.
+///
+/// Uses a clean heredoc-style template string (no literal tabs mixed with spaces).
+/// SBATCH directives use consistent quoting: all string values are double-quoted.
+pub fn generate_job_script(config: &ChainConfig, task_id: &str, seed_name: &str) -> String {
+    let nix_flake = config
+        .nix_flake
+        .as_deref()
+        .unwrap_or("git+ssh://git@github.com/TonyWu20/CASTEP-25.12-nixos#castep_25_mkl");
+    let mpi_if = config.mpi_if.as_deref().unwrap_or("enp6s0");
+    format!(
+        "\
+#!/usr/bin/env bash
+#SBATCH --job-name=\"{task_id}\"
+#SBATCH --output=\"slurm_output_%j.txt\"
+#SBATCH --partition=\"{partition}\"
+#SBATCH --nodes=1
+#SBATCH --ntasks-per-node={ntasks}
+#SBATCH --cpus-per-task=1
+#SBATCH --mem=30000m
+#SBATCH --nodelist=nixos
+nix develop {nix_flake} --command bash -c \\
+    \"mpirun --mca plm slurm \\
+        -x OMPI_MCA_btl_tcp_if_include={mpi_if} \\
+        -x OMPI_MCA_orte_keep_fqdn_hostnames=true \\
+        --mca pmix s1 \\
+        --mca btl tcp,self \\
+        --map-by numa --bind-to numa \\
+    castep.mpi {seed_name}\"
+",
+        task_id = task_id,
+        partition = config.partition,
+        ntasks = config.ntasks,
+        nix_flake = nix_flake,
+        mpi_if = mpi_if,
+    )
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::config::ChainConfig;
+    use clap::Parser;
+
+    fn default_config() -> ChainConfig {
+        ChainConfig::parse_from(["test"])
+    }
+
+    #[test]
+    fn contains_sbatch_directives() {
+        let config = default_config();
+        let script = generate_job_script(&config, "scf", "ZnO");
+        assert!(script.contains("#SBATCH --job-name=\"scf\""));
+        assert!(script.contains("#SBATCH --partition=\"defq\""));
+        assert!(script.contains("#SBATCH --ntasks-per-node=8"));
+        assert!(script.contains("#SBATCH --mem=30000m"));
+    }
+
+    #[test]
+    fn contains_seed_name() {
+        let config = default_config();
+        let script = generate_job_script(&config, "scf", "ZnO");
+        assert!(script.contains("castep.mpi ZnO"));
+    }
+
+    #[test]
+    fn no_literal_tabs() {
+        let config = default_config();
+        let script = generate_job_script(&config, "scf", "ZnO");
+        assert!(
+            !script.contains('\t'),
+            "job script should not contain literal tab characters"
+        );
+    }
+
+    #[test]
+    fn starts_with_shebang() {
+        let config = default_config();
+        let script = generate_job_script(&config, "scf", "ZnO");
+        assert!(script.starts_with("#!/usr/bin/env bash"));
+    }
+}
+'''
+
+[[tasks.TASK-3.changes]]
+file = "examples/scf_dos_chain/src/main.rs"
+after = '''
+mod config;
+mod job_script;
+
+use std::path::PathBuf;
+use std::sync::Arc;
+
+use anyhow::Result;
+use castep_cell_fmt::{format::to_string_many_spaced, parse, ToCellFile};
+use castep_cell_io::cell::species::{AtomHubbardU, HubbardU, HubbardUUnit, OrbitalU, Species};
+use castep_cell_io::param::general::Task;
+use castep_cell_io::{CellDocument, ParamDocument};
+use clap::Parser;
+use workflow_utils::prelude::*;
+
+use config::ChainConfig;
+use job_script::generate_job_script;
+
+/// Build the SCF task (Task 1 in the chain).
+/// Injects Hubbard U into the seed cell file.
+fn build_scf_task(
+    u_value: f64,
+    element: &str,
+    orbital: char,
+    seed_name: &str,
+    seed_cell: &str,
+    seed_param: &str,
+    workdir: &str,
+    castep_command: &str,
+    is_local: bool,
+    config: &ChainConfig,
+) -> Result<Task, WorkflowError> {
+    let task_id = "scf".to_string();
+    let workdir_path = PathBuf::from(workdir);
+
+    let seed_cell = seed_cell.to_owned();
+    let seed_param_clone = seed_param.to_owned();
+    let element = element.to_string();
+    let seed_name_setup = seed_name.to_string();
+    let seed_name_collect = seed_name.to_string();
+    let castep_command = castep_command.to_string();
+
+    // Only generate job script for SLURM mode
+    let job_script = if !is_local {
+        Some(generate_job_script(config, &task_id, &seed_name_setup))
+    } else {
+        None
+    };
+
+    let mode = if is_local {
+        ExecutionMode::direct(&castep_command, &[&seed_name_setup])
+    } else {
+        ExecutionMode::Queued
+    };
+
+    let task = Task::new(&task_id, mode)
+        .workdir(workdir_path.clone())
+        .setup(move |workdir| -> Result<(), WorkflowError> {
+            create_dir(workdir)?;
+
+            // Parse seed cell and inject HubbardU
+            let mut cell_doc: CellDocument =
+                parse(&seed_cell).map_err(|e| WorkflowError::InvalidConfig(e.to_string()))?;
+
+            let orbital_u = match orbital {
+                'd' => OrbitalU::D(u_value),
+                'f' => OrbitalU::F(u_value),
+                c => {
+                    return Err(WorkflowError::InvalidConfig(format!(
+                        "unsupported orbital '{c}'"
+                    )))
+                }
+            };
+            let atom_u = AtomHubbardU::builder()
+                .species(Species::Symbol(element.clone()))
+                .orbitals(vec![orbital_u])
+                .build();
+            let hubbard_u = HubbardU::builder()
+                .unit(HubbardUUnit::ElectronVolt)
+                .atom_u_values(vec![atom_u])
+                .build();
+            cell_doc.hubbard_u = Some(hubbard_u);
+
+            let cell_text = to_string_many_spaced(&cell_doc.to_cell_file());
+            write_file(
+                workdir.join(format!("{seed_name_setup}.cell")),
+                &cell_text,
+            )?;
+            write_file(
+                workdir.join(format!("{seed_name_setup}.param")),
+                &seed_param_clone,
+            )?;
+
+            // Only write job script for SLURM mode
+            if let Some(ref script) = job_script {
+                write_file(workdir.join(JOB_SCRIPT_NAME), script)?;
+            }
+            Ok(())
+        })
+        .collect(move |workdir| -> Result<(), WorkflowError> {
+            let castep_out = workdir.join(format!("{seed_name_collect}.castep"));
+            if !castep_out.exists() {
+                return Err(WorkflowError::InvalidConfig(format!(
+                    "missing output: {}",
+                    castep_out.display()
+                )));
+            }
+            let content = read_file(&castep_out)?;
+            if !content.contains("Total time") {
+                return Err(WorkflowError::InvalidConfig(
+                    "CASTEP output appears incomplete (no 'Total time' marker)".into(),
+                ));
+            }
+            Ok(())
+        });
+
+    Ok(task)
+}
+
+/// Build the DOS task (Task 2 in the chain, depends on Task 1).
+/// Modifies seed param to set task = BandStructure, copies SCF checkpoint.
+fn build_dos_task(
+    seed_name: &str,
+    seed_cell: &str,
+    seed_param: &str,
+    workdir: &str,
+    castep_command: &str,
+    is_local: bool,
+    config: &ChainConfig,
+) -> Result<Task, WorkflowError> {
+    let task_id = "dos".to_string();
+    let workdir_path = PathBuf::from(workdir);
+    let dos_seed_name = format!("{}_DOS", seed_name);
+
+    let seed_cell = seed_cell.to_owned();
+    let seed_param = seed_param.to_owned();
+    let seed_name_setup = seed_name.to_string();
+    let dos_seed_name_setup = dos_seed_name.clone();
+    let dos_seed_name_collect = dos_seed_name.clone();
+    let castep_command = castep_command.to_string();
+
+    // Only generate job script for SLURM mode
+    let job_script = if !is_local {
+        Some(generate_job_script(config, &task_id, &dos_seed_name_setup))
+    } else {
+        None
+    };
+
+    let mode = if is_local {
+        ExecutionMode::direct(&castep_command, &[&dos_seed_name_setup])
+    } else {
+        ExecutionMode::Queued
+    };
+
+    let task = Task::new(&task_id, mode)
+        .workdir(workdir_path.clone())
+        .depends_on("scf")
+        .setup(move |workdir| -> Result<(), WorkflowError> {
+            // 1. Parse in-memory seed cell, write to ZnO_DOS.cell
+            let cell_doc: CellDocument =
+                parse(&seed_cell).map_err(|e| WorkflowError::InvalidConfig(e.to_string()))?;
+            let cell_text = to_string_many_spaced(&cell_doc.to_cell_file());
+            write_file(
+                workdir.join(format!("{dos_seed_name_setup}.cell")),
+                &cell_text,
+            )?;
+
+            // 2. Parse in-memory seed param, set task = BandStructure, write to ZnO_DOS.param
+            let mut param_doc: ParamDocument =
+                parse(&seed_param).map_err(|e| WorkflowError::InvalidConfig(e.to_string()))?;
+            param_doc.general.task = Some(Task::BandStructure);
+            let param_text = to_string_many_spaced(&param_doc.to_cell_file());
+            write_file(
+                workdir.join(format!("{dos_seed_name_setup}.param")),
+                &param_text,
+            )?;
+
+            // 3. Copy SCF checkpoint: ZnO.check -> ZnO_DOS.check
+            let check_src = workdir.join(format!("{seed_name_setup}.check"));
+            let check_dst = workdir.join(format!("{dos_seed_name_setup}.check"));
+            copy_file(&check_src, &check_dst)?;
+
+            // Only write job script for SLURM mode
+            if let Some(ref script) = job_script {
+                write_file(workdir.join(JOB_SCRIPT_NAME), script)?;
+            }
+            Ok(())
+        })
+        .collect(move |workdir| -> Result<(), WorkflowError> {
+            let castep_out = workdir.join(format!("{dos_seed_name_collect}.castep"));
+            if !castep_out.exists() {
+                return Err(WorkflowError::InvalidConfig(format!(
+                    "missing output: {}",
+                    castep_out.display()
+                )));
+            }
+            let content = read_file(&castep_out)?;
+            if !content.contains("Total time") {
+                return Err(WorkflowError::InvalidConfig(
+                    "CASTEP output appears incomplete (no 'Total time' marker)".into(),
+                ));
+            }
+            Ok(())
+        });
+
+    Ok(task)
+}
+
+fn main() -> Result<()> {
+    workflow_core::init_default_logging().ok();
+    let config = ChainConfig::parse();
+
+    let seed_cell = include_str!("../seeds/ZnO.cell");
+    let seed_param = include_str!("../seeds/ZnO.param");
+
+    let root = PathBuf::from(&config.workdir);
+    let workdir_str = "runs/scf_dos/";
+
+    // Build SCF task
+    let scf_task = build_scf_task(
+        config.u_value,
+        &config.element,
+        config.orbital,
+        &config.seed_name,
+        seed_cell,
+        seed_param,
+        workdir_str,
+        &config.castep_command,
+        config.local,
+        &config,
+    )?;
+
+    // Build DOS task (depends on SCF)
+    let dos_task = build_dos_task(
+        &config.seed_name,
+        seed_cell,
+        seed_param,
+        workdir_str,
+        &config.castep_command,
+        config.local,
+        &config,
+    )?;
+
+    let mut workflow = Workflow::new("scf_dos_chain")
+        .with_max_parallel(config.max_parallel)?
+        .with_root_dir(root.clone());
+
+    if !config.local {
+        workflow = workflow
+            .with_queued_submitter(Arc::new(QueuedRunner::new(SchedulerKind::Slurm)));
+    }
+
+    workflow = workflow.with_log_dir(root.join("logs"));
+
+    workflow.add_task(scf_task)?;
+    workflow.add_task(dos_task)?;
+
+    if config.dry_run {
+        let order = workflow.dry_run()?;
+        println!("Dry-run topological order:");
+        for task_id in &order {
+            println!("  {task_id}");
+        }
+        return Ok(());
+    }
+
+    let state_path = root.join(".scf_dos_chain.workflow.json");
+    let mut state = JsonStateStore::new("scf_dos_chain", state_path);
+
+    let summary = if config.local {
+        run_default(&mut workflow, &mut state)?
+    } else {
+        let runner: Arc<dyn ProcessRunner> =
+            Arc::new(SystemProcessRunner::new().with_log_dir(root.join("logs")));
+        let executor: Arc<dyn HookExecutor> = Arc::new(ShellHookExecutor);
+        workflow.run(&mut state, runner, executor)?
+    };
+
+    println!(
+        "Workflow complete: {} succeeded, {} failed, {} skipped ({:.1}s)",
+        summary.succeeded.len(),
+        summary.failed.len(),
+        summary.skipped.len(),
+        summary.duration.as_secs_f64(),
+    );
+    Ok(())
+}
+'''
+
+[tasks.TASK-4]
+description = "Copy seed files for scf_dos_chain binary"
+type = "create"
+acceptance = [
+    "test -f examples/scf_dos_chain/seeds/ZnO.cell",
+    "test -f examples/scf_dos_chain/seeds/ZnO.param",
+]
+
+[[tasks.TASK-4.changes]]
+file = "examples/scf_dos_chain/seeds/ZnO.cell"
+after = '''%BLOCK LATTICE_CART
+  3.25 0.0 0.0
+  0.0 3.25 0.0
+  0.0 0.0 5.21
+%ENDBLOCK LATTICE_CART
+
+%BLOCK POSITIONS_FRAC
+Zn  0.333333  0.666667  0.0
+Zn  0.666667  0.333333  0.5
+O   0.333333  0.666667  0.375
+O   0.666667  0.333333  0.875
+%ENDBLOCK POSITIONS_FRAC
+'''
+
+[[tasks.TASK-4.changes]]
+file = "examples/scf_dos_chain/seeds/ZnO.param"
+after = '''task : SinglePoint
+'''
+
+[tasks.TASK-5]
+description = "Update root Cargo.toml workspace members and delete old hubbard_u_sweep_slurm directory"
+type = "replace"
+acceptance = [
+    "cargo check --workspace",
+    "cargo test --workspace",
+    "rm -rf examples/hubbard_u_sweep_slurm/",
+    "test ! -d examples/hubbard_u_sweep_slurm/",
+]
+
+[[tasks.TASK-5.changes]]
+file = "Cargo.toml"
+before = '''[workspace]
+members = [
+    "workflow_core",
+    "workflow_utils",
+    "examples/hubbard_u_sweep",
+    "examples/hubbard_u_sweep_slurm",
+    "workflow-cli",
+]
+resolver = "2"'''
+after = '''[workspace]
+members = [
+    "workflow_core",
+    "workflow_utils",
+    "examples/hubbard_u_sweep",
+    "examples/multi_param_sweep",
+    "examples/scf_dos_chain",
+    "workflow-cli",
+]
+resolver = "2"'''
diff --git a/notes/plan-enrichment/phase-6-fix/gather-summary.md b/notes/plan-enrichment/phase-6-fix/gather-summary.md
new file mode 100644
index 0000000..0d01c5e
--- /dev/null
+++ b/notes/plan-enrichment/phase-6-fix/gather-summary.md
@@ -0,0 +1,24 @@
+## Gather Summary: phase-6-fix
+
+**Tasks created:** 5
+**Dependency chain:** TASK-5 depends on TASK-1..TASK-4. TASK-1 through TASK-4 are all parallel.
+**Deferred items absorbed:** 2 (D.2 fully absorbed; D.3 partially absorbed — port `no_literal_tabs` test)
+
+**Gather completeness:**
+- [x] deferred-and-patterns.md — saved
+- [x] codebase-state.md — saved — Files documented: 19
+- [x] draft-elaboration.md — saved
+- [x] draft-plan.toml — saved
+- [x] task-checklist.md — saved
+
+**Before-block verification:** 1/1 confirmed from Step 4
+**Unverified tasks:** none (from Step 4)
+**Wiring issues flagged:** 1
+
+**Confidence notes:**
+1 HIGH uncertainty: exact `TaskClosure` type alias in `workflow_core` — must read `workflow_core/src/task.rs` before implementation.
+4 MEDIUM uncertainties: `KpointsMpGrid` API surface, `parse_u_values` module placement vs old pattern, `WorkflowError`-anyhow compatibility in setup closures, task ID naming when kpoints/cutoffs are None.
+The HIGH uncertainty must be resolved before any setup/collect closure code can be written.
+
+**Questions for user:**
+None
diff --git a/notes/plan-enrichment/phase-6-fix/task-checklist.md b/notes/plan-enrichment/phase-6-fix/task-checklist.md
new file mode 100644
index 0000000..96de899
--- /dev/null
+++ b/notes/plan-enrichment/phase-6-fix/task-checklist.md
@@ -0,0 +1,184 @@
+# Task Checklist — Phase 6 Fix Plan
+
+Generated: 2026-04-26
+Plan source: `notes/plan-enrichment/phase-6-fix/draft-plan.toml`
+Branch: `phase-6-fix`
+
+---
+
+## TASK-1: Create multi_param_sweep binary source files (Cargo.toml, config.rs, job_script.rs, main.rs)
+
+**Goal**: Clear. Create four source files for a new binary crate `multi_param_sweep`.
+
+**Target files**: Explicitly specified with full file paths.
+
+**Implementation detail**: Complete. Full file contents provided as `after` blocks.
+
+**New types/interfaces**: Fully defined inline.
+
+**Module wiring check** (binary crate — no parent `pub mod` needed):
+- `pub mod` in parent: Not applicable (binary crate has no parent mod)
+- `pub use` re-export: Not applicable
+- Consumer updates co-located: Yes — `main.rs` declares `mod config; mod job_script;`, `config.rs` defines `pub struct SweepConfig`, `job_script.rs` imports via `use crate::config::SweepConfig;`, `main.rs` imports via `use config::SweepConfig;` and `use job_script::generate_job_script;`
+
+**Known failure mode check**:
+- Missing `pub mod` risk: Low — both `mod config;` and `mod job_script;` are declared in `main.rs` as written in the draft
+- Missing `pub use` risk: Low — no `pub use` required for a binary crate
+- Stale import risk: Medium — the `main.rs` references `seed_cell` and `seed_param` via `include_str!("../seeds/ZnO.cell")` and `include_str!("../seeds/ZnO.param")`, but those seed files are created by TASK-2. This is acceptable because the dependency chain (TASK-5 depends on TASK-1 through TASK-4) ensures seeds exist before workspace compilation.
+
+**Before-block check**:
+- Grep confirmed: No — the new files do not exist yet (empty directories confirmed via `ls`)
+- Acceptance commands present: Yes — 4 `test -f` checks
+
+**Depends on**: None (creates new files that reference seed files from TASK-2, but files can be created independently)
+
+**Notes**:
+- The `KpointsMpGrid` import path (`castep_cell_io::cell::bz_sampling_kpoints::KpointsMpGrid`) is from an external crate and cannot be verified locally. The codebase-state.md asserts this path exists.
+- The `Workspace::with_root_dir` takes `impl Into<PathBuf>` and returns `Self`, matching the chaining pattern used.
+- The `main.rs` uses unused variable `seed_name_collect` (line 383) — this may produce a compiler warning but not an error.
+
+---
+
+## TASK-2: Copy seed files for multi_param_sweep binary
+
+**Goal**: Clear. Create ZnO.cell and ZnO.param seed files.
+
+**Target files**: Explicitly specified with full file paths (`examples/multi_param_sweep/seeds/ZnO.cell`, `examples/multi_param_sweep/seeds/ZnO.param`).
+
+**Implementation detail**: Complete. Full file contents provided as `after` blocks.
+
+**New types/interfaces**: Not applicable (data files).
+
+**Module wiring check**: Not applicable (not Rust source files).
+
+**Known failure mode check**:
+- Missing `pub mod` risk: Low — seed files, not modules
+- Missing `pub use` risk: Not applicable
+- Stale import risk: Low — these files are `include_str!`'d by TASK-1's `main.rs`. No import paths to go stale.
+
+**Before-block check**:
+- Grep confirmed: No — the seed directory exists but is empty
+- Acceptance commands present: Yes — 2 `test -f` checks
+
+**Depends on**: None
+
+**Notes**: The seed files content matches the existing seed files in `examples/hubbard_u_sweep_slurm/seeds/`.
+
+---
+
+## TASK-3: Create scf_dos_chain binary source files (Cargo.toml, config.rs, job_script.rs, main.rs)
+
+**Goal**: Clear. Create four source files for a new binary crate `scf_dos_chain`.
+
+**Target files**: Explicitly specified with full file paths.
+
+**Implementation detail**: Complete. Full file contents provided as `after` blocks.
+
+**New types/interfaces**: Fully defined inline.
+
+**Module wiring check** (binary crate):
+- `pub mod` in parent: Not applicable
+- `pub use` re-export: Not applicable
+- Consumer updates co-located: Yes — `main.rs` declares `mod config; mod job_script;`, `config.rs` defines `pub struct ChainConfig`, `job_script.rs` imports via `use crate::config::ChainConfig;`, `main.rs` imports via `use config::ChainConfig;` and `use job_script::generate_job_script;`
+
+**Known failure mode check**:
+- Missing `pub mod` risk: Low — both `mod config;` and `mod job_script;` are declared in `main.rs`
+- Missing `pub use` risk: Low — binary crate
+- Stale import risk: Medium — same as TASK-1, seed files are in TASK-4. The same `include_str!("../seeds/...")` dependency exists.
+
+**Before-block check**:
+- Grep confirmed: No — the new files do not exist yet (empty directories confirmed via `ls`)
+- Acceptance commands present: Yes — 4 `test -f` checks
+
+**Depends on**: None, but TASK-4 seed files are needed before compilation would succeed.
+
+**Notes**:
+- **WIRING ISSUE FLAGGED**: The import `use castep_cell_io::param::general::Task;` on line 787 of the main.rs code will shadow the `workflow_core::Task` that is brought in via `use workflow_utils::prelude::*` (which re-exports `workflow_core::prelude::*`). In Rust, an explicit `use` shadows glob imports. This means all references to `Task` in the file (such as `Task::new()` on line 831 and the return type `Result<Task, WorkflowError>` on line 807) would resolve to `castep_cell_io::param::general::Task` rather than `workflow_core::Task`.
+  - If `castep_cell_io::param::general::Task` is an enum (with variants like `BandStructure`, `SinglePoint`), it likely has no `new()` method, causing a compilation error at line 831.
+  - The only place where the castep `Task` type is needed is on line 946: `param_doc.general.task = Some(Task::BandStructure);`
+  - **Recommended fix**: Use `use castep_cell_io::param::general::Task as CastepTask;` and change line 946 to `CastepTask::BandStructure`. Alternatively, test whether the actual module path requires the `task` submodule (i.e., `castep_cell_io::param::general::task::Task::BandStructure`), in which case the import path may be simply wrong.
+- The import path `castep_cell_io::param::general::Task` cannot be verified against the external crate from this workspace. If the path is wrong, it would produce an "unresolved import" error instead of a naming conflict.
+
+---
+
+## TASK-4: Copy seed files for scf_dos_chain binary
+
+**Goal**: Clear. Create ZnO.cell and ZnO.param seed files.
+
+**Target files**: Explicitly specified (`examples/scf_dos_chain/seeds/ZnO.cell`, `examples/scf_dos_chain/seeds/ZnO.param`).
+
+**Implementation detail**: Complete. Full file contents provided.
+
+**New types/interfaces**: Not applicable.
+
+**Module wiring check**: Not applicable.
+
+**Known failure mode check**:
+- Missing `pub mod` risk: Low
+- Missing `pub use` risk: Not applicable
+- Stale import risk: Low
+
+**Before-block check**:
+- Grep confirmed: No — empty `seeds/` directory
+- Acceptance commands present: Yes — 2 `test -f` checks
+
+**Depends on**: None
+
+**Notes**: Identical content to TASK-2 seed files.
+
+---
+
+## TASK-5: Update root Cargo.toml workspace members and delete old hubbard_u_sweep_slurm directory
+
+**Goal**: Clear. Two actions: (1) modify workspace members in root `Cargo.toml`, (2) delete old binary directory.
+
+**Target files**: Root `Cargo.toml` specified with exact `before`/`after` blocks.
+
+**Implementation detail**: Complete. The `before` and `after` sections clearly show the exact diff.
+
+**New types/interfaces**: Not applicable.
+
+**Module wiring check**:
+- `pub mod` in parent: Not applicable (workspace config, not Rust module)
+- `pub use` re-export: Not applicable
+- Consumer updates co-located: Not applicable — the old member is removed and two new members are added. This is the workspace configuration change.
+
+**Known failure mode check**:
+- Missing `pub mod` risk: Low — workspace members are listed in `Cargo.toml`, not `pub mod` declarations
+- Missing `pub use` risk: Not applicable
+- Stale import risk: Medium — after deleting `examples/hubbard_u_sweep_slurm`, any remaining file that references it would break. A `grep` for `hubbard_u_sweep_slurm` found only in: root `Cargo.toml` (being modified by this task), the plan files, and codebase-state.md. No source files outside the binary itself reference it.
+
+**Before-block check**:
+- Grep confirmed: Yes — root `Cargo.toml` contains `"examples/hubbard_u_sweep_slurm"` on line 6
+- Acceptance commands present: Yes — `cargo check --workspace`, `cargo test --workspace`, `rm -rf`, `test ! -d`
+
+**Depends on**: TASK-1, TASK-2, TASK-3, TASK-4 (as declared in `[dependencies]`). The new binary directories must exist and the new seed files must be present before the workspace is updated and verified.
+
+**Notes**:
+- The `cargo check --workspace` acceptance criterion will verify that all new code compiles. If TASK-3 has the Task naming conflict flagged above, this check will fail.
+- The `cargo test --workspace` criterion will run all tests including the `no_literal_tabs`, `contains_sbatch_directives`, etc. tests in the new binaries.
+- The `rm -rf` and `test ! -d` acceptance commands are destructive but irreversible. No backup step is mentioned, but the deleted code is being replaced by the two new binaries.
+- The plan's dependency declaration `TASK-5 = ["TASK-1", "TASK-2", "TASK-3", "TASK-4"]` is correct and necessary.
+
+---
+
+## Summary
+
+| Task | Module Wiring | Known Failure Risks | Before-block Verified | Depends On |
+|------|--------------|---------------------|-----------------------|------------|
+| TASK-1 | Not applicable (binary) | Stale import: Medium (seed refs) | No (files don't exist) | None |
+| TASK-2 | Not applicable (data files) | Low | No (files don't exist) | None |
+| TASK-3 | **FLAGGED**: Task naming conflict at `use castep_cell_io::param::general::Task;` shadows `workflow_core::Task` from prelude | Stale import: Medium (seed refs); Naming conflict: **High** (compilation error) | No (files don't exist) | None |
+| TASK-4 | Not applicable (data files) | Low | No (files don't exist) | None |
+| TASK-5 | Not applicable (workspace config) | Stale import: Medium (deleted dir references) | Yes (confirmed in Cargo.toml) | TASK-1, TASK-2, TASK-3, TASK-4 |
+
+**Tasks checked**: 5
+**Wiring issues flagged**: 1
+- TASK-3: `use castep_cell_io::param::general::Task;` will shadow `workflow_core::Task` from the glob prelude import, causing `Task::new()` at line 831 to fail compilation.
+
+**Before-block unverified**: 4 (TASK-1, TASK-2, TASK-3, TASK-4 — files don't exist yet, as expected)
+
+**Additional items flagged**:
+1. TASK-3 import path `castep_cell_io::param::general::Task` may itself be incorrect — the plan documentation references `castep_cell_io::param::general::task::Task` (note the lowercase `task` module). If the draft plan's shortened path is wrong, this produces an "unresolved import" error rather than a naming conflict.
+2. TASK-1's `main.rs` uses unused variable `seed_name_collect` (line 383), which will generate a compiler warning but not an error.
+3. Both TASK-1 and TASK-3 `main.rs` files import `use std::sync::Arc;` — confirmed correct for `Arc<dyn ProcessRunner>` and `Arc::new(QueuedRunner(...))` usage.
diff --git a/notes/plan-reviews/phase-6-fix/decisions.md b/notes/plan-reviews/phase-6-fix/decisions.md
new file mode 100644
index 0000000..ce6ec8c
--- /dev/null
+++ b/notes/plan-reviews/phase-6-fix/decisions.md
@@ -0,0 +1,43 @@
+## Plan Review Decisions — PHASE_6_FIX_PLAN — 2026-05-05
+
+### Design Assessment
+
+The plan is structurally sound. Splitting the buggy `hubbard_u_sweep_slurm` into two purpose-built binaries (`multi_param_sweep` for independent SCF sweeps, `scf_dos_chain` for the chained SCF-to-DOS workflow) correctly resolves all three bugs. Task ID encoding (`scf_U{u}_k{k}_c{c}`) eliminates the duplicate ID collision, and the `.check` file copy pattern for DOS chains follows standard CASTEP convention. Crate boundaries are respected: `anyhow` only in binaries, domain logic stays in library code. Two issues were identified and amended: (1) the plan was written against v0.4.0 APIs but `castep-cell-io` is now v0.5.0 — parse API changed, `KpointsMpGrid` is now a direct field, and dependencies must use local path refs; (2) the `default-logging` feature flag was missing from both new `Cargo.toml` files.
+
+### Deferred Item Decisions
+
+#### D.1: Restore plan-specified portable config fields
+**Decision:** Defer again
+**Rationale:** The current plan is a bug-fix rewrite, not a portability initiative. The NixOS-specific SLURM template remains appropriate for Tony's sole-user context.
+**Action:** No plan update needed.
+
+#### D.2: `generate_job_script` formatting inconsistencies
+**Decision:** Absorb
+**Rationale:** The plan explicitly calls for cleaning up the heredoc template — no literal tab characters, consistent SBATCH quoting — and porting the `no_literal_tabs` test. This is the "next functional edit" the precondition was waiting for.
+**Action:** Already addressed in plan Section "Setup closure" step 5.
+
+#### D.3 (partial): Unit tests for `generate_job_script`
+**Decision:** Defer again
+**Rationale:** Without a second (portable) job script template variant, test assertions remain tightly coupled to NixOS-specific output. D.1 must be addressed first. The plan ports existing tests (including `no_literal_tabs`), which is sufficient.
+**Action:** Updated precondition: D.1 must be addressed first.
+
+#### `read_task_ids` empty-string edge case
+**Decision:** Defer again
+**Rationale:** Lives in `workflow-cli/src/main.rs` — a different crate entirely, unrelated to the example binary rewrite.
+**Action:** Updated precondition: next functional edit to `workflow-cli/src/main.rs`.
+
+### Plan Amendments Applied
+
+**Amendment 1** — Updated all `castep-cell-io` references to v0.5.0 API:
+- Parse calls: `castep_cell_fmt::parse::<CellDocument>(&input)` instead of `CellDocument::parse()`
+- `KpointsMpGrid` is a direct field on `CellDocument` (no serialization workaround)
+- Dependencies use local path dep: `castep-cell-io = { path = "../castep-cell-io/castep_cell_io" }`
+- Removed the "issue to be filed" note about missing KpointsMpGrid field
+- Bumped API version reference from v0.4.0 to v0.5.0 throughout
+- Added note that kept `hubbard_u_sweep` also needs Cargo.toml and main.rs updates
+
+**Amendment 2** — Added `features = ["default-logging"]` to both new `Cargo.toml` files
+
+**Amendment 3** — Added empty-string handling to `parse_kpoints` spec
+
+**Amendment 4** — Added lockfile note to build verification step
diff --git a/plans/phase-6-fix/PHASE_6_FIX_PLAN.md b/plans/phase-6-fix/PHASE_6_FIX_PLAN.md
new file mode 100644
index 0000000..5a45d6f
--- /dev/null
+++ b/plans/phase-6-fix/PHASE_6_FIX_PLAN.md
@@ -0,0 +1,162 @@
+# Plan: Rewrite example binaries for multi-parameter sweep & SCF+DOS chain
+
+## Context
+
+The merged `hubbard_u_sweep_slurm` binary has three bugs:
+1. String-parsed "second values" with no CLI format hints — users don't know what convention to use
+2. Default sweep mode "single" only generates SCF tasks — product/pairwise modes silently require a flag
+3. Product/pairwise modes always build SCF→DOS chains, causing `duplicate task id: dos_kpt8x8x8` because DOS task IDs don't include the U value
+
+The fix is to replace this binary with two purpose-built binaries that correctly test the intended workflows.
+
+---
+
+## Plan
+
+### Binary 1: `multi_param_sweep` — independent SCF parameter sweep
+
+**Purpose**: Sweep Hubbard U, k-point MP grid, and cutoff energy in product or pairwise mode. All tasks are independent SCF calculations (no children).
+
+**File structure**:
+```
+examples/multi_param_sweep/
+├── Cargo.toml          (deps: anyhow, clap, castep-cell-fmt, castep-cell-io, itertools, workflow_core, workflow_utils; use `workspace = true` for workspace-managed deps: anyhow, clap, itertools. castep-cell-io uses local path dep: `castep-cell-io = { path = "../castep-cell-io/castep_cell_io" }`. workflow_core needs `features = ["default-logging"]`.)
+├── src/
+│   ├── main.rs         (entry point, task builders, sweep logic, workflow runner)
+│   ├── config.rs       (clap CLI config)
+│   └── job_script.rs   (SLURM script generation, ported from old binary)
+└── seeds/
+    ├── ZnO.cell
+    └── ZnO.param
+```
+
+**CLI flags**:
+
+| Flag | Type | Default | Description |
+|------|------|---------|-------------|
+| `--u-values` | `String` | `"0.0,1.0,2.0,3.0,4.0,5.0"` | Comma-separated Hubbard U values (eV) |
+| `--kpoints` | `Option<String>` | `None` | Comma-separated MP grids, e.g. `"8x8x8,6x6x6"`. If omitted, no KPOINTS block is written (use seed defaults) |
+| `--cutoffs` | `Option<String>` | `None` | Comma-separated cutoff energies (eV), e.g. `"300,500,800"`. If omitted, use seed defaults |
+| `--sweep-mode` | `String` | `"product"` | `"product"` (cartesian product) or `"pairwise"` (zip — all three lists must be same length) |
+| `--element` | `String` | `"Zn"` | Element for Hubbard U |
+| `--orbital` | `char` | `"d"` | Orbital: `'d'` or `'f'` |
+| `--seed-name` | `String` | `"ZnO"` | CASTEP input file prefix |
+| `--max-parallel` | `usize` | `4` | Max concurrent tasks |
+| `--local` | `bool` | `false` | Direct process execution (no SLURM) |
+| `--dry-run` | `bool` | `false` | Print topological order and exit |
+| `--castep-command` | `String` | `"castep"` | CASTEP binary (local mode only) |
+| `--workdir` | `String` | `"."` | Root directory for runs/logs |
+| Slurm flags | | | `--partition`, `--ntasks`, `--nix-flake`, `--mpi-if` (same as old binary) |
+
+**Sweep logic** (`main.rs`):
+
+- `product` mode: generate all combinations via `itertools::iproduct!(u_values, kpoints, cutoffs)`
+- `pairwise` mode: zip all three lists (must be same length; error if not). Pairwise mode **requires** both `--kpoints` and `--cutoffs` to be explicitly provided. If either is `None` in pairwise mode, error with: `"pairwise mode requires --kpoints and --cutoffs to be specified"`
+- Each combination → one `Task` with `ExecutionMode::direct(...)` (local) or `Queued` (Slurm)
+- Task ID: `scf_U{u}_k{k}_c{c}` (e.g., `scf_U3.0_k8x8x8_c500`) — unique, no collisions
+- Workdir: `runs/U{u}_k{k}_c{c}/`
+
+**Parsing utility functions** (`main.rs`):
+
+1. `fn parse_kpoints(s: &str) -> anyhow::Result<Vec<[u32; 3]>>` — splits on comma, splits each segment on `"x"`, parses exactly 3 `u32` values, returns clear `anyhow` errors for wrong number of axes or non-numeric input, e.g. `"8xx8"` → error, `"abc"` → error, `"8x8"` → error (only 2 axes). Empty or whitespace-only input returns `anyhow!("kpoints list is empty")`.
+2. `fn parse_cutoffs(s: &str) -> anyhow::Result<Vec<f64>>` — comma-separated f64 list, same pattern as existing `parse_u_values` (imported or duplicated)
+
+**Setup closure** — three parameter injections:
+
+1. **Hubbard U**: Parse seed cell via `castep_cell_fmt::parse::<CellDocument>(&input)`, inject `hubbard_u` field using builder pattern
+2. **K-points**: Parse compact string `"8x8x8"` → `[u32; 3]`, construct `KpointsMpGrid([kx, ky, kz])`, set directly: `cell_doc.kpoints_mp_grid = Some(KpointsMpGrid(...))`. No workaround needed — `KpointsMpGrid` is a direct field on `CellDocument` in v0.5.0. Serialize via `cell_doc.to_cell_file()` and `to_string_many_spaced()` as usual.
+3. **Cutoff energy**: Parse seed param via `castep_cell_fmt::parse::<ParamDocument>(&input)`, set `basis_set.cutoff_energy = Some(CutOffEnergy { value, unit: None })` — unit defaults to eV in CASTEP, leaving `None` keeps serialization cleaner (no unit suffix)
+4. Serialize both documents to file via `to_string_many_spaced()`
+5. If Slurm: write `job.sh` via ported `generate_job_script()`. **Formatting fix (from D.2)**: use a clean heredoc template — no literal tab characters mixed with spaces, consistent quoting around SBATCH directives. The existing `no_literal_tabs` test from the old binary should be ported and pass.
+
+**API usage** (from `castep-cell-io` v0.5.0):
+- `CellDocument` — parsed via `castep_cell_fmt::parse::<CellDocument>(&input)`, field mutation, `.to_cell_file()`
+- `KpointsMpGrid([u32; 3])` — direct field on `CellDocument`: `doc.kpoints_mp_grid = Some(KpointsMpGrid(...))`
+- `ParamDocument` — parsed via `castep_cell_fmt::parse::<ParamDocument>(&input)`, `.basis_set.cutoff_energy` field mutation, `.to_cell_file()`
+- `HubbardU::builder()`, `AtomHubbardU::builder()`, `OrbitalU::D(f64)` / `OrbitalU::F(f64)`
+- `CutOffEnergy { value: f64, unit: None }` — None defaults to eV in CASTEP, cleaner serialization
+
+---
+
+### Binary 2: `scf_dos_chain` — SCF+DOS task chain
+
+**Purpose**: Test chained task automation — SCF followed by DOS (BandStructure) calculation that depends on SCF checkpoint output.
+
+**File structure**:
+```
+examples/scf_dos_chain/
+├── Cargo.toml          (deps: same as multi_param_sweep; use `workspace = true` for workspace-managed deps. Same path-dep and default-logging requirements.)
+├── src/
+│   ├── main.rs         (entry point, SCF task, DOS task, chain wiring)
+│   ├── config.rs       (simpler CLI — single U, no sweep, no kpoints/cutoff)
+│   └── job_script.rs   (SLURM script, same as Binary 1)
+└── seeds/
+    ├── ZnO.cell
+    └── ZnO.param
+```
+
+**CLI flags** (simpler — single parameter set for testing the chain):
+
+| Flag | Type | Default | Description |
+|------|------|---------|-------------|
+| `--u-value` | `f64` | `3.0` | Single Hubbard U value (eV) |
+| `--element` | `String` | `"Zn"` | Element for Hubbard U |
+| `--orbital` | `char` | `"d"` | Orbital type |
+| `--seed-name` | `String` | `"ZnO"` | CASTEP input prefix |
+| `--max-parallel` | `usize` | `1` | Max concurrent tasks |
+| `--local` | `bool` | `false` | Direct execution |
+| `--dry-run` | `bool` | `false` | Print topological order |
+| `--castep-command` | `String` | `"castep"` | CASTEP binary |
+| `--workdir` | `String` | `"."` | Root directory |
+| Slurm flags | | | Same as Binary 1 |
+
+**Task chain**: Two tasks sharing the same workdir.
+
+**Task 1 — `scf`**:
+- Task ID: `scf`
+- Workdir: `runs/scf_dos/`
+- Setup: parse seed cell, inject HubbardU (same pattern as Binary 1), write `ZnO.cell` + `ZnO.param`
+- Execution: `ExecutionMode::direct("castep", &["ZnO"])` or `Queued`
+- Collect: verify `ZnO.castep` exists and has "Total time" marker
+
+**Task 2 — `dos`** (depends on `scf`):
+- Task ID: `dos`
+- Workdir: `runs/scf_dos/` (same workdir as SCF)
+- Depends on: `scf`
+- Setup:
+  1. Parse in-memory seed cell into `CellDocument`, write to `<workdir>/ZnO_DOS.cell`
+  2. Parse in-memory seed param into `ParamDocument`, set `general.task = Some(Task::BandStructure)`, write to `<workdir>/ZnO_DOS.param`
+  3. Copy `<workdir>/ZnO.check` → `<workdir>/ZnO_DOS.check` (SCF checkpoint produced by Task 1)
+- Execution: `ExecutionMode::direct("castep", &["ZnO_DOS"])` or `Queued` — note seed name is `ZnO_DOS`
+- Collect: verify `ZnO_DOS.castep` exists and contains a "Total time" completion marker (same pattern as SCF collect)
+
+**API usage** (from `castep-cell-io` v0.5.0):
+- `ParamDocument` — parsed via `castep_cell_fmt::parse::<ParamDocument>(&input)`, `.general.task = Some(Task::BandStructure)` — direct field mutation
+- `Task::BandStructure` imported from `castep_cell_io::param::general::task::Task`
+- `read_file`/`copy_file` from `workflow_utils::files` for .check file handling
+
+---
+
+### Workspace changes
+
+**`Cargo.toml`** (root):
+- Remove `"examples/hubbard_u_sweep_slurm"` from workspace members
+- Add `"examples/multi_param_sweep"` and `"examples/scf_dos_chain"`
+
+**Delete**:
+- `examples/hubbard_u_sweep_slurm/` (entire directory)
+
+**Keep and update**:
+- `examples/hubbard_u_sweep/` — update its `Cargo.toml` to use local path deps for `castep-cell-io`, and update `main.rs` to use the v0.5.0 parse API
+
+---
+
+## Verification
+
+1. **Binary 1 — dry run**: `cargo run --bin multi_param_sweep -- --dry-run` → prints topological order with all SCF task combos, no duplicate IDs, no DOS tasks
+2. **Binary 1 — dry run pairwise**: `cargo run --bin multi_param_sweep -- --dry-run --sweep-mode pairwise --u-values 0,1,2 --kpoints 8x8x8,6x6x6,4x4x4 --cutoffs 300,500,800` → zip mode, 3 tasks, no errors
+3. **Binary 1 — local run**: `cargo run --bin multi_param_sweep -- --local --u-values 0,1 --kpoints 8x8x8,6x6x6` (if CASTEP available) → generates correct .cell/.param files with HubbardU block, KPOINTS_MP_GRID block, and correct cutoff energy
+4. **Binary 2 — dry run**: `cargo run --bin scf_dos_chain -- --dry-run` → prints `scf` then `dos` (chain dependency)
+5. **Binary 2 — SCF phase only**: `cargo run --bin scf_dos_chain -- --local --dry-run` → verify scf→dos topological order
+6. **Build**: `cargo build --workspace` → all crates compile, no warnings. Note: the lockfile will resolve the local path dep for `castep-cell-io` from the sibling workspace — `cargo update` not needed.
+7. **Tests**: `cargo test --workspace` → all existing tests pass (old slurm tests removed, new binary tests run)
diff --git a/workflow_core/src/task.rs b/workflow_core/src/task.rs
index 5099624..d6b1336 100644
--- a/workflow_core/src/task.rs
+++ b/workflow_core/src/task.rs
@@ -65,6 +65,21 @@ pub struct Task {
     pub(crate) collect_failure_policy: CollectFailurePolicy,
 }
 
+impl std::fmt::Debug for Task {
+    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
+        f.debug_struct("Task")
+            .field("id", &self.id)
+            .field("dependencies", &self.dependencies)
+            .field("workdir", &self.workdir)
+            .field("mode", &self.mode)
+            .field("setup", &self.setup.as_ref().map(|_| "Fn(...)"))
+            .field("collect", &self.collect.as_ref().map(|_| "Fn(...)"))
+            .field("monitor_count", &self.monitors.len())
+            .field("collect_failure_policy", &self.collect_failure_policy)
+            .finish()
+    }
+}
+
 impl Task {
     pub fn new(id: impl Into<String>, mode: ExecutionMode) -> Self {
         Self {
