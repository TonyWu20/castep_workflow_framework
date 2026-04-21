#!/usr/bin/env bash
set -euo pipefail
# TASK-1: Remove set_task_graph from StateStore trait, make it inherent on JsonStateStore, store computed successor map on Workflow, expose via successor_map()
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-4/fix-plan.toml
# Type: replace
# File: workflow_core/src/state.rs
python3 "$(dirname "$0")/TASK-1.py"
