#!/usr/bin/env python3
"""TASK-1: Remove set_task_graph from StateStore trait, make it inherent on JsonStateStore, store computed successor map on Workflow, expose via successor_map()"""
import base64, json, subprocess, sys

TASK_ID = "TASK-1"
STEPS = json.loads('[{"before_b64": "ICAgIC8vLyBQZXJzaXN0cyB0aGUgY3VycmVudCBzdGF0ZSB0byBkaXNrLgogICAgZm4gc2F2ZSgmc2VsZikgLT4gUmVzdWx0PCgpLCBXb3JrZmxvd0Vycm9yPjsKCiAgICAvLy8gUGVyc2lzdHMgdGhlIHRhc2sgZGVwZW5kZW5jeSBncmFwaCAoc3VjY2Vzc29ycyBtYXApIGZvciBncmFwaC1hd2FyZSByZXRyeS4KICAgIC8vLyBEZWZhdWx0IGlzIGEgbm8tb3A7IGBKc29uU3RhdGVTdG9yZWAgb3ZlcnJpZGVzIHRoaXMuCiAgICBmbiBzZXRfdGFza19ncmFwaCgmbXV0IHNlbGYsIF9zdWNjZXNzb3JzOiBIYXNoTWFwPFN0cmluZywgVmVjPFN0cmluZz4+KSB7fQp9", "after_b64": "ICAgIC8vLyBQZXJzaXN0cyB0aGUgY3VycmVudCBzdGF0ZSB0byBkaXNrLgogICAgZm4gc2F2ZSgmc2VsZikgLT4gUmVzdWx0PCgpLCBXb3JrZmxvd0Vycm9yPjsKfQ==", "target": "workflow_core/src/state.rs", "index": 0}, {"before_b64": "ICAgIGZuIHNhdmUoJnNlbGYpIC0+IFJlc3VsdDwoKSwgV29ya2Zsb3dFcnJvcj4gewogICAgICAgIHNlbGYucGVyc2lzdCgpCiAgICB9CgogICAgZm4gc2V0X3Rhc2tfZ3JhcGgoJm11dCBzZWxmLCBzdWNjZXNzb3JzOiBIYXNoTWFwPFN0cmluZywgVmVjPFN0cmluZz4+KSB7CiAgICAgICAgc2VsZi50YXNrX3N1Y2Nlc3NvcnMgPSBzdWNjZXNzb3JzOwogICAgfQp9", "after_b64": "ICAgIGZuIHNhdmUoJnNlbGYpIC0+IFJlc3VsdDwoKSwgV29ya2Zsb3dFcnJvcj4gewogICAgICAgIHNlbGYucGVyc2lzdCgpCiAgICB9Cn0=", "target": "workflow_core/src/state.rs", "index": 1}, {"before_b64": "ICAgIC8vLyBSZXR1cm5zIHRoZSB0YXNrIHN1Y2Nlc3NvciBncmFwaCBwZXJzaXN0ZWQgZnJvbSB0aGUgbGFzdCB3b3JrZmxvdyBydW4uCiAgICBwdWIgZm4gdGFza19zdWNjZXNzb3JzKCZzZWxmKSAtPiAmSGFzaE1hcDxTdHJpbmcsIFZlYzxTdHJpbmc+PiB7CiAgICAgICAgJnNlbGYudGFza19zdWNjZXNzb3JzCiAgICB9Cn0=", "after_b64": "ICAgIC8vLyBSZXR1cm5zIHRoZSB0YXNrIHN1Y2Nlc3NvciBncmFwaCBwZXJzaXN0ZWQgZnJvbSB0aGUgbGFzdCB3b3JrZmxvdyBydW4uCiAgICBwdWIgZm4gdGFza19zdWNjZXNzb3JzKCZzZWxmKSAtPiAmSGFzaE1hcDxTdHJpbmcsIFZlYzxTdHJpbmc+PiB7CiAgICAgICAgJnNlbGYudGFza19zdWNjZXNzb3JzCiAgICB9CgogICAgLy8vIFNldHMgdGhlIHRhc2sgZGVwZW5kZW5jeSBncmFwaCAoc3VjY2Vzc29ycyBtYXApIGZvciBncmFwaC1hd2FyZSByZXRyeS4KICAgIHB1YiBmbiBzZXRfdGFza19ncmFwaCgmbXV0IHNlbGYsIHN1Y2Nlc3NvcnM6IEhhc2hNYXA8U3RyaW5nLCBWZWM8U3RyaW5nPj4pIHsKICAgICAgICBzZWxmLnRhc2tfc3VjY2Vzc29ycyA9IHN1Y2Nlc3NvcnM7CiAgICB9Cn0=", "target": "workflow_core/src/state.rs", "index": 2}, {"before_b64": "cHViIHN0cnVjdCBXb3JrZmxvdyB7CiAgICBwdWIgbmFtZTogU3RyaW5nLAogICAgdGFza3M6IEhhc2hNYXA8U3RyaW5nLCBUYXNrPiwKICAgIG1heF9wYXJhbGxlbDogdXNpemUsCiAgICBwdWIoY3JhdGUpIGludGVycnVwdDogQXJjPEF0b21pY0Jvb2w+LAogICAgbG9nX2RpcjogT3B0aW9uPHN0ZDo6cGF0aDo6UGF0aEJ1Zj4sCiAgICBxdWV1ZWRfc3VibWl0dGVyOiBPcHRpb248QXJjPGR5biBjcmF0ZTo6cHJvY2Vzczo6UXVldWVkU3VibWl0dGVyPj4sCn0=", "after_b64": "cHViIHN0cnVjdCBXb3JrZmxvdyB7CiAgICBwdWIgbmFtZTogU3RyaW5nLAogICAgdGFza3M6IEhhc2hNYXA8U3RyaW5nLCBUYXNrPiwKICAgIG1heF9wYXJhbGxlbDogdXNpemUsCiAgICBwdWIoY3JhdGUpIGludGVycnVwdDogQXJjPEF0b21pY0Jvb2w+LAogICAgbG9nX2RpcjogT3B0aW9uPHN0ZDo6cGF0aDo6UGF0aEJ1Zj4sCiAgICBxdWV1ZWRfc3VibWl0dGVyOiBPcHRpb248QXJjPGR5biBjcmF0ZTo6cHJvY2Vzczo6UXVldWVkU3VibWl0dGVyPj4sCiAgICBjb21wdXRlZF9zdWNjZXNzb3JzOiBPcHRpb248SGFzaE1hcDxTdHJpbmcsIFZlYzxTdHJpbmc+Pj4sCn0=", "target": "workflow_core/src/workflow.rs", "index": 3}, {"before_b64": "ICAgICAgICBTZWxmIHsKICAgICAgICAgICAgbmFtZTogbmFtZS5pbnRvKCksCiAgICAgICAgICAgIHRhc2tzOiBIYXNoTWFwOjpuZXcoKSwKICAgICAgICAgICAgbWF4X3BhcmFsbGVsLAogICAgICAgICAgICBpbnRlcnJ1cHQ6IEFyYzo6bmV3KEF0b21pY0Jvb2w6Om5ldyhmYWxzZSkpLAogICAgICAgICAgICBsb2dfZGlyOiBOb25lLAogICAgICAgICAgICBxdWV1ZWRfc3VibWl0dGVyOiBOb25lLAogICAgICAgIH0=", "after_b64": "ICAgICAgICBTZWxmIHsKICAgICAgICAgICAgbmFtZTogbmFtZS5pbnRvKCksCiAgICAgICAgICAgIHRhc2tzOiBIYXNoTWFwOjpuZXcoKSwKICAgICAgICAgICAgbWF4X3BhcmFsbGVsLAogICAgICAgICAgICBpbnRlcnJ1cHQ6IEFyYzo6bmV3KEF0b21pY0Jvb2w6Om5ldyhmYWxzZSkpLAogICAgICAgICAgICBsb2dfZGlyOiBOb25lLAogICAgICAgICAgICBxdWV1ZWRfc3VibWl0dGVyOiBOb25lLAogICAgICAgICAgICBjb21wdXRlZF9zdWNjZXNzb3JzOiBOb25lLAogICAgICAgIH0=", "target": "workflow_core/src/workflow.rs", "index": 4}, {"before_b64": "ICAgICAgICBsZXQgZGFnID0gc2VsZi5idWlsZF9kYWcoKT87CgogICAgICAgIC8vIFBlcnNpc3QgdGFzayBkZXBlbmRlbmN5IGdyYXBoIGZvciBDTEkgcmV0cnkKICAgICAgICBsZXQgc3VjY2Vzc29yczogSGFzaE1hcDxTdHJpbmcsIFZlYzxTdHJpbmc+PiA9IGRhZy50YXNrX2lkcygpCiAgICAgICAgICAgIC5tYXAofGlkfCAoaWQuY2xvbmUoKSwgZGFnLnN1Y2Nlc3NvcnMoaWQpKSkKICAgICAgICAgICAgLmNvbGxlY3QoKTsKICAgICAgICBzdGF0ZS5zZXRfdGFza19ncmFwaChzdWNjZXNzb3JzKTsKCiAgICAgICAgLy8gSW5pdGlhbGl6ZSBzdGF0ZSBmb3IgYWxsIHRhc2tz", "after_b64": "ICAgICAgICBsZXQgZGFnID0gc2VsZi5idWlsZF9kYWcoKT87CgogICAgICAgIC8vIENvbXB1dGUgYW5kIHN0b3JlIHRhc2sgZGVwZW5kZW5jeSBncmFwaCBmb3IgQ0xJIHJldHJpZXZhbAogICAgICAgIGxldCBzdWNjZXNzb3JzOiBIYXNoTWFwPFN0cmluZywgVmVjPFN0cmluZz4+ID0gZGFnLnRhc2tfaWRzKCkKICAgICAgICAgICAgLm1hcCh8aWR8IChpZC5jbG9uZSgpLCBkYWcuc3VjY2Vzc29ycyhpZCkpKQogICAgICAgICAgICAuY29sbGVjdCgpOwogICAgICAgIHNlbGYuY29tcHV0ZWRfc3VjY2Vzc29ycyA9IFNvbWUoc3VjY2Vzc29ycyk7CgogICAgICAgIC8vIEluaXRpYWxpemUgc3RhdGUgZm9yIGFsbCB0YXNrcw==", "target": "workflow_core/src/workflow.rs", "index": 5}, {"before_b64": "ICAgIHB1YiBmbiBhZGRfdGFzaygmbXV0IHNlbGYsIHRhc2s6IFRhc2spIC0+IFJlc3VsdDwoKSwgV29ya2Zsb3dFcnJvcj4gew==", "after_b64": "ICAgIC8vLyBSZXR1cm5zIHRoZSBjb21wdXRlZCBzdWNjZXNzb3IgbWFwIGFmdGVyIGBydW4oKWAgaGFzIGJlZW4gY2FsbGVkLgogICAgLy8vIFJldHVybnMgYE5vbmVgIGlmIGBydW4oKWAgaGFzIG5vdCB5ZXQgYmVlbiBjYWxsZWQuCiAgICBwdWIgZm4gc3VjY2Vzc29yX21hcCgmc2VsZikgLT4gT3B0aW9uPCZIYXNoTWFwPFN0cmluZywgVmVjPFN0cmluZz4+PiB7CiAgICAgICAgc2VsZi5jb21wdXRlZF9zdWNjZXNzb3JzLmFzX3JlZigpCiAgICB9CgogICAgcHViIGZuIGFkZF90YXNrKCZtdXQgc2VsZiwgdGFzazogVGFzaykgLT4gUmVzdWx0PCgpLCBXb3JrZmxvd0Vycm9yPiB7", "target": "workflow_core/src/workflow.rs", "index": 6}]')

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
