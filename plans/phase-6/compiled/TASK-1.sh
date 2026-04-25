#!/usr/bin/env bash
set -euo pipefail
# TASK-1: Add CollectFailurePolicy enum, field on Task/InFlightTask, and re-exports
# Source: /Users/tony/programming/castep_workflow_framework/plans/phase-6/phase6_implementation.toml
# Type: replace
# File: /Users/tony/programming/castep_workflow_framework/workflow_core/src/task.rs
python3 "$(dirname "$0")/TASK-1.py"
