#!/usr/bin/env bash
set -euo pipefail
# TASK-3: Introduce pub struct TaskSuccessors(HashMap<String, Vec<String>>) with get() and is_empty(); use it in JsonStateStore field/getter/setter, Workflow computed_successors/successor_map(), and downstream_tasks in CLI
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-4/fix-plan.toml
# Type: replace
# File: workflow_core/src/state.rs
python3 "$(dirname "$0")/TASK-3.py"
