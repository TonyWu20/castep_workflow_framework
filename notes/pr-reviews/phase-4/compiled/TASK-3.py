#!/usr/bin/env python3
"""TASK-3: Add doc comments to ProcessHandle trait"""
import base64, json, subprocess, sys

TASK_ID = "TASK-3"
STEPS = json.loads('[{"before_b64": "cHViIHRyYWl0IFByb2Nlc3NIYW5kbGU6IFNlbmQgewogICAgZm4gaXNfcnVubmluZygmbXV0IHNlbGYpIC0+IGJvb2w7CiAgICBmbiB0ZXJtaW5hdGUoJm11dCBzZWxmKSAtPiBSZXN1bHQ8KCksIFdvcmtmbG93RXJyb3I+OwogICAgZm4gd2FpdCgmbXV0IHNlbGYpIC0+IFJlc3VsdDxQcm9jZXNzUmVzdWx0LCBXb3JrZmxvd0Vycm9yPjsKfQ==", "after_b64": "Ly8vIEEgaGFuZGxlIHRvIGEgcnVubmluZyAob3IgZmluaXNoZWQpIHByb2Nlc3MsIHVzZWQgdG8gcG9sbCwgd2FpdCwgb3IgdGVybWluYXRlIGl0LgovLy8KLy8vIEltcGxlbWVudGF0aW9ucyBtdXN0IGJlIGBTZW5kYCBzbyBoYW5kbGVzIGNhbiBiZSBzdG9yZWQgYWNyb3NzIHRocmVhZCBib3VuZGFyaWVzLgpwdWIgdHJhaXQgUHJvY2Vzc0hhbmRsZTogU2VuZCB7CiAgICAvLy8gUmV0dXJucyBgdHJ1ZWAgaWYgdGhlIHByb2Nlc3MgaXMgc3RpbGwgcnVubmluZy4KICAgIC8vLwogICAgLy8vIEltcGxlbWVudGF0aW9ucyBtYXkgY2FjaGUgdGhlIHJlc3VsdCBhbmQgb25seSByZS1wb2xsIHBlcmlvZGljYWxseS4KICAgIGZuIGlzX3J1bm5pbmcoJm11dCBzZWxmKSAtPiBib29sOwoKICAgIC8vLyBSZXF1ZXN0cyB0ZXJtaW5hdGlvbiBvZiB0aGUgcHJvY2Vzcy4KICAgIC8vLwogICAgLy8vIEJlc3QtZWZmb3J0OiB0aGUgcHJvY2VzcyBtYXkgYWxyZWFkeSBoYXZlIGV4aXRlZC4KICAgIGZuIHRlcm1pbmF0ZSgmbXV0IHNlbGYpIC0+IFJlc3VsdDwoKSwgV29ya2Zsb3dFcnJvcj47CgogICAgLy8vIFJldHVybnMgdGhlIHByb2Nlc3MgcmVzdWx0IG9uY2UgdGhlIHByb2Nlc3MgaGFzIGZpbmlzaGVkLgogICAgLy8vCiAgICAvLy8gRm9yIHF1ZXVlZCAoSFBDKSBoYW5kbGVzIHRoaXMgbWF5IHJldHVybiBpbW1lZGlhdGVseSB3aXRoIGBPbkRpc2tgIG91dHB1dAogICAgLy8vIHBhdGhzIHJhdGhlciB0aGFuIGNhcHR1cmVkIG91dHB1dC4gQ2FsbGVycyBzaG91bGQgZW5zdXJlIGBpc19ydW5uaW5nKClgCiAgICAvLy8gaGFzIHJldHVybmVkIGBmYWxzZWAgYmVmb3JlIGNhbGxpbmcgYHdhaXQoKWAsIGFzIGJlaGF2aW91ciB3aGVuIGNhbGxlZAogICAgLy8vIG9uIGEgc3RpbGwtcnVubmluZyBwcm9jZXNzIGlzIGltcGxlbWVudGF0aW9uLWRlZmluZWQuCiAgICBmbiB3YWl0KCZtdXQgc2VsZikgLT4gUmVzdWx0PFByb2Nlc3NSZXN1bHQsIFdvcmtmbG93RXJyb3I+Owp9", "target": "workflow_core/src/process.rs", "index": 0}]')

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
