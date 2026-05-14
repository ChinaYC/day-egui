#!/bin/bash

echo "==========================================================="
echo "  eframe_template Android APK Build Script"
echo "==========================================================="

MODE="${1:-debug}"

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo could not be found. Please install Rust first."
    exit 1
fi

# Check for cargo-apk
if ! command -v cargo-apk &> /dev/null; then
    echo "cargo-apk not found. Installing cargo-apk..."
    cargo install cargo-apk
fi

# Add Android targets
echo "Adding Android Rust targets..."
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android

# Ensure ANDROID_SDK_ROOT and ANDROID_NDK_ROOT are set
if [ -z "$ANDROID_SDK_ROOT" ]; then
    if [ -d "./.android-sdk" ]; then
        export ANDROID_SDK_ROOT="$(pwd)/.android-sdk"
        echo "Using repo-local ANDROID_SDK_ROOT: $ANDROID_SDK_ROOT"
    elif [ -d "/Volumes/T7/android-sdk" ]; then
        export ANDROID_SDK_ROOT="/Volumes/T7/android-sdk"
        echo "Using T7 ANDROID_SDK_ROOT: $ANDROID_SDK_ROOT"
    elif [ -d "$HOME/Library/Android/sdk" ]; then
        export ANDROID_SDK_ROOT="$HOME/Library/Android/sdk"
        echo "Auto-detected ANDROID_SDK_ROOT: $ANDROID_SDK_ROOT"
    fi
fi

if [ -z "$ANDROID_HOME" ] && [ -n "$ANDROID_SDK_ROOT" ]; then
    export ANDROID_HOME="$ANDROID_SDK_ROOT"
fi

if [ -z "$ANDROID_NDK_ROOT" ]; then
    if [ -n "$ANDROID_SDK_ROOT" ] && [ -d "$ANDROID_SDK_ROOT/ndk" ]; then
        NDK_VER=$(ls "$ANDROID_SDK_ROOT/ndk" | sort -V | tail -n 1)
        export ANDROID_NDK_ROOT="$ANDROID_SDK_ROOT/ndk/$NDK_VER"
        echo "Auto-detected ANDROID_NDK_ROOT: $ANDROID_NDK_ROOT"
    else
        echo "⚠️ Warning: ANDROID_NDK_ROOT is not set!"
        echo "Please set it before building, e.g.:"
        echo "export ANDROID_SDK_ROOT=/Volumes/T7/android-sdk"
        echo "export ANDROID_NDK_ROOT=\$ANDROID_SDK_ROOT/ndk/<version>"
        exit 1
    fi
fi

echo "Building APK..."
PROFILE_DIR="debug"
RELEASE_FLAG=""
if [ "$MODE" = "release" ]; then
    PROFILE_DIR="release"
    RELEASE_FLAG="--release"
fi

set -o pipefail
mkdir -p dist

if cargo apk build --lib -p eframe_template --target aarch64-linux-android $RELEASE_FLAG 2>&1 | tee "dist/build_android_${PROFILE_DIR}.log"; then
    APK_PATH="$(ls -1 "target/${PROFILE_DIR}/apk/"*.apk | head -n 1)"
    OUT_APK="dist/EfficiencyTool-android-arm64-${PROFILE_DIR}.apk"
    cp "$APK_PATH" "$OUT_APK"
    echo "✅ APK 构建成功: $OUT_APK"
else
    echo "❌ APK 构建失败，日志已保存: dist/build_android_${PROFILE_DIR}.log"
    if [ "$MODE" = "release" ]; then
        echo "提示：release 模式需要配置签名（keystore）。"
        echo "可先用 debug 模式：bash ./build_apk.sh"
    fi
    exit 1
fi
