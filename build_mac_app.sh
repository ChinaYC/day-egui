#!/bin/bash
# ./build_mac_app.sh 
# 1. 编译发布版本
echo "正在构建 release 二进制..."
cargo build --release

# 2. 创建应用包结构
APP_NAME="EfficiencyTool"
DEST_DIR="${1:-/Applications}"
APP_DIR="$DEST_DIR/$APP_NAME.app"

SUDO=""
if [ ! -w "$DEST_DIR" ]; then
  SUDO="sudo"
fi

echo "正在创建应用包结构: $APP_DIR"
$SUDO mkdir -p "$APP_DIR/Contents/MacOS"
$SUDO mkdir -p "$APP_DIR/Contents/Resources"

# 3. 复制二进制文件到应用包
echo "正在复制二进制到应用包..."
$SUDO cp target/release/eframe_template "$APP_DIR/Contents/MacOS/$APP_NAME"

# 4. 创建 Info.plist 配置文件
echo "正在生成 Info.plist..."
cat <<EOF | $SUDO tee "$APP_DIR/Contents/Info.plist" >/dev/null
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>$APP_NAME</string>
    <key>CFBundleIdentifier</key>
    <string>com.liam.efficiencytool</string>
    <key>CFBundleName</key>
    <string>$APP_NAME</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.11</string>
</dict>
</plist>
EOF

# 5. 赋予可执行权限
$SUDO chmod +x "$APP_DIR/Contents/MacOS/$APP_NAME"

echo "✅ 已生成 App: $APP_DIR"
echo "你现在可以在 Finder 里双击运行。"
