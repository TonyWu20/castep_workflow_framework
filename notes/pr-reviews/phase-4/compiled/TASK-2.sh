#!/usr/bin/env bash
set -euo pipefail
# TASK-2: Change task_successors field from HashMap to Option<HashMap> in JsonStateStore; None = pre-graph state file, Some(empty) = graph with no edges; update getter, setter, constructor, and cmd_retry fallback
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-4/fix-plan.toml
# Type: replace
# File: workflow_core/src/state.rs
python3 "$(dirname "$0")/TASK-2.py"
