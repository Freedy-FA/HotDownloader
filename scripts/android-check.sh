#!/usr/bin/env bash
# 在本机用 NDK 交叉编译检查 Android target（aarch64），无需完整 APK 构建。
# 用法：bash scripts/android-check.sh
set -euo pipefail

NDK_DIR="D:/languages/Android/ndk/28.2.13676358"
NDK_BIN="$NDK_DIR/toolchains/llvm/prebuilt/windows-x86_64/bin"

export CC_aarch64_linux_android="$NDK_BIN/aarch64-linux-android24-clang.cmd"
export CXX_aarch64_linux_android="$NDK_BIN/aarch64-linux-android24-clang++.cmd"
export AR_aarch64_linux_android="$NDK_BIN/llvm-ar.exe"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$NDK_BIN/aarch64-linux-android24-clang.cmd"
export ANDROID_NDK_HOME="$NDK_DIR"
export ANDROID_NDK_ROOT="$NDK_DIR"

cd "$(dirname "$0")/../src-tauri"
cargo check --target aarch64-linux-android "$@"
