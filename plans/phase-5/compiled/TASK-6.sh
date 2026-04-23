#!/usr/bin/env bash
set -euo pipefail
# TASK-6: Add unit tests for generate_job_script verifying SBATCH directives, seed name substitution, and absence of literal tabs
# Source: /Users/tony/programming/castep_workflow_framework/plans/phase-5/PHASE5B_IMPL.toml
# Type: replace
# File: examples/hubbard_u_sweep_slurm/src/job_script.rs
python3 "$(dirname "$0")/TASK-6.py"
