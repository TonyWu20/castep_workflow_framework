#!/usr/bin/env bash
set -euo pipefail
# TASK-2: Add TaskSuccessors to the pub use line in workflow_core/src/lib.rs so it is accessible at the crate root like its sibling types
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-4/fix-plan.toml
# Type: replace
# File: workflow_core/src/lib.rs
python3 "$(dirname "$0")/TASK-2.py"
