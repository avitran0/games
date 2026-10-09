#!/usr/bin/env python3
import json
from pathlib import Path
import shutil
import subprocess
import sys
import xml.etree.ElementTree as ET

PORT_LAUNCH_SCRIPT = """#!/bin/sh
set -eu
PORTS_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
GAME_DIR="$(basename -- "$0" .sh)"
export LD_LIBRARY_PATH=/lib:/usr/lib
cd "$PORTS_DIR/$GAME_DIR"
exec ./game "$@" > game.log 2>&1
"""

ES_FIELDS = {
    "description": "desc",
    "author": "developer",
    "players": "players",
}


def fail(message):
    raise SystemExit(message)


def add_text(parent, tag, text):
    element = ET.SubElement(parent, tag)
    element.text = str(text)


if len(sys.argv) != 3:
    raise SystemExit("usage: stage-binaries.py BUILD_DIR OUTPUT_DIR")

build_dir = Path(sys.argv[1]).resolve()
ports_dir = Path(sys.argv[2]).resolve()
repo_root = Path(__file__).resolve().parents[2]
converter = build_dir / "pxs-to-png"

metadata = json.loads(
    subprocess.check_output(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=repo_root,
        text=True,
    )
)
packages = {package["id"]: package for package in metadata["packages"]}
binaries = [
    (target["name"], packages[package_id])
    for package_id in metadata["workspace_default_members"]
    for target in packages[package_id]["targets"]
    if "bin" in target["kind"]
]
if not binaries:
    fail("No binary targets found among workspace default members")

if ports_dir.exists():
    shutil.rmtree(ports_dir)
ports_dir.mkdir(parents=True)

game_list = ET.Element("gameList")
for binary_name, package in binaries:
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

    package_metadata = package.get("metadata") or {}
    game_metadata = package_metadata.get("game")
    if game_metadata is not None:
        if not isinstance(game_metadata, dict):
            fail(f"{package['name']}: [package.metadata.game] must be a table")
        allowed = {"title", "description", "author", "players", "cover", "logo"}
        unknown = set(game_metadata) - allowed
        if unknown:
            fail(f"{package['name']}: unsupported game metadata keys: {', '.join(sorted(unknown))}")

        entry = ET.SubElement(game_list, "game")
        add_text(entry, "path", f"./{game_name}.sh")
        add_text(entry, "name", game_metadata.get("title", game_name))
        for source_field, es_field in ES_FIELDS.items():
            value = game_metadata.get(source_field)
            if value is not None:
                add_text(entry, es_field, value)

        image_path = None
        for source_field, output_name in (("cover", "cover.png"), ("logo", "logo.png")):
            artwork = game_metadata.get(source_field)
            if artwork is None:
                continue
            artwork_path = Path(package["manifest_path"]).parent / artwork
            if not artwork_path.is_file():
                fail(f"{package['name']}: {source_field} file does not exist: {artwork_path}")
            if artwork_path.suffix.lower() != ".pxs":
                fail(f"{package['name']}: {source_field} must be a .pxs sprite: {artwork_path}")
            if not converter.is_file():
                fail(f"PXS converter not found: {converter}; build the pxs-to-png package first")
            artwork_output = game_dir / output_name
            subprocess.run([str(converter), str(artwork_path), str(artwork_output)], check=True)
            relative_image_path = f"./{game_name}/{output_name}"
            if source_field == "cover" or image_path is None:
                image_path = relative_image_path
            if source_field == "logo":
                add_text(entry, "marquee", relative_image_path)
        if image_path is not None:
            add_text(entry, "image", image_path)

    print(f"Prepared {game_name} port")

if len(game_list):
    ET.indent(game_list, space="    ")
    ET.ElementTree(game_list).write(
        ports_dir / "gamelist-metadata.xml",
        encoding="utf-8",
        xml_declaration=True,
    )
    print("Prepared EmulationStation metadata")

print(f"Prepared Ports directory: {ports_dir}")
