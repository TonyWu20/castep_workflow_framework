#!/usr/bin/env bash
set -euo pipefail
# TASK-11: Update ARCHITECTURE.md and ARCHITECTURE_STATUS.md to reflect all Phase 5B changes (prelude, run_default, ExecutionMode::direct, downstream_of generic, --local flag)
# Source: /Users/tony/programming/castep_workflow_framework/plans/phase-5/PHASE5B_IMPL.toml
# Type: replace
# File: ARCHITECTURE_STATUS.md
python3 "$(dirname "$0")/TASK-11.py"
