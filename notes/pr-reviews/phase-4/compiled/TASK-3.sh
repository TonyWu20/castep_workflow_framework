#!/usr/bin/env bash
set -euo pipefail
# TASK-3: Move BFS traversal downstream_tasks from workflow-cli/src/main.rs into workflow_core/src/state.rs as pub fn TaskSuccessors::downstream_of; update CLI to call through the library; move unit tests with it
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-4/fix-plan.toml
# Type: replace
# File: workflow_core/src/state.rs
python3 "$(dirname "$0")/TASK-3.py"
