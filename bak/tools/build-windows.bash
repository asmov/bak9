#!/bin/bash
set -euo pipefail
PROJECT_DIR="$(realpath "$(dirname "$0")/..")"
source "${PROJECT_DIR}/tools/common.lib.bash"

target="$1"

cd "${PROJECT_DIR}"

log "Began building windows release: ${target}"

log "Debug testing: ${target}"
$CARGO test --target="${target}"

log "Building release: ${target}"
$CARGO build --release --target="${target}"

log "Testing release: ${target}"
$CARGO test --release --target="${target}"

log "Building msi: ${target}"
$CARGO wix

log "Finished building windows release: ${target}"

