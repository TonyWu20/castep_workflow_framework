#!/usr/bin/env python3
"""TASK-1: Extract hardcoded 'job.sh' literal into a pub const JOB_SCRIPT_NAME in queued module"""
import base64, json, subprocess, sys

TASK_ID = "TASK-1"
STEPS = json.loads('[{"before_b64": "dXNlIHdvcmtmbG93X2NvcmU6OmVycm9yOjpXb3JrZmxvd0Vycm9yOwp1c2Ugd29ya2Zsb3dfY29yZTo6cHJvY2Vzczo6e091dHB1dExvY2F0aW9uLCBQcm9jZXNzSGFuZGxlLCBQcm9jZXNzUmVzdWx0fTsKCi8vLyBUaGUgdHlwZSBvZiBIUEMgam9iIHNjaGVkdWxlciB0byB0YXJnZXQuCiNbZGVyaXZlKERlYnVnLCBDbG9uZSwgQ29weSldCnB1YiBlbnVtIFNjaGVkdWxlcktpbmQgew==", "after_b64": "dXNlIHdvcmtmbG93X2NvcmU6OmVycm9yOjpXb3JrZmxvd0Vycm9yOwp1c2Ugd29ya2Zsb3dfY29yZTo6cHJvY2Vzczo6e091dHB1dExvY2F0aW9uLCBQcm9jZXNzSGFuZGxlLCBQcm9jZXNzUmVzdWx0fTsKCi8vLyBEZWZhdWx0IGpvYiBzY3JpcHQgZmlsZW5hbWUgdXNlZCBieSBbYFF1ZXVlZFJ1bm5lcjo6c3VibWl0YF0uCnB1YiBjb25zdCBKT0JfU0NSSVBUX05BTUU6ICZzdHIgPSAiam9iLnNoIjsKCi8vLyBUaGUgdHlwZSBvZiBIUEMgam9iIHNjaGVkdWxlciB0byB0YXJnZXQuCiNbZGVyaXZlKERlYnVnLCBDbG9uZSwgQ29weSldCnB1YiBlbnVtIFNjaGVkdWxlcktpbmQgew==", "target": "workflow_utils/src/queued.rs", "index": 0}, {"before_b64": "ICAgICAgICAuYXJncyhbIi1vIiwgJnN0ZG91dF9wYXRoLnRvX3N0cmluZ19sb3NzeSgpLCAiLWUiLCAmc3RkZXJyX3BhdGgudG9fc3RyaW5nX2xvc3N5KCldKQogICAgICAgIC5hcmcoImpvYi5zaCIpCiAgICAgICAgLmN1cnJlbnRfZGlyKHdvcmtkaXIp", "after_b64": "ICAgICAgICAuYXJncyhbIi1vIiwgJnN0ZG91dF9wYXRoLnRvX3N0cmluZ19sb3NzeSgpLCAiLWUiLCAmc3RkZXJyX3BhdGgudG9fc3RyaW5nX2xvc3N5KCldKQogICAgICAgIC5hcmcoSk9CX1NDUklQVF9OQU1FKQogICAgICAgIC5jdXJyZW50X2Rpcih3b3JrZGlyKQ==", "target": "workflow_utils/src/queued.rs", "index": 1}]')

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
