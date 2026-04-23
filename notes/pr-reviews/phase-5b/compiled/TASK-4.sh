#!/usr/bin/env bash
set -euo pipefail
# TASK-4: Wrap 'workflow_core' in backticks in the doc comment (line 1) and ensure file ends with trailing newline.
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-5b/fix-plan-compilable.toml
# Type: replace
# File: workflow_core/src/prelude.rs
python3 "$(dirname "$0")/TASK-4.py"
