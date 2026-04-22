#!/usr/bin/env python3
"""TASK-4: Add doc comment to QueuedProcessHandle::wait() explaining the three exit-code states; fix the stale 'accounting query in wait() may refine' comment in is_running()"""
import base64, json, subprocess, sys

TASK_ID = "TASK-4"
STEPS = json.loads('[{"before_b64": "ICAgICAgICAgICAgICAgIGlmICFydW5uaW5nIHsKICAgICAgICAgICAgICAgICAgICBzZWxmLmZpbmlzaGVkX2V4aXRfY29kZSA9IFNvbWUoMCk7IC8vIGRlZmF1bHQ7IGFjY291bnRpbmcgcXVlcnkgaW4gd2FpdCgpIG1heSByZWZpbmUKICAgICAgICAgICAgICAgIH0=", "after_b64": "ICAgICAgICAgICAgICAgIGlmICFydW5uaW5nIHsKICAgICAgICAgICAgICAgICAgICAvLyBKb2Igbm8gbG9uZ2VyIGFwcGVhcnMgaW4gdGhlIHF1ZXVlOyBhc3N1bWUgc3VjY2VzcyAoZXhpdCBjb2RlIDApLgogICAgICAgICAgICAgICAgICAgIC8vIFRoZSBzY2hlZHVsZXIgZG9lcyBub3QgcHJvdmlkZSB0aGUgYWN0dWFsIGV4aXQgY29kZSBhdCBwb2xsIHRpbWUuCiAgICAgICAgICAgICAgICAgICAgc2VsZi5maW5pc2hlZF9leGl0X2NvZGUgPSBTb21lKDApOwogICAgICAgICAgICAgICAgfQ==", "target": "workflow_utils/src/queued.rs", "index": 0}, {"before_b64": "ICAgIGZuIHdhaXQoJm11dCBzZWxmKSAtPiBSZXN1bHQ8UHJvY2Vzc1Jlc3VsdCwgV29ya2Zsb3dFcnJvcj4gewogICAgICAgIE9rKFByb2Nlc3NSZXN1bHQgewogICAgICAgICAgICBleGl0X2NvZGU6IHNlbGYuZmluaXNoZWRfZXhpdF9jb2RlLA==", "after_b64": "ICAgIC8vLyBSZXR1cm5zIHRoZSBwcm9jZXNzIHJlc3VsdCBhZnRlciBgaXNfcnVubmluZygpYCBoYXMgcmV0dXJuZWQgYGZhbHNlYC4KICAgIC8vLwogICAgLy8vICMgRXhpdCBjb2RlIHNlbWFudGljcyAoYXBwcm94aW1hdGUpCiAgICAvLy8KICAgIC8vLyAtIGBTb21lKDApYCDigJQgam9iIGxlZnQgdGhlIHNjaGVkdWxlciBxdWV1ZSBub3JtYWxseSAoYXNzdW1lZCBzdWNjZXNzKS4KICAgIC8vLyAtIGBTb21lKC0xKWAg4oCUIHRoZSBzY2hlZHVsZXIgc3RhdHVzIHF1ZXJ5IGNvbW1hbmQgaXRzZWxmIGZhaWxlZCAoSS9PIGVycm9yKTsKICAgIC8vLyAgIHRoaXMgY29uZmxhdGVzICJjYW5ub3QgcmVhY2ggc2NoZWR1bGVyIiB3aXRoIGFuIGFjdHVhbCAtMSBleGl0IGNvZGUuCiAgICAvLy8gLSBgTm9uZWAg4oCUIGBpc19ydW5uaW5nKClgIHdhcyBuZXZlciBjYWxsZWQgb3IgbmV2ZXIgdHJhbnNpdGlvbmVkIHRvIGZpbmlzaGVkOwogICAgLy8vICAgY2FsbGVycyBzaG91bGQgdHJlYXQgYE5vbmVgIGFzIGFuIHVua25vd24gb3V0Y29tZS4KICAgIC8vLwogICAgLy8vIFRoZSBjYWxsZXIgaW4gYHdvcmtmbG93LnJzYCBoYW5kbGVzIGFsbCB0aHJlZSBjYXNlcyBkZWZlbnNpdmVseSB2aWEgYHVud3JhcF9vcigtMSlgLgogICAgZm4gd2FpdCgmbXV0IHNlbGYpIC0+IFJlc3VsdDxQcm9jZXNzUmVzdWx0LCBXb3JrZmxvd0Vycm9yPiB7CiAgICAgICAgT2soUHJvY2Vzc1Jlc3VsdCB7CiAgICAgICAgICAgIGV4aXRfY29kZTogc2VsZi5maW5pc2hlZF9leGl0X2NvZGUs", "target": "workflow_utils/src/queued.rs", "index": 1}]')

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
