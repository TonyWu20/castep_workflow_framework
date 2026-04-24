#!/usr/bin/env bash
set -euo pipefail
# TASK-7: Restructure hubbard_u_sweep_slurm/main.rs: extract build_one_task + build_sweep_tasks functions, add --local flag + castep_command to SweepConfig, use iterator chain, fix anyhow conversion
# Source: /Users/tony/programming/castep_workflow_framework/plans/phase-5/PHASE5B_IMPL.toml
# Type: replace
# File: examples/hubbard_u_sweep_slurm/src/config.rs
python3 "$(dirname "$0")/TASK-7.py"
