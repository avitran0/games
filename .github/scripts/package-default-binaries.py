#!/usr/bin/env python3
import json
from pathlib import Path
import shutil
import subprocess
import sys

if len(sys.argv) != 3:
    raise SystemExit("usage: package-default-binaries.py BUILD_DIR OUTPUT_DIR")

build_dir = Path(sys.argv[1]).resolve()
output_dir = Path(sys.argv[2]).resolve()
repo_root = Path(__file__).resolve().parents[2]

metadata = json.loads(
    subprocess.check_output(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=repo_root,
        text=True,
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

if output_dir.exists():
    shutil.rmtree(output_dir)
output_dir.mkdir(parents=True)

suffix = ".exe" if sys.platform == "win32" else ""
for name in binaries:
    binary = build_dir / f"{name}{suffix}"
    if not binary.is_file():
        raise FileNotFoundError(f"Built executable not found: {binary}")
    shutil.copy2(binary, output_dir / binary.name)
    print(f"Packaged {binary.name}")

print(f"Packaged default-member binaries: {output_dir}")
