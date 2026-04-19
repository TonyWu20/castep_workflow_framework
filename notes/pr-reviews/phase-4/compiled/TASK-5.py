#!/usr/bin/env python3
"""TASK-5: Update `Queued` match arm in `workflow.rs` for unit variant"""
import base64, json, subprocess, sys

TASK_ID = "TASK-5"
STEPS = json.loads('[{"before_b64": "ICAgICAgICAgICAgICAgICAgICAgICAgICAgIEV4ZWN1dGlvbk1vZGU6OlF1ZXVlZCB7IHN1Ym1pdF9jbWQsIHBvbGxfY21kLCBjYW5jZWxfY21kIH0gPT4gew==", "after_b64": "ICAgICAgICAgICAgICAgICAgICAgICAgICAgIEV4ZWN1dGlvbk1vZGU6OlF1ZXVlZCA9PiB7", "target": "workflow_core/src/workflow.rs", "index": 0}]')

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
