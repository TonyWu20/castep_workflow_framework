#!/usr/bin/env bash
set -euo pipefail
# TASK-9: Reduce periodic hook test sleep from 8s to 2s
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-4/fix-plan.md
# Type: replace
# File: workflow_core/tests/hook_recording.rs
python3 "$(dirname "$0")/TASK-9.py"
