#!/usr/bin/env bash
set -euo pipefail
# TASK-6: Replace `pub use queued::*` with explicit re-exports
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-4/fix-plan.md
# Type: replace
# File: workflow_utils/src/lib.rs
python3 "$(dirname "$0")/TASK-6.py"
