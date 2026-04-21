#!/usr/bin/env python3
"""TASK-5: Remove the second (duplicate) computation of stdout_path and stderr_path in QueuedRunner::submit"""
import base64, json, subprocess, sys

TASK_ID = "TASK-5"
STEPS = json.loads('[{"before_b64": "ICAgICAgICBsZXQgc3Rkb3V0ID0gU3RyaW5nOjpmcm9tX3V0ZjhfbG9zc3koJm91dHB1dC5zdGRvdXQpOwogICAgICAgIGxldCBqb2JfaWQgPSBzZWxmLnBhcnNlX2pvYl9pZCgmc3Rkb3V0KT87CgogICAgICAgIGxldCBzdGRvdXRfcGF0aCA9IGxvZ19kaXIuam9pbihmb3JtYXQhKCJ7fS5zdGRvdXQiLCB0YXNrX2lkKSk7CiAgICAgICAgbGV0IHN0ZGVycl9wYXRoID0gbG9nX2Rpci5qb2luKGZvcm1hdCEoInt9LnN0ZGVyciIsIHRhc2tfaWQpKTsKCiAgICAgICAgT2soQm94OjpuZXcoUXVldWVkUHJvY2Vzc0hhbmRsZSB7", "after_b64": "ICAgICAgICBsZXQgc3Rkb3V0ID0gU3RyaW5nOjpmcm9tX3V0ZjhfbG9zc3koJm91dHB1dC5zdGRvdXQpOwogICAgICAgIGxldCBqb2JfaWQgPSBzZWxmLnBhcnNlX2pvYl9pZCgmc3Rkb3V0KT87CgogICAgICAgIE9rKEJveDo6bmV3KFF1ZXVlZFByb2Nlc3NIYW5kbGUgew==", "target": "workflow_utils/src/queued.rs", "index": 0}]')

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
