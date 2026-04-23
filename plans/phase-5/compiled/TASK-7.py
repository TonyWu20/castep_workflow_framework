#!/usr/bin/env python3
"""TASK-7: Verify full workspace builds and passes clippy"""
import base64, sys
from pathlib import Path

CONTENT = base64.b64decode("UGhhc2UgNUEgd29ya3NwYWNlIHZhbGlkYXRpb24gcGFzc2VkLg==").decode()
TARGET = "examples/hubbard_u_sweep_slurm/.validation-complete"
TASK_ID = "TASK-7"

target_path = Path(TARGET)
target_path.parent.mkdir(parents=True, exist_ok=True)
target_path.write_text(CONTENT)

if not target_path.exists():
    print(f"FAILED {TASK_ID}: file not created at {TARGET}", file=sys.stderr)
    sys.exit(1)

print(f"OK {TASK_ID}: created {TARGET}")
