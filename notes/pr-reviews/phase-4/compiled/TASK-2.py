#!/usr/bin/env python3
"""TASK-2: Eliminate shell injection in QueuedRunner::submit"""
import base64, json, subprocess, sys

TASK_ID = "TASK-2"
STEPS = json.loads('[{"before_b64": "ICAgIGZuIGJ1aWxkX3N1Ym1pdF9jbWQoJnNlbGYsIHNjcmlwdF9wYXRoOiAmc3RyLCB0YXNrX2lkOiAmc3RyLCBsb2dfZGlyOiAmUGF0aCkgLT4gU3RyaW5nIHsKICAgICAgICBsZXQgc3Rkb3V0X3BhdGggPSBsb2dfZGlyLmpvaW4oZm9ybWF0ISgie30uc3Rkb3V0IiwgdGFza19pZCkpOwogICAgICAgIGxldCBzdGRlcnJfcGF0aCA9IGxvZ19kaXIuam9pbihmb3JtYXQhKCJ7fS5zdGRlcnIiLCB0YXNrX2lkKSk7CiAgICAgICAgbWF0Y2ggc2VsZi5zY2hlZHVsZXIgewogICAgICAgICAgICBTY2hlZHVsZXJLaW5kOjpTbHVybSA9PiBmb3JtYXQhKAogICAgICAgICAgICAgICAgInNiYXRjaCAtbyB7fSAtZSB7fSB7fSIsCiAgICAgICAgICAgICAgICBzdGRvdXRfcGF0aC5kaXNwbGF5KCksIHN0ZGVycl9wYXRoLmRpc3BsYXkoKSwgc2NyaXB0X3BhdGgKICAgICAgICAgICAgKSwKICAgICAgICAgICAgU2NoZWR1bGVyS2luZDo6UGJzID0+IGZvcm1hdCEoCiAgICAgICAgICAgICAgICAicXN1YiAtbyB7fSAtZSB7fSB7fSIsCiAgICAgICAgICAgICAgICBzdGRvdXRfcGF0aC5kaXNwbGF5KCksIHN0ZGVycl9wYXRoLmRpc3BsYXkoKSwgc2NyaXB0X3BhdGgKICAgICAgICAgICAgKSwKICAgICAgICB9CiAgICB9CgogICAgZm4gYnVpbGRfcG9sbF9jbWQoJnNlbGYpIC0+IFN0cmluZyB7", "after_b64": "ICAgIGZuIGJ1aWxkX3BvbGxfY21kKCZzZWxmKSAtPiBTdHJpbmcgew==", "target": "workflow_utils/src/queued.rs", "index": 0}, {"before_b64": "ICAgICAgICBsZXQgc3VibWl0X2NtZCA9IHNlbGYuYnVpbGRfc3VibWl0X2NtZCgKICAgICAgICAgICAgJndvcmtkaXIuam9pbigiam9iLnNoIikudG9fc3RyaW5nX2xvc3N5KCksIHRhc2tfaWQsIGxvZ19kaXIKICAgICAgICApOwogICAgICAgIGxldCBvdXRwdXQgPSBDb21tYW5kOjpuZXcoInNoIikKICAgICAgICAgICAgLmFyZ3MoWyItYyIsICZzdWJtaXRfY21kXSkKICAgICAgICAgICAgLmN1cnJlbnRfZGlyKHdvcmtkaXIpCiAgICAgICAgICAgIC5vdXRwdXQoKQogICAgICAgICAgICAubWFwX2VycihXb3JrZmxvd0Vycm9yOjpJbyk/Ow==", "after_b64": "ICAgICAgICBsZXQgc3Rkb3V0X3BhdGggPSBsb2dfZGlyLmpvaW4oZm9ybWF0ISgie30uc3Rkb3V0IiwgdGFza19pZCkpOwogICAgICAgIGxldCBzdGRlcnJfcGF0aCA9IGxvZ19kaXIuam9pbihmb3JtYXQhKCJ7fS5zdGRlcnIiLCB0YXNrX2lkKSk7CiAgICAgICAgbGV0IHNjcmlwdF9wYXRoID0gd29ya2Rpci5qb2luKCJqb2Iuc2giKTsKCiAgICAgICAgbGV0IG91dHB1dCA9IG1hdGNoIHNlbGYuc2NoZWR1bGVyIHsKICAgICAgICAgICAgU2NoZWR1bGVyS2luZDo6U2x1cm0gPT4gQ29tbWFuZDo6bmV3KCJzYmF0Y2giKSwKICAgICAgICAgICAgU2NoZWR1bGVyS2luZDo6UGJzID0+IENvbW1hbmQ6Om5ldygicXN1YiIpLAogICAgICAgIH0KICAgICAgICAuYXJncyhbIi1vIiwgJnN0ZG91dF9wYXRoLnRvX3N0cmluZ19sb3NzeSgpLCAiLWUiLCAmc3RkZXJyX3BhdGgudG9fc3RyaW5nX2xvc3N5KCldKQogICAgICAgIC5hcmcoJnNjcmlwdF9wYXRoKQogICAgICAgIC5jdXJyZW50X2Rpcih3b3JrZGlyKQogICAgICAgIC5vdXRwdXQoKQogICAgICAgIC5tYXBfZXJyKFdvcmtmbG93RXJyb3I6OklvKT87", "target": "workflow_utils/src/queued.rs", "index": 1}]')

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
