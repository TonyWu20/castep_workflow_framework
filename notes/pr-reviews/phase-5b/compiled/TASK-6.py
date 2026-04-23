#!/usr/bin/env python3
"""TASK-6: Branch on config.local at the run site: local mode uses run_default(&mut workflow, &mut state), SLURM mode keeps manual Arc wiring."""
import base64, json, subprocess, sys

TASK_ID = "TASK-6"
STEPS = json.loads('[{"before_b64": "ICAgIGxldCBzdGF0ZV9wYXRoID0gc3RkOjpwYXRoOjpQYXRoQnVmOjpmcm9tKCIuaHViYmFyZF91X3N3ZWVwX3NsdXJtLndvcmtmbG93Lmpzb24iKTsKICAgIGxldCBtdXQgc3RhdGUgPSBKc29uU3RhdGVTdG9yZTo6bmV3KCJodWJiYXJkX3Vfc3dlZXBfc2x1cm0iLCBzdGF0ZV9wYXRoKTsKICAgIGxldCBydW5uZXI6IEFyYzxkeW4gUHJvY2Vzc1J1bm5lcj4gPSBBcmM6Om5ldyhTeXN0ZW1Qcm9jZXNzUnVubmVyOjpuZXcoKSk7CiAgICBsZXQgZXhlY3V0b3I6IEFyYzxkeW4gSG9va0V4ZWN1dG9yPiA9IEFyYzo6bmV3KFNoZWxsSG9va0V4ZWN1dG9yKTsKCiAgICBsZXQgc3VtbWFyeSA9IHdvcmtmbG93LnJ1bigmbXV0IHN0YXRlLCBydW5uZXIsIGV4ZWN1dG9yKT87CiAgICBwcmludGxuISgKICAgICAgICAiV29ya2Zsb3cgY29tcGxldGU6IHt9IHN1Y2NlZWRlZCwge30gZmFpbGVkLCB7fSBza2lwcGVkICh7Oi4xfXMpIiwKICAgICAgICBzdW1tYXJ5LnN1Y2NlZWRlZC5sZW4oKSwKICAgICAgICBzdW1tYXJ5LmZhaWxlZC5sZW4oKSwKICAgICAgICBzdW1tYXJ5LnNraXBwZWQubGVuKCksCiAgICAgICAgc3VtbWFyeS5kdXJhdGlvbi5hc19zZWNzX2Y2NCgpLAogICAgKTsKICAgIE9rKCgpKQ==", "after_b64": "ICAgIGxldCBzdGF0ZV9wYXRoID0gc3RkOjpwYXRoOjpQYXRoQnVmOjpmcm9tKCIuaHViYmFyZF91X3N3ZWVwX3NsdXJtLndvcmtmbG93Lmpzb24iKTsKICAgIGxldCBtdXQgc3RhdGUgPSBKc29uU3RhdGVTdG9yZTo6bmV3KCJodWJiYXJkX3Vfc3dlZXBfc2x1cm0iLCBzdGF0ZV9wYXRoKTsKCiAgICBsZXQgc3VtbWFyeSA9IGlmIGNvbmZpZy5sb2NhbCB7CiAgICAgICAgcnVuX2RlZmF1bHQoJm11dCB3b3JrZmxvdywgJm11dCBzdGF0ZSk/CiAgICB9IGVsc2UgewogICAgICAgIGxldCBydW5uZXI6IEFyYzxkeW4gUHJvY2Vzc1J1bm5lcj4gPSBBcmM6Om5ldyhTeXN0ZW1Qcm9jZXNzUnVubmVyOjpuZXcoKSk7CiAgICAgICAgbGV0IGV4ZWN1dG9yOiBBcmM8ZHluIEhvb2tFeGVjdXRvcj4gPSBBcmM6Om5ldyhTaGVsbEhvb2tFeGVjdXRvcik7CiAgICAgICAgd29ya2Zsb3cucnVuKCZtdXQgc3RhdGUsIHJ1bm5lciwgZXhlY3V0b3IpPwogICAgfTsKCiAgICBwcmludGxuISgKICAgICAgICAiV29ya2Zsb3cgY29tcGxldGU6IHt9IHN1Y2NlZWRlZCwge30gZmFpbGVkLCB7fSBza2lwcGVkICh7Oi4xfXMpIiwKICAgICAgICBzdW1tYXJ5LnN1Y2NlZWRlZC5sZW4oKSwKICAgICAgICBzdW1tYXJ5LmZhaWxlZC5sZW4oKSwKICAgICAgICBzdW1tYXJ5LnNraXBwZWQubGVuKCksCiAgICAgICAgc3VtbWFyeS5kdXJhdGlvbi5hc19zZWNzX2Y2NCgpLAogICAgKTsKICAgIE9rKCgpKQ==", "target": "examples/hubbard_u_sweep_slurm/src/main.rs", "index": 0}]')

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
