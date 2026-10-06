#!/usr/bin/env python3
"""Build workspace game binaries and assemble NextUI .pak directories."""

import json
from pathlib import Path
import shutil
import shlex
import subprocess

ROOT_DIR = Path(__file__).resolve().parent
TARGET = "aarch64-unknown-linux-gnu"
OUT_DIR = ROOT_DIR / "out" / "paks"
LAUNCH_SCRIPT = """#!/bin/sh
set -eu

PAK_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
cd "$PAK_DIR"
GAME=@GAME@
: "${LOGS_PATH:?LOGS_PATH is not set}"
mkdir -p "$LOGS_PATH"
exec ./game --fullscreen "$@" > "$LOGS_PATH/$GAME.log" 2>&1
"""


def run(
    *command: str, capture_output: bool = False
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command, cwd=ROOT_DIR, check=True, text=True, capture_output=capture_output
    )


def main() -> None:
    metadata = json.loads(
        run(
            "cargo", "metadata", "--no-deps", "--format-version", "1",
            capture_output=True,
        ).stdout
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

    run("cross", "build", "--target", TARGET, "--release")

    target_dir = Path(metadata["target_directory"])
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    for binary_name in binaries:
        pak_dir = OUT_DIR / f"{binary_name[:1].upper()}{binary_name[1:]}.pak"
        binary = target_dir / TARGET / "release" / binary_name
        if not binary.is_file():
            raise FileNotFoundError(f"Built binary not found: {binary}")

        if pak_dir.exists():
            shutil.rmtree(pak_dir)
        pak_dir.mkdir(parents=True)
        shutil.copy2(binary, pak_dir / "game")
        launch_script = pak_dir / "launch.sh"
        game_name = binary_name.replace("_", " ").replace("-", " ").title()
        launch_script.write_text(
            LAUNCH_SCRIPT.replace("@GAME@", shlex.quote(game_name)), encoding="utf-8"
        )
        launch_script.chmod(0o755)

        print(f"Created {pak_dir}")


if __name__ == "__main__":
    main()
