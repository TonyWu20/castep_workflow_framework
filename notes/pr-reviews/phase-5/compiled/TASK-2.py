#!/usr/bin/env python3
"""TASK-2: Use JOB_SCRIPT_NAME constant instead of hardcoded 'job.sh' in queued integration tests"""
import base64, json, subprocess, sys

TASK_ID = "TASK-2"
STEPS = json.loads('[{"before_b64": "dXNlIHdvcmtmbG93X3V0aWxzOjp7UXVldWVkUnVubmVyLCBTY2hlZHVsZXJLaW5kfTs=", "after_b64": "dXNlIHdvcmtmbG93X3V0aWxzOjp7UXVldWVkUnVubmVyLCBTY2hlZHVsZXJLaW5kLCBKT0JfU0NSSVBUX05BTUV9Ow==", "target": "workflow_utils/tests/queued_integration.rs", "index": 0}, {"before_b64": "ICAgIGxldCB3b3JrZGlyID0gZGlyLnBhdGgoKS5qb2luKCJ3b3JrIik7CiAgICBzdGQ6OmZzOjpjcmVhdGVfZGlyX2FsbCgmd29ya2RpcikudW53cmFwKCk7CiAgICBzdGQ6OmZzOjp3cml0ZSh3b3JrZGlyLmpvaW4oImpvYi5zaCIpLCAiIyEvYmluL3NoXG5lY2hvIGhlbGxvXG4iKS51bndyYXAoKTsKCiAgICAvLyBSZXN0cmljdCBQQVRIIHRvIGFuIGVtcHR5IGRpcmVjdG9yeSBzbyBgc2JhdGNoYCBjYW5ub3QgYmUgZm91bmQu", "after_b64": "ICAgIGxldCB3b3JrZGlyID0gZGlyLnBhdGgoKS5qb2luKCJ3b3JrIik7CiAgICBzdGQ6OmZzOjpjcmVhdGVfZGlyX2FsbCgmd29ya2RpcikudW53cmFwKCk7CiAgICBzdGQ6OmZzOjp3cml0ZSh3b3JrZGlyLmpvaW4oSk9CX1NDUklQVF9OQU1FKSwgIiMhL2Jpbi9zaFxuZWNobyBoZWxsb1xuIikudW53cmFwKCk7CgogICAgLy8gUmVzdHJpY3QgUEFUSCB0byBhbiBlbXB0eSBkaXJlY3Rvcnkgc28gYHNiYXRjaGAgY2Fubm90IGJlIGZvdW5kLg==", "target": "workflow_utils/tests/queued_integration.rs", "index": 1}, {"before_b64": "ICAgIGxldCB3b3JrZGlyID0gZGlyLnBhdGgoKS5qb2luKCJ3b3JrIik7CiAgICBzdGQ6OmZzOjpjcmVhdGVfZGlyX2FsbCgmd29ya2RpcikudW53cmFwKCk7CiAgICBzdGQ6OmZzOjp3cml0ZSh3b3JrZGlyLmpvaW4oImpvYi5zaCIpLCAiIyEvYmluL3NoXG5lY2hvIGhlbGxvXG4iKS51bndyYXAoKTsKCiAgICAvLyBNb2NrIGBzYmF0Y2hgIHRoYXQgcHJpbnRzIGEgU0xVUk0tc3R5bGUgc3VibWlzc2lvbiBsaW5lIGFuZCBleGl0cyAwLg==", "after_b64": "ICAgIGxldCB3b3JrZGlyID0gZGlyLnBhdGgoKS5qb2luKCJ3b3JrIik7CiAgICBzdGQ6OmZzOjpjcmVhdGVfZGlyX2FsbCgmd29ya2RpcikudW53cmFwKCk7CiAgICBzdGQ6OmZzOjp3cml0ZSh3b3JrZGlyLmpvaW4oSk9CX1NDUklQVF9OQU1FKSwgIiMhL2Jpbi9zaFxuZWNobyBoZWxsb1xuIikudW53cmFwKCk7CgogICAgLy8gTW9jayBgc2JhdGNoYCB0aGF0IHByaW50cyBhIFNMVVJNLXN0eWxlIHN1Ym1pc3Npb24gbGluZSBhbmQgZXhpdHMgMC4=", "target": "workflow_utils/tests/queued_integration.rs", "index": 2}]')

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
