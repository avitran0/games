#!/usr/bin/env bash
set -euo pipefail

GAME_ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
PACKAGE_DIR="$GAME_ROOT/rocknix/avitrano"
CACHE_HOME="${XDG_CACHE_HOME:-$HOME/.cache}"
DISTRO_DIR="${ROCKNIX_BUILD_DIR:-$CACHE_HOME/rocknix-distribution}"
ROCKNIX_COMMIT="c445081a59518f37d9776e5412dd7b14910696f7"
PACKAGE_DEST="$DISTRO_DIR/projects/ROCKNIX/packages/games/avitrano"
OUTPUT_DIR="$GAME_ROOT/out/rocknix-ports"

if ! command -v podman >/dev/null 2>&1; then
  echo "podman is required on the Fedora build host" >&2
  exit 1
fi

if [ ! -d "$DISTRO_DIR/.git" ]; then
  mkdir -p "$(dirname -- "$DISTRO_DIR")"
  git clone --branch next https://github.com/ROCKNIX/distribution.git "$DISTRO_DIR"
fi
git -C "$DISTRO_DIR" fetch --quiet origin next
git -C "$DISTRO_DIR" checkout --detach --force "$ROCKNIX_COMMIT"

mkdir -p "$(dirname -- "$PACKAGE_DEST")"
rm -rf "$PACKAGE_DEST"
mkdir -p "$PACKAGE_DEST/sources"
cp "$PACKAGE_DIR/package.mk" "$PACKAGE_DEST/package.mk"

archive="$(mktemp)"
trap 'rm -f "$archive"' EXIT
VERSION="$(git -C "$GAME_ROOT" rev-parse HEAD)"
git -C "$GAME_ROOT" archive --format=tar --prefix=games/ HEAD | gzip -n > "$archive"
SOURCE_SHA256="$(sha256sum "$archive" | cut -d ' ' -f1)"
tar -xzf "$archive" --strip-components=1 -C "$PACKAGE_DEST/sources"
rm -f "$archive"
trap - EXIT
echo "Building games commit $VERSION"

export PROJECT=ROCKNIX DEVICE=H700 ARCH=aarch64
export AVITRANO_GAMES_VERSION="$VERSION"
export AVITRANO_GAMES_SHA256="$SOURCE_SHA256"
(
  cd "$DISTRO_DIR"
  ./scripts/get_env > .env
  podman pull ghcr.io/rocknix/rocknix-build:latest
  podman run --rm --init --userns=keep-id \
    --user "$(id -u):$(id -g)" --env-file .env \
    -v "$DISTRO_DIR:$DISTRO_DIR:Z" -w "$DISTRO_DIR" \
    ghcr.io/rocknix/rocknix-build:latest \
    bash -lc './scripts/build_mt avitrano'
)

STAGED_PORTS="$DISTRO_DIR/build.ROCKNIX-H700.aarch64/install_pkg/avitrano-${VERSION}/usr/share/avitrano/ports"
if [ ! -d "$STAGED_PORTS" ]; then
  echo "ROCKNIX build completed but staged ports were not found at:" >&2
  echo "  $STAGED_PORTS" >&2
  exit 1
fi

rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR"
cp -a "$STAGED_PORTS/." "$OUTPUT_DIR/"
echo "Built ROCKNIX ports from the local checkout in $OUTPUT_DIR"
