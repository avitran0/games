#!/usr/bin/env python3
"""Build games on ROCKNIX and stage them as Ports for manual copying."""

import json
from pathlib import Path
import shlex
import shutil
import subprocess

ROOT_DIR = Path(__file__).resolve().parent
ROCKNIX_MARKER = Path("/usr/bin/rocknix-info")
PORTS_DIR = ROOT_DIR / "out" / "rocknix-ports"
PORT_LAUNCH_SCRIPT = """#!/bin/sh
set -eu

PORTS_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
GAME_DIR=@GAME_DIR@
cd "$PORTS_DIR/$GAME_DIR"
exec ./game --fullscreen "$@" > game.log 2>&1
"""


def run(*command: str, capture_output: bool = False) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        cwd=ROOT_DIR,
        check=True,
        text=True,
        capture_output=capture_output,
    )


def workspace_binaries() -> list[str]:
    metadata = json.loads(
        run(
            "cargo",
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
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
    return binaries


def main() -> None:
    if not ROCKNIX_MARKER.is_file():
        raise SystemExit(
            f"{ROCKNIX_MARKER} is missing; this packaging script only runs on ROCKNIX"
        )
    binaries = workspace_binaries()
    run("cargo", "build", "--release")

    metadata = json.loads(
        run(
            "cargo",
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            capture_output=True,
        ).stdout
    )
    target_dir = Path(metadata["target_directory"])
    PORTS_DIR.mkdir(parents=True, exist_ok=True)

    for binary_name in binaries:
        game_name = binary_name.replace("_", " ").replace("-", " ").title()
        binary = target_dir / "release" / binary_name
        if not binary.is_file():
            raise FileNotFoundError(f"Built binary not found: {binary}")

        game_dir = PORTS_DIR / game_name
        game_dir.mkdir(parents=True, exist_ok=True)
        shutil.copy2(binary, game_dir / "game")

        launcher = PORTS_DIR / f"{game_name}.sh"
        launcher.write_text(
            PORT_LAUNCH_SCRIPT.replace("@GAME_DIR@", shlex.quote(game_name)),
            encoding="utf-8",
        )
        launcher.chmod(0o755)
        print(f"Installed {game_name} port: {launcher}")


if __name__ == "__main__":
    main()
