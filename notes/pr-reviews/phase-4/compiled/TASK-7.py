#!/usr/bin/env python3
"""TASK-7: Add #[serial] to PATH-mutating queued integration tests"""
import base64, json, subprocess, sys

TASK_ID = "TASK-7"
STEPS = json.loads('[{"before_b64": "W2Rldi1kZXBlbmRlbmNpZXNdCnRlbXBmaWxlID0gIjMi", "after_b64": "W2Rldi1kZXBlbmRlbmNpZXNdCnNlcmlhbF90ZXN0ID0gIjMiCnRlbXBmaWxlID0gIjMi", "target": "workflow_utils/Cargo.toml", "index": 0}, {"before_b64": "dXNlIHdvcmtmbG93X2NvcmU6OnByb2Nlc3M6OlF1ZXVlZFN1Ym1pdHRlcjsKdXNlIHdvcmtmbG93X3V0aWxzOjp7UXVldWVkUnVubmVyLCBTY2hlZHVsZXJLaW5kfTs=", "after_b64": "dXNlIHNlcmlhbF90ZXN0OjpzZXJpYWw7CnVzZSB3b3JrZmxvd19jb3JlOjpwcm9jZXNzOjpRdWV1ZWRTdWJtaXR0ZXI7CnVzZSB3b3JrZmxvd191dGlsczo6e1F1ZXVlZFJ1bm5lciwgU2NoZWR1bGVyS2luZH07", "target": "workflow_utils/tests/queued_integration.rs", "index": 1}, {"before_b64": "I1t0ZXN0XQpmbiBzdWJtaXRfcmV0dXJuc19lcnJfd2hlbl9zYmF0Y2hfdW5hdmFpbGFibGUoKSB7", "after_b64": "I1t0ZXN0XQojW3NlcmlhbF0KZm4gc3VibWl0X3JldHVybnNfZXJyX3doZW5fc2JhdGNoX3VuYXZhaWxhYmxlKCkgew==", "target": "workflow_utils/tests/queued_integration.rs", "index": 2}, {"before_b64": "I1t0ZXN0XQpmbiBzdWJtaXRfd2l0aF9tb2NrX3NiYXRjaF9yZXR1cm5zX29uX2Rpc2tfaGFuZGxlKCkgew==", "after_b64": "I1t0ZXN0XQojW3NlcmlhbF0KZm4gc3VibWl0X3dpdGhfbW9ja19zYmF0Y2hfcmV0dXJuc19vbl9kaXNrX2hhbmRsZSgpIHs=", "target": "workflow_utils/tests/queued_integration.rs", "index": 3}]')

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
