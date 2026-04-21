#!/usr/bin/env bash
set -euo pipefail
# TASK-1: Remove pub fn inner() from TaskSuccessors; it is dead code and exposes the raw HashMap backing type, defeating the newtype abstraction
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-4/fix-plan.toml
# Type: replace
# File: workflow_core/src/state.rs
python3 "$(dirname "$0")/TASK-1.py"
