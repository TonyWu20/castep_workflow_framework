#!/usr/bin/env bash
set -euo pipefail
# TASK-1: Remove unused ProcessHandle import in queued integration test
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-4/fix-plan.md
# Type: replace
# File: workflow_utils/tests/queued_integration.rs
python3 "$(dirname "$0")/TASK-1.py"
