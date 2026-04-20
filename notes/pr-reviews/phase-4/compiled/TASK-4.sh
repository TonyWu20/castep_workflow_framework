#!/usr/bin/env bash
set -euo pipefail
# TASK-4: Default log_dir to task workdir instead of "."
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-4/fix-plan.md
# Type: replace
# File: workflow_core/src/workflow.rs
python3 "$(dirname "$0")/TASK-4.py"
