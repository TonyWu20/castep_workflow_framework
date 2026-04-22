#!/usr/bin/env bash
set -euo pipefail
# TASK-1: Extract hardcoded 'job.sh' literal into a pub const JOB_SCRIPT_NAME in queued module
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-5/fix-plan.toml
# Type: replace
# File: workflow_utils/src/queued.rs
python3 "$(dirname "$0")/TASK-1.py"
