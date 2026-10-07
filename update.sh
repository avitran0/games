#!/bin/sh
set -eu

PORTS_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
LOG_FILE=${UPDATE_LOG:-"$PORTS_DIR/update.log"}
LOG_NAME=$(basename -- "$LOG_FILE")
exec >>"$LOG_FILE" 2>&1

log() {
    printf '[%s] %s\n' "$(date '+%Y-%m-%dT%H:%M:%S%z')" "$*"
}

TMP_DIR=
cleanup() {
    if [ -n "$TMP_DIR" ]; then
        rm -rf -- "$TMP_DIR" || log "WARNING: could not remove temporary directory $TMP_DIR"
    fi
}

finish() {
    status=$?
    trap - EXIT HUP INT TERM
    if [ "$status" -eq 0 ]; then
        log "Update completed successfully"
    else
        log "ERROR: update failed with exit status $status"
    fi
    cleanup
}

on_signal() {
    signal=$1
    code=$2
    log "ERROR: update interrupted by $signal"
    exit "$code"
}

trap finish EXIT
trap 'on_signal HUP 129' HUP
trap 'on_signal INT 130' INT
trap 'on_signal TERM 143' TERM

log "Starting update (pid $$); log file: $LOG_FILE"

API=https://api.github.com/repos/avitran0/games
TMP_DIR=$(mktemp -d)

log "Fetching latest successful workflow run"
curl -fsSL "$API/actions/workflows/games.yml/runs?branch=main&status=success&per_page=1" \
    -o "$TMP_DIR/runs.json"
RUN_ID=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["workflow_runs"][0]["id"])' "$TMP_DIR/runs.json")

log "Fetching artifact metadata for workflow run $RUN_ID"
curl -fsSL "$API/actions/runs/$RUN_ID/artifacts" -o "$TMP_DIR/artifacts.json"
ARTIFACT_ID=$(python3 -c 'import json,sys; print(next(a["id"] for a in json.load(open(sys.argv[1]))["artifacts"] if a["name"] == "games-aarch64" and not a["expired"]))' "$TMP_DIR/artifacts.json")

log "Downloading and extracting artifact $ARTIFACT_ID"
curl -fLsS "$API/actions/artifacts/$ARTIFACT_ID/zip" -o "$TMP_DIR/build.zip"
unzip -oq "$TMP_DIR/build.zip" -d "$TMP_DIR/build"

log "Installing build into $PORTS_DIR"
for SOURCE in "$TMP_DIR/build"/*; do
    [ -e "$SOURCE" ] || continue
    NAME=${SOURCE##*/}
    [ "$NAME" = "$(basename -- "$0")" ] && continue
    [ "$NAME" = "$LOG_NAME" ] && continue
    rm -rf "$PORTS_DIR/$NAME"
    cp -R "$SOURCE" "$PORTS_DIR/$NAME"
done

for FILE in "$PORTS_DIR"/*.sh "$PORTS_DIR"/*/game; do
    if [ -f "$FILE" ]; then chmod 755 "$FILE"; fi
done

log "Installed latest build in $PORTS_DIR"
