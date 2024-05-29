#!/bin/bash
set -euo pipefail
PROJECT_DIR="$(realpath "$(dirname "$0")/..")"
source "${PROJECT_DIR}/tools/common.lib.bash"

target="$1"

cd "${PROJECT_DIR}"

echo "building macos release: $target"

echo "testing macos debugging: ${target}"
cargo test --target="${target}"

echo "building macos release: ${target}"
cargo build --release --target="${target}"

echo "testing macos release: ${target}"
cargo test --release --target="${target}"

echo "finished building macos release"

