#!/bin/bash
set -euo pipefail
PROJECT_DIR="$(realpath "$(dirname "$0")/..")"
source "${PROJECT_DIR}/tools/common.lib.bash"
source_pkg_cfg

MACOS_SSH_PACKAGE_DIR="${MACOS_SSH_WORKSPACE_DIR}/${PACKAGE_SUBDIR}"

log "Began remote building native macos releases"

log "Pulling latest changes from git"

if [ -n "${MACOS_SSH_GIT_IDENTITY_FILE}" ]; then
    ssh -t "$MACOS_SSH_HOST" "cd "${MACOS_SSH_PACKAGE_DIR}" && eval \$(ssh-agent -s) && ssh-add "${MACOS_SSH_GIT_IDENTITY_FILE}" && git pull"
else
    ssh "$MACOS_SSH_HOST" "cd "${MACOS_SSH_PACKAGE_DIR}" && git pull"
fi

for target in "${MACOS_NATIVE_RELEASE_TARGETS[@]}"; do
    log "Remote building native macos release: ${target}"
    ssh "$MACOS_SSH_HOST" "cd "${MACOS_SSH_PACKAGE_DIR}" && ./tools/build-macos.bash ${target}"

    log "Downloading build artifacts: ${target}"
    mkdir -p "${TARGET_DIR}/${target}/release"
    scp "${MACOS_SSH_HOST}:${MACOS_SSH_WORKSPACE_DIR}/target/${target}/release/${CARGO_BIN_NAME}" "${TARGET_DIR}/${target}/release"
done

log "Finished remote building native macos releases"
