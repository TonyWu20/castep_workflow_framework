#!/usr/bin/env bash
set -euo pipefail
# TASK-1: Change TaskSuccessors::downstream_of to accept &[S] where S: AsRef<str> instead of &[String]
# Source: /Users/tony/programming/castep_workflow_framework/plans/phase-5/PHASE5B_IMPL.toml
# Type: replace
# File: workflow_core/src/state.rs
python3 "$(dirname "$0")/TASK-1.py"
