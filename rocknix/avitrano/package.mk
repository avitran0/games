# SPDX-License-Identifier: EUPL-1.2

PKG_NAME="avitrano"
PKG_VERSION="${AVITRANO_GAMES_VERSION:?build helper must set AVITRANO_GAMES_VERSION}"
PKG_SHA256="${AVITRANO_GAMES_SHA256:?build helper must set AVITRANO_GAMES_SHA256}"
PKG_LICENSE="EUPL-1.2"
PKG_SITE="https://codeberg.org/avitrano/games"
PKG_URL=""
PKG_DEPENDS_HOST="cargo:host"
PKG_DEPENDS_TARGET="SDL3"
PKG_LONGDESC="avitrano's games"
PKG_TOOLCHAIN="manual"

# The project-local config is for native development, not ROCKNIX cross-builds.
# Use the linker and pkg-config configuration installed by ROCKNIX's Rust package.
pre_configure_target() {
  rm -rf "${PKG_BUILD}/.cargo"
}

make_target() {
  cd "${PKG_BUILD}"
  cargo build --locked --release --target "${TARGET_NAME}"
}

makeinstall_target() {
  local ports_dir="${INSTALL}/usr/share/avitrano/ports"
  local bin_dir="${CARGO_TARGET_DIR}/${TARGET_NAME}/release"
  local binary_targets

  cd "${PKG_BUILD}"
  binary_targets="$(cargo metadata --no-deps --format-version 1 | python3 -c '
import json
import sys

metadata = json.load(sys.stdin)
packages = {package["id"]: package for package in metadata["packages"]}
for package_id in metadata["workspace_default_members"]:
    for target in packages[package_id]["targets"]:
        if "bin" in target["kind"]:
            name = target["name"]
            display_name = name.replace("_", " ").replace("-", " ").title()
            print(f"{name}\t{display_name}")
')"
  if [ -z "${binary_targets}" ]; then
    echo "No binary targets found among the workspace default members" >&2
    return 1
  fi

  install_port() {
    local binary_name="$1"
    local display_name="$2"
    local game_dir="${ports_dir}/${display_name}"
    local launcher="${ports_dir}/${display_name}.sh"

    install -d "${game_dir}"
    install -m 0755 "${bin_dir}/${binary_name}" "${game_dir}/game"

    cat >"${launcher}" <<EOF
#!/bin/sh
set -eu
PORTS_DIR="\$(CDPATH= cd -- "\$(dirname -- "\$0")" && pwd)"
cd "\$PORTS_DIR/${display_name}"
exec ./game --fullscreen "\$@" > game.log 2>&1
EOF
    chmod 0755 "${launcher}"
  }

  while IFS=$'\t' read -r binary_name display_name; do
    [ -n "${binary_name}" ] && install_port "${binary_name}" "${display_name}"
  done <<<"${binary_targets}"
}
