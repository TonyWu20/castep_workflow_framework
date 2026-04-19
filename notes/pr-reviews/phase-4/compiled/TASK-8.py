#!/usr/bin/env python3
"""TASK-8: Add `#[derive(Default)]` to `SystemProcessRunner`"""
import base64, json, subprocess, sys

TASK_ID = "TASK-8"
STEPS = json.loads('[{"before_b64": "Ly8vIENvbmNyZXRlIGltcGxlbWVudGF0aW9uIG9mIHRoZSBQcm9jZXNzUnVubmVyIHRyYWl0IGZvciBzeXN0ZW0gcHJvY2Vzc2VzLgovLy8gV3JhcHMgYHN0ZDo6cHJvY2Vzczo6Q2hpbGRgIHdpdGggb3V0cHV0IGNhcHR1cmUgYW5kIHRpbWluZy4KcHViIHN0cnVjdCBTeXN0ZW1Qcm9jZXNzUnVubmVyIHs=", "after_b64": "Ly8vIENvbmNyZXRlIGltcGxlbWVudGF0aW9uIG9mIHRoZSBQcm9jZXNzUnVubmVyIHRyYWl0IGZvciBzeXN0ZW0gcHJvY2Vzc2VzLgovLy8gV3JhcHMgYHN0ZDo6cHJvY2Vzczo6Q2hpbGRgIHdpdGggb3V0cHV0IGNhcHR1cmUgYW5kIHRpbWluZy4KI1tkZXJpdmUoRGVmYXVsdCldCnB1YiBzdHJ1Y3QgU3lzdGVtUHJvY2Vzc1J1bm5lciB7", "target": "workflow_utils/src/executor.rs", "index": 0}]')

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
