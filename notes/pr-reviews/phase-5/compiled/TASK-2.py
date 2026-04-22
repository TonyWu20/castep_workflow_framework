#!/usr/bin/env python3
"""TASK-2: Re-export JOB_SCRIPT_NAME from workflow_utils top-level lib.rs"""
import base64, json, subprocess, sys

TASK_ID = "TASK-2"
STEPS = json.loads('[{"before_b64": "cHViIHVzZSBxdWV1ZWQ6OntRdWV1ZWRSdW5uZXIsIFNjaGVkdWxlcktpbmR9Ow==", "after_b64": "cHViIHVzZSBxdWV1ZWQ6OntRdWV1ZWRSdW5uZXIsIFNjaGVkdWxlcktpbmQsIEpPQl9TQ1JJUFRfTkFNRX07", "target": "workflow_utils/src/lib.rs", "index": 0}]')

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
