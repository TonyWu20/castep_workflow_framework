#!/usr/bin/env python3
"""TASK-6: Add #[must_use] annotation to JsonStateStore::task_successors() getter"""
import base64, json, subprocess, sys

TASK_ID = "TASK-6"
STEPS = json.loads('[{"before_b64": "ICAgIC8vLyBSZXR1cm5zIHRoZSB0YXNrIHN1Y2Nlc3NvciBncmFwaCBwZXJzaXN0ZWQgZnJvbSB0aGUgbGFzdCB3b3JrZmxvdyBydW4uCiAgICAvLy8gUmV0dXJucyBgTm9uZWAgZm9yIHN0YXRlIGZpbGVzIGNyZWF0ZWQgYmVmb3JlIGdyYXBoIHBlcnNpc3RlbmNlIHdhcyBhZGRlZC4KICAgIHB1YiBmbiB0YXNrX3N1Y2Nlc3NvcnMoJnNlbGYpIC0+IE9wdGlvbjwmVGFza1N1Y2Nlc3NvcnM+IHs=", "after_b64": "ICAgIC8vLyBSZXR1cm5zIHRoZSB0YXNrIHN1Y2Nlc3NvciBncmFwaCBwZXJzaXN0ZWQgZnJvbSB0aGUgbGFzdCB3b3JrZmxvdyBydW4uCiAgICAvLy8gUmV0dXJucyBgTm9uZWAgZm9yIHN0YXRlIGZpbGVzIGNyZWF0ZWQgYmVmb3JlIGdyYXBoIHBlcnNpc3RlbmNlIHdhcyBhZGRlZC4KICAgICNbbXVzdF91c2VdCiAgICBwdWIgZm4gdGFza19zdWNjZXNzb3JzKCZzZWxmKSAtPiBPcHRpb248JlRhc2tTdWNjZXNzb3JzPiB7", "target": "workflow_core/src/state.rs", "index": 0}]')

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
