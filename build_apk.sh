#!/bin/bash

echo "==========================================================="
echo "  eframe_template Android APK Build Script"
echo "==========================================================="

MODE="${1:-debug}"
APK_TARGETS="${APK_TARGETS:-aarch64-linux-android armv7-linux-androideabi}"
FORCE_ANDROID_HOME="${FORCE_ANDROID_HOME:-0}"

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
if [ "$FORCE_ANDROID_HOME" != "1" ] && [ -d "./.android-sdk" ]; then
    export ANDROID_HOME="$(pwd)/.android-sdk"
    echo "Using repo-local ANDROID_HOME: $ANDROID_HOME"
elif [ -z "$ANDROID_HOME" ]; then
    if [ -d "/Volumes/T7/android-sdk" ]; then
        export ANDROID_HOME="/Volumes/T7/android-sdk"
        echo "Using T7 ANDROID_HOME: $ANDROID_HOME"
    elif [ -d "$HOME/Library/Android/sdk" ]; then
        export ANDROID_HOME="$HOME/Library/Android/sdk"
        echo "Auto-detected ANDROID_HOME: $ANDROID_HOME"
    fi
fi

if [ -z "$ANDROID_SDK_ROOT" ] && [ -n "$ANDROID_HOME" ]; then
    export ANDROID_SDK_ROOT="$ANDROID_HOME"
fi

if [ -z "$ANDROID_NDK_ROOT" ]; then
    if [ -n "$ANDROID_HOME" ] && [ -d "$ANDROID_HOME/ndk" ]; then
        NDK_VER=$(ls "$ANDROID_HOME/ndk" | sort -V | tail -n 1)
        export ANDROID_NDK_ROOT="$ANDROID_HOME/ndk/$NDK_VER"
        echo "Auto-detected ANDROID_NDK_ROOT: $ANDROID_NDK_ROOT"
    else
        echo "⚠️ Warning: ANDROID_NDK_ROOT is not set!"
        echo "Please set it before building, e.g.:"
        echo "export ANDROID_HOME=/Volumes/T7/android-sdk"
        echo "export ANDROID_NDK_ROOT=\$ANDROID_HOME/ndk/<version>"
        exit 1
    fi
fi

echo "Building APK..."
PROFILE_DIR="debug"
BUILD_PROFILE_ARGS=""
if [ "$MODE" = "android" ]; then
    PROFILE_DIR="android"
    BUILD_PROFILE_ARGS="--profile android"
elif [ "$MODE" = "release" ]; then
    PROFILE_DIR="release"
    BUILD_PROFILE_ARGS="--release"
fi

set -o pipefail
mkdir -p dist

BUILD_TOOLS_VER="$(ls "$ANDROID_HOME/build-tools" | sort -V | tail -n 1)"
APKSIGNER="$ANDROID_HOME/build-tools/$BUILD_TOOLS_VER/apksigner"
ZIPALIGN="$ANDROID_HOME/build-tools/$BUILD_TOOLS_VER/zipalign"

KS="${ANDROID_KEYSTORE_PATH:-$(pwd)/android-debug.keystore}"
if [ ! -f "$KS" ]; then
    keytool -genkeypair -v -keystore "$KS" -storepass android -keypass android -alias androiddebugkey \
        -keyalg RSA -keysize 2048 -validity 10000 \
        -dname "CN=Android Debug,O=Android,C=US"
fi

for TARGET in $APK_TARGETS; do
    ABI="unknown"
    if [ "$TARGET" = "aarch64-linux-android" ]; then
        ABI="arm64"
    elif [ "$TARGET" = "armv7-linux-androideabi" ]; then
        ABI="armv7"
    elif [ "$TARGET" = "i686-linux-android" ]; then
        ABI="x86"
    elif [ "$TARGET" = "x86_64-linux-android" ]; then
        ABI="x86_64"
    fi

    LOG="dist/build_android_${ABI}_${PROFILE_DIR}.log"
    echo "------------------------------"
    echo "Target: $TARGET ($ABI), profile: $PROFILE_DIR"
    echo "------------------------------"

    if ! cargo apk build --lib -p eframe_template --target "$TARGET" $BUILD_PROFILE_ARGS 2>&1 | tee "$LOG"; then
        echo "❌ 构建失败: $TARGET，日志: $LOG"
        if [ "$MODE" = "release" ]; then
            echo "提示：release 模式 cargo-apk 默认要求配置 release keystore。"
            echo "建议先用：bash ./build_apk.sh android"
        fi
        exit 1
    fi

    APK_PATH="$(ls -t "target/${PROFILE_DIR}/apk/"*.apk | head -n 1)"
    RAW_APK="dist/EfficiencyTool-android-${ABI}-${PROFILE_DIR}.apk"
    ALIGNED_APK="dist/EfficiencyTool-android-${ABI}-${PROFILE_DIR}-aligned.apk"
    SIGNED_APK="dist/EfficiencyTool-android-${ABI}-${PROFILE_DIR}-signed.apk"
    rm -f "$RAW_APK" "$ALIGNED_APK" "$SIGNED_APK"
    cp "$APK_PATH" "$RAW_APK"

    "$ZIPALIGN" -p -f 4 "$RAW_APK" "$ALIGNED_APK"
    "$APKSIGNER" sign \
        --ks "$KS" \
        --ks-pass pass:android \
        --key-pass pass:android \
        --ks-key-alias androiddebugkey \
        --v1-signing-enabled true \
        --v2-signing-enabled true \
        --v3-signing-enabled true \
        --out "$SIGNED_APK" \
        "$ALIGNED_APK"

    "$APKSIGNER" verify --verbose "$SIGNED_APK" >/dev/null
    echo "✅ 已输出可安装 APK: $SIGNED_APK"
done
