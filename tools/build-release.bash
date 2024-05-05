#!/bin/bash
# Build all releases
set -euo pipefail
PROJECT_DIR="$(realpath "$(dirname "$0")/..")"
source "${PROJECT_DIR}/tools/common.lib.bash"

echo "began building releases"

for target in "${RELEASE_TARGETS[@]}"; do
    echo "building release: ${target}"
    cross build --release --target "${target}"
done

echo "finished building releases"
