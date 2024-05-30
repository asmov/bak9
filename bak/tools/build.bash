#!/bin/bash
# Build all releases
set -euo pipefail
PROJECT_DIR="$(realpath "$(dirname "$0")/..")"
source "${PROJECT_DIR}/tools/common.lib.bash"


usage() {
    cat 1>&2 << EOF 
Usage: $(basename "$0") [OPTIONS]
Build all releases.

OPTIONS:
  -l    Local. Skip remote building.
EOF
    exit 1
}

SKIP_REMOTE=0

while getopts ":l" opt; do
    case "${opt}" in
        l)
            SKIP_REMOTE=1
            ;;
        *)
            usage
            ;;
    esac
done

shift $((OPTIND-1))

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

if [ -f "${PROJECT_DIR}/pkg.cfg.bash" ] && [ $SKIP_REMOTE -eq 0 ]; then
    source "${PROJECT_DIR}/pkg.cfg.bash"

    "${PROJECT_DIR}/tools/remote-build-windows.bash"

    "${PROJECT_DIR}/tools/remote-build-macos.bash"

    echo "remote building native macos releases"

    echo "finished remote building native macos releases"
else
    if [ $SKIP_REMOTE -eq 1 ]; then
        echo "skipping remote building of native releases"
    else
        echo "pkg.cfg not found, skipping remote building of native releases"
    fi
fi

echo "finished building releases"
