#!/usr/bin/env python3
"""TASK-2: Change `second` parameter of `build_one_task` and `build_chain` to `Option<&str>`; update all call sites; restore single-mode task IDs to original format"""
import base64, json, subprocess, sys
from pathlib import Path

TASK_ID = "TASK-2"
STEPS = json.loads('[{"before_b64": "Zm4gYnVpbGRfb25lX3Rhc2soCiAgICBjb25maWc6ICZTd2VlcENvbmZpZywKICAgIHU6IGY2NCwKICAgIHNlY29uZDogJnN0ciwKICAgIHNlZWRfY2VsbDogJnN0ciwKICAgIHNlZWRfcGFyYW06ICZzdHIsCikgLT4gUmVzdWx0PFRhc2ssIFdvcmtmbG93RXJyb3I+IHsKICAgIGxldCB0YXNrX2lkID0gZm9ybWF0ISgic2NmX1V7dTouMX1fe3NlY29uZH0iKTsKICAgIGxldCB3b3JrZGlyID0gc3RkOjpwYXRoOjpQYXRoQnVmOjpmcm9tKGZvcm1hdCEoInJ1bnMvVXt1Oi4xfS97c2Vjb25kfSIpKTs=", "after_b64": "Zm4gYnVpbGRfb25lX3Rhc2soCiAgICBjb25maWc6ICZTd2VlcENvbmZpZywKICAgIHU6IGY2NCwKICAgIHNlY29uZDogT3B0aW9uPCZzdHI+LAogICAgc2VlZF9jZWxsOiAmc3RyLAogICAgc2VlZF9wYXJhbTogJnN0ciwKKSAtPiBSZXN1bHQ8VGFzaywgV29ya2Zsb3dFcnJvcj4gewogICAgbGV0IHRhc2tfaWQgPSBtYXRjaCBzZWNvbmQgewogICAgICAgIFNvbWUocykgPT4gZm9ybWF0ISgic2NmX1V7dTouMX1fe3N9IiksCiAgICAgICAgTm9uZSA9PiBmb3JtYXQhKCJzY2ZfVXt1Oi4xfSIpLAogICAgfTsKICAgIGxldCB3b3JrZGlyID0gbWF0Y2ggc2Vjb25kIHsKICAgICAgICBTb21lKHMpID0+IHN0ZDo6cGF0aDo6UGF0aEJ1Zjo6ZnJvbShmb3JtYXQhKCJydW5zL1V7dTouMX0ve3N9IikpLAogICAgICAgIE5vbmUgPT4gc3RkOjpwYXRoOjpQYXRoQnVmOjpmcm9tKGZvcm1hdCEoInJ1bnMvVXt1Oi4xfSIpKSwKICAgIH07", "target": "examples/hubbard_u_sweep_slurm/src/main.rs", "index": 0, "is_create": false}, {"before_b64": "Zm4gYnVpbGRfY2hhaW4oCiAgICBjb25maWc6ICZTd2VlcENvbmZpZywKICAgIHU6IGY2NCwKICAgIHNlY29uZDogJnN0ciwKICAgIHNlZWRfY2VsbDogJnN0ciwKICAgIHNlZWRfcGFyYW06ICZzdHIsCikgLT4gUmVzdWx0PFZlYzxUYXNrPiwgV29ya2Zsb3dFcnJvcj4gewogICAgbGV0IHNjZiA9IGJ1aWxkX29uZV90YXNrKGNvbmZpZywgdSwgc2Vjb25kLCBzZWVkX2NlbGwsIHNlZWRfcGFyYW0pPzsKICAgIC8vIERPUyB0YXNrIGRlcGVuZHMgb24gU0NGIGNvbXBsZXRpbmcgc3VjY2Vzc2Z1bGx5CiAgICBsZXQgZG9zX2lkID0gZm9ybWF0ISgiZG9zX3tzZWNvbmR9Iik7CiAgICBsZXQgZG9zX3dvcmtkaXIgPSBzdGQ6OnBhdGg6OlBhdGhCdWY6OmZyb20oZm9ybWF0ISgicnVucy9Ve3U6LjF9L3tzZWNvbmR9L2RvcyIpKTs=", "after_b64": "Zm4gYnVpbGRfY2hhaW4oCiAgICBjb25maWc6ICZTd2VlcENvbmZpZywKICAgIHU6IGY2NCwKICAgIHNlY29uZDogT3B0aW9uPCZzdHI+LAogICAgc2VlZF9jZWxsOiAmc3RyLAogICAgc2VlZF9wYXJhbTogJnN0ciwKKSAtPiBSZXN1bHQ8VmVjPFRhc2s+LCBXb3JrZmxvd0Vycm9yPiB7CiAgICBsZXQgc2NmID0gYnVpbGRfb25lX3Rhc2soY29uZmlnLCB1LCBzZWNvbmQsIHNlZWRfY2VsbCwgc2VlZF9wYXJhbSk/OwogICAgLy8gRE9TIHRhc2sgZGVwZW5kcyBvbiBTQ0YgY29tcGxldGluZyBzdWNjZXNzZnVsbHkKICAgIGxldCBkb3NfaWQgPSBtYXRjaCBzZWNvbmQgewogICAgICAgIFNvbWUocykgPT4gZm9ybWF0ISgiZG9zX3tzfSIpLAogICAgICAgIE5vbmUgPT4gImRvcyIudG9fc3RyaW5nKCksCiAgICB9OwogICAgbGV0IGRvc193b3JrZGlyID0gbWF0Y2ggc2Vjb25kIHsKICAgICAgICBTb21lKHMpID0+IHN0ZDo6cGF0aDo6UGF0aEJ1Zjo6ZnJvbShmb3JtYXQhKCJydW5zL1V7dTouMX0ve3N9L2RvcyIpKSwKICAgICAgICBOb25lID0+IHN0ZDo6cGF0aDo6UGF0aEJ1Zjo6ZnJvbShmb3JtYXQhKCJydW5zL1V7dTouMX0vZG9zIikpLAogICAgfTs=", "target": "examples/hubbard_u_sweep_slurm/src/main.rs", "index": 1, "is_create": false}, {"before_b64": "ICAgICAgICAgICAgICAgIHRhc2tzLmV4dGVuZChidWlsZF9jaGFpbihjb25maWcsIHUsICZzZWNvbmQsIHNlZWRfY2VsbCwgc2VlZF9wYXJhbSk/KTs=", "after_b64": "ICAgICAgICAgICAgICAgIHRhc2tzLmV4dGVuZChidWlsZF9jaGFpbihjb25maWcsIHUsIFNvbWUoJnNlY29uZCksIHNlZWRfY2VsbCwgc2VlZF9wYXJhbSk/KTs=", "target": "examples/hubbard_u_sweep_slurm/src/main.rs", "index": 2, "is_create": false}, {"before_b64": "ICAgICAgICAgICAgICAgIHRhc2tzLmV4dGVuZChidWlsZF9jaGFpbihjb25maWcsICp1LCBzZWNvbmQsIHNlZWRfY2VsbCwgc2VlZF9wYXJhbSk/KTs=", "after_b64": "ICAgICAgICAgICAgICAgIHRhc2tzLmV4dGVuZChidWlsZF9jaGFpbihjb25maWcsICp1LCBTb21lKHNlY29uZCksIHNlZWRfY2VsbCwgc2VlZF9wYXJhbSk/KTs=", "target": "examples/hubbard_u_sweep_slurm/src/main.rs", "index": 3, "is_create": false}, {"before_b64": "ICAgICAgICAgICAgICAgIC5tYXAofHV8IGJ1aWxkX29uZV90YXNrKGNvbmZpZywgdSwgImRlZmF1bHQiLCBzZWVkX2NlbGwsIHNlZWRfcGFyYW0pLm1hcF9lcnIoSW50bzo6aW50bykp", "after_b64": "ICAgICAgICAgICAgICAgIC5tYXAofHV8IGJ1aWxkX29uZV90YXNrKGNvbmZpZywgdSwgTm9uZSwgc2VlZF9jZWxsLCBzZWVkX3BhcmFtKS5tYXBfZXJyKEludG86OmludG8pKQ==", "target": "examples/hubbard_u_sweep_slurm/src/main.rs", "index": 4, "is_create": false}]')

