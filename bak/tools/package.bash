#!/bin/bash
# Package all releases for every package type
# Expects tools/build-release.bash to have been run
set -euo pipefail
PROJECT_DIR="$(realpath "$(dirname "$0")/..")"
source "${PROJECT_DIR}/tools/common.lib.bash"

log "Began packaging linux releases"

"${PROJECT_DIR}/tools/package-tarball.bash"
"${PROJECT_DIR}/tools/package-debian.bash"
"${PROJECT_DIR}/tools/package-rpm.bash"
"${PROJECT_DIR}/tools/package-snap.bash"

log "Finished packaging linux releases"

