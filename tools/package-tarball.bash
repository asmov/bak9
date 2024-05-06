#!/bin/bash
# Package tarballs for all releases
# Expects tools/build-release.bash to have been run
set -euo pipefail
PROJECT_DIR="$(realpath "$(dirname "$0")/..")"
source "${PROJECT_DIR}/tools/common.lib.bash"

echo "began packaging tarballs"

TARBALL_DIR="${PROJECT_DIR}/target/pkg/tarball"
rm -rf "${TARBALL_DIR}"
mkdir -p "${TARBALL_DIR}"

# Create a template for the rest of the releases
TARBALL_TEMPLATE_DIR="${TARBALL_DIR}/template"
mkdir -p "${TARBALL_TEMPLATE_DIR}"
cp "${PROJECT_DIR}/README.md" "${TARBALL_TEMPLATE_DIR}"
cp "${PROJECT_DIR}/LICENSE.txt" "${TARBALL_TEMPLATE_DIR}"
cp "${PROJECT_DIR}/COPYING.txt" "${TARBALL_TEMPLATE_DIR}"

for target in "${RELEASE_TARGETS[@]}"; do
    echo "packaging tarball: ${target}"
    package_dir_name="${CARGO_NAME}_${CARGO_VERSION}_${target//_/-}"
    package_dir="${TARBALL_DIR}/${package_dir_name}"
    mkdir -p "${package_dir}"
    rsync -a "${TARBALL_TEMPLATE_DIR}/" "${package_dir}"

    if [[ "$target" != *"windows"* ]]; then
        cp "${PROJECT_DIR}/target/${target}/release/${CARGO_BIN_NAME}" "${package_dir}"
    else
        cp "${PROJECT_DIR}/target/${target}/release/${CARGO_BIN_NAME}.exe" "${package_dir}"
    fi

    cd "${package_dir}/.."
    tar cf "${package_dir_name}.tar.xz" --use-compress-program='xz -T0' "${package_dir_name}"
done

echo "finished packaging tarballs"

