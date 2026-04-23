#!/usr/bin/env python3
"""TASK-1: Use JOB_SCRIPT_NAME constant instead of hardcoded 'job.sh' in hubbard_u_sweep_slurm consumer"""
import base64, json, subprocess, sys

TASK_ID = "TASK-1"
STEPS = json.loads('[{"before_b64": "dXNlIHdvcmtmbG93X3V0aWxzOjp7CiAgICBjcmVhdGVfZGlyLCByZWFkX2ZpbGUsIHdyaXRlX2ZpbGUsIFF1ZXVlZFJ1bm5lciwgU2NoZWR1bGVyS2luZCwgU2hlbGxIb29rRXhlY3V0b3IsCiAgICBTeXN0ZW1Qcm9jZXNzUnVubmVyLAp9Ow==", "after_b64": "dXNlIHdvcmtmbG93X3V0aWxzOjp7CiAgICBjcmVhdGVfZGlyLCByZWFkX2ZpbGUsIHdyaXRlX2ZpbGUsIFF1ZXVlZFJ1bm5lciwgU2NoZWR1bGVyS2luZCwgU2hlbGxIb29rRXhlY3V0b3IsCiAgICBTeXN0ZW1Qcm9jZXNzUnVubmVyLCBKT0JfU0NSSVBUX05BTUUsCn07", "target": "examples/hubbard_u_sweep_slurm/src/main.rs", "index": 0}, {"before_b64": "ICAgICAgICAgICAgICAgIHdyaXRlX2ZpbGUod29ya2Rpci5qb2luKCJqb2Iuc2giKSwgJmpvYl9zY3JpcHQpPzs=", "after_b64": "ICAgICAgICAgICAgICAgIHdyaXRlX2ZpbGUod29ya2Rpci5qb2luKEpPQl9TQ1JJUFRfTkFNRSksICZqb2Jfc2NyaXB0KT87", "target": "examples/hubbard_u_sweep_slurm/src/main.rs", "index": 1}]')

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
