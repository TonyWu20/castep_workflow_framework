#!/usr/bin/env python3
"""TASK-3: Create examples/hubbard_u_sweep_slurm/Cargo.toml and seed files"""
import base64, sys
from pathlib import Path

CONTENT = base64.b64decode("W3BhY2thZ2VdCm5hbWUgPSAiaHViYmFyZF91X3N3ZWVwX3NsdXJtIgp2ZXJzaW9uID0gIjAuMS4wIgplZGl0aW9uID0gIjIwMjEiCgpbW2Jpbl1dCm5hbWUgPSAiaHViYmFyZF91X3N3ZWVwX3NsdXJtIgpwYXRoID0gInNyYy9tYWluLnJzIgoKW2RlcGVuZGVuY2llc10KYW55aG93ID0geyB3b3Jrc3BhY2UgPSB0cnVlIH0KY2xhcCA9IHsgd29ya3NwYWNlID0gdHJ1ZSB9CmNhc3RlcC1jZWxsLWZtdCA9ICIwLjEuMCIKY2FzdGVwLWNlbGwtaW8gPSAiMC40LjAiCndvcmtmbG93X2NvcmUgPSB7IHBhdGggPSAiLi4vLi4vd29ya2Zsb3dfY29yZSIsIGZlYXR1cmVzID0gWyJkZWZhdWx0LWxvZ2dpbmciXSB9CndvcmtmbG93X3V0aWxzID0geyBwYXRoID0gIi4uLy4uL3dvcmtmbG93X3V0aWxzIiB9").decode()
TARGET = "examples/hubbard_u_sweep_slurm/Cargo.toml"
TASK_ID = "TASK-3"

target_path = Path(TARGET)
target_path.parent.mkdir(parents=True, exist_ok=True)
target_path.write_text(CONTENT)

if not target_path.exists():
    print(f"FAILED {TASK_ID}: file not created at {TARGET}", file=sys.stderr)
    sys.exit(1)

print(f"OK {TASK_ID}: created {TARGET}")
