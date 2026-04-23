#!/usr/bin/env bash
set -euo pipefail
# TASK-1: Use JOB_SCRIPT_NAME constant instead of hardcoded 'job.sh' in hubbard_u_sweep_slurm consumer
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-5/fix-plan.toml
# Type: replace
# File: examples/hubbard_u_sweep_slurm/src/main.rs
python3 "$(dirname "$0")/TASK-1.py"
