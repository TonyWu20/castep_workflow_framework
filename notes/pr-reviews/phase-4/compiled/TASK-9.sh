#!/usr/bin/env bash
set -euo pipefail
# TASK-9: Add integration test verifying Workflow::run completes a Queued-mode task using a stub QueuedSubmitter
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-4/fix-plan.toml
# Type: create
# File: workflow_core/tests/queued_workflow.rs
python3 "$(dirname "$0")/TASK-9.py"
