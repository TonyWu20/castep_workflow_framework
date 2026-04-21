#!/usr/bin/env python3
"""TASK-7: Replace println! with panic! for unexpected error variant in submit_returns_err_when_sbatch_unavailable test"""
import base64, json, subprocess, sys

TASK_ID = "TASK-7"
STEPS = json.loads('[{"before_b64": "ICAgICAgICBFcnIoZSkgPT4gcHJpbnRsbiEoImV4cGVjdGVkIFF1ZXVlU3VibWl0RmFpbGVkLCBnb3Qgezo/fSIsIGUpLA==", "after_b64": "ICAgICAgICBFcnIoZSkgPT4gcGFuaWMhKCJleHBlY3RlZCBRdWV1ZVN1Ym1pdEZhaWxlZCwgZ290IHs6P30iLCBlKSw=", "target": "workflow_utils/tests/queued_integration.rs", "index": 0}]')

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
