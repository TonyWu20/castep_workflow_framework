#!/usr/bin/env bash
set -euo pipefail
# TASK-2: Change `second` parameter of `build_one_task` and `build_chain` to `Option<&str>`; update all call sites; restore single-mode task IDs to original format
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-6/fix-plan.toml
# Type: replace
# File: examples/hubbard_u_sweep_slurm/src/main.rs
python3 "$(dirname "$0")/TASK-2.py"
