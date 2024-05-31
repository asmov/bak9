#!/bin/bash
set -euo pipefail
PROJECT_DIR="$(realpath "$(dirname "$0")/..")"
source "${PROJECT_DIR}/tools/common.lib.bash"
source_pkg_cfg

WINDOWS_SSH_PACKAGE_DIR="${WINDOWS_SSH_WORKSPACE_DIR}/${PACKAGE_SUBDIR}"

log "Began building native windows releases"

log "Pulling latest changes from git"

if [ -n "${WINDOWS_SSH_GIT_IDENTITY_FILE}" ]; then
    ssh -t "$WINDOWS_SSH_HOST" "cd "${WINDOWS_SSH_PACKAGE_DIR}" ; git pull"
    #ssh -t "$WINDOWS_SSH_HOST" "cd "${WINDOWS_SSH_PACKAGE_DIR}" ; ssh-add "${WINDOWS_SSH_GIT_IDENTITY_FILE}" ; git pull"
    #ssh -t "$WINDOWS_SSH_HOST" "cd "${WINDOWS_SSH_PACKAGE_DIR}" && eval \$(ssh-agent -s) && ssh-add "${WINDOWS_SSH_GIT_IDENTITY_FILE}" && git pull"
else
    ssh "$WINDOWS_SSH_HOST" "cd "${WINDOWS_SSH_PACKAGE_DIR}" ; git pull"
fi

mkdir -p "${TARGET_DIR}/pkg/msi"

for target in "${WINDOWS_NATIVE_RELEASE_TARGETS[@]}"; do
    log "remote building native windows release: ${target}"
    ssh "$WINDOWS_SSH_HOST" "cd "${WINDOWS_SSH_PACKAGE_DIR}" ; Set-ExecutionPolicy Bypass -Scope Process ; ./tools/build-windows.ps1 -Target '${target}' -Package '${PACKAGE_NAME}'"

    log "downloading build artifacts: ${target}"
    mkdir -p "${TARGET_DIR}/${target}/release"
    scp "${WINDOWS_SSH_HOST}:${WINDOWS_SSH_WORKSPACE_DIR}/target/${target}/release/${CARGO_BIN_NAME}.exe" "${TARGET_DIR}/${target}/release"
done

scp "${WINDOWS_SSH_HOST}:${WINDOWS_SSH_WORKSPACE_DIR}/target/wix/*.msi" "${TARGET_DIR}/pkg/msi"

for msi in "${TARGET_DIR}/pkg/msi"/*.msi; do
    sha256sum -b "${msi}" > "${msi}.sha256"
done

log "Finished remote building native windows releases"

