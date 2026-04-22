#!/usr/bin/env bash
set -euo pipefail
# TASK-7: Verify full workspace builds and passes clippy
# Source: /Users/tony/programming/castep_workflow_framework/plans/phase-5/phase5a_implementation.toml
# Type: create
# File: examples/hubbard_u_sweep_slurm/.validation-complete
python3 "$(dirname "$0")/TASK-7.py"
