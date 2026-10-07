#!/usr/bin/env python3
"""Copy workspace default-member executables into a CI artifact directory."""

import json
from pathlib import Path
import shutil
import subprocess
import sys

if len(sys.argv) != 3:
    raise SystemExit("usage: stage-binaries.py BUILD_DIR OUTPUT_DIR")

build_dir = Path(sys.argv[1])
output_dir = Path(sys.argv[2])
metadata = json.loads(
    subprocess.check_output(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"], text=True
    )
)
packages = {package["id"]: package for package in metadata["packages"]}
binaries = [
    target["name"]
    for package_id in metadata["workspace_default_members"]
    for target in packages[package_id]["targets"]
    if "bin" in target["kind"]
]
if not binaries:
    raise SystemExit("No binary targets found among workspace default members")

output_dir.mkdir(parents=True, exist_ok=True)
for name in binaries:
    source = build_dir / name
    if not source.is_file():
        raise FileNotFoundError(f"Built executable not found: {source}")
    shutil.copy2(source, output_dir / name)
    print(f"Staged {source} -> {output_dir / name}")
