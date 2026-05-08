#!/bin/bash

echo "==========================================================="
echo "  eframe_template Android APK Build Script"
echo "==========================================================="

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
if [ -z "$ANDROID_NDK_ROOT" ]; then
    if [ -d "$HOME/Library/Android/sdk/ndk" ]; then
        # Auto-detect newest NDK on macOS
        NDK_VER=$(ls "$HOME/Library/Android/sdk/ndk" | sort -V | tail -n 1)
        export ANDROID_NDK_ROOT="$HOME/Library/Android/sdk/ndk/$NDK_VER"
        echo "Auto-detected ANDROID_NDK_ROOT: $ANDROID_NDK_ROOT"
    else
        echo "⚠️ Warning: ANDROID_NDK_ROOT is not set!"
        echo "Please set it before building, e.g.:"
        echo "export ANDROID_NDK_ROOT=\$HOME/Library/Android/sdk/ndk/<version>"
        exit 1
    fi
fi

if [ -z "$ANDROID_SDK_ROOT" ]; then
    if [ -d "$HOME/Library/Android/sdk" ]; then
        export ANDROID_SDK_ROOT="$HOME/Library/Android/sdk"
        echo "Auto-detected ANDROID_SDK_ROOT: $ANDROID_SDK_ROOT"
    fi
fi

echo "Building APK..."
cargo apk build --release

if [ $? -eq 0 ]; then
    echo "✅ APK built successfully!"
    echo "You can find it in: target/release/apk/EfficiencyTool.apk"
else
    echo "❌ APK build failed."
fi
