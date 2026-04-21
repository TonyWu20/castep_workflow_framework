#!/usr/bin/env bash
set -euo pipefail
# TASK-4: Add doc comment to QueuedProcessHandle::wait() explaining the three exit-code states; fix the stale 'accounting query in wait() may refine' comment in is_running()
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-4/fix-plan.toml
# Type: replace
# File: workflow_utils/src/queued.rs
python3 "$(dirname "$0")/TASK-4.py"
