#!/bin/bash
# Common library for tools
set -euo pipefail

WSL='/mnt/c/Program Files/WSL/wsl.exe'
if [ -f "${WSL}" ]; then
    CARGO=""${WSL}" cargo.exe"
else
    CARGO="cargo"
fi

WORKSPACE_DIR="$(realpath "$(dirname "$($CARGO locate-project --workspace --message-format=plain)")")"
PACKAGE_DIR="$(realpath "$(dirname "$($CARGO locate-project --message-format=plain)")")"
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

COLOR_RED='\033[0;31m'
COLOR_BOLD_RED='\033[1;31m'
COLOR_GREEN='\033[0;32m'
COLOR_NONE='\033[0m'

log() {
  log_prefix "bak9" "${1}"
}

log_prefix() {
  echo -e "${COLOR_GREEN}[$(date "+%H:%M:%S") ${1}]${COLOR_NONE} ${2}"
}

echo_error() {
  echo -e "${COLOR_BOLD_RED}error:${COLOR_NONE} ${1}" 1>&2
}

log_error_prefix() {
  echo -e "${COLOR_RED}[$(date "+%H:%M:%S") ${1}]${COLOR_NONE} ${COLOR_BOLD_RED}error:${COLOR_NONE} ${2}"
}

log_error() {
  log_error_prefix "bak9" "${1}"
  exit 1
}

error() {
  echo_error "${1}"
  exit 1
}

source_pkg_cfg() {
  if [ -f "${PACKAGE_DIR}/pkg.cfg.bash" ]; then
    source "${PACKAGE_DIR}/pkg.cfg.bash"
  else
    error "File not found: ${PACKAGE_DIR}/pkg.cfg.bash"
  fi
}

catch_err() {
    log_error "Unable to proceed"
}

trap 'catch_err' ERR

