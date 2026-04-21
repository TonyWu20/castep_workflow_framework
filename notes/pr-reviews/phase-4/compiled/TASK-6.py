#!/usr/bin/env python3
"""TASK-6: Change pub scheduler field to private, add pub fn scheduler() getter on QueuedRunner"""
import base64, json, subprocess, sys

TASK_ID = "TASK-6"
STEPS = json.loads('[{"before_b64": "cHViIHN0cnVjdCBRdWV1ZWRSdW5uZXIgewogICAgLy8vIFdoaWNoIHNjaGVkdWxlciBkaWFsZWN0IHRvIHVzZSBmb3IgY29tbWFuZCBjb25zdHJ1Y3Rpb24uCiAgICBwdWIgc2NoZWR1bGVyOiBTY2hlZHVsZXJLaW5kLAp9", "after_b64": "cHViIHN0cnVjdCBRdWV1ZWRSdW5uZXIgewogICAgLy8vIFdoaWNoIHNjaGVkdWxlciBkaWFsZWN0IHRvIHVzZSBmb3IgY29tbWFuZCBjb25zdHJ1Y3Rpb24uCiAgICBzY2hlZHVsZXI6IFNjaGVkdWxlcktpbmQsCn0=", "target": "workflow_utils/src/queued.rs", "index": 0}, {"before_b64": "ICAgIHB1YiBmbiBuZXcoc2NoZWR1bGVyOiBTY2hlZHVsZXJLaW5kKSAtPiBTZWxmIHsKICAgICAgICBTZWxmIHsgc2NoZWR1bGVyIH0KICAgIH0KCiAgICBmbiBidWlsZF9wb2xsX2NtZA==", "after_b64": "ICAgIHB1YiBmbiBuZXcoc2NoZWR1bGVyOiBTY2hlZHVsZXJLaW5kKSAtPiBTZWxmIHsKICAgICAgICBTZWxmIHsgc2NoZWR1bGVyIH0KICAgIH0KCiAgICAvLy8gUmV0dXJucyB0aGUgc2NoZWR1bGVyIGtpbmQgdGhpcyBydW5uZXIgdGFyZ2V0cy4KICAgIHB1YiBmbiBzY2hlZHVsZXIoJnNlbGYpIC0+IFNjaGVkdWxlcktpbmQgewogICAgICAgIHNlbGYuc2NoZWR1bGVyCiAgICB9CgogICAgZm4gYnVpbGRfcG9sbF9jbWQ=", "target": "workflow_utils/src/queued.rs", "index": 1}]')

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
