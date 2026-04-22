#!/usr/bin/env python3
"""TASK-3: Make parse_u_values return Result<Vec<f64>, String> instead of silently dropping unparseable values"""
import base64, json, subprocess, sys

TASK_ID = "TASK-3"
STEPS = json.loads('[{"before_b64": "aW1wbCBTd2VlcENvbmZpZyB7CiAgICBwdWIgZm4gcGFyc2VfdV92YWx1ZXMoJnNlbGYpIC0+IFZlYzxmNjQ+IHsKICAgICAgICBzZWxmLnVfdmFsdWVzCiAgICAgICAgICAgIC5zcGxpdCgnLCcpCiAgICAgICAgICAgIC5maWx0ZXJfbWFwKHxzfCBzLnRyaW0oKS5wYXJzZTo6PGY2ND4oKS5vaygpKQogICAgICAgICAgICAuY29sbGVjdCgpCiAgICB9Cn0=", "after_b64": "aW1wbCBTd2VlcENvbmZpZyB7CiAgICBwdWIgZm4gcGFyc2VfdV92YWx1ZXMoJnNlbGYpIC0+IFJlc3VsdDxWZWM8ZjY0PiwgU3RyaW5nPiB7CiAgICAgICAgc2VsZi51X3ZhbHVlcwogICAgICAgICAgICAuc3BsaXQoJywnKQogICAgICAgICAgICAubWFwKHxzfCB7CiAgICAgICAgICAgICAgICBzLnRyaW0oKQogICAgICAgICAgICAgICAgICAgIC5wYXJzZTo6PGY2ND4oKQogICAgICAgICAgICAgICAgICAgIC5tYXBfZXJyKHxlfCBmb3JtYXQhKCJpbnZhbGlkIFUgdmFsdWUgJ3t9Jzoge30iLCBzLnRyaW0oKSwgZSkpCiAgICAgICAgICAgIH0pCiAgICAgICAgICAgIC5jb2xsZWN0Ojo8UmVzdWx0PFZlYzxfPiwgXz4+KCkKICAgIH0KfQ==", "target": "examples/hubbard_u_sweep_slurm/src/config.rs", "index": 0}]')

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
