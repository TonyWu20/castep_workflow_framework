#!/usr/bin/env python3
"""TASK-5: Fix remaining uninlined_format_args clippy warnings in touched files: hubbard_u_sweep/main.rs lines 19-20 (format!("scf_U{:.1}", u)), task.rs line 138 (format!("{:?}", mode)), and job_script.rs test line 85 (format!("...{}", config.mpi_if)). Also fix 3.14 approx_constant warning in config.rs tests."""
import base64, json, subprocess, sys

TASK_ID = "TASK-5"
STEPS = json.loads('[{"before_b64": "ICAgICAgICBsZXQgdGFza19pZCA9IGZvcm1hdCEoInNjZl9VezouMX0iLCB1KTsKICAgICAgICBsZXQgd29ya2RpciA9IHN0ZDo6cGF0aDo6UGF0aEJ1Zjo6ZnJvbShmb3JtYXQhKCJydW5zL1V7Oi4xfSIsIHUpKTs=", "after_b64": "ICAgICAgICBsZXQgdGFza19pZCA9IGZvcm1hdCEoInNjZl9Ve3U6LjF9Iik7CiAgICAgICAgbGV0IHdvcmtkaXIgPSBzdGQ6OnBhdGg6OlBhdGhCdWY6OmZyb20oZm9ybWF0ISgicnVucy9Ve3U6LjF9IikpOw==", "target": "examples/hubbard_u_sweep/src/main.rs", "index": 0}, {"before_b64": "ICAgICAgICBsZXQgZGJnID0gZm9ybWF0ISgiezo/fSIsIG1vZGUpOw==", "after_b64": "ICAgICAgICBsZXQgZGJnID0gZm9ybWF0ISgie21vZGU6P30iKTs=", "target": "workflow_core/src/task.rs", "index": 1}, {"before_b64": "ICAgICAgICBhc3NlcnQhKHNjcmlwdC5jb250YWlucygmZm9ybWF0ISgiT01QSV9NQ0FfYnRsX3RjcF9pZl9pbmNsdWRlPXt9IiwgY29uZmlnLm1waV9pZikpKTs=", "after_b64": "ICAgICAgICBhc3NlcnQhKHNjcmlwdC5jb250YWlucygmZm9ybWF0ISgiT01QSV9NQ0FfYnRsX3RjcF9pZl9pbmNsdWRlPXttcGlfaWZ9IiwgbXBpX2lmID0gY29uZmlnLm1waV9pZikpKTs=", "target": "examples/hubbard_u_sweep_slurm/src/job_script.rs", "index": 2}, {"before_b64": "ICAgIGZuIHBhcnNlX3NpbmdsZV92YWx1ZSgpIHsKICAgICAgICBsZXQgdmFscyA9IHBhcnNlX3VfdmFsdWVzKCIzLjE0IikudW53cmFwKCk7CiAgICAgICAgYXNzZXJ0X2VxISh2YWxzLCB2ZWMhWzMuMTRdKTs=", "after_b64": "ICAgIGZuIHBhcnNlX3NpbmdsZV92YWx1ZSgpIHsKICAgICAgICBsZXQgdmFscyA9IHBhcnNlX3VfdmFsdWVzKCIyLjcxODI4IikudW53cmFwKCk7CiAgICAgICAgYXNzZXJ0X2VxISh2YWxzLCB2ZWMhWzIuNzE4MjhdKTs=", "target": "examples/hubbard_u_sweep_slurm/src/config.rs", "index": 3}]')

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
