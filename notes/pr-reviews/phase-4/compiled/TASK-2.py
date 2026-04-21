#!/usr/bin/env python3
"""TASK-2: Change task_successors field from HashMap to Option<HashMap> in JsonStateStore; None = pre-graph state file, Some(empty) = graph with no edges; update getter, setter, constructor, and cmd_retry fallback"""
import base64, json, subprocess, sys

TASK_ID = "TASK-2"
STEPS = json.loads('[{"before_b64": "ICAgICNbc2VyZGUoZGVmYXVsdCldCiAgICB0YXNrX3N1Y2Nlc3NvcnM6IEhhc2hNYXA8U3RyaW5nLCBWZWM8U3RyaW5nPj4sCiAgICBwYXRoOiBQYXRoQnVmLA==", "after_b64": "ICAgICNbc2VyZGUoZGVmYXVsdCldCiAgICB0YXNrX3N1Y2Nlc3NvcnM6IE9wdGlvbjxIYXNoTWFwPFN0cmluZywgVmVjPFN0cmluZz4+PiwKICAgIHBhdGg6IFBhdGhCdWYs", "target": "workflow_core/src/state.rs", "index": 0}, {"before_b64": "ICAgICAgICAgICAgdGFza19zdWNjZXNzb3JzOiBIYXNoTWFwOjpuZXcoKSwKICAgICAgICAgICAgcGF0aCw=", "after_b64": "ICAgICAgICAgICAgdGFza19zdWNjZXNzb3JzOiBOb25lLAogICAgICAgICAgICBwYXRoLA==", "target": "workflow_core/src/state.rs", "index": 1}, {"before_b64": "ICAgIC8vLyBSZXR1cm5zIHRoZSB0YXNrIHN1Y2Nlc3NvciBncmFwaCBwZXJzaXN0ZWQgZnJvbSB0aGUgbGFzdCB3b3JrZmxvdyBydW4uCiAgICBwdWIgZm4gdGFza19zdWNjZXNzb3JzKCZzZWxmKSAtPiAmSGFzaE1hcDxTdHJpbmcsIFZlYzxTdHJpbmc+PiB7CiAgICAgICAgJnNlbGYudGFza19zdWNjZXNzb3JzCiAgICB9CgogICAgLy8vIFNldHMgdGhlIHRhc2sgZGVwZW5kZW5jeSBncmFwaCAoc3VjY2Vzc29ycyBtYXApIGZvciBncmFwaC1hd2FyZSByZXRyeS4KICAgIHB1YiBmbiBzZXRfdGFza19ncmFwaCgmbXV0IHNlbGYsIHN1Y2Nlc3NvcnM6IEhhc2hNYXA8U3RyaW5nLCBWZWM8U3RyaW5nPj4pIHsKICAgICAgICBzZWxmLnRhc2tfc3VjY2Vzc29ycyA9IHN1Y2Nlc3NvcnM7CiAgICB9", "after_b64": "ICAgIC8vLyBSZXR1cm5zIHRoZSB0YXNrIHN1Y2Nlc3NvciBncmFwaCBwZXJzaXN0ZWQgZnJvbSB0aGUgbGFzdCB3b3JrZmxvdyBydW4uCiAgICAvLy8gUmV0dXJucyBgTm9uZWAgZm9yIHN0YXRlIGZpbGVzIGNyZWF0ZWQgYmVmb3JlIGdyYXBoIHBlcnNpc3RlbmNlIHdhcyBhZGRlZC4KICAgIHB1YiBmbiB0YXNrX3N1Y2Nlc3NvcnMoJnNlbGYpIC0+IE9wdGlvbjwmSGFzaE1hcDxTdHJpbmcsIFZlYzxTdHJpbmc+Pj4gewogICAgICAgIHNlbGYudGFza19zdWNjZXNzb3JzLmFzX3JlZigpCiAgICB9CgogICAgLy8vIFNldHMgdGhlIHRhc2sgZGVwZW5kZW5jeSBncmFwaCAoc3VjY2Vzc29ycyBtYXApIGZvciBncmFwaC1hd2FyZSByZXRyeS4KICAgIHB1YiBmbiBzZXRfdGFza19ncmFwaCgmbXV0IHNlbGYsIHN1Y2Nlc3NvcnM6IEhhc2hNYXA8U3RyaW5nLCBWZWM8U3RyaW5nPj4pIHsKICAgICAgICBzZWxmLnRhc2tfc3VjY2Vzc29ycyA9IFNvbWUoc3VjY2Vzc29ycyk7CiAgICB9", "target": "workflow_core/src/state.rs", "index": 2}, {"before_b64": "ICAgIGxldCBzdWNjZXNzb3JzID0gc3RhdGUudGFza19zdWNjZXNzb3JzKCkuY2xvbmUoKTsKICAgIGlmIHN1Y2Nlc3NvcnMuaXNfZW1wdHkoKSB7CiAgICAgICAgZXByaW50bG4hKCJ3YXJuOiBzdGF0ZSBmaWxlIGxhY2tzIGRlcGVuZGVuY3kgaW5mbzsgZmFsbGluZyBiYWNrIHRvIGdsb2JhbCByZXNldCIpOwogICAgICAgIGxldCB0b19yZXNldDogVmVjPFN0cmluZz4gPSBzdGF0ZQogICAgICAgICAgICAuYWxsX3Rhc2tzKCkKICAgICAgICAgICAgLmludG9faXRlcigpCiAgICAgICAgICAgIC5maWx0ZXIofChfLCBzKXwgbWF0Y2hlcyEocywgVGFza1N0YXR1czo6U2tpcHBlZER1ZVRvRGVwZW5kZW5jeUZhaWx1cmUpKQogICAgICAgICAgICAubWFwKHwoaWQsIF8pfCBpZCkKICAgICAgICAgICAgLmNvbGxlY3QoKTsKICAgICAgICBmb3IgaWQgaW4gdG9fcmVzZXQgewogICAgICAgICAgICBzdGF0ZS5tYXJrX3BlbmRpbmcoJmlkKTsKICAgICAgICB9CiAgICB9IGVsc2UgewogICAgICAgIGxldCBkb3duc3RyZWFtID0gZG93bnN0cmVhbV90YXNrcyh0YXNrX2lkcywgJnN1Y2Nlc3NvcnMpOwogICAgICAgIGxldCB0b19yZXNldDogVmVjPFN0cmluZz4gPSBzdGF0ZQogICAgICAgICAgICAuYWxsX3Rhc2tzKCkKICAgICAgICAgICAgLmludG9faXRlcigpCiAgICAgICAgICAgIC5maWx0ZXIofChpZCwgcyl8IHsKICAgICAgICAgICAgICAgIG1hdGNoZXMhKHMsIFRhc2tTdGF0dXM6OlNraXBwZWREdWVUb0RlcGVuZGVuY3lGYWlsdXJlKQogICAgICAgICAgICAgICAgICAgICYmIGRvd25zdHJlYW0uY29udGFpbnMoaWQpCiAgICAgICAgICAgIH0pCiAgICAgICAgICAgIC5tYXAofChpZCwgXyl8IGlkKQogICAgICAgICAgICAuY29sbGVjdCgpOwogICAgICAgIGZvciBpZCBpbiB0b19yZXNldCB7CiAgICAgICAgICAgIHN0YXRlLm1hcmtfcGVuZGluZygmaWQpOwogICAgICAgIH0KICAgIH0=", "after_b64": "ICAgIG1hdGNoIHN0YXRlLnRhc2tfc3VjY2Vzc29ycygpLmNsb25lZCgpIHsKICAgICAgICBOb25lID0+IHsKICAgICAgICAgICAgZXByaW50bG4hKCJ3YXJuOiBzdGF0ZSBmaWxlIGxhY2tzIGRlcGVuZGVuY3kgaW5mbzsgZmFsbGluZyBiYWNrIHRvIGdsb2JhbCByZXNldCIpOwogICAgICAgICAgICBsZXQgdG9fcmVzZXQ6IFZlYzxTdHJpbmc+ID0gc3RhdGUKICAgICAgICAgICAgICAgIC5hbGxfdGFza3MoKQogICAgICAgICAgICAgICAgLmludG9faXRlcigpCiAgICAgICAgICAgICAgICAuZmlsdGVyKHwoXywgcyl8IG1hdGNoZXMhKHMsIFRhc2tTdGF0dXM6OlNraXBwZWREdWVUb0RlcGVuZGVuY3lGYWlsdXJlKSkKICAgICAgICAgICAgICAgIC5tYXAofChpZCwgXyl8IGlkKQogICAgICAgICAgICAgICAgLmNvbGxlY3QoKTsKICAgICAgICAgICAgZm9yIGlkIGluIHRvX3Jlc2V0IHsKICAgICAgICAgICAgICAgIHN0YXRlLm1hcmtfcGVuZGluZygmaWQpOwogICAgICAgICAgICB9CiAgICAgICAgfQogICAgICAgIFNvbWUoc3VjY2Vzc29ycykgPT4gewogICAgICAgICAgICBsZXQgZG93bnN0cmVhbSA9IGRvd25zdHJlYW1fdGFza3ModGFza19pZHMsICZzdWNjZXNzb3JzKTsKICAgICAgICAgICAgbGV0IHRvX3Jlc2V0OiBWZWM8U3RyaW5nPiA9IHN0YXRlCiAgICAgICAgICAgICAgICAuYWxsX3Rhc2tzKCkKICAgICAgICAgICAgICAgIC5pbnRvX2l0ZXIoKQogICAgICAgICAgICAgICAgLmZpbHRlcih8KGlkLCBzKXwgewogICAgICAgICAgICAgICAgICAgIG1hdGNoZXMhKHMsIFRhc2tTdGF0dXM6OlNraXBwZWREdWVUb0RlcGVuZGVuY3lGYWlsdXJlKQogICAgICAgICAgICAgICAgICAgICAgICAmJiBkb3duc3RyZWFtLmNvbnRhaW5zKGlkKQogICAgICAgICAgICAgICAgfSkKICAgICAgICAgICAgICAgIC5tYXAofChpZCwgXyl8IGlkKQogICAgICAgICAgICAgICAgLmNvbGxlY3QoKTsKICAgICAgICAgICAgZm9yIGlkIGluIHRvX3Jlc2V0IHsKICAgICAgICAgICAgICAgIHN0YXRlLm1hcmtfcGVuZGluZygmaWQpOwogICAgICAgICAgICB9CiAgICAgICAgfQogICAgfQ==", "target": "workflow-cli/src/main.rs", "index": 3}]')

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
