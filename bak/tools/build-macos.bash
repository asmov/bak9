#!/bin/bash
set -euo pipefail
PROJECT_DIR="$(realpath "$(dirname "$0")/..")"
source "${PROJECT_DIR}/tools/common.lib.bash"

target="$1"

cd "${PROJECT_DIR}"

echo "began building macos release: ${target}"

echo "debug testing: ${target}"
cargo test --target="${target}"

echo "building release: ${target}"
cargo build --release --target="${target}"

echo "testing release: ${target}"
cargo test --release --target="${target}"

echo "finished building macos release: ${target}"

