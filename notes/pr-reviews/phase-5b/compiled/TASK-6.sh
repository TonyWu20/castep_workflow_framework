#!/usr/bin/env bash
set -euo pipefail
# TASK-6: Branch on config.local at the run site: local mode uses run_default(&mut workflow, &mut state), SLURM mode keeps manual Arc wiring.
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-5b/fix-plan-compilable.toml
# Type: replace
# File: examples/hubbard_u_sweep_slurm/src/main.rs
python3 "$(dirname "$0")/TASK-6.py"
