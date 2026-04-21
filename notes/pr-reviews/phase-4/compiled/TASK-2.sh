#!/usr/bin/env bash
set -euo pipefail
# TASK-2: Replace shell-injected poll_cmd/cancel_cmd String fields with direct Command construction using SchedulerKind
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-4/fix-plan.toml
# Type: replace
# File: workflow_utils/src/queued.rs
python3 "$(dirname "$0")/TASK-2.py"
