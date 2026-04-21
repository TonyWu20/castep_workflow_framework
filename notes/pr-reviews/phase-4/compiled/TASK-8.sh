#!/usr/bin/env bash
set -euo pipefail
# TASK-8: Set deprecated TASK_STATE env var alongside TASK_PHASE for backwards compatibility with existing hook scripts
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-4/fix-plan.toml
# Type: replace
# File: workflow_utils/src/monitoring.rs
python3 "$(dirname "$0")/TASK-8.py"
