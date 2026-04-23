#!/usr/bin/env bash
set -euo pipefail
# TASK-5: Replace individual workflow_core/workflow_utils imports with use workflow_utils::prelude::*; in both example binaries.
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-5b/fix-plan-compilable.toml
# Type: replace
# File: examples/hubbard_u_sweep/src/main.rs
python3 "$(dirname "$0")/TASK-5.py"
