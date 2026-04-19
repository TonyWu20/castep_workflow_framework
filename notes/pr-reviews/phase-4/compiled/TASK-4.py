#!/usr/bin/env python3
"""TASK-4: Simplify `ExecutionMode::Queued` to unit-like variant"""
import base64, json, subprocess, sys

TASK_ID = "TASK-4"
STEPS = json.loads('[{"before_b64": "ICAgIC8vLyBOb3QgeWV0IGltcGxlbWVudGVkLiBDb25zdHJ1Y3RpbmcgYSB0YXNrIHdpdGggdGhpcyBtb2RlIHdpbGwgY2F1c2UKICAgIC8vLyBgV29ya2Zsb3c6OnJ1bigpYCB0byByZXR1cm4gYEVycihXb3JrZmxvd0Vycm9yOjpJbnZhbGlkQ29uZmlnKWAuCiAgICAvLy8gUmVzZXJ2ZWQgZm9yIGZ1dHVyZSBIUEMgcXVldWUgaW50ZWdyYXRpb24gKFNMVVJNL1BCUykuCiAgICBRdWV1ZWQgewogICAgICAgIHN1Ym1pdF9jbWQ6IFN0cmluZywKICAgICAgICBwb2xsX2NtZDogU3RyaW5nLAogICAgICAgIGNhbmNlbF9jbWQ6IFN0cmluZywKICAgIH0s", "after_b64": "ICAgIC8vLyBRdWV1ZWQgZXhlY3V0aW9uIHZpYSBhbiBIUEMgc2NoZWR1bGVyIChTTFVSTS9QQlMpLgogICAgLy8vIFRoZSBhY3R1YWwgc3VibWl0L3BvbGwvY2FuY2VsIGNvbW1hbmRzIGFyZSBvd25lZCBieSB0aGUgYFF1ZXVlZFN1Ym1pdHRlcmAKICAgIC8vLyBpbXBsZW1lbnRhdGlvbiBzZXQgdmlhIGBXb3JrZmxvdzo6d2l0aF9xdWV1ZWRfc3VibWl0dGVyKClgLgogICAgUXVldWVkLA==", "target": "workflow_core/src/task.rs", "index": 0}]')

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
