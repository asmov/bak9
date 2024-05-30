#!/bin/bash
set -euo pipefail
PROJECT_DIR="$(realpath "$(dirname "$0")/..")"
EXE=".exe"
source "${PROJECT_DIR}/tools/common.lib.bash"

target="$1"

cd "${PROJECT_DIR}"

log "Began building windows release: ${target}"

log "Debug testing: ${target}"
cargo.exe test --target="${target}"

log "Building release: ${target}"
cargo.exe build --release --target="${target}"

log "Testing release: ${target}"
cargo.exe test --release --target="${target}"

log "Building msi: ${target}"
cargo.exe wix

log "Finished building windows release: ${target}"

