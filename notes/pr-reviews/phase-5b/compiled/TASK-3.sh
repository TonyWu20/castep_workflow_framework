#!/usr/bin/env bash
set -euo pipefail
# TASK-3: Inline format args on lines 102 and 108 of config.rs: change '"... {}", err' to '"... {err}"' in assert! messages.
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-5b/fix-plan-compilable.toml
# Type: replace
# File: examples/hubbard_u_sweep_slurm/src/config.rs
python3 "$(dirname "$0")/TASK-3.py"
