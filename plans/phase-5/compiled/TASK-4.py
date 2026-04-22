#!/usr/bin/env python3
"""TASK-4: Remove extra blank line between cmd_inspect and cmd_retry in workflow-cli/src/main.rs"""
import base64, json, subprocess, sys

TASK_ID = "TASK-4"
STEPS = json.loads('[{"before_b64": "ICAgIH0KfQoKCgpmbiBjbWRfcmV0cnkoc3RhdGU6ICZtdXQgSnNvblN0YXRlU3RvcmUsIHRhc2tfaWRzOiAmW1N0cmluZ10pIC0+IGFueWhvdzo6UmVzdWx0PCgpPiB7", "after_b64": "ICAgIH0KfQoKZm4gY21kX3JldHJ5KHN0YXRlOiAmbXV0IEpzb25TdGF0ZVN0b3JlLCB0YXNrX2lkczogJltTdHJpbmddKSAtPiBhbnlob3c6OlJlc3VsdDwoKT4gew==", "target": "workflow-cli/src/main.rs", "index": 0}]')

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
