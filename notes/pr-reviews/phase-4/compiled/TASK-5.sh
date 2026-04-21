#!/usr/bin/env bash
set -euo pipefail
# TASK-5: Add two tests in workflow_core/src/state.rs: set_task_graph round-trip through save/load, and backwards-compatible deserialization of old state files without task_successors
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-4/fix-plan.toml
# Type: replace
# File: workflow_core/src/state.rs
python3 "$(dirname "$0")/TASK-5.py"
