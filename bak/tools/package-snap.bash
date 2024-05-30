#!/bin/bash
# Package Snapcraft .snap files for all releases
# Expects tools/build-release.bash to have been run
set -euo pipefail
shopt -s extglob
PROJECT_DIR="$(realpath "$(dirname "$0")/..")"
source "${PROJECT_DIR}/tools/common.lib.bash"

log "Began packaging snaps"

SNAP_DIR="${TARGET_DIR}/pkg/snap"
rm -rf "${SNAP_DIR}"
mkdir -p "${SNAP_DIR}"

cd "${PROJECT_DIR}"
rm -f "${PROJECT_DIR}"/*.snap

snapcraft

mv "${PROJECT_DIR}"/*.snap "${SNAP_DIR}"

mv "${SNAP_DIR}"/bak9_*_amd64.snap "${SNAP_DIR}/bak9_${CARGO_VERSION}_amd64.snap"
mv "${SNAP_DIR}"/bak9_*_arm64.snap "${SNAP_DIR}/bak9_${CARGO_VERSION}_arm64.snap"
mv "${SNAP_DIR}"/bak9_*_armhf.snap "${SNAP_DIR}/bak9_${CARGO_VERSION}_armhf.snap"

for snap in "${SNAP_DIR}"/*.snap; do
    sha256sum -b "${snap}" > "${snap}.sha256"
done

log "Finished packaging snaps"
