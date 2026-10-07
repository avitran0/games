#!/usr/bin/env python3
"""Build a Ports-layout ZIP from the workspace's default-member binaries."""

import json
from pathlib import Path
import shutil
import subprocess
import sys
import zipfile

PORT_LAUNCH_SCRIPT = """#!/bin/sh
set -eu
PORTS_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
GAME_DIR="$(basename -- "$0" .sh)"
cd "$PORTS_DIR/$GAME_DIR"
exec ./game "$@" > game.log 2>&1
"""

if len(sys.argv) != 3:
    raise SystemExit("usage: stage-binaries.py BUILD_DIR OUTPUT_DIR")

build_dir = Path(sys.argv[1])
output_dir = Path(sys.argv[2])
ports_dir = output_dir / "ports"
archive_path = output_dir / "games-ports-aarch64.zip"

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

if ports_dir.exists():
    shutil.rmtree(ports_dir)
if archive_path.exists():
    archive_path.unlink()
ports_dir.mkdir(parents=True)

for binary_name in binaries:
    game_name = binary_name.replace("_", " ").replace("-", " ").title()
    binary = build_dir / binary_name
    if not binary.is_file():
        raise FileNotFoundError(f"Built executable not found: {binary}")

    game_dir = ports_dir / game_name
    game_dir.mkdir()
    shutil.copy2(binary, game_dir / "game")

    launcher = ports_dir / f"{game_name}.sh"
    launcher.write_text(PORT_LAUNCH_SCRIPT, encoding="utf-8")
    launcher.chmod(0o755)
    print(f"Prepared {game_name} port")

with zipfile.ZipFile(archive_path, "w", compression=zipfile.ZIP_DEFLATED) as archive:
    for path in sorted(ports_dir.rglob("*")):
        if path.is_file():
            archive.write(path, path.relative_to(ports_dir))

print(f"Created {archive_path}")
