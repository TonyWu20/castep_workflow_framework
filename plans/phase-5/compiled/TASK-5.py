#!/usr/bin/env python3
"""TASK-5: Fix generate_job_script formatting: replace literal tabs with spaces, use consistent indentation"""
import base64, json, subprocess, sys

TASK_ID = "TASK-5"
STEPS = json.loads('[{"before_b64": "dXNlIGNyYXRlOjpjb25maWc6OlN3ZWVwQ29uZmlnOwoKcHViIGZuIGdlbmVyYXRlX2pvYl9zY3JpcHQoY29uZmlnOiAmU3dlZXBDb25maWcsIHRhc2tfaWQ6ICZzdHIsIHNlZWRfbmFtZTogJnN0cikgLT4gU3RyaW5nIHsKICAgIGZvcm1hdCEoCiAgICAgICAgIiMhL3Vzci9iaW4vZW52IGJhc2hcblwKI1NCQVRDSCAtLWpvYi1uYW1lPVwie3Rhc2tfaWR9XCJcblwKI1NCQVRDSCAtLW91dHB1dD1zbHVybV9vdXRwdXRfJWoudHh0XG5cCiNTQkFUQ0ggLS1wYXJ0aXRpb249e3BhcnRpdGlvbn1cblwKI1NCQVRDSCAtLW5vZGVzPTFcblwKI1NCQVRDSCAtLW50YXNrcy1wZXItbm9kZT17bnRhc2tzfVxuXAojU0JBVENIIC0tY3B1cy1wZXItdGFzaz0xXG5cCiNTQkFUQ0ggLS1tZW09MzAwMDBtXG5cCiNTQkFUQ0ggLS1ub2RlbGlzdD1uaXhvcwpuaXggZGV2ZWxvcCB7bml4X2ZsYWtlfSAtLWNvbW1hbmQgYmFzaCAtYyBcXFxuXAogICAgXCJtcGlydW4gLS1tY2EgcGxtIHNsdXJtIFxcXG5cCiAgICAgICAgLXggT01QSV9NQ0FfYnRsX3RjcF9pZl9pbmNsdWRlPXttcGlfaWZ9IFxcXG5cCiAgICAgICAgLXggT01QSV9NQ0Ffb3J0ZV9rZWVwX2ZxZG5faG9zdG5hbWVzPXRydWUgXFxcblwKICAgICAgIC0tbWNhIHBtaXggczEgXFxcblwKICAgICAgIC0tbWNhIGJ0bCB0Y3Asc2VsZiBcXFxuXApcdC0tbWFwLWJ5IG51bWEgLS1iaW5kLXRvIG51bWEgXFxcblwKICAgIGNhc3RlcC5tcGkge3NlZWRfbmFtZX1cIlxuIiwKICAgICAgICB0YXNrX2lkID0gdGFza19pZCwKICAgICAgICBwYXJ0aXRpb24gPSBjb25maWcucGFydGl0aW9uLAogICAgICAgIG50YXNrcyA9IGNvbmZpZy5udGFza3MsCiAgICAgICAgbml4X2ZsYWtlID0gY29uZmlnLm5peF9mbGFrZSwKICAgICAgICBtcGlfaWYgPSBjb25maWcubXBpX2lmLAogICAgKQp9", "after_b64": "dXNlIGNyYXRlOjpjb25maWc6OlN3ZWVwQ29uZmlnOwoKcHViIGZuIGdlbmVyYXRlX2pvYl9zY3JpcHQoY29uZmlnOiAmU3dlZXBDb25maWcsIHRhc2tfaWQ6ICZzdHIsIHNlZWRfbmFtZTogJnN0cikgLT4gU3RyaW5nIHsKICAgIGZvcm1hdCEoCiAgICAgICAgIlwKIyEvdXNyL2Jpbi9lbnYgYmFzaAojU0JBVENIIC0tam9iLW5hbWU9XCJ7dGFza19pZH1cIgojU0JBVENIIC0tb3V0cHV0PXNsdXJtX291dHB1dF8lai50eHQKI1NCQVRDSCAtLXBhcnRpdGlvbj17cGFydGl0aW9ufQojU0JBVENIIC0tbm9kZXM9MQojU0JBVENIIC0tbnRhc2tzLXBlci1ub2RlPXtudGFza3N9CiNTQkFUQ0ggLS1jcHVzLXBlci10YXNrPTEKI1NCQVRDSCAtLW1lbT0zMDAwMG0KI1NCQVRDSCAtLW5vZGVsaXN0PW5peG9zCm5peCBkZXZlbG9wIHtuaXhfZmxha2V9IC0tY29tbWFuZCBiYXNoIC1jIFxcCiAgICBcIm1waXJ1biAtLW1jYSBwbG0gc2x1cm0gXFwKICAgICAgICAteCBPTVBJX01DQV9idGxfdGNwX2lmX2luY2x1ZGU9e21waV9pZn0gXFwKICAgICAgICAteCBPTVBJX01DQV9vcnRlX2tlZXBfZnFkbl9ob3N0bmFtZXM9dHJ1ZSBcXAogICAgICAgIC0tbWNhIHBtaXggczEgXFwKICAgICAgICAtLW1jYSBidGwgdGNwLHNlbGYgXFwKICAgICAgICAtLW1hcC1ieSBudW1hIC0tYmluZC10byBudW1hIFxcCiAgICBjYXN0ZXAubXBpIHtzZWVkX25hbWV9XCIKIiwKICAgICAgICB0YXNrX2lkID0gdGFza19pZCwKICAgICAgICBwYXJ0aXRpb24gPSBjb25maWcucGFydGl0aW9uLAogICAgICAgIG50YXNrcyA9IGNvbmZpZy5udGFza3MsCiAgICAgICAgbml4X2ZsYWtlID0gY29uZmlnLm5peF9mbGFrZSwKICAgICAgICBtcGlfaWYgPSBjb25maWcubXBpX2lmLAogICAgKQp9", "target": "examples/hubbard_u_sweep_slurm/src/job_script.rs", "index": 0}]')

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
