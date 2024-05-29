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

    echo "began remote building native windows releases"

    for target in "${WINDOWS_NATIVE_RELEASE_TARGETS[@]}"; do
    #todo: call remote build instead
        ssh "$WINDOWS_SSH_HOST" "cd "$WINDOWS_SSH_WORKSPACE_DIR/${PACKAGE_SUBDIR}" && ./tools/build-windows.bash "$target""
    done

    echo "finished remote building native windows releases"


    echo "remote building native macos releases"

    for target in "${MACOS_NATIVE_RELEASE_TARGETS[@]}"; do
        echo "remote building native macos release: ${target}"
        ssh "$MACOS_SSH_HOST" "cd "$MACOS_SSH_WORKSPACE_DIR" && cargo build --release --target="${target}""
        mkdir -p "${TARGET_DIR}/${target}/release"
        echo "downloading build artifacts: ${target}"
        scp "${MACOS_SSH_HOST}:${MACOS_SSH_WORKSPACE_DIR}/target/${target}/release/${CARGO_BIN_NAME}" "${TARGET_DIR}/${target}/release"
    done

    echo "finished remote building native macos releases"
else
    echo "pkg.cfg not found, skipping remote building of native releases"
fi

echo "finished building releases"
