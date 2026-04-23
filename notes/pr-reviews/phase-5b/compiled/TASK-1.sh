#!/usr/bin/env bash
set -euo pipefail
# TASK-1: Change 2.71828 to 42.0 in parse_single_value test to avoid clippy treating it as std::f64::consts::E (approx_constant lint).
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-5b/fix-plan-compilable.toml
# Type: replace
# File: examples/hubbard_u_sweep_slurm/src/config.rs
python3 "$(dirname "$0")/TASK-1.py"
