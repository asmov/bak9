#!/bin/bash
# Build all releases
set -euo pipefail
PROJECT_DIR="$(realpath "$(dirname "$0")/..")"
source "${PROJECT_DIR}/tools/common.lib.bash"

echo "began building releases"

for target in "${LINUX_RELEASE_TARGETS[@]}"; do
    echo "building linux release: ${target}"
    cross build --release --target "${target}"
done

for target in "${WINDOWS_CROSS_RELEASE_TARGETS[@]}"; do
    echo "building windows cross release: ${target}"
    cross build --release --target "${target}"
done

for target in "${MACOS_CROSS_RELEASE_TARGETS[@]}"; do
    echo "building macos cross release: ${target}"
    cross build --release --target "${target}"
done

if [ -f "${PROJECT_DIR}/pkg.cfg" ]; then
    source "${PROJECT_DIR}/pkg.cfg"

    echo "remote building native windows releases"

    mkdir -p "${PROJECT_DIR}/target/pkg/msi"

    for target in "${WINDOWS_NATIVE_RELEASE_TARGETS[@]}"; do
        echo "remote building native windows release: ${target}"
        ssh "$WINDOWS_SSH_HOST" "cd "$WINDOWS_SSH_PROJECT_DIR" ; cargo build --release --target="${target}""
        echo "remote building msi: ${target}"
        ssh "$WINDOWS_SSH_HOST" "cd "$WINDOWS_SSH_PROJECT_DIR" ; cargo wix"
        mkdir -p "${PROJECT_DIR}/target/${target}/release"
        echo "downloading build artifacts: ${target}"
        scp "${WINDOWS_SSH_HOST}:${WINDOWS_SSH_PROJECT_DIR}/target/${target}/release/${CARGO_BIN_NAME}.exe" "${PROJECT_DIR}/target/${target}/release"
    done
    
    scp "${WINDOWS_SSH_HOST}:${WINDOWS_SSH_PROJECT_DIR}/target/wix/*.msi" "${PROJECT_DIR}/target/pkg/msi"

    for msi in "${PROJECT_DIR}/target/pkg/msi"/*.msi; do
        sha256sum -b "${msi}" > "${msi}.sha256"
    done

    echo "finished remote building native windows releases"
    echo "remote building native macos releases"

    for target in "${MACOS_NATIVE_RELEASE_TARGETS[@]}"; do
        echo "remote building native macos release: ${target}"
        ssh "$MACOS_SSH_HOST" "cd "$MACOS_SSH_PROJECT_DIR" && cargo build --release --target="${target}""
        mkdir -p "${PROJECT_DIR}/target/${target}/release"
        echo "downloading build artifacts: ${target}"
        scp "${MACOS_SSH_HOST}:${MACOS_SSH_PROJECT_DIR}/target/${target}/release/${CARGO_BIN_NAME}" "${PROJECT_DIR}/target/${target}/release"
    done

    echo "finished remote building native macos releases"
else
    echo "pkg.cfg not found, skipping remote building of native releases"
fi

echo "finished building releases"
