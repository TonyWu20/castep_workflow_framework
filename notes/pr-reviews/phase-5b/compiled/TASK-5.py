#!/usr/bin/env python3
"""TASK-5: Replace individual workflow_core/workflow_utils imports with use workflow_utils::prelude::*; in both example binaries."""
import base64, json, subprocess, sys

TASK_ID = "TASK-5"
STEPS = json.loads('[{"before_b64": "dXNlIHdvcmtmbG93X2NvcmU6OnN0YXRlOjpKc29uU3RhdGVTdG9yZTsKdXNlIHdvcmtmbG93X2NvcmU6OnRhc2s6OntFeGVjdXRpb25Nb2RlLCBUYXNrfTsKdXNlIHdvcmtmbG93X2NvcmU6OndvcmtmbG93OjpXb3JrZmxvdzsKdXNlIHdvcmtmbG93X2NvcmU6OldvcmtmbG93RXJyb3I7CnVzZSB3b3JrZmxvd191dGlsczo6e2NyZWF0ZV9kaXIsIHdyaXRlX2ZpbGV9Ow==", "after_b64": "dXNlIHdvcmtmbG93X3V0aWxzOjpwcmVsdWRlOjoqOw==", "target": "examples/hubbard_u_sweep/src/main.rs", "index": 0}, {"before_b64": "dXNlIHN0ZDo6c3luYzo6QXJjOwp1c2Ugd29ya2Zsb3dfY29yZTo6c3RhdGU6Okpzb25TdGF0ZVN0b3JlOwp1c2Ugd29ya2Zsb3dfY29yZTo6dGFzazo6e0V4ZWN1dGlvbk1vZGUsIFRhc2t9Owp1c2Ugd29ya2Zsb3dfY29yZTo6d29ya2Zsb3c6OldvcmtmbG93Owp1c2Ugd29ya2Zsb3dfY29yZTo6e0hvb2tFeGVjdXRvciwgUHJvY2Vzc1J1bm5lciwgV29ya2Zsb3dFcnJvcn07CnVzZSB3b3JrZmxvd191dGlsczo6ewogICAgY3JlYXRlX2RpciwgcmVhZF9maWxlLCB3cml0ZV9maWxlLCBRdWV1ZWRSdW5uZXIsIFNjaGVkdWxlcktpbmQsIFNoZWxsSG9va0V4ZWN1dG9yLAogICAgU3lzdGVtUHJvY2Vzc1J1bm5lciwgSk9CX1NDUklQVF9OQU1FLAp9Ow==", "after_b64": "dXNlIHN0ZDo6c3luYzo6QXJjOwp1c2Ugd29ya2Zsb3dfdXRpbHM6OnByZWx1ZGU6Oio7", "target": "examples/hubbard_u_sweep_slurm/src/main.rs", "index": 1}]')

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
