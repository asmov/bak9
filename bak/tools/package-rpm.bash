#!/bin/bash
# Package RPMs for all releases
# Expects tools/build-release.bash to have been run
set -euo pipefail
shopt -s extglob
PROJECT_DIR="$(realpath "$(dirname "$0")/..")"
source "${PROJECT_DIR}/tools/common.lib.bash"

echo "began packaging for Red Hat"

RPM_DIR="${TARGET_DIR}/pkg/rpm"
rm -rf "${RPM_DIR}"
mkdir -p "${RPM_DIR}"

cd "${PROJECT_DIR}"

for target in "${LINUX_RELEASE_TARGETS[@]}"; do
    echo "packaging rpm: ${target}"
    cargo generate-rpm --target "${target}" --output "${RPM_DIR}"
done

for rpm in "${RPM_DIR}"/*.rpm; do
    sha256sum -b "${rpm}" > "${rpm}.sha256"
done

echo "finished packaging for Red Hat"

