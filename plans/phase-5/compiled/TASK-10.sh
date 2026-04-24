#!/usr/bin/env bash
set -euo pipefail
# TASK-10: Fix uninlined_format_args pedantic clippy warnings in touched files only. Run clippy per crate, inline format arguments (e.g. format!("{}", x) -> format!("{x}")), skip needless_pass_by_value on Workflow::run(). Scope: config.rs, job_script.rs, main.rs in hubbard_u_sweep_slurm; main.rs in hubbard_u_sweep; state.rs, task.rs in workflow_core.
# Source: /Users/tony/programming/castep_workflow_framework/plans/phase-5/PHASE5B_IMPL.toml
# Type: replace
# File: examples/hubbard_u_sweep_slurm/src/main.rs
python3 "$(dirname "$0")/TASK-10.py"
