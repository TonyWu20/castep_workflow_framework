#!/usr/bin/env python3
"""TASK-7: Add queued_task_polls_before_completing test using DelayedHandle (AtomicUsize counter) that returns is_running()=true for first 2 polls then false, exercising the polling loop"""
import base64, json, subprocess, sys

TASK_ID = "TASK-7"
STEPS = json.loads('[{"before_b64": "I1t0ZXN0XQpmbiBxdWV1ZWRfdGFza19jb21wbGV0ZXNfdmlhX3dvcmtmbG93X3J1bigpIC0+IFJlc3VsdDwoKSwgV29ya2Zsb3dFcnJvcj4gew==", "after_b64": "c3RydWN0IERlbGF5ZWRRdWV1ZWRTdWJtaXR0ZXIgewogICAgZGVsYXlfcG9sbHM6IHVzaXplLAp9CgppbXBsIFF1ZXVlZFN1Ym1pdHRlciBmb3IgRGVsYXllZFF1ZXVlZFN1Ym1pdHRlciB7CiAgICBmbiBzdWJtaXQoCiAgICAgICAgJnNlbGYsCiAgICAgICAgX3dvcmtkaXI6ICZQYXRoLAogICAgICAgIHRhc2tfaWQ6ICZzdHIsCiAgICAgICAgbG9nX2RpcjogJlBhdGgsCiAgICApIC0+IFJlc3VsdDxCb3g8ZHluIFByb2Nlc3NIYW5kbGU+LCBXb3JrZmxvd0Vycm9yPiB7CiAgICAgICAgT2soQm94OjpuZXcoRGVsYXllZEhhbmRsZSB7CiAgICAgICAgICAgIHBvbGxfY291bnQ6IHN0ZDo6c3luYzo6YXRvbWljOjpBdG9taWNVc2l6ZTo6bmV3KDApLAogICAgICAgICAgICBkZWxheV9wb2xsczogc2VsZi5kZWxheV9wb2xscywKICAgICAgICAgICAgc3Rkb3V0X3BhdGg6IGxvZ19kaXIuam9pbihmb3JtYXQhKCJ7fS5zdGRvdXQiLCB0YXNrX2lkKSksCiAgICAgICAgICAgIHN0ZGVycl9wYXRoOiBsb2dfZGlyLmpvaW4oZm9ybWF0ISgie30uc3RkZXJyIiwgdGFza19pZCkpLAogICAgICAgICAgICBzdGFydDogSW5zdGFudDo6bm93KCksCiAgICAgICAgfSkpCiAgICB9Cn0KCnN0cnVjdCBEZWxheWVkSGFuZGxlIHsKICAgIHBvbGxfY291bnQ6IHN0ZDo6c3luYzo6YXRvbWljOjpBdG9taWNVc2l6ZSwKICAgIGRlbGF5X3BvbGxzOiB1c2l6ZSwKICAgIHN0ZG91dF9wYXRoOiBzdGQ6OnBhdGg6OlBhdGhCdWYsCiAgICBzdGRlcnJfcGF0aDogc3RkOjpwYXRoOjpQYXRoQnVmLAogICAgc3RhcnQ6IEluc3RhbnQsCn0KCmltcGwgUHJvY2Vzc0hhbmRsZSBmb3IgRGVsYXllZEhhbmRsZSB7CiAgICBmbiBpc19ydW5uaW5nKCZtdXQgc2VsZikgLT4gYm9vbCB7CiAgICAgICAgbGV0IGNvdW50ID0gc2VsZi5wb2xsX2NvdW50LmZldGNoX2FkZCgxLCBzdGQ6OnN5bmM6OmF0b21pYzo6T3JkZXJpbmc6OlNlcUNzdCk7CiAgICAgICAgY291bnQgPCBzZWxmLmRlbGF5X3BvbGxzCiAgICB9CgogICAgZm4gdGVybWluYXRlKCZtdXQgc2VsZikgLT4gUmVzdWx0PCgpLCBXb3JrZmxvd0Vycm9yPiB7CiAgICAgICAgT2soKCkpCiAgICB9CgogICAgZm4gd2FpdCgmbXV0IHNlbGYpIC0+IFJlc3VsdDxQcm9jZXNzUmVzdWx0LCBXb3JrZmxvd0Vycm9yPiB7CiAgICAgICAgT2soUHJvY2Vzc1Jlc3VsdCB7CiAgICAgICAgICAgIGV4aXRfY29kZTogU29tZSgwKSwKICAgICAgICAgICAgb3V0cHV0OiBPdXRwdXRMb2NhdGlvbjo6T25EaXNrIHsKICAgICAgICAgICAgICAgIHN0ZG91dF9wYXRoOiBzZWxmLnN0ZG91dF9wYXRoLmNsb25lKCksCiAgICAgICAgICAgICAgICBzdGRlcnJfcGF0aDogc2VsZi5zdGRlcnJfcGF0aC5jbG9uZSgpLAogICAgICAgICAgICB9LAogICAgICAgICAgICBkdXJhdGlvbjogc2VsZi5zdGFydC5lbGFwc2VkKCksCiAgICAgICAgfSkKICAgIH0KfQoKI1t0ZXN0XQpmbiBxdWV1ZWRfdGFza19wb2xsc19iZWZvcmVfY29tcGxldGluZygpIC0+IFJlc3VsdDwoKSwgV29ya2Zsb3dFcnJvcj4gewogICAgbGV0IGRpciA9IHRlbXBmaWxlOjp0ZW1wZGlyKCkudW53cmFwKCk7CiAgICBsZXQgbG9nX2RpciA9IGRpci5wYXRoKCkuam9pbigibG9ncyIpOwogICAgc3RkOjpmczo6Y3JlYXRlX2Rpcl9hbGwoJmxvZ19kaXIpLnVud3JhcCgpOwoKICAgIGxldCBtdXQgd2YgPSBXb3JrZmxvdzo6bmV3KCJxdWV1ZWRfcG9sbF90ZXN0IikKICAgICAgICAud2l0aF9tYXhfcGFyYWxsZWwoNCk/CiAgICAgICAgLndpdGhfbG9nX2RpcigmbG9nX2RpcikKICAgICAgICAud2l0aF9xdWV1ZWRfc3VibWl0dGVyKEFyYzo6bmV3KERlbGF5ZWRRdWV1ZWRTdWJtaXR0ZXIgeyBkZWxheV9wb2xsczogMiB9KSk7CgogICAgd2YuYWRkX3Rhc2soCiAgICAgICAgVGFzazo6bmV3KCJxdWV1ZWRfZGVsYXllZCIsIEV4ZWN1dGlvbk1vZGU6OlF1ZXVlZCkKICAgICAgICAgICAgLndvcmtkaXIoZGlyLnBhdGgoKS50b19wYXRoX2J1ZigpKSwKICAgICk/OwoKICAgIGxldCBzdGF0ZV9wYXRoID0gZGlyLnBhdGgoKS5qb2luKCIucXVldWVkX3BvbGxfdGVzdC53b3JrZmxvdy5qc29uIik7CiAgICBsZXQgbXV0IHN0YXRlID0gSnNvblN0YXRlU3RvcmU6Om5ldygicXVldWVkX3BvbGxfdGVzdCIsIHN0YXRlX3BhdGgpOwoKICAgIGxldCBzdW1tYXJ5ID0gd2YucnVuKAogICAgICAgICZtdXQgc3RhdGUsCiAgICAgICAgQXJjOjpuZXcoVW51c2VkUnVubmVyKSwKICAgICAgICBBcmM6Om5ldyhOb29wSG9va0V4ZWN1dG9yKSwKICAgICk/OwoKICAgIGFzc2VydF9lcSEoc3VtbWFyeS5zdWNjZWVkZWQubGVuKCksIDEpOwogICAgYXNzZXJ0IShzdW1tYXJ5LnN1Y2NlZWRlZC5jb250YWlucygmInF1ZXVlZF9kZWxheWVkIi50b19zdHJpbmcoKSkpOwogICAgYXNzZXJ0IShzdW1tYXJ5LmZhaWxlZC5pc19lbXB0eSgpKTsKCiAgICBhc3NlcnQhKG1hdGNoZXMhKAogICAgICAgIHN0YXRlLmdldF9zdGF0dXMoInF1ZXVlZF9kZWxheWVkIiksCiAgICAgICAgU29tZSh3b3JrZmxvd19jb3JlOjpzdGF0ZTo6VGFza1N0YXR1czo6Q29tcGxldGVkKQogICAgKSk7CgogICAgT2soKCkpCn0KCiNbdGVzdF0KZm4gcXVldWVkX3Rhc2tfY29tcGxldGVzX3ZpYV93b3JrZmxvd19ydW4oKSAtPiBSZXN1bHQ8KCksIFdvcmtmbG93RXJyb3I+IHs=", "target": "workflow_core/tests/queued_workflow.rs", "index": 0}]')

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
