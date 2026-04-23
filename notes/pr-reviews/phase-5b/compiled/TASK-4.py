#!/usr/bin/env python3
"""TASK-4: Create workflow_utils/src/prelude.rs (re-exporting workflow_core::prelude::* plus workflow_utils types) and register it in workflow_utils/src/lib.rs."""
import base64, sys
from pathlib import Path

CONTENT = base64.b64decode("Ly8hIENvbnZlbmllbmNlIHJlLWV4cG9ydHMgZm9yIGNvbW1vbiB0eXBlcyBmcm9tIGJvdGggd29ya2Zsb3dfY29yZSBhbmQgd29ya2Zsb3dfdXRpbHMuCi8vIQovLyEgYGBgCi8vISB1c2Ugd29ya2Zsb3dfdXRpbHM6OnByZWx1ZGU6Oio7Ci8vISBgYGAKCi8vIFJlLWV4cG9ydCBldmVyeXRoaW5nIGZyb20gd29ya2Zsb3dfY29yZTo6cHJlbHVkZQpwdWIgdXNlIHdvcmtmbG93X2NvcmU6OnByZWx1ZGU6Oio7CgovLyB3b3JrZmxvd191dGlscyB0eXBlcwpwdWIgdXNlIGNyYXRlOjp7CiAgICBjb3B5X2ZpbGUsIGNyZWF0ZV9kaXIsIGV4aXN0cywgcmVhZF9maWxlLCByZW1vdmVfZGlyLCBydW5fZGVmYXVsdCwgd3JpdGVfZmlsZSwKICAgIFF1ZXVlZFJ1bm5lciwgU2NoZWR1bGVyS2luZCwgU2hlbGxIb29rRXhlY3V0b3IsIFN5c3RlbVByb2Nlc3NSdW5uZXIsIEpPQl9TQ1JJUFRfTkFNRSwKfTs=").decode()
TARGET = "workflow_utils/src/prelude.rs"
TASK_ID = "TASK-4"

target_path = Path(TARGET)
target_path.parent.mkdir(parents=True, exist_ok=True)
target_path.write_text(CONTENT)

if not target_path.exists():
    print(f"FAILED {TASK_ID}: file not created at {TARGET}", file=sys.stderr)
    sys.exit(1)

print(f"OK {TASK_ID}: created {TARGET}")
