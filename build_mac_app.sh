#!/bin/bash
# 打包执行 bash ./build_mac_app.sh ./dist
# ./build_mac_app.sh 
# 1. 编译发布版本
echo "正在构建 release 二进制..."
cargo build --release

# 2. 创建应用包结构
APP_NAME="EfficiencyTool"
DEST_DIR="${1:-./dist}"
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
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.11</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

# 5. 赋予可执行权限
$SUDO chmod +x "$APP_DIR/Contents/MacOS/$APP_NAME"

# 6. 签名与公证 (可选)
# 使用环境变量: 
# APPLE_ID_SIGNING_IDENTITY: "Developer ID Application: Your Name (TEAMID)"
# NOTARY_PROFILE: "notarytool 存储的 Profile 名称"

ENTITLEMENTS="entitlements.plist"

if [ -n "$APPLE_ID_SIGNING_IDENTITY" ]; then
    echo "正在使用 $APPLE_ID_SIGNING_IDENTITY 进行签名 (Hardened Runtime)..."
    codesign --force --options runtime --entitlements "$ENTITLEMENTS" --deep --sign "$APPLE_ID_SIGNING_IDENTITY" "$APP_DIR"
else
    echo "未设置 APPLE_ID_SIGNING_IDENTITY, 正在执行 Ad-hoc 签名..."
    if command -v codesign >/dev/null 2>&1; then
      codesign --force --deep --sign - "$APP_DIR" >/dev/null 2>&1 || true
    fi
fi

if [ -n "$NOTARY_PROFILE" ] && [ -n "$APPLE_ID_SIGNING_IDENTITY" ]; then
    echo "正在提交公证 (Notarization)..."
    ZIP_FOR_NOTARY="$DEST_DIR/${APP_NAME}_to_notarize.zip"
    ditto -c -k --sequesterRsrc --keepParent "$APP_DIR" "$ZIP_FOR_NOTARY"
    
    xcrun notarytool submit "$ZIP_FOR_NOTARY" --keychain-profile "$NOTARY_PROFILE" --wait
    
    echo "正在将公证票据附加到 App (Stapling)..."
    xcrun stapler staple "$APP_DIR"
    rm "$ZIP_FOR_NOTARY"
fi

if command -v ditto >/dev/null 2>&1; then
  ZIP_PATH="$DEST_DIR/${APP_NAME}-macos-aarch64.zip"
  rm -f "$ZIP_PATH"
  ditto -c -k --sequesterRsrc --keepParent "$APP_DIR" "$ZIP_PATH"
  echo "✅ 已生成 ZIP: $ZIP_PATH"
fi

# 7. 生成 DMG
DMG_PATH="$DEST_DIR/${APP_NAME}-macos.dmg"
rm -f "$DMG_PATH"

if command -v create-dmg >/dev/null 2>&1; then
    echo "正在使用 create-dmg 生成美化版 DMG: $DMG_PATH"
    create-dmg \
      --volname "${APP_NAME} Installer" \
      --window-pos 200 120 \
      --window-size 800 400 \
      --icon-size 100 \
      --icon "${APP_NAME}.app" 200 190 \
      --hide-extension "${APP_NAME}.app" \
      --app-drop-link 600 185 \
      "$DMG_PATH" \
      "$APP_DIR"
else
    echo "未找到 create-dmg，正在使用 hdiutil 生成标准 DMG: $DMG_PATH"
    # 创建临时目录
    TMP_DMG_DIR=$(mktemp -d)
    cp -R "$APP_DIR" "$TMP_DMG_DIR/"
    # 创建软链接到 /Applications
    ln -s /Applications "$TMP_DMG_DIR/Applications"
    
    # 生成 DMG
    hdiutil create -volname "${APP_NAME} Installer" -srcfolder "$TMP_DMG_DIR" -ov -format UDZO "$DMG_PATH"
    
    # 清理
    rm -rf "$TMP_DMG_DIR"
fi

if [ -f "$DMG_PATH" ]; then
    echo "✅ 已生成 DMG: $DMG_PATH"
fi

echo "✅ 已生成 App: $APP_DIR"
echo "你现在可以在 Finder 里双击运行。"
