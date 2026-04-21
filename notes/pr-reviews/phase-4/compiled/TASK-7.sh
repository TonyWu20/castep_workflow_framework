#!/usr/bin/env bash
set -euo pipefail
# TASK-7: Add queued_task_polls_before_completing test using DelayedHandle (AtomicUsize counter) that returns is_running()=true for first 2 polls then false, exercising the polling loop
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-4/fix-plan.toml
# Type: replace
# File: workflow_core/tests/queued_workflow.rs
python3 "$(dirname "$0")/TASK-7.py"
