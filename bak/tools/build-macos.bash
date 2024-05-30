#!/bin/bash
set -euo pipefail
PROJECT_DIR="$(realpath "$(dirname "$0")/..")"
source "${PROJECT_DIR}/tools/common.lib.bash"

target="$1"

cd "${PROJECT_DIR}"

echo "Began building macos release: ${target}"

echo "Debug testing: ${target}"
cargo test --target="${target}"

echo "Building release: ${target}"
cargo build --release --target="${target}"

echo "Testing release: ${target}"
cargo test --release --target="${target}"

echo "Finished building macos release: ${target}"

