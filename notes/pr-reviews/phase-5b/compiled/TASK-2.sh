#!/usr/bin/env bash
set -euo pipefail
# TASK-2: Add 2 missing parse_u_values test cases specified in plan D.3a: empty string and negative values
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-5b/fix-plan.toml
# Type: replace
# File: examples/hubbard_u_sweep_slurm/src/config.rs
python3 "$(dirname "$0")/TASK-2.py"
