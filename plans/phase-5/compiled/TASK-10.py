#!/usr/bin/env python3
"""TASK-10: Fix uninlined_format_args pedantic clippy warnings in touched files only. Run clippy per crate, inline format arguments (e.g. format!("{}", x) -> format!("{x}")), skip needless_pass_by_value on Workflow::run(). Scope: config.rs, job_script.rs, main.rs in hubbard_u_sweep_slurm; main.rs in hubbard_u_sweep; state.rs, task.rs in workflow_core."""
import base64, json, subprocess, sys

TASK_ID = "TASK-10"
STEPS = json.loads('[{"before_b64": "ICAgIGxldCB0YXNrX2lkID0gZm9ybWF0ISgic2NmX1V7Oi4xfSIsIHUpOwogICAgbGV0IHdvcmtkaXIgPSBzdGQ6OnBhdGg6OlBhdGhCdWY6OmZyb20oZm9ybWF0ISgicnVucy9VezouMX0iLCB1KSk7", "after_b64": "ICAgIGxldCB0YXNrX2lkID0gZm9ybWF0ISgic2NmX1V7dTouMX0iKTsKICAgIGxldCB3b3JrZGlyID0gc3RkOjpwYXRoOjpQYXRoQnVmOjpmcm9tKGZvcm1hdCEoInJ1bnMvVXt1Oi4xfSIpKTs=", "target": "examples/hubbard_u_sweep_slurm/src/main.rs", "index": 0}, {"before_b64": "ICAgICAgICAgICAgICAgICAgICByZXR1cm4gRXJyKFdvcmtmbG93RXJyb3I6OkludmFsaWRDb25maWcoZm9ybWF0ISgKICAgICAgICAgICAgICAgICAgICAgICAgInVuc3VwcG9ydGVkIG9yYml0YWwgJ3t9JyIsCiAgICAgICAgICAgICAgICAgICAgICAgIGMKICAgICAgICAgICAgICAgICAgICApKSk=", "after_b64": "ICAgICAgICAgICAgICAgICAgICByZXR1cm4gRXJyKFdvcmtmbG93RXJyb3I6OkludmFsaWRDb25maWcoZm9ybWF0ISgKICAgICAgICAgICAgICAgICAgICAgICAgInVuc3VwcG9ydGVkIG9yYml0YWwgJ3tjfSciCiAgICAgICAgICAgICAgICAgICAgKSkp", "target": "examples/hubbard_u_sweep_slurm/src/main.rs", "index": 1}, {"before_b64": "ICAgICAgICAgICAgbGV0IGNhc3RlcF9vdXQgPSB3b3JrZGlyLmpvaW4oZm9ybWF0ISgie30uY2FzdGVwIiwgc2VlZF9uYW1lX2NvbGxlY3QpKTsKICAgICAgICAgICAgaWYgIWNhc3RlcF9vdXQuZXhpc3RzKCkgewogICAgICAgICAgICAgICAgcmV0dXJuIEVycihXb3JrZmxvd0Vycm9yOjpJbnZhbGlkQ29uZmlnKGZvcm1hdCEoCiAgICAgICAgICAgICAgICAgICAgIm1pc3Npbmcgb3V0cHV0OiB7fSIsCiAgICAgICAgICAgICAgICAgICAgY2FzdGVwX291dC5kaXNwbGF5KCkKICAgICAgICAgICAgICAgICkpKTs=", "after_b64": "ICAgICAgICAgICAgbGV0IGNhc3RlcF9vdXQgPSB3b3JrZGlyLmpvaW4oZm9ybWF0ISgie3NlZWRfbmFtZV9jb2xsZWN0fS5jYXN0ZXAiKSk7CiAgICAgICAgICAgIGlmICFjYXN0ZXBfb3V0LmV4aXN0cygpIHsKICAgICAgICAgICAgICAgIHJldHVybiBFcnIoV29ya2Zsb3dFcnJvcjo6SW52YWxpZENvbmZpZyhmb3JtYXQhKAogICAgICAgICAgICAgICAgICAgICJtaXNzaW5nIG91dHB1dDoge30iLAogICAgICAgICAgICAgICAgICAgIGNhc3RlcF9vdXQuZGlzcGxheSgpCiAgICAgICAgICAgICAgICApKSk7", "target": "examples/hubbard_u_sweep_slurm/src/main.rs", "index": 2}]')

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
