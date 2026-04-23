#!/usr/bin/env bash
set -euo pipefail
# TASK-1: Fix 6 test call sites in state.rs that use .into() with downstream_of — now ambiguous because S: AsRef<str> conflicts with tracing_core::Field. Replace &["a".into()] with &["a"] (string slice literals, which was the goal of the ergonomic improvement).
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-5b/fix-plan.toml
# Type: replace
# File: workflow_core/src/state.rs
python3 "$(dirname "$0")/TASK-1.py"
