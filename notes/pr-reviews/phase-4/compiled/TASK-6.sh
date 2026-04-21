#!/usr/bin/env bash
set -euo pipefail
# TASK-6: Add #[must_use] annotation to JsonStateStore::task_successors() getter
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-4/fix-plan.toml
# Type: replace
# File: workflow_core/src/state.rs
python3 "$(dirname "$0")/TASK-6.py"
