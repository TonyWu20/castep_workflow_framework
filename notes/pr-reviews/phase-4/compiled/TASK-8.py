#!/usr/bin/env python3
"""TASK-8: Set deprecated TASK_STATE env var alongside TASK_PHASE for backwards compatibility with existing hook scripts"""
import base64, json, subprocess, sys

TASK_ID = "TASK-8"
STEPS = json.loads('[{"before_b64": "ICAgICAgICAgICAgLmVudigiVEFTS19JRCIsICZjdHgudGFza19pZCkKICAgICAgICAgICAgLmVudigiVEFTS19QSEFTRSIsIGN0eC5waGFzZS50b19zdHJpbmcoKS5hc19zdHIoKSkKICAgICAgICAgICAgLmVudigiV09SS0RJUiIsIGN0eC53b3JrZGlyLnRvX3N0cmluZ19sb3NzeSgpLmFzX3JlZigpKQ==", "after_b64": "ICAgICAgICAgICAgLmVudigiVEFTS19JRCIsICZjdHgudGFza19pZCkKICAgICAgICAgICAgLmVudigiVEFTS19QSEFTRSIsIGN0eC5waGFzZS50b19zdHJpbmcoKS5hc19zdHIoKSkKICAgICAgICAgICAgLy8gRGVwcmVjYXRlZDogVEFTS19TVEFURSBpcyB0aGUgb2xkIG5hbWUgZm9yIFRBU0tfUEhBU0UuCiAgICAgICAgICAgIC8vIEtlcHQgZm9yIGJhY2t3YXJkcyBjb21wYXRpYmlsaXR5IHdpdGggZXhpc3RpbmcgaG9vayBzY3JpcHRzLgogICAgICAgICAgICAuZW52KCJUQVNLX1NUQVRFIiwgY3R4LnBoYXNlLnRvX3N0cmluZygpLmFzX3N0cigpKQogICAgICAgICAgICAuZW52KCJXT1JLRElSIiwgY3R4LndvcmtkaXIudG9fc3RyaW5nX2xvc3N5KCkuYXNfcmVmKCkp", "target": "workflow_utils/src/monitoring.rs", "index": 0}]')

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
