#!/usr/bin/env python3
"""TASK-3: Declare `pub mod prelude` in workflow_core/src/lib.rs — the prelude.rs file exists but is unreachable because lib.rs has no module declaration for it."""
import base64, json, subprocess, sys

TASK_ID = "TASK-3"
STEPS = json.loads('[{"before_b64": "cHViIG1vZCBkYWc7CnB1YiBtb2QgZXJyb3I7Cm1vZCBtb25pdG9yaW5nOwpwdWIgbW9kIHByb2Nlc3M7CnB1YiBtb2Qgc3RhdGU7CnB1YiBtb2QgdGFzazsKcHViIG1vZCB3b3JrZmxvdzs=", "after_b64": "cHViIG1vZCBkYWc7CnB1YiBtb2QgZXJyb3I7Cm1vZCBtb25pdG9yaW5nOwpwdWIgbW9kIHByZWx1ZGU7CnB1YiBtb2QgcHJvY2VzczsKcHViIG1vZCBzdGF0ZTsKcHViIG1vZCB0YXNrOwpwdWIgbW9kIHdvcmtmbG93Ow==", "target": "workflow_core/src/lib.rs", "index": 0}]')

for step in STEPS:
    before = base64.b64decode(step["before_b64"]).decode()
    after = base64.b64decode(step["after_b64"]).decode()
    target = step["target"]
    idx = step["index"]

    content = open(target).read()
    if before not in content:
        print(f"FAILED {TASK_ID} change {idx}: pattern not found in {target}", file=sys.stderr)
        print(f"Expected (first 200 chars): {repr(before[:200])}", file=sys.stderr)
        sys.exit(1)

    result = subprocess.run(
        ["sd", "-F", "-A", "-n", "1", before, after, target],
        capture_output=True, text=True,
    )
    if result.returncode != 0:
        print(f"FAILED {TASK_ID} change {idx}: sd error: {result.stderr}", file=sys.stderr)
        sys.exit(result.returncode)

    new_content = open(target).read()
    if after and after not in new_content:
        print(f"FAILED {TASK_ID} change {idx}: replacement not found after apply", file=sys.stderr)
        sys.exit(1)

    print(f"OK {TASK_ID} change {idx}: applied to {target}")

print(f"OK {TASK_ID}: all changes applied")
