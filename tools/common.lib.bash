#!/bin/bash
# Common library for tools
set -euo pipefail

TARGET_LINUX_X86_64="x86_64-unknown-linux-gnu"
TARGET_LINUX_ARM_64="aarch64-unknown-linux-gnu"
TARGET_LINUX_ARM_V7="armv7-unknown-linux-gnueabihf"
TARGET_WINDOWS_X86_64="x86_64-pc-windows-gnu"

RELEASE_TARGETS=(
  "${TARGET_LINUX_X86_64}"
  "${TARGET_LINUX_ARM_64}"
  "${TARGET_LINUX_ARM_V7}"
  "${TARGET_WINDOWS_X86_64}"
)

CARGO_NAME="$(grep -m1 '^name' "${PROJECT_DIR}/Cargo.toml" | cut -d '"' -f 2)"
CARGO_VERSION="$(grep '^version' "${PROJECT_DIR}/Cargo.toml" | cut -d '"' -f 2)"
CARGO_BIN_NAME="$(sed -n '/\[\[bin\]\]/,$p' "${PROJECT_DIR}/Cargo.toml" | grep '^name' | cut -d '"' -f 2)"
