#!/usr/bin/env python3

# Finds a Rust binary artifact based on environment variables provided by
# docker, and copies it to /root/output/run

import os
import shutil
from pathlib import Path

build_target_type, build_target_name = os.environ["RUST_BUILD_TARGET"].split("=")
build_target_type = build_target_type[2:]
target_triple = os.environ["RUST_TARGET"]
profile_out_dir = os.environ["RUST_PROFILE_OUT_DIR"]

artifact_dir = Path(f"target/{target_triple}/{profile_out_dir}")
if build_target_type == "example":
    artifact_path = artifact_dir / f"examples/{build_target_name}"
else:
    artifact_path = artifact_dir / f"{build_target_name}"

output_dir = Path("/root/output")
output_dir.mkdir(parents=True)
output_path = output_dir / "run"

shutil.copy(artifact_path, output_path)
