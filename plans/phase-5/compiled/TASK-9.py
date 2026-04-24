#!/usr/bin/env python3
"""TASK-9: Create prelude modules for workflow_core and workflow_utils, then update both examples to use them"""
import base64, sys
from pathlib import Path

CONTENT = base64.b64decode("Ly8hIENvbnZlbmllbmNlIHJlLWV4cG9ydHMgZm9yIGNvbW1vbiB3b3JrZmxvd19jb3JlIHR5cGVzLgovLyEKLy8hIGBgYAovLyEgdXNlIHdvcmtmbG93X2NvcmU6OnByZWx1ZGU6Oio7Ci8vISBgYGAKCnB1YiB1c2UgY3JhdGU6OmVycm9yOjpXb3JrZmxvd0Vycm9yOwpwdWIgdXNlIGNyYXRlOjpzdGF0ZTo6e0pzb25TdGF0ZVN0b3JlLCBTdGF0ZVN0b3JlLCBTdGF0ZVN0b3JlRXh0LCBUYXNrU3RhdHVzfTsKcHViIHVzZSBjcmF0ZTo6dGFzazo6e0V4ZWN1dGlvbk1vZGUsIFRhc2t9OwpwdWIgdXNlIGNyYXRlOjp3b3JrZmxvdzo6e1dvcmtmbG93LCBXb3JrZmxvd1N1bW1hcnl9OwpwdWIgdXNlIGNyYXRlOjp7SG9va0V4ZWN1dG9yLCBQcm9jZXNzUnVubmVyfTs=").decode()
TARGET = "workflow_core/src/prelude.rs"
TASK_ID = "TASK-9"

target_path = Path(TARGET)
target_path.parent.mkdir(parents=True, exist_ok=True)
target_path.write_text(CONTENT)

if not target_path.exists():
    print(f"FAILED {TASK_ID}: file not created at {TARGET}", file=sys.stderr)
    sys.exit(1)

print(f"OK {TASK_ID}: created {TARGET}")
