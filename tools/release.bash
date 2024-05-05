#!/bin/bash
# Build all releases and then package everything
set -euo pipefail
PROJECT_DIR="$(realpath "$(dirname "$0")/..")"

echo "began releasing"

"${PROJECT_DIR}/tools/build-release.bash"
"${PROJECT_DIR}/tools/package-all.bash"

echo "finished releasing"