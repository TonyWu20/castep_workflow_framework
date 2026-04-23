#!/usr/bin/env bash
set -euo pipefail
# TASK-3: Declare `pub mod prelude` in workflow_core/src/lib.rs — the prelude.rs file exists but is unreachable because lib.rs has no module declaration for it.
# Source: /Users/tony/programming/castep_workflow_framework/notes/pr-reviews/phase-5b/fix-plan.toml
# Type: replace
# File: workflow_core/src/lib.rs
python3 "$(dirname "$0")/TASK-3.py"
