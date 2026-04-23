#!/usr/bin/env python3
"""TASK-4: Add unit tests for the free function parse_u_values covering happy path, whitespace, empty token, and invalid input"""
import base64, json, subprocess, sys

TASK_ID = "TASK-4"
STEPS = json.loads('[{"before_b64": "ICAgIHB1YiBmbiBwYXJzZV91X3ZhbHVlcygmc2VsZikgLT4gUmVzdWx0PFZlYzxmNjQ+LCBTdHJpbmc+IHsKICAgICAgICBwYXJzZV91X3ZhbHVlcygmc2VsZi51X3ZhbHVlcykKICAgIH0KfQ==", "after_b64": "ICAgIHB1YiBmbiBwYXJzZV91X3ZhbHVlcygmc2VsZikgLT4gUmVzdWx0PFZlYzxmNjQ+LCBTdHJpbmc+IHsKICAgICAgICBwYXJzZV91X3ZhbHVlcygmc2VsZi51X3ZhbHVlcykKICAgIH0KfQoKI1tjZmcodGVzdCldCm1vZCB0ZXN0cyB7CiAgICB1c2Ugc3VwZXI6Oio7CgogICAgI1t0ZXN0XQogICAgZm4gcGFyc2VfYmFzaWNfdmFsdWVzKCkgewogICAgICAgIGxldCB2YWxzID0gcGFyc2VfdV92YWx1ZXMoIjAuMCwxLjAsMi4wIikudW53cmFwKCk7CiAgICAgICAgYXNzZXJ0X2VxISh2YWxzLCB2ZWMhWzAuMCwgMS4wLCAyLjBdKTsKICAgIH0KCiAgICAjW3Rlc3RdCiAgICBmbiBwYXJzZV93aXRoX3doaXRlc3BhY2UoKSB7CiAgICAgICAgbGV0IHZhbHMgPSBwYXJzZV91X3ZhbHVlcygiICAwLjAgLCAxLjAgLCAyLjAgICIpLnVud3JhcCgpOwogICAgICAgIGFzc2VydF9lcSEodmFscywgdmVjIVswLjAsIDEuMCwgMi4wXSk7CiAgICB9CgogICAgI1t0ZXN0XQogICAgZm4gcGFyc2Vfc2luZ2xlX3ZhbHVlKCkgewogICAgICAgIGxldCB2YWxzID0gcGFyc2VfdV92YWx1ZXMoIjMuMTQiKS51bndyYXAoKTsKICAgICAgICBhc3NlcnRfZXEhKHZhbHMsIHZlYyFbMy4xNF0pOwogICAgfQoKICAgICNbdGVzdF0KICAgIGZuIHBhcnNlX2ludmFsaWRfdG9rZW4oKSB7CiAgICAgICAgbGV0IGVyciA9IHBhcnNlX3VfdmFsdWVzKCIxLjAsYWJjLDIuMCIpLnVud3JhcF9lcnIoKTsKICAgICAgICBhc3NlcnQhKGVyci5jb250YWlucygiYWJjIiksICJlcnJvciBzaG91bGQgbWVudGlvbiB0aGUgaW52YWxpZCB0b2tlbjoge30iLCBlcnIpOwogICAgfQoKICAgICNbdGVzdF0KICAgIGZuIHBhcnNlX2VtcHR5X3Rva2VuKCkgewogICAgICAgIGxldCBlcnIgPSBwYXJzZV91X3ZhbHVlcygiMS4wLCwyLjAiKS51bndyYXBfZXJyKCk7CiAgICAgICAgYXNzZXJ0IShlcnIuY29udGFpbnMoImludmFsaWQiKSwgImVycm9yIHNob3VsZCByZXBvcnQgcGFyc2UgZmFpbHVyZToge30iLCBlcnIpOwogICAgfQp9", "target": "examples/hubbard_u_sweep_slurm/src/config.rs", "index": 0}]')

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
