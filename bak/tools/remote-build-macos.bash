#!/bin/bash
set -euo pipefail
PROJECT_DIR="$(realpath "$(dirname "$0")/..")"
source "${PROJECT_DIR}/tools/common.lib.bash"
source_pkg_cfg

echo "began remote building native macos releases"

echo "pulling latest changes from git"
ssh -t "$MACOS_SSH_HOST" "cd "${MACOS_SSH_WORKSPACE_DIR}/${PACKAGE_SUBDIR}" && eval \$(ssh-agent -s) && ssh-add ${MACOS_SSH_IDENTITY} && git pull"
echo "hi"

for target in "${MACOS_NATIVE_RELEASE_TARGETS[@]}"; do
    echo "remote building native macos release: ${target}"
    ssh "$MACOS_SSH_HOST" "cd "${MACOS_SSH_WORKSPACE_DIR}/${PACKAGE_SUBDIR}" && ./tools/build-macos.bash ${target}"

    echo "downloading build artifacts: ${target}"
    mkdir -p "${TARGET_DIR}/${target}/release"
    scp "${MACOS_SSH_HOST}:${MACOS_SSH_WORKSPACE_DIR}/target/${target}/release/${CARGO_BIN_NAME}" "${TARGET_DIR}/${target}/release"
done

echo "finished remote building native macos releases"
