#!/usr/bin/env python3
"""TASK-2: Add 2 missing parse_u_values test cases specified in plan D.3a: empty string and negative values"""
import base64, json, subprocess, sys

TASK_ID = "TASK-2"
STEPS = json.loads('[{"before_b64": "ICAgICNbdGVzdF0KICAgIGZuIHBhcnNlX2VtcHR5X3Rva2VuKCkgewogICAgICAgIGxldCBlcnIgPSBwYXJzZV91X3ZhbHVlcygiMS4wLCwyLjAiKS51bndyYXBfZXJyKCk7CiAgICAgICAgYXNzZXJ0IShlcnIuY29udGFpbnMoImludmFsaWQiKSwgImVycm9yIHNob3VsZCByZXBvcnQgcGFyc2UgZmFpbHVyZToge2Vycn0iKTsKICAgIH0KfQ==", "after_b64": "ICAgICNbdGVzdF0KICAgIGZuIHBhcnNlX2VtcHR5X3Rva2VuKCkgewogICAgICAgIGxldCBlcnIgPSBwYXJzZV91X3ZhbHVlcygiMS4wLCwyLjAiKS51bndyYXBfZXJyKCk7CiAgICAgICAgYXNzZXJ0IShlcnIuY29udGFpbnMoImludmFsaWQiKSwgImVycm9yIHNob3VsZCByZXBvcnQgcGFyc2UgZmFpbHVyZToge2Vycn0iKTsKICAgIH0KCiAgICAjW3Rlc3RdCiAgICBmbiBwYXJzZV9lbXB0eV9zdHJpbmcoKSB7CiAgICAgICAgLy8gVGhlIHdob2xlIGlucHV0IGlzIGVtcHR5IChkaXN0aW5jdCBmcm9tIGFuIGVtcHR5IHRva2VuIGluIHRoZSBtaWRkbGUpCiAgICAgICAgbGV0IGVyciA9IHBhcnNlX3VfdmFsdWVzKCIiKS51bndyYXBfZXJyKCk7CiAgICAgICAgYXNzZXJ0ISghZXJyLmlzX2VtcHR5KCkpOwogICAgfQoKICAgICNbdGVzdF0KICAgIGZuIHBhcnNlX25lZ2F0aXZlX3ZhbHVlcygpIHsKICAgICAgICBsZXQgdmFscyA9IHBhcnNlX3VfdmFsdWVzKCItMS4wLDIuMCIpLnVud3JhcCgpOwogICAgICAgIGFzc2VydF9lcSEodmFscywgdmVjIVstMS4wLCAyLjBdKTsKICAgIH0KfQ==", "target": "examples/hubbard_u_sweep_slurm/src/config.rs", "index": 0}]')

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
