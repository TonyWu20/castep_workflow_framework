#!/usr/bin/env python3
"""TASK-6: Add unit tests for generate_job_script verifying SBATCH directives, seed name substitution, and absence of literal tabs"""
import base64, json, subprocess, sys

TASK_ID = "TASK-6"
STEPS = json.loads('[{"before_b64": "ICAgICAgICB0YXNrX2lkID0gdGFza19pZCwKICAgICAgICBwYXJ0aXRpb24gPSBjb25maWcucGFydGl0aW9uLAogICAgICAgIG50YXNrcyA9IGNvbmZpZy5udGFza3MsCiAgICAgICAgbml4X2ZsYWtlID0gY29uZmlnLm5peF9mbGFrZSwKICAgICAgICBtcGlfaWYgPSBjb25maWcubXBpX2lmLAogICAgKQp9", "after_b64": "ICAgICAgICB0YXNrX2lkID0gdGFza19pZCwKICAgICAgICBwYXJ0aXRpb24gPSBjb25maWcucGFydGl0aW9uLAogICAgICAgIG50YXNrcyA9IGNvbmZpZy5udGFza3MsCiAgICAgICAgbml4X2ZsYWtlID0gY29uZmlnLm5peF9mbGFrZSwKICAgICAgICBtcGlfaWYgPSBjb25maWcubXBpX2lmLAogICAgKQp9CgojW2NmZyh0ZXN0KV0KbW9kIHRlc3RzIHsKICAgIHVzZSBzdXBlcjo6KjsKICAgIHVzZSBjcmF0ZTo6Y29uZmlnOjpTd2VlcENvbmZpZzsKICAgIHVzZSBjbGFwOjpQYXJzZXI7CgogICAgZm4gZGVmYXVsdF9jb25maWcoKSAtPiBTd2VlcENvbmZpZyB7CiAgICAgICAgU3dlZXBDb25maWc6OnBhcnNlX2Zyb20oWyJ0ZXN0Il0pCiAgICB9CgogICAgI1t0ZXN0XQogICAgZm4gY29udGFpbnNfc2JhdGNoX2RpcmVjdGl2ZXMoKSB7CiAgICAgICAgbGV0IGNvbmZpZyA9IGRlZmF1bHRfY29uZmlnKCk7CiAgICAgICAgbGV0IHNjcmlwdCA9IGdlbmVyYXRlX2pvYl9zY3JpcHQoJmNvbmZpZywgInNjZl9VMS4wIiwgIlpuTyIpOwogICAgICAgIGFzc2VydCEoc2NyaXB0LmNvbnRhaW5zKCIjU0JBVENIIC0tam9iLW5hbWU9XCJzY2ZfVTEuMFwiIikpOwogICAgICAgIGFzc2VydCEoc2NyaXB0LmNvbnRhaW5zKCIjU0JBVENIIC0tcGFydGl0aW9uPWRlYnVnIikpOwogICAgICAgIGFzc2VydCEoc2NyaXB0LmNvbnRhaW5zKCIjU0JBVENIIC0tbnRhc2tzLXBlci1ub2RlPTE2IikpOwogICAgICAgIGFzc2VydCEoc2NyaXB0LmNvbnRhaW5zKCIjU0JBVENIIC0tbWVtPTMwMDAwbSIpKTsKICAgIH0KCiAgICAjW3Rlc3RdCiAgICBmbiBjb250YWluc19zZWVkX25hbWUoKSB7CiAgICAgICAgbGV0IGNvbmZpZyA9IGRlZmF1bHRfY29uZmlnKCk7CiAgICAgICAgbGV0IHNjcmlwdCA9IGdlbmVyYXRlX2pvYl9zY3JpcHQoJmNvbmZpZywgInNjZl9VMC4wIiwgIlpuTyIpOwogICAgICAgIGFzc2VydCEoc2NyaXB0LmNvbnRhaW5zKCJjYXN0ZXAubXBpIFpuTyIpKTsKICAgIH0KCiAgICAjW3Rlc3RdCiAgICBmbiBub19saXRlcmFsX3RhYnMoKSB7CiAgICAgICAgbGV0IGNvbmZpZyA9IGRlZmF1bHRfY29uZmlnKCk7CiAgICAgICAgbGV0IHNjcmlwdCA9IGdlbmVyYXRlX2pvYl9zY3JpcHQoJmNvbmZpZywgInNjZl9VMC4wIiwgIlpuTyIpOwogICAgICAgIGFzc2VydCEoIXNjcmlwdC5jb250YWlucygnXHQnKSwgImpvYiBzY3JpcHQgc2hvdWxkIG5vdCBjb250YWluIGxpdGVyYWwgdGFiIGNoYXJhY3RlcnMiKTsKICAgIH0KCiAgICAjW3Rlc3RdCiAgICBmbiBzdGFydHNfd2l0aF9zaGViYW5nKCkgewogICAgICAgIGxldCBjb25maWcgPSBkZWZhdWx0X2NvbmZpZygpOwogICAgICAgIGxldCBzY3JpcHQgPSBnZW5lcmF0ZV9qb2Jfc2NyaXB0KCZjb25maWcsICJzY2ZfVTAuMCIsICJabk8iKTsKICAgICAgICBhc3NlcnQhKHNjcmlwdC5zdGFydHNfd2l0aCgiIyEvdXNyL2Jpbi9lbnYgYmFzaCIpKTsKICAgIH0KCiAgICAjW3Rlc3RdCiAgICBmbiBjb250YWluc19uaXhfZGV2ZWxvcCgpIHsKICAgICAgICBsZXQgY29uZmlnID0gZGVmYXVsdF9jb25maWcoKTsKICAgICAgICBsZXQgc2NyaXB0ID0gZ2VuZXJhdGVfam9iX3NjcmlwdCgmY29uZmlnLCAic2NmX1UwLjAiLCAiWm5PIik7CiAgICAgICAgYXNzZXJ0IShzY3JpcHQuY29udGFpbnMoIm5peCBkZXZlbG9wIikpOwogICAgICAgIGFzc2VydCEoc2NyaXB0LmNvbnRhaW5zKCZjb25maWcubml4X2ZsYWtlKSk7CiAgICB9CgogICAgI1t0ZXN0XQogICAgZm4gY29udGFpbnNfbXBpX2ludGVyZmFjZSgpIHsKICAgICAgICBsZXQgY29uZmlnID0gZGVmYXVsdF9jb25maWcoKTsKICAgICAgICBsZXQgc2NyaXB0ID0gZ2VuZXJhdGVfam9iX3NjcmlwdCgmY29uZmlnLCAic2NmX1UwLjAiLCAiWm5PIik7CiAgICAgICAgYXNzZXJ0IShzY3JpcHQuY29udGFpbnMoJmZvcm1hdCEoIk9NUElfTUNBX2J0bF90Y3BfaWZfaW5jbHVkZT17fSIsIGNvbmZpZy5tcGlfaWYpKSk7CiAgICB9Cn0=", "target": "examples/hubbard_u_sweep_slurm/src/job_script.rs", "index": 0}]')

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
