#!/usr/bin/env bash
set -euo pipefail
# TASK-2: Use JOB_SCRIPT_NAME constant instead of hardcoded 'job.sh' in queued integration tests
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-5/fix-plan.toml
# Type: replace
# File: workflow_utils/tests/queued_integration.rs
python3 "$(dirname "$0")/TASK-2.py"
