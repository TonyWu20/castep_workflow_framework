#!/usr/bin/env bash
set -euo pipefail
# TASK-6: Update ARCHITECTURE.md/ARCHITECTURE_STATUS.md, fix config assertion, trailing newline, and clippy
# Source: /Users/tony/programming/castep_workflow_framework/plans/phase-6/phase6_implementation.toml
# Type: replace
# File: /Users/tony/programming/castep_workflow_framework/examples/hubbard_u_sweep_slurm/src/config.rs
python3 "$(dirname "$0")/TASK-6.py"
