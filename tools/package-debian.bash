#!/bin/bash
# Package Debian .deb files for all releases
# Expects tools/build-release.bash to have been run
set -euo pipefail
PROJECT_DIR="$(realpath "$(dirname "$0")/..")"
source "${PROJECT_DIR}/tools/common.lib.bash"

echo "began packaging for Debian"

TARBALL_DIR="${PROJECT_DIR}/target/pkg/debian"
rm -rf "${TARBALL_DIR}"
mkdir -p "${TARBALL_DIR}"

for target in "${RELEASE_TARGETS[@]}"; do
    [[ "$target" != *"linux"* ]] &&
        continue

    echo "packaging .deb: ${target}"
    cargo deb --target "${target}" --no-build --output="${TARBALL_DIR}"
done
 
 echo "finished packaging for Debian"