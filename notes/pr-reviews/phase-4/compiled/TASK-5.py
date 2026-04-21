#!/usr/bin/env python3
"""TASK-5: Add two tests in workflow_core/src/state.rs: set_task_graph round-trip through save/load, and backwards-compatible deserialization of old state files without task_successors"""
import base64, json, subprocess, sys

TASK_ID = "TASK-5"
STEPS = json.loads('[{"before_b64": "ICAgICNbdGVzdF0KICAgIGZuIHdvcmtmbG93X25hbWVfcHJlc2VydmVkKCkgewogICAgICAgIGxldCBkaXIgPSB0ZW1wZGlyKCkudW53cmFwKCk7CiAgICAgICAgbGV0IHBhdGggPSBkaXIucGF0aCgpLmpvaW4oIm5hbWUuanNvbiIpOwogICAgICAgIGxldCBzID0gSnNvblN0YXRlU3RvcmU6Om5ldygibXlfd29ya2Zsb3ciLCBwYXRoLmNsb25lKCkpOwogICAgICAgIHMuc2F2ZSgpLnVud3JhcCgpOwogICAgICAgIGxldCBsb2FkZWQgPSBKc29uU3RhdGVTdG9yZTo6bG9hZCgmcGF0aCkudW53cmFwKCk7CiAgICAgICAgYXNzZXJ0X2VxIShsb2FkZWQud29ya2Zsb3dfbmFtZSgpLCAibXlfd29ya2Zsb3ciKTsKICAgIH0KfQ==", "after_b64": "ICAgICNbdGVzdF0KICAgIGZuIHdvcmtmbG93X25hbWVfcHJlc2VydmVkKCkgewogICAgICAgIGxldCBkaXIgPSB0ZW1wZGlyKCkudW53cmFwKCk7CiAgICAgICAgbGV0IHBhdGggPSBkaXIucGF0aCgpLmpvaW4oIm5hbWUuanNvbiIpOwogICAgICAgIGxldCBzID0gSnNvblN0YXRlU3RvcmU6Om5ldygibXlfd29ya2Zsb3ciLCBwYXRoLmNsb25lKCkpOwogICAgICAgIHMuc2F2ZSgpLnVud3JhcCgpOwogICAgICAgIGxldCBsb2FkZWQgPSBKc29uU3RhdGVTdG9yZTo6bG9hZCgmcGF0aCkudW53cmFwKCk7CiAgICAgICAgYXNzZXJ0X2VxIShsb2FkZWQud29ya2Zsb3dfbmFtZSgpLCAibXlfd29ya2Zsb3ciKTsKICAgIH0KCiAgICAjW3Rlc3RdCiAgICBmbiBzZXRfdGFza19ncmFwaF9wZXJzaXN0c190aHJvdWdoX3NhdmVfbG9hZCgpIHsKICAgICAgICBsZXQgZGlyID0gdGVtcGRpcigpLnVud3JhcCgpOwogICAgICAgIGxldCBwYXRoID0gZGlyLnBhdGgoKS5qb2luKCJncmFwaC5qc29uIik7CiAgICAgICAgbGV0IG11dCBzID0gSnNvblN0YXRlU3RvcmU6Om5ldygiZ3JhcGhfdGVzdCIsIHBhdGguY2xvbmUoKSk7CiAgICAgICAgbGV0IG11dCBtYXAgPSBIYXNoTWFwOjpuZXcoKTsKICAgICAgICBtYXAuaW5zZXJ0KCJhIi50b19zdHJpbmcoKSwgdmVjIVsiYiIudG9fc3RyaW5nKCksICJjIi50b19zdHJpbmcoKV0pOwogICAgICAgIG1hcC5pbnNlcnQoImIiLnRvX3N0cmluZygpLCB2ZWMhWyJkIi50b19zdHJpbmcoKV0pOwogICAgICAgIHMuc2V0X3Rhc2tfZ3JhcGgoVGFza1N1Y2Nlc3NvcnM6Om5ldyhtYXApKTsKICAgICAgICBzLnNhdmUoKS51bndyYXAoKTsKCiAgICAgICAgbGV0IGxvYWRlZCA9IEpzb25TdGF0ZVN0b3JlOjpsb2FkKCZwYXRoKS51bndyYXAoKTsKICAgICAgICBsZXQgc3VjYyA9IGxvYWRlZC50YXNrX3N1Y2Nlc3NvcnMoKS5leHBlY3QoInRhc2tfc3VjY2Vzc29ycyBzaG91bGQgYmUgU29tZSBhZnRlciByb3VuZC10cmlwIik7CiAgICAgICAgbGV0IGFfc3VjY3MgPSBzdWNjLmdldCgiYSIpLnVud3JhcCgpOwogICAgICAgIGFzc2VydCEoYV9zdWNjcy5jb250YWlucygmImIiLnRvX3N0cmluZygpKSk7CiAgICAgICAgYXNzZXJ0IShhX3N1Y2NzLmNvbnRhaW5zKCYiYyIudG9fc3RyaW5nKCkpKTsKICAgICAgICBhc3NlcnRfZXEhKHN1Y2MuZ2V0KCJiIikudW53cmFwKCksICZbImQiLnRvX3N0cmluZygpXSk7CiAgICB9CgogICAgI1t0ZXN0XQogICAgZm4gb2xkX3N0YXRlX2ZpbGVfZGVzZXJpYWxpemVzX3dpdGhvdXRfdGFza19zdWNjZXNzb3JzKCkgewogICAgICAgIGxldCBkaXIgPSB0ZW1wZGlyKCkudW53cmFwKCk7CiAgICAgICAgbGV0IHBhdGggPSBkaXIucGF0aCgpLmpvaW4oIm9sZF9zdGF0ZS5qc29uIik7CiAgICAgICAgbGV0IGpzb24gPSBzZXJkZV9qc29uOjpqc29uISh7CiAgICAgICAgICAgICJ3b3JrZmxvd19uYW1lIjogIm9sZF93ZiIsCiAgICAgICAgICAgICJjcmVhdGVkX2F0IjogIjIwMjQtMDEtMDFUMDA6MDA6MDBaIiwKICAgICAgICAgICAgImxhc3RfdXBkYXRlZCI6ICIyMDI0LTAxLTAxVDAwOjAwOjAwWiIsCiAgICAgICAgICAgICJ0YXNrcyI6IHt9LAogICAgICAgICAgICAicGF0aCI6IHBhdGgudG9fc3RyaW5nX2xvc3N5KCkKICAgICAgICB9KQogICAgICAgIC50b19zdHJpbmcoKTsKICAgICAgICBzdGQ6OmZzOjp3cml0ZSgmcGF0aCwganNvbikudW53cmFwKCk7CgogICAgICAgIGxldCBsb2FkZWQgPSBKc29uU3RhdGVTdG9yZTo6bG9hZCgmcGF0aCkudW53cmFwKCk7CiAgICAgICAgYXNzZXJ0ISgKICAgICAgICAgICAgbG9hZGVkLnRhc2tfc3VjY2Vzc29ycygpLmlzX25vbmUoKSwKICAgICAgICAgICAgIm9sZCBzdGF0ZSBmaWxlcyB3aXRob3V0IHRhc2tfc3VjY2Vzc29ycyBrZXkgc2hvdWxkIGRlc2VyaWFsaXplIGFzIE5vbmUiCiAgICAgICAgKTsKICAgIH0KfQ==", "target": "workflow_core/src/state.rs", "index": 0}]')

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
