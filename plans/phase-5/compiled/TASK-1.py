#!/usr/bin/env python3
"""TASK-1: Change TaskSuccessors::downstream_of to accept &[S] where S: AsRef<str> instead of &[String]"""
import base64, json, subprocess, sys

TASK_ID = "TASK-1"
STEPS = json.loads('[{"before_b64": "ICAgIHB1YiBmbiBkb3duc3RyZWFtX29mKCZzZWxmLCBzdGFydDogJltTdHJpbmddKSAtPiBzdGQ6OmNvbGxlY3Rpb25zOjpIYXNoU2V0PFN0cmluZz4gewogICAgICAgIGxldCBtdXQgdmlzaXRlZCA9IHN0ZDo6Y29sbGVjdGlvbnM6Okhhc2hTZXQ6Om5ldygpOwogICAgICAgIGxldCBtdXQgcXVldWU6IHN0ZDo6Y29sbGVjdGlvbnM6OlZlY0RlcXVlPFN0cmluZz4gPSBzdGFydC5pdGVyKCkuY2xvbmVkKCkuY29sbGVjdCgpOw==", "after_b64": "ICAgIHB1YiBmbiBkb3duc3RyZWFtX29mPFM6IEFzUmVmPHN0cj4+KCZzZWxmLCBzdGFydDogJltTXSkgLT4gc3RkOjpjb2xsZWN0aW9uczo6SGFzaFNldDxTdHJpbmc+IHsKICAgICAgICBsZXQgbXV0IHZpc2l0ZWQgPSBzdGQ6OmNvbGxlY3Rpb25zOjpIYXNoU2V0OjpuZXcoKTsKICAgICAgICBsZXQgbXV0IHF1ZXVlOiBzdGQ6OmNvbGxlY3Rpb25zOjpWZWNEZXF1ZTxTdHJpbmc+ID0KICAgICAgICAgICAgc3RhcnQuaXRlcigpLm1hcCh8c3wgcy5hc19yZWYoKS50b19vd25lZCgpKS5jb2xsZWN0KCk7", "target": "workflow_core/src/state.rs", "index": 0}]')

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
