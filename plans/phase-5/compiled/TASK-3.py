#!/usr/bin/env python3
"""TASK-3: Extract a free function parse_u_values(s: &str) from SweepConfig::parse_u_values, fix double trim, and have the method delegate to it"""
import base64, json, subprocess, sys

TASK_ID = "TASK-3"
STEPS = json.loads('[{"before_b64": "aW1wbCBTd2VlcENvbmZpZyB7CiAgICBwdWIgZm4gcGFyc2VfdV92YWx1ZXMoJnNlbGYpIC0+IFJlc3VsdDxWZWM8ZjY0PiwgU3RyaW5nPiB7CiAgICAgICAgc2VsZi51X3ZhbHVlcwogICAgICAgICAgICAuc3BsaXQoJywnKQogICAgICAgICAgICAubWFwKHxzfCB7CiAgICAgICAgICAgICAgICBzLnRyaW0oKQogICAgICAgICAgICAgICAgICAgIC5wYXJzZTo6PGY2ND4oKQogICAgICAgICAgICAgICAgICAgIC5tYXBfZXJyKHxlfCBmb3JtYXQhKCJpbnZhbGlkIFUgdmFsdWUgJ3t9Jzoge30iLCBzLnRyaW0oKSwgZSkpCiAgICAgICAgICAgIH0pCiAgICAgICAgICAgIC5jb2xsZWN0Ojo8UmVzdWx0PFZlYzxfPiwgXz4+KCkKICAgIH0KfQ==", "after_b64": "Ly8vIFBhcnNlcyBhIGNvbW1hLXNlcGFyYXRlZCBzdHJpbmcgb2YgZjY0IHZhbHVlcy4KLy8vCi8vLyBFYWNoIHNlZ21lbnQgaXMgdHJpbW1lZCBiZWZvcmUgcGFyc2luZy4KLy8vIFJldHVybnMgYW4gZXJyb3Igc3RyaW5nIGlkZW50aWZ5aW5nIHRoZSBvZmZlbmRpbmcgdG9rZW4gb24gZmFpbHVyZS4KcHViIGZuIHBhcnNlX3VfdmFsdWVzKHM6ICZzdHIpIC0+IFJlc3VsdDxWZWM8ZjY0PiwgU3RyaW5nPiB7CiAgICBzLnNwbGl0KCcsJykKICAgICAgICAubWFwKHxzZWdtZW50fCB7CiAgICAgICAgICAgIGxldCB0cmltbWVkID0gc2VnbWVudC50cmltKCk7CiAgICAgICAgICAgIHRyaW1tZWQKICAgICAgICAgICAgICAgIC5wYXJzZTo6PGY2ND4oKQogICAgICAgICAgICAgICAgLm1hcF9lcnIofGV8IGZvcm1hdCEoImludmFsaWQgVSB2YWx1ZSAne30nOiB7fSIsIHRyaW1tZWQsIGUpKQogICAgICAgIH0pCiAgICAgICAgLmNvbGxlY3Q6OjxSZXN1bHQ8VmVjPF8+LCBfPj4oKQp9CgppbXBsIFN3ZWVwQ29uZmlnIHsKICAgIHB1YiBmbiBwYXJzZV91X3ZhbHVlcygmc2VsZikgLT4gUmVzdWx0PFZlYzxmNjQ+LCBTdHJpbmc+IHsKICAgICAgICBwYXJzZV91X3ZhbHVlcygmc2VsZi51X3ZhbHVlcykKICAgIH0KfQ==", "target": "examples/hubbard_u_sweep_slurm/src/config.rs", "index": 0}]')

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
