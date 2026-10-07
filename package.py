#!/usr/bin/env python3
"""Build workspace game binaries and assemble NextUI .pak and EmulationStation ports."""

import json
from pathlib import Path
import shutil
import shlex
import subprocess

ROOT_DIR = Path(__file__).resolve().parent
TARGET = "aarch64-unknown-linux-gnu"
PAKS_DIR = ROOT_DIR / "out" / "paks"
PORTS_DIR = ROOT_DIR / "out" / "ports"
PAK_LAUNCH_SCRIPT = """#!/bin/sh
set -eu

PAK_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
cd "$PAK_DIR"
GAME=@GAME@
: "${LOGS_PATH:?LOGS_PATH is not set}"
mkdir -p "$LOGS_PATH"
exec ./game --fullscreen "$@" > "$LOGS_PATH/$GAME.log" 2>&1
"""
PORT_LAUNCH_SCRIPT = """#!/bin/sh
set -eu

PORTS_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
GAME_DIR=@GAME_DIR@
cd "$PORTS_DIR/$GAME_DIR"
exec ./game --fullscreen "$@" > game.log 2>&1
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
    PAKS_DIR.mkdir(parents=True, exist_ok=True)
    if PORTS_DIR.exists():
        shutil.rmtree(PORTS_DIR)
    PORTS_DIR.mkdir(parents=True)

    for binary_name in binaries:
        game_name = binary_name.replace("_", " ").replace("-", " ").title()
        pak_dir = PAKS_DIR / f"{binary_name[:1].upper()}{binary_name[1:]}.pak"
        binary = target_dir / TARGET / "release" / binary_name
        if not binary.is_file():
            raise FileNotFoundError(f"Built binary not found: {binary}")

        if pak_dir.exists():
            shutil.rmtree(pak_dir)
        pak_dir.mkdir(parents=True)
        shutil.copy2(binary, pak_dir / "game")
        launch_script = pak_dir / "launch.sh"
        launch_script.write_text(
            PAK_LAUNCH_SCRIPT.replace("@GAME@", shlex.quote(game_name)), encoding="utf-8"
        )
        launch_script.chmod(0o755)

        port_game_dir = PORTS_DIR / game_name
        port_game_dir.mkdir(parents=True)
        shutil.copy2(binary, port_game_dir / "game")
        port_launcher = PORTS_DIR / f"{game_name}.sh"
        port_launcher.write_text(
            PORT_LAUNCH_SCRIPT.replace("@GAME_DIR@", shlex.quote(game_name)),
            encoding="utf-8",
        )
        port_launcher.chmod(0o755)

        print(f"Created {pak_dir}")
        print(f"Created {port_launcher} and {port_game_dir}")


if __name__ == "__main__":
    main()
