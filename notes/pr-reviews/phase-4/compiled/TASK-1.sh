#!/usr/bin/env bash
set -euo pipefail
# TASK-1: Add task_successors field to JsonStateStore, set_task_graph to StateStore trait, persist graph in Workflow::run, rewrite cmd_retry for graph-aware downstream-only reset
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-4/fix-plan.toml
# Type: replace
# File: workflow_core/src/state.rs
python3 "$(dirname "$0")/TASK-1.py"
