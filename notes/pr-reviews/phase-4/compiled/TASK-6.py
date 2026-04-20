#!/usr/bin/env python3
"""TASK-6: Add doc comments to SchedulerKind and QueuedRunner public API"""
import base64, json, subprocess, sys

TASK_ID = "TASK-6"
STEPS = json.loads('[{"before_b64": "I1tkZXJpdmUoRGVidWcsIENsb25lLCBDb3B5KV0KcHViIGVudW0gU2NoZWR1bGVyS2luZCB7CiAgICBTbHVybSwKICAgIFBicywKfQoKcHViIHN0cnVjdCBRdWV1ZWRSdW5uZXIgewogICAgcHViIHNjaGVkdWxlcjogU2NoZWR1bGVyS2luZCwKfQ==", "after_b64": "Ly8vIFRoZSB0eXBlIG9mIEhQQyBqb2Igc2NoZWR1bGVyIHRvIHRhcmdldC4KI1tkZXJpdmUoRGVidWcsIENsb25lLCBDb3B5KV0KcHViIGVudW0gU2NoZWR1bGVyS2luZCB7CiAgICAvLy8gU0xVUk0gV29ya2xvYWQgTWFuYWdlciAoYHNiYXRjaGAgLyBgc3F1ZXVlYCAvIGBzY2FuY2VsYCkuCiAgICBTbHVybSwKICAgIC8vLyBQb3J0YWJsZSBCYXRjaCBTeXN0ZW0gKGBxc3ViYCAvIGBxc3RhdGAgLyBgcWRlbGApLgogICAgUGJzLAp9CgovLy8gU3VibWl0cyBhbmQgbWFuYWdlcyBqb2JzIHZpYSBhbiBIUEMgYmF0Y2ggc2NoZWR1bGVyLgovLy8KLy8vIEltcGxlbWVudHMgW2BRdWV1ZWRTdWJtaXR0ZXJgXSh3b3JrZmxvd19jb3JlOjpwcm9jZXNzOjpRdWV1ZWRTdWJtaXR0ZXIpIHRvCi8vLyBpbnRlZ3JhdGUgd2l0aCB0aGUgd29ya2Zsb3cgZW5naW5lJ3MgYFF1ZXVlZGAgZXhlY3V0aW9uIG1vZGUuCnB1YiBzdHJ1Y3QgUXVldWVkUnVubmVyIHsKICAgIC8vLyBXaGljaCBzY2hlZHVsZXIgZGlhbGVjdCB0byB1c2UgZm9yIGNvbW1hbmQgY29uc3RydWN0aW9uLgogICAgcHViIHNjaGVkdWxlcjogU2NoZWR1bGVyS2luZCwKfQ==", "target": "workflow_utils/src/queued.rs", "index": 0}]')

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
