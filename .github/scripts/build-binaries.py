#!/usr/bin/env python3
"""Build each default-member binary with static-link flags on its final link only."""

import json
import subprocess

TARGET = "aarch64-unknown-linux-gnu"

metadata = json.loads(
    subprocess.check_output(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"], text=True
    )
)
packages = {package["id"]: package for package in metadata["packages"]}
binaries = [
    (packages[package_id]["name"], target["name"])
    for package_id in metadata["workspace_default_members"]
    for target in packages[package_id]["targets"]
    if "bin" in target["kind"]
]
if not binaries:
    raise SystemExit("No binary targets found among workspace default members")

for package, binary in binaries:
    print(f"Building {binary} with static-link flags on the executable only")
    subprocess.run(
        [
            "cargo",
            "rustc",
            "--release",
            "--target",
            TARGET,
            "--package",
            package,
            "--bin",
            binary,
            "--",
            "-C",
            "target-feature=+crt-static",
            "-C",
            "link-arg=-static",
        ],
        check=True,
    )
