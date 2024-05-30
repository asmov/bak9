#!/bin/bash
# Common library for tools
set -euo pipefail

WORKSPACE_DIR="$(realpath "$(dirname "$(cargo locate-project --workspace --message-format=plain)")")"
PACKAGE_DIR="$(realpath "$(dirname "$(cargo locate-project --message-format=plain)")")"
PACKAGE_SUBDIR="${PACKAGE_DIR##${WORKSPACE_DIR}/}"
TARGET_DIR="${WORKSPACE_DIR}/target"

TARGET_LINUX_X86_64="x86_64-unknown-linux-gnu"
TARGET_LINUX_ARM_64="aarch64-unknown-linux-gnu"
TARGET_LINUX_ARM_V7="armv7-unknown-linux-gnueabihf"
TARGET_WINDOWS_X86_64_MSVC="x86_64-pc-windows-msvc"
TARGET_WINDOWS_ARM_64="aarch64-pc-windows-msvc"
TARGET_WINDOWS_X86_64_GNU="x86_64-pc-windows-gnu"
TARGET_MACOS_ARM_64="aarch64-apple-darwin"

RELEASE_TARGETS=(
  "${TARGET_LINUX_X86_64}"
  "${TARGET_LINUX_ARM_64}"
  "${TARGET_LINUX_ARM_V7}"
  "${TARGET_WINDOWS_X86_64_MSVC}"
  "${TARGET_WINDOWS_ARM_64}"
  "${TARGET_WINDOWS_X86_64_GNU}"
  "${TARGET_MACOS_ARM_64}"
)

LINUX_RELEASE_TARGETS=(
  "${TARGET_LINUX_X86_64}"
  "${TARGET_LINUX_ARM_64}"
  "${TARGET_LINUX_ARM_V7}"
)

WINDOWS_NATIVE_RELEASE_TARGETS=(
  "${TARGET_WINDOWS_X86_64_MSVC}"
)

WINDOWS_CROSS_RELEASE_TARGETS=(
  "${TARGET_WINDOWS_X86_64_GNU}"
)

MACOS_CROSS_RELEASE_TARGETS=()

MACOS_NATIVE_RELEASE_TARGETS=(
  "${TARGET_MACOS_ARM_64}"
)

CARGO_NAME="$(grep -m1 '^name' "${PACKAGE_DIR}/Cargo.toml" | cut -d '"' -f 2)"
CARGO_VERSION="$(grep '^version' "${PACKAGE_DIR}/Cargo.toml" | cut -d '"' -f 2)"
CARGO_VERSION_EXT="${CARGO_VERSION}-1"
CARGO_BIN_NAME="$(sed -n '/\[\[bin\]\]/,$p' "${PACKAGE_DIR}/Cargo.toml" | grep '^name' | cut -d '"' -f 2)"

GIT_BRANCH="$(git rev-parse --abbrev-ref HEAD)"

COLOR_BOLD_RED='\033[1;31m'
COLOR_NONE='\033[0m'

echo_error() {
  echo -e "${COLOR_BOLD_RED}error:${COLOR_NONE} ${1}" 1>&2
}

error() {
  echo_error "${1}"
  exit 1
}

source_pkg_cfg() {
  if [ -f "${PACKAGE_DIR}/pkg.cfg" ]; then
    source "${PACKAGE_DIR}/pkg.cfg"
  else
    error "File not found: ${PACKAGE_DIR}/pkg.cfg"
  fi
}

