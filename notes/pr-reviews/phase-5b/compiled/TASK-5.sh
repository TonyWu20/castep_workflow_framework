#!/usr/bin/env bash
set -euo pipefail
# TASK-5: Fix remaining uninlined_format_args clippy warnings in touched files: hubbard_u_sweep/main.rs lines 19-20 (format!("scf_U{:.1}", u)), task.rs line 138 (format!("{:?}", mode)), and job_script.rs test line 85 (format!("...{}", config.mpi_if)). Also fix 3.14 approx_constant warning in config.rs tests.
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-5b/fix-plan.toml
# Type: replace
# File: examples/hubbard_u_sweep/src/main.rs
python3 "$(dirname "$0")/TASK-5.py"
