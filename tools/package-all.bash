#!/bin/bash
# Package all releases for every package type
# Expects tools/build-release.bash to have been run
set -euo pipefail
PROJECT_DIR="$(realpath "$(dirname "$0")/..")"

echo "began packaging everything"

"${PROJECT_DIR}/tools/package-tarball.bash"
"${PROJECT_DIR}/tools/package-debian.bash"
"${PROJECT_DIR}/tools/package-rpm.bash"
"${PROJECT_DIR}/tools/package-snap.bash"

echo "finished packaging everything"

