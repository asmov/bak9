#!/bin/bash
set -euo pipefail
PROJECT_DIR="$(realpath "$(dirname "$0")/..")"
source "${PROJECT_DIR}/tools/common.lib.bash"

target="$1"

cd "${PROJECT_DIR}"

echo "building windows release: $target"

echo "testing windows debugging: ${target}"
cargo test --target="${target}"

echo "building windows release: ${target}"
cargo build --release --target="${target}"

echo "testing windows release: ${target}"
cargo test --release --target="${target}"

echo "building msi: ${target}"
cargo wix

echo "finished building windows release"

