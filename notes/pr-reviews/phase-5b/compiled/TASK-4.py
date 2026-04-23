#!/usr/bin/env python3
"""TASK-4: Wrap 'workflow_core' in backticks in the doc comment (line 1) and ensure file ends with trailing newline."""
import base64, json, subprocess, sys

TASK_ID = "TASK-4"
STEPS = json.loads('[{"before_b64": "Ly8hIENvbnZlbmllbmNlIHJlLWV4cG9ydHMgZm9yIGNvbW1vbiB3b3JrZmxvd19jb3JlIHR5cGVzLg==", "after_b64": "Ly8hIENvbnZlbmllbmNlIHJlLWV4cG9ydHMgZm9yIGNvbW1vbiBgd29ya2Zsb3dfY29yZWAgdHlwZXMu", "target": "workflow_core/src/prelude.rs", "index": 0}, {"before_b64": "cHViIHVzZSBjcmF0ZTo6e0hvb2tFeGVjdXRvciwgUHJvY2Vzc1J1bm5lcn07", "after_b64": "cHViIHVzZSBjcmF0ZTo6e0hvb2tFeGVjdXRvciwgUHJvY2Vzc1J1bm5lcn07", "target": "workflow_core/src/prelude.rs", "index": 1}]')

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
