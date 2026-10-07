#!/bin/sh
set -eu

PORTS_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
API=https://api.github.com/repos/avitran0/games
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT HUP INT TERM

curl -fsSL "$API/actions/workflows/games.yml/runs?branch=main&status=success&per_page=1" \
    -o "$TMP_DIR/runs.json"
RUN_ID=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["workflow_runs"][0]["id"])' "$TMP_DIR/runs.json")

curl -fsSL "$API/actions/runs/$RUN_ID/artifacts" -o "$TMP_DIR/artifacts.json"
ARTIFACT_ID=$(python3 -c 'import json,sys; print(next(a["id"] for a in json.load(open(sys.argv[1]))["artifacts"] if a["name"] == "games-aarch64" and not a["expired"]))' "$TMP_DIR/artifacts.json")

curl -fLsS "$API/actions/artifacts/$ARTIFACT_ID/zip" -o "$TMP_DIR/build.zip"
unzip -oq "$TMP_DIR/build.zip" -d "$TMP_DIR/build"

for SOURCE in "$TMP_DIR/build"/*; do
    [ -e "$SOURCE" ] || continue
    NAME=${SOURCE##*/}
    [ "$NAME" = "$(basename -- "$0")" ] && continue
    rm -rf "$PORTS_DIR/$NAME"
    cp -R "$SOURCE" "$PORTS_DIR/$NAME"
done

for FILE in "$PORTS_DIR"/*.sh "$PORTS_DIR"/*/game; do
    if [ -f "$FILE" ]; then chmod 755 "$FILE"; fi
done

echo "Installed latest build in $PORTS_DIR"
