#!/usr/bin/env python3
"""TASK-7: Implement `QueuedSubmitter` trait for `QueuedRunner`"""

import base64, json, subprocess, sys

TASK_ID = "TASK-7"
STEPS = json.loads(
    '[{"before_b64": "ICAgIHB1YiBmbiBzdWJtaXQoCiAgICAgICAgJnNlbGYsCiAgICAgICAgd29ya2RpcjogJlBhdGgsCiAgICAgICAgdGFza19pZDogJnN0ciwKICAgICAgICBsb2dfZGlyOiAmUGF0aCwKICAgICkgLT4gUmVzdWx0PEJveDxkeW4gUHJvY2Vzc0hhbmRsZT4sIFdvcmtmbG93RXJyb3I+IHsKICAgICAgICBsZXQgc3VibWl0X2NtZCA9IHNlbGYuYnVpbGRfc3VibWl0X2NtZCgKICAgICAgICAgICAgJndvcmtkaXIuam9pbigiam9iLnNoIikudG9fc3RyaW5nX2xvc3N5KCksIHRhc2tfaWQsIGxvZ19kaXIKICAgICAgICApOwogICAgICAgIGxldCBvdXRwdXQgPSBDb21tYW5kOjpuZXcoInNoIikKICAgICAgICAgICAgLmFyZ3MoWyItYyIsICZzdWJtaXRfY21kXSkKICAgICAgICAgICAgLmN1cnJlbnRfZGlyKHdvcmtkaXIpCiAgICAgICAgICAgIC5vdXRwdXQoKQogICAgICAgICAgICAubWFwX2VycihXb3JrZmxvd0Vycm9yOjpJbyk/OwoKICAgICAgICBpZiAhb3V0cHV0LnN0YXR1cy5zdWNjZXNzKCkgewogICAgICAgICAgICByZXR1cm4gRXJyKFdvcmtmbG93RXJyb3I6OlF1ZXVlU3VibWl0RmFpbGVkKAogICAgICAgICAgICAgICAgU3RyaW5nOjpmcm9tX3V0ZjhfbG9zc3koJm91dHB1dC5zdGRlcnIpLmludG9fb3duZWQoKQogICAgICAgICAgICApKTsKICAgICAgICB9CgogICAgICAgIGxldCBzdGRvdXQgPSBTdHJpbmc6OmZyb21fdXRmOF9sb3NzeSgmb3V0cHV0LnN0ZG91dCk7CiAgICAgICAgbGV0IGpvYl9pZCA9IHNlbGYucGFyc2Vfam9iX2lkKCZzdGRvdXQpPzsKCiAgICAgICAgbGV0IHN0ZG91dF9wYXRoID0gbG9nX2Rpci5qb2luKGZvcm1hdCEoInt9LnN0ZG91dCIsIHRhc2tfaWQpKTsKICAgICAgICBsZXQgc3RkZXJyX3BhdGggPSBsb2dfZGlyLmpvaW4oZm9ybWF0ISgie30uc3RkZXJyIiwgdGFza19pZCkpOwoKICAgICAgICBPayhCb3g6Om5ldyhRdWV1ZWRQcm9jZXNzSGFuZGxlIHsKICAgICAgICAgICAgam9iX2lkLAogICAgICAgICAgICBwb2xsX2NtZDogc2VsZi5idWlsZF9wb2xsX2NtZCgpLAogICAgICAgICAgICBjYW5jZWxfY21kOiBzZWxmLmJ1aWxkX2NhbmNlbF9jbWQoKSwKICAgICAgICAgICAgd29ya2Rpcjogd29ya2Rpci50b19wYXRoX2J1ZigpLAogICAgICAgICAgICBzdGRvdXRfcGF0aCwKICAgICAgICAgICAgc3RkZXJyX3BhdGgsCiAgICAgICAgICAgIGxhc3RfcG9sbDogSW5zdGFudDo6bm93KCksCiAgICAgICAgICAgIHBvbGxfaW50ZXJ2YWw6IER1cmF0aW9uOjpmcm9tX3NlY3MoMTUpLAogICAgICAgICAgICBjYWNoZWRfcnVubmluZzogdHJ1ZSwKICAgICAgICAgICAgZmluaXNoZWRfZXhpdF9jb2RlOiBOb25lLAogICAgICAgICAgICBzdGFydGVkX2F0OiBJbnN0YW50Ojpub3coKSwKICAgICAgICB9KSkKICAgIH0KfQ==", "after_b64": "fQoKaW1wbCB3b3JrZmxvd19jb3JlOjpwcm9jZXNzOjpRdWV1ZWRTdWJtaXR0ZXIgZm9yIFF1ZXVlZFJ1bm5lciB7CiAgICBmbiBzdWJtaXQoCiAgICAgICAgJnNlbGYsCiAgICAgICAgd29ya2RpcjogJlBhdGgsCiAgICAgICAgdGFza19pZDogJnN0ciwKICAgICAgICBsb2dfZGlyOiAmUGF0aCwKICAgICkgLT4gUmVzdWx0PEJveDxkeW4gUHJvY2Vzc0hhbmRsZT4sIFdvcmtmbG93RXJyb3I+IHsKICAgICAgICBsZXQgc3VibWl0X2NtZCA9IHNlbGYuYnVpbGRfc3VibWl0X2NtZCgKICAgICAgICAgICAgJndvcmtkaXIuam9pbigiam9iLnNoIikudG9fc3RyaW5nX2xvc3N5KCksIHRhc2tfaWQsIGxvZ19kaXIKICAgICAgICApOwogICAgICAgIGxldCBvdXRwdXQgPSBDb21tYW5kOjpuZXcoInNoIikKICAgICAgICAgICAgLmFyZ3MoWyItYyIsICZzdWJtaXRfY21kXSkKICAgICAgICAgICAgLmN1cnJlbnRfZGlyKHdvcmtkaXIpCiAgICAgICAgICAgIC5vdXRwdXQoKQogICAgICAgICAgICAubWFwX2VycihXb3JrZmxvd0Vycm9yOjpJbyk/OwoKICAgICAgICBpZiAhb3V0cHV0LnN0YXR1cy5zdWNjZXNzKCkgewogICAgICAgICAgICByZXR1cm4gRXJyKFdvcmtmbG93RXJyb3I6OlF1ZXVlU3VibWl0RmFpbGVkKAogICAgICAgICAgICAgICAgU3RyaW5nOjpmcm9tX3V0ZjhfbG9zc3koJm91dHB1dC5zdGRlcnIpLmludG9fb3duZWQoKQogICAgICAgICAgICApKTsKICAgICAgICB9CgogICAgICAgIGxldCBzdGRvdXQgPSBTdHJpbmc6OmZyb21fdXRmOF9sb3NzeSgmb3V0cHV0LnN0ZG91dCk7CiAgICAgICAgbGV0IGpvYl9pZCA9IHNlbGYucGFyc2Vfam9iX2lkKCZzdGRvdXQpPzsKCiAgICAgICAgbGV0IHN0ZG91dF9wYXRoID0gbG9nX2Rpci5qb2luKGZvcm1hdCEoInt9LnN0ZG91dCIsIHRhc2tfaWQpKTsKICAgICAgICBsZXQgc3RkZXJyX3BhdGggPSBsb2dfZGlyLmpvaW4oZm9ybWF0ISgie30uc3RkZXJyIiwgdGFza19pZCkpOwoKICAgICAgICBPayhCb3g6Om5ldyhRdWV1ZWRQcm9jZXNzSGFuZGxlIHsKICAgICAgICAgICAgam9iX2lkLAogICAgICAgICAgICBwb2xsX2NtZDogc2VsZi5idWlsZF9wb2xsX2NtZCgpLAogICAgICAgICAgICBjYW5jZWxfY21kOiBzZWxmLmJ1aWxkX2NhbmNlbF9jbWQoKSwKICAgICAgICAgICAgd29ya2Rpcjogd29ya2Rpci50b19wYXRoX2J1ZigpLAogICAgICAgICAgICBzdGRvdXRfcGF0aCwKICAgICAgICAgICAgc3RkZXJyX3BhdGgsCiAgICAgICAgICAgIGxhc3RfcG9sbDogSW5zdGFudDo6bm93KCksCiAgICAgICAgICAgIHBvbGxfaW50ZXJ2YWw6IER1cmF0aW9uOjpmcm9tX3NlY3MoMTUpLAogICAgICAgICAgICBjYWNoZWRfcnVubmluZzogdHJ1ZSwKICAgICAgICAgICAgZmluaXNoZWRfZXhpdF9jb2RlOiBOb25lLAogICAgICAgICAgICBzdGFydGVkX2F0OiBJbnN0YW50Ojpub3coKSwKICAgICAgICB9KSkKICAgIH0KfQ==", "target": "workflow_utils/src/queued.rs", "index": 0}]'
)

for step in STEPS:
    before = base64.b64decode(step["before_b64"]).decode()
    after = base64.b64decode(step["after_b64"]).decode()
    target = step["target"]
    idx = step["index"]

    content = open(target).read()
    if before not in content:
        print(
            f"FAILED {TASK_ID} change {idx}: pattern not found in {target}",
            file=sys.stderr,
        )
        print(f"Expected (first 200 chars): {repr(before[:200])}", file=sys.stderr)
        sys.exit(1)

    result = subprocess.run(
        ["sd", "-F", "-A", "-n", "1", before, after, target],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print(
            f"FAILED {TASK_ID} change {idx}: sd error: {result.stderr}", file=sys.stderr
        )
        sys.exit(result.returncode)

    new_content = open(target).read()
    if after and after not in new_content:
        print(
            f"FAILED {TASK_ID} change {idx}: replacement not found after apply",
            file=sys.stderr,
        )
        sys.exit(1)

    print(f"OK {TASK_ID} change {idx}: applied to {target}")

print(f"OK {TASK_ID}: all changes applied")
