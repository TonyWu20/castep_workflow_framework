#!/usr/bin/env bash
set -euo pipefail
# TASK-2: Remove the unused `use std::collections::HashMap` from the task.rs test module — it was left over from before tests were updated to use ExecutionMode::direct().
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-5b/fix-plan.toml
# Type: replace
# File: workflow_core/src/task.rs
python3 "$(dirname "$0")/TASK-2.py"
