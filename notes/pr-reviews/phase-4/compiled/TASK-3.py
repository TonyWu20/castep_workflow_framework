#!/usr/bin/env python3
"""TASK-3: Remove `.clone()` on `TaskPhase` in `fire_hooks`"""
import base64, json, subprocess, sys

TASK_ID = "TASK-3"
STEPS = json.loads('[{"before_b64": "Zm4gZmlyZV9ob29rcygKICAgIG1vbml0b3JzOiAmW2NyYXRlOjptb25pdG9yaW5nOjpNb25pdG9yaW5nSG9va10sCiAgICB3b3JrZGlyOiAmc3RkOjpwYXRoOjpQYXRoLAogICAgcGhhc2U6IGNyYXRlOjptb25pdG9yaW5nOjpUYXNrUGhhc2UsCiAgICBleGl0X2NvZGU6IE9wdGlvbjxpMzI+LAogICAgdGFza19pZDogJnN0ciwKICAgIGhvb2tfZXhlY3V0b3I6ICZkeW4gSG9va0V4ZWN1dG9yLAopIHsKICAgIGxldCBjdHggPSBjcmF0ZTo6bW9uaXRvcmluZzo6SG9va0NvbnRleHQgewogICAgICAgIHRhc2tfaWQ6IHRhc2tfaWQudG9fc3RyaW5nKCksCiAgICAgICAgd29ya2Rpcjogd29ya2Rpci50b19wYXRoX2J1ZigpLAogICAgICAgIHBoYXNlOiBwaGFzZS5jbG9uZSgpLAogICAgICAgIGV4aXRfY29kZSwKICAgIH07CiAgICBmb3IgaG9vayBpbiBtb25pdG9ycyB7CiAgICAgICAgbGV0IHNob3VsZF9maXJlID0gbWF0Y2hlcyEoCiAgICAgICAgICAgICgmaG9vay50cmlnZ2VyLCBwaGFzZS5jbG9uZSgpKSwKICAgICAgICAgICAgKGNyYXRlOjptb25pdG9yaW5nOjpIb29rVHJpZ2dlcjo6T25TdGFydCwgY3JhdGU6Om1vbml0b3Jpbmc6OlRhc2tQaGFzZTo6UnVubmluZykKICAgICAgICAgICAgICAgIHwgKGNyYXRlOjptb25pdG9yaW5nOjpIb29rVHJpZ2dlcjo6T25Db21wbGV0ZSwgY3JhdGU6Om1vbml0b3Jpbmc6OlRhc2tQaGFzZTo6Q29tcGxldGVkKQogICAgICAgICAgICAgICAgfCAoY3JhdGU6Om1vbml0b3Jpbmc6Okhvb2tUcmlnZ2VyOjpPbkZhaWx1cmUsIGNyYXRlOjptb25pdG9yaW5nOjpUYXNrUGhhc2U6OkZhaWxlZCkKICAgICAgICApOw==", "after_b64": "Zm4gZmlyZV9ob29rcygKICAgIG1vbml0b3JzOiAmW2NyYXRlOjptb25pdG9yaW5nOjpNb25pdG9yaW5nSG9va10sCiAgICB3b3JrZGlyOiAmc3RkOjpwYXRoOjpQYXRoLAogICAgcGhhc2U6IGNyYXRlOjptb25pdG9yaW5nOjpUYXNrUGhhc2UsCiAgICBleGl0X2NvZGU6IE9wdGlvbjxpMzI+LAogICAgdGFza19pZDogJnN0ciwKICAgIGhvb2tfZXhlY3V0b3I6ICZkeW4gSG9va0V4ZWN1dG9yLAopIHsKICAgIGxldCBjdHggPSBjcmF0ZTo6bW9uaXRvcmluZzo6SG9va0NvbnRleHQgewogICAgICAgIHRhc2tfaWQ6IHRhc2tfaWQudG9fc3RyaW5nKCksCiAgICAgICAgd29ya2Rpcjogd29ya2Rpci50b19wYXRoX2J1ZigpLAogICAgICAgIHBoYXNlLAogICAgICAgIGV4aXRfY29kZSwKICAgIH07CiAgICBmb3IgaG9vayBpbiBtb25pdG9ycyB7CiAgICAgICAgbGV0IHNob3VsZF9maXJlID0gbWF0Y2hlcyEoCiAgICAgICAgICAgICgmaG9vay50cmlnZ2VyLCBwaGFzZSksCiAgICAgICAgICAgIChjcmF0ZTo6bW9uaXRvcmluZzo6SG9va1RyaWdnZXI6Ok9uU3RhcnQsIGNyYXRlOjptb25pdG9yaW5nOjpUYXNrUGhhc2U6OlJ1bm5pbmcpCiAgICAgICAgICAgICAgICB8IChjcmF0ZTo6bW9uaXRvcmluZzo6SG9va1RyaWdnZXI6Ok9uQ29tcGxldGUsIGNyYXRlOjptb25pdG9yaW5nOjpUYXNrUGhhc2U6OkNvbXBsZXRlZCkKICAgICAgICAgICAgICAgIHwgKGNyYXRlOjptb25pdG9yaW5nOjpIb29rVHJpZ2dlcjo6T25GYWlsdXJlLCBjcmF0ZTo6bW9uaXRvcmluZzo6VGFza1BoYXNlOjpGYWlsZWQpCiAgICAgICAgKTs=", "target": "workflow_core/src/workflow.rs", "index": 0}]')

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
