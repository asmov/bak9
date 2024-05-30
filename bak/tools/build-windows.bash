#!/bin/bash
set -euo pipefail
PROJECT_DIR="$(realpath "$(dirname "$0")/..")"
source "${PROJECT_DIR}/tools/common.lib.bash"

target="$1"

cd "${PROJECT_DIR}"

echo "began building windows release: ${target}"

echo "debug testing: ${target}"
cargo.exe test --target="${target}"

echo "building release: ${target}"
cargo.exe build --release --target="${target}"

echo "testing release: ${target}"
cargo.exe test --release --target="${target}"

echo "building msi: ${target}"
cargo.exe wix

echo "finished building windows release: ${target}"

