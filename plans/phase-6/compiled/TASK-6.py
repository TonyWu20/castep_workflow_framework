#!/usr/bin/env python3
"""TASK-6: Update ARCHITECTURE.md/ARCHITECTURE_STATUS.md, fix config assertion, trailing newline, and clippy"""
import base64, json, subprocess, sys
from pathlib import Path

TASK_ID = "TASK-6"
STEPS = json.loads('[{"before_b64": "ICAgICNbdGVzdF0KICAgIGZuIHBhcnNlX2VtcHR5X3N0cmluZygpIHsKICAgICAgICAvLyBUaGUgd2hvbGUgaW5wdXQgaXMgZW1wdHkgKGRpc3RpbmN0IGZyb20gYW4gZW1wdHkgdG9rZW4gaW4gdGhlIG1pZGRsZSkKICAgICAgICBsZXQgZXJyID0gcGFyc2VfdV92YWx1ZXMoIiIpLnVud3JhcF9lcnIoKTsKICAgICAgICBhc3NlcnQhKCFlcnIuaXNfZW1wdHkoKSk7CiAgICB9", "after_b64": "ICAgICNbdGVzdF0KICAgIGZuIHBhcnNlX2VtcHR5X3N0cmluZygpIHsKICAgICAgICAvLyBUaGUgd2hvbGUgaW5wdXQgaXMgZW1wdHkgKGRpc3RpbmN0IGZyb20gYW4gZW1wdHkgdG9rZW4gaW4gdGhlIG1pZGRsZSkKICAgICAgICBsZXQgZXJyID0gcGFyc2VfdV92YWx1ZXMoIiIpLnVud3JhcF9lcnIoKTsKICAgICAgICBhc3NlcnQhKGVyci5jb250YWlucygiaW52YWxpZCIpLCAiZXhwZWN0ZWQgcGFyc2UgZmFpbHVyZSBvbiBlbXB0eSBpbnB1dCwgZ290OiB7ZXJyfSIpOwogICAgfQ==", "target": "/Users/tony/programming/castep_workflow_framework/examples/hubbard_u_sweep_slurm/src/config.rs", "index": 0, "is_create": false}, {"before_b64": "Ly8gd29ya2Zsb3dfdXRpbHMgdHlwZXMKcHViIHVzZSBjcmF0ZTo6ewogICAgY29weV9maWxlLCBjcmVhdGVfZGlyLCBleGlzdHMsIHJlYWRfZmlsZSwgcmVtb3ZlX2RpciwgcnVuX2RlZmF1bHQsIHdyaXRlX2ZpbGUsCiAgICBRdWV1ZWRSdW5uZXIsIFNjaGVkdWxlcktpbmQsIFNoZWxsSG9va0V4ZWN1dG9yLCBTeXN0ZW1Qcm9jZXNzUnVubmVyLCBKT0JfU0NSSVBUX05BTUUsCn07", "after_b64": "Ly8gd29ya2Zsb3dfdXRpbHMgdHlwZXMKcHViIHVzZSBjcmF0ZTo6ewogICAgY29weV9maWxlLCBjcmVhdGVfZGlyLCBleGlzdHMsIHJlYWRfZmlsZSwgcmVtb3ZlX2RpciwgcnVuX2RlZmF1bHQsIHdyaXRlX2ZpbGUsCiAgICBRdWV1ZWRSdW5uZXIsIFNjaGVkdWxlcktpbmQsIFNoZWxsSG9va0V4ZWN1dG9yLCBTeXN0ZW1Qcm9jZXNzUnVubmVyLCBKT0JfU0NSSVBUX05BTUUsCn07", "target": "/Users/tony/programming/castep_workflow_framework/workflow_utils/src/prelude.rs", "index": 1, "is_create": false}, {"before_b64": "aW1wbCBKc29uU3RhdGVTdG9yZSB7CiAgICBwdWIgZm4gbmV3KG5hbWU6IGltcGwgSW50bzxTdHJpbmc+LCBwYXRoOiBQYXRoQnVmKSAtPiBTZWxmOwoKICAgIC8vIGNyYXNoLXJlY292ZXJ5OiByZXNldHMgRmFpbGVkL1J1bm5pbmcvU2tpcHBlZER1ZVRvRGVwZW5kZW5jeUZhaWx1cmUg4oaSIFBlbmRpbmcKICAgIHB1YiBmbiBsb2FkKCZtdXQgc2VsZikgLT4gUmVzdWx0PCgpLCBXb3JrZmxvd0Vycm9yPjsKCiAgICAvLyByZWFkLW9ubHkgaW5zcGVjdGlvbiB3aXRob3V0IGNyYXNoLXJlY292ZXJ5IHJlc2V0cyAodXNlZCBieSBDTEkgc3RhdHVzL2luc3BlY3QpCiAgICBwdWIgZm4gbG9hZF9yYXcoJnNlbGYpIC0+IFJlc3VsdDxXb3JrZmxvd1N0YXRlLCBXb3JrZmxvd0Vycm9yPjsKfQ==", "after_b64": "aW1wbCBKc29uU3RhdGVTdG9yZSB7CiAgICBwdWIgZm4gbmV3KG5hbWU6IGltcGwgSW50bzxTdHJpbmc+LCBwYXRoOiBQYXRoQnVmKSAtPiBTZWxmOwoKICAgIC8vIGNyYXNoLXJlY292ZXJ5OiByZXNldHMgRmFpbGVkL1J1bm5pbmcvU2tpcHBlZER1ZVRvRGVwZW5kZW5jeUZhaWx1cmUg4oaSIFBlbmRpbmcKICAgIHB1YiBmbiBsb2FkKHBhdGg6IGltcGwgQXNSZWY8UGF0aD4pIC0+IFJlc3VsdDxTZWxmLCBXb3JrZmxvd0Vycm9yPjsKCiAgICAvLyByZWFkLW9ubHkgaW5zcGVjdGlvbiB3aXRob3V0IGNyYXNoLXJlY292ZXJ5IHJlc2V0cyAodXNlZCBieSBDTEkgc3RhdHVzL2luc3BlY3QpCiAgICBwdWIgZm4gbG9hZF9yYXcocGF0aDogaW1wbCBBc1JlZjxQYXRoPikgLT4gUmVzdWx0PFNlbGYsIFdvcmtmbG93RXJyb3I+Owp9", "target": "/Users/tony/programming/castep_workflow_framework/ARCHITECTURE.md", "index": 2, "is_create": false}, {"before_b64": "ICAgIC8vLyBTZXQgc2V0dXAgY2xvc3VyZSAocnVucyBiZWZvcmUgZXhlY3V0aW9uKQogICAgcHViIGZuIHNldHVwPEY+KHNlbGYsIGY6IEYpIC0+IFNlbGYKICAgIHdoZXJlIEY6IEZuKCZQYXRoKSAtPiBSZXN1bHQ8KCksIFdvcmtmbG93RXJyb3I+ICsgU2VuZCArIFN5bmMgKyAnc3RhdGljOwoKICAgIC8vLyBTZXQgY29sbGVjdCBjbG9zdXJlIChydW5zIGFmdGVyIHN1Y2Nlc3NmdWwgZXhlY3V0aW9uIHRvIHZhbGlkYXRlIG91dHB1dCkKICAgIHB1YiBmbiBjb2xsZWN0PEY+KHNlbGYsIGY6IEYpIC0+IFNlbGYKICAgIHdoZXJlIEY6IEZuKCZQYXRoKSAtPiBSZXN1bHQ8KCksIFdvcmtmbG93RXJyb3I+ICsgU2VuZCArIFN5bmMgKyAnc3RhdGljOw==", "after_b64": "ICAgIC8vLyBTZXQgc2V0dXAgY2xvc3VyZSAocnVucyBiZWZvcmUgZXhlY3V0aW9uKS4KICAgIHB1YiBmbiBzZXR1cDxGLCBFPihzZWxmLCBmOiBGKSAtPiBTZWxmCiAgICB3aGVyZQogICAgICAgIEY6IEZuKCZQYXRoKSAtPiBSZXN1bHQ8KCksIEU+ICsgU2VuZCArIFN5bmMgKyAnc3RhdGljLAogICAgICAgIEU6IHN0ZDo6ZXJyb3I6OkVycm9yICsgU2VuZCArIFN5bmMgKyAnc3RhdGljOwoKICAgIC8vLyBTZXQgY29sbGVjdCBjbG9zdXJlIChydW5zIGFmdGVyIHN1Y2Nlc3NmdWwgZXhlY3V0aW9uIHRvIHZhbGlkYXRlIG91dHB1dCkuCiAgICBwdWIgZm4gY29sbGVjdDxGLCBFPihzZWxmLCBmOiBGKSAtPiBTZWxmCiAgICB3aGVyZQogICAgICAgIEY6IEZuKCZQYXRoKSAtPiBSZXN1bHQ8KCksIEU+ICsgU2VuZCArIFN5bmMgKyAnc3RhdGljLAogICAgICAgIEU6IHN0ZDo6ZXJyb3I6OkVycm9yICsgU2VuZCArIFN5bmMgKyAnc3RhdGljOw==", "target": "/Users/tony/programming/castep_workflow_framework/ARCHITECTURE.md", "index": 3, "is_create": false}, {"before_b64": "LSBgVGFza2AgZ2FpbnMgYHNldHVwYC9gY29sbGVjdGAgY2xvc3VyZSBmaWVsZHM7IGBUYXNrQ2xvc3VyZSA9IEJveDxkeW4gRm4oJlBhdGgpIC0+IFJlc3VsdDwoKSwgV29ya2Zsb3dFcnJvcj4gKyBTZW5kICsgU3luYz5gIHR5cGUgYWxpYXM=", "after_b64": "LSBgVGFza2AgZ2FpbnMgYHNldHVwYC9gY29sbGVjdGAgY2xvc3VyZSBmaWVsZHM7IGBUYXNrQ2xvc3VyZSA9IEJveDxkeW4gRm4oJlBhdGgpIC0+IFJlc3VsdDwoKSwgQm94PGR5biBzdGQ6OmVycm9yOjpFcnJvciArIFNlbmQgKyBTeW5jPj4gKyBTZW5kICsgU3luYz5gIHR5cGUgYWxpYXMKLSBgQ29sbGVjdEZhaWx1cmVQb2xpY3lgIGVudW06IGBGYWlsVGFza2AgKGRlZmF1bHQpIGFuZCBgV2Fybk9ubHlgIGZvciBnb3Zlcm5pbmcgY29sbGVjdCBjbG9zdXJlIGZhaWx1cmVz", "target": "/Users/tony/programming/castep_workflow_framework/ARCHITECTURE_STATUS.md", "index": 4, "is_create": false}, {"before_b64": "LSBgZG93bnN0cmVhbV9vZjxTOiBBc1JlZjxzdHI+PmAgZ2VuZXJpYyBzaWduYXR1cmUg4oCUIGNhbGxlcnMgcGFzcyBgJlsmc3RyXWAgd2l0aG91dCBhbGxvY2F0aW5n", "after_b64": "LSBgZG93bnN0cmVhbV9vZjxTOiBBc1JlZjxzdHI+PmAgZ2VuZXJpYyBzaWduYXR1cmUg4oCUIGNhbGxlcnMgcGFzcyBgJlsmc3RyXWAgd2l0aG91dCBhbGxvY2F0aW5nCi0gYENvbGxlY3RGYWlsdXJlUG9saWN5YCByZS1leHBvcnRlZCBmcm9tIGB3b3JrZmxvd19jb3JlOjpwcmVsdWRlYCBhbmQgYHdvcmtmbG93X2NvcmU6OmxpYmA=", "target": "/Users/tony/programming/castep_workflow_framework/ARCHITECTURE_STATUS.md", "index": 5, "is_create": false}]')

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
