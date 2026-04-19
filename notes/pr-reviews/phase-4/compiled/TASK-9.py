#!/usr/bin/env python3
"""TASK-9: Reduce periodic hook test sleep from 8s to 2s"""
import base64, json, subprocess, sys

TASK_ID = "TASK-9"
STEPS = json.loads('[{"before_b64": "ICAgIHdmLmFkZF90YXNrKAogICAgICAgIFRhc2s6Om5ldygibG9uZ190YXNrIiwgZGlyZWN0X3dpdGhfYXJncygic2xlZXAiLCAmWyI4Il0pKQogICAgICAgICAgICAubW9uaXRvcnModmVjIVtwZXJpb2RpY19ob29rXSkKICAgICkudW53cmFwKCk7", "after_b64": "ICAgIHdmLmFkZF90YXNrKAogICAgICAgIFRhc2s6Om5ldygibG9uZ190YXNrIiwgZGlyZWN0X3dpdGhfYXJncygic2xlZXAiLCAmWyIyIl0pKQogICAgICAgICAgICAubW9uaXRvcnModmVjIVtwZXJpb2RpY19ob29rXSkKICAgICkudW53cmFwKCk7", "target": "workflow_core/tests/hook_recording.rs", "index": 0}]')

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
