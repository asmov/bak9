#!/bin/bash
set -euo pipefail
PROJECT_DIR="$(realpath "$(dirname "$0")/..")"
source "${PROJECT_DIR}/tools/common.lib.bash"
source_pkg_cfg

echo "began remote building native windows releases"

mkdir -p "${TARGET_DIR}/pkg/msi"

for target in "${WINDOWS_NATIVE_RELEASE_TARGETS[@]}"; do
    echo "remote building native windows release: ${target}"
    ssh "$WINDOWS_SSH_HOST" "cd "${WINDOWS_SSH_WORKSPACE_DIR}/${PACKAGE_SUBDIR}" ; ./tools/build-windows.bash ${target}"

    echo "downloading build artifacts: ${target}"
    mkdir -p "${TARGET_DIR}/${target}/release"
    scp "${WINDOWS_SSH_HOST}:${WINDOWS_SSH_WORKSPACE_DIR}/target/${target}/release/${CARGO_BIN_NAME}.exe" "${TARGET_DIR}/${target}/release"
done

scp "${WINDOWS_SSH_HOST}:${WINDOWS_SSH_WORKSPACE_DIR}/target/wix/*.msi" "${TARGET_DIR}/pkg/msi"

for msi in "${TARGET_DIR}/pkg/msi"/*.msi; do
    sha256sum -b "${msi}" > "${msi}.sha256"
done

echo "finished remote building native windows releases"
