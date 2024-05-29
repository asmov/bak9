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

CARGO_NAME="$(grep -m1 '^name' "${PROJECT_DIR}/Cargo.toml" | cut -d '"' -f 2)"
CARGO_VERSION="$(grep '^version' "${PROJECT_DIR}/Cargo.toml" | cut -d '"' -f 2)"
CARGO_VERSION_EXT="${CARGO_VERSION}-1"
CARGO_BIN_NAME="$(sed -n '/\[\[bin\]\]/,$p' "${PROJECT_DIR}/Cargo.toml" | grep '^name' | cut -d '"' -f 2)"

GIT_BRANCH="$(git rev-parse --abbrev-ref HEAD)"
