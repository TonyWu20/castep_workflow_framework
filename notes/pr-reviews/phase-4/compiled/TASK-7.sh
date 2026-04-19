#!/usr/bin/env bash
set -euo pipefail
# TASK-7: Remove dead `workdir` field from `QueuedProcessHandle`
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-4/fix-plan.md
# Type: replace
# File: workflow_utils/src/queued.rs
python3 "$(dirname "$0")/TASK-7.py"
