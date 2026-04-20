#!/usr/bin/env python3
"""TASK-5: Add unit tests for parse_job_id (SLURM and PBS)"""
import base64, json, subprocess, sys

TASK_ID = "TASK-5"
STEPS = json.loads('[{"before_b64": "ICAgIH0KfQ==", "after_b64": "ICAgIH0KfQoKI1tjZmcodGVzdCldCm1vZCB0ZXN0cyB7CiAgICB1c2Ugc3VwZXI6Oio7CgogICAgI1t0ZXN0XQogICAgZm4gcGFyc2Vfc2x1cm1fam9iX2lkX2Zyb21fc3VibWl0X291dHB1dCgpIHsKICAgICAgICBsZXQgcnVubmVyID0gUXVldWVkUnVubmVyOjpuZXcoU2NoZWR1bGVyS2luZDo6U2x1cm0pOwogICAgICAgIGxldCBpZCA9IHJ1bm5lci5wYXJzZV9qb2JfaWQoIlN1Ym1pdHRlZCBiYXRjaCBqb2IgMTIzNDUiKS51bndyYXAoKTsKICAgICAgICBhc3NlcnRfZXEhKGlkLCAiMTIzNDUiKTsKICAgIH0KCiAgICAjW3Rlc3RdCiAgICBmbiBwYXJzZV9zbHVybV9qb2JfaWRfc2luZ2xlX3dvcmQoKSB7CiAgICAgICAgbGV0IHJ1bm5lciA9IFF1ZXVlZFJ1bm5lcjo6bmV3KFNjaGVkdWxlcktpbmQ6OlNsdXJtKTsKICAgICAgICBsZXQgaWQgPSBydW5uZXIucGFyc2Vfam9iX2lkKCI5OTk5OSIpLnVud3JhcCgpOwogICAgICAgIGFzc2VydF9lcSEoaWQsICI5OTk5OSIpOwogICAgfQoKICAgICNbdGVzdF0KICAgIGZuIHBhcnNlX3NsdXJtX2pvYl9pZF9lbXB0eV9mYWlscygpIHsKICAgICAgICBsZXQgcnVubmVyID0gUXVldWVkUnVubmVyOjpuZXcoU2NoZWR1bGVyS2luZDo6U2x1cm0pOwogICAgICAgIGFzc2VydCEocnVubmVyLnBhcnNlX2pvYl9pZCgiIikuaXNfZXJyKCkpOwogICAgfQoKICAgICNbdGVzdF0KICAgIGZuIHBhcnNlX3Bic19qb2JfaWRfdHlwaWNhbCgpIHsKICAgICAgICBsZXQgcnVubmVyID0gUXVldWVkUnVubmVyOjpuZXcoU2NoZWR1bGVyS2luZDo6UGJzKTsKICAgICAgICBsZXQgaWQgPSBydW5uZXIucGFyc2Vfam9iX2lkKCIxMjM0LnBicy1zZXJ2ZXJcbiIpLnVud3JhcCgpOwogICAgICAgIGFzc2VydF9lcSEoaWQsICIxMjM0LnBicy1zZXJ2ZXIiKTsKICAgIH0KCiAgICAjW3Rlc3RdCiAgICBmbiBwYXJzZV9wYnNfam9iX2lkX2VtcHR5X2ZhaWxzKCkgewogICAgICAgIGxldCBydW5uZXIgPSBRdWV1ZWRSdW5uZXI6Om5ldyhTY2hlZHVsZXJLaW5kOjpQYnMpOwogICAgICAgIGFzc2VydCEocnVubmVyLnBhcnNlX2pvYl9pZCgiIikuaXNfZXJyKCkpOwogICAgfQoKICAgICNbdGVzdF0KICAgIGZuIHBhcnNlX3Bic19qb2JfaWRfd2hpdGVzcGFjZV9vbmx5X2ZhaWxzKCkgewogICAgICAgIGxldCBydW5uZXIgPSBRdWV1ZWRSdW5uZXI6Om5ldyhTY2hlZHVsZXJLaW5kOjpQYnMpOwogICAgICAgIGFzc2VydCEocnVubmVyLnBhcnNlX2pvYl9pZCgiICAgXG4gICIpLmlzX2VycigpKTsKICAgIH0KfQ==", "target": "workflow_utils/src/queued.rs", "index": 0}]')

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
