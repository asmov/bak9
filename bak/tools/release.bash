#!/bin/bash
# Build all releases and then package everything
set -euo pipefail
PROJECT_DIR="$(realpath "$(dirname "$0")/..")"

log "Began cleaning"

cd "$PROJECT_DIR"
cargo clean

if [ -f "${PROJECT_DIR}/pkg.cfg" ]; then
    source "${PROJECT_DIR}/pkg.cfg"

    ssh "$WINDOWS_SSH_HOST" "cd "$WINDOWS_SSH_WORKSPACE_DIR" && cargo clean"
    ssh "$MACOS_SSH_HOST" "cd "$MACOS_SSH_WORKSPACE_DIR" && cargo clean"
else
    log "No pkg.cfg file found, skipping remote cleaning"
fi

log "Finished cleaning"

log "Began releasing"

cargo build
cargo test
cargo build --release
cargo test --release

"${PROJECT_DIR}/tools/build.bash"
"${PROJECT_DIR}/tools/package.bash"

log "Finished releasing"