for step in STEPS:
    before = base64.b64decode(step["before_b64"]).decode()
    after = base64.b64decode(step["after_b64"]).decode()
    target = step["target"]
    idx = step["index"]
    is_create = step["is_create"]

    if is_create:
        target_path = Path(target)
        target_path.parent.mkdir(parents=True, exist_ok=True)
        target_path.write_text(after)
        print(f"OK {TASK_ID} change {idx}: created {target}")
    else:
        target_path = Path(target)
        content = target_path.read_text()
        if before not in content:
            print(f"FAILED {TASK_ID} change {idx}: pattern not found in {target}", file=sys.stderr)
            print(f"Expected (first 200 chars): {repr(before[:200])}", file=sys.stderr)
            sys.exit(1)

        result = subprocess.run(
            ["sd", "-F", "-A", "-n", "1", "--", before, after, target],
            capture_output=True, text=True,
        )
        if result.returncode != 0:
            print(f"FAILED {TASK_ID} change {idx}: sd error: {result.stderr}", file=sys.stderr)
            sys.exit(result.returncode)

        new_content = target_path.read_text()
        if after and after not in new_content:
            print(f"FAILED {TASK_ID} change {idx}: replacement not found after apply", file=sys.stderr)
            sys.exit(1)

        print(f"OK {TASK_ID} change {idx}: applied to {target}")

print(f"OK {TASK_ID}: all changes applied")
