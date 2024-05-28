#!/bin/bash
# Package Debian .deb files for all releases
# Expects tools/build-release.bash to have been run
set -euo pipefail
shopt -s extglob
PROJECT_DIR="$(realpath "$(dirname "$0")/..")"
source "${PROJECT_DIR}/tools/common.lib.bash"

echo "began packaging for Debian"

DEB_DIR="${TARGET_DIR}/pkg/debian"
rm -rf "${DEB_DIR}"
mkdir -p "${DEB_DIR}"

for target in "${LINUX_RELEASE_TARGETS[@]}"; do
    echo "packaging .deb: ${target}"
    cargo deb --target "${target}" --no-build --no-strip --output="${DEB_DIR}"
done

for deb in "${DEB_DIR}"/*.deb; do
    sha256sum -b "${deb}" > "${deb}.sha256"
done
 
 echo "finished packaging for Debian"