# ⚡ Efficiency Tool / 效率工具 (LeetCode & Todo)

A lightweight, local-first productivity application built with Rust and `egui` (via `eframe_template`).
一个使用 Rust 和 `egui` 构建的轻量级本地优先效率工具。

## 🎯 Features / 功能特点

### 1. LeetCode Daily Practice / LeetCode 每日打卡
- **Offline Code Editor / 离线代码输入区**: Write and save your daily solutions locally. (支持本地离线编写代码)。
- **One-Click Copy / 一键复制**: Quickly copy your solution to clipboard for submission. (一键复制答案，方便粘贴到网页提交)。
- **Simulated Submission / 模拟提交**: Mock submission tracking, designed with API extensibility in mind for future automated sync. (模拟提交记录，架构已预留 API 对接空间，方便后续实现自动同步)。

### 2. Daily Todo List / 日常待办清单
- **Task Management / 任务管理**: Add, complete, and delete daily tasks. (增删改查日常任务)。
- **Persistent Storage / 持久化存储**: All data is saved to your local disk and persists across app restarts. (支持离线保存，数据自动持久化到本地磁盘)。

## 🏗 Architecture & Tech Stack / 架构与技术栈

- **Language / 语言**: [Rust](https://www.rust-lang.org/)
- **GUI Framework / UI 框架**: [egui](https://github.com/emilk/egui) / eframe
- **State Management / 状态管理**: Uses `serde` for automatic serialization/deserialization to local app data directories. (采用 `serde` 自动序列化状态至本地缓存)。
- **Extensibility / 扩展性设计**: The architecture separates the `LeetCodeState` and `TodoState` into distinct modules (`leetcode.rs`, `todo.rs`). This allows easy drop-in of network requests (`reqwest`) when moving from local to online. (模块化拆分了 `LeetCode` 和 `Todo` 的状态，后期从离线版切换到联网版时，可无缝接入 `reqwest` 等网络库)。

## 🚀 How to Run / 如何运行

```bash
# Make sure you have Rust installed / 请确保已安装 Rust
cargo run --release
```

## 📦 How to Build macOS App / 如何打包 macOS 应用

You can build and package the project into a macOS `.app` bundle by running the following commands:
您可以通过运行以下命令将项目打包成 macOS 的 `.app` 应用程序包：

```bash
# 1. Build the release binary / 编译发布版本
cargo build --release

# 2. Create the app bundle structure / 创建应用包结构
APP_NAME="EfficiencyTool"
APP_DIR="$HOME/Downloads/$APP_NAME.app"
mkdir -p "$APP_DIR/Contents/MacOS"
mkdir -p "$APP_DIR/Contents/Resources"

# 3. Copy the binary to the app bundle / 复制二进制文件到应用包
cp target/release/eframe_template "$APP_DIR/Contents/MacOS/$APP_NAME"

# 4. Create Info.plist / 创建 Info.plist 配置文件
cat > "$APP_DIR/Contents/Info.plist" <<EOF
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

echo "App successfully built at / 应用程序打包成功，路径为: $APP_DIR"
```

## 🛠 Future Roadmap / 后续优化预留

1. **API Integration / 接口集成**: Add HTTP client to fetch daily questions from LeetCode and submit code automatically. (增加 HTTP 请求模块，实现自动获取每日一题及自动提交)。
2. **Cloud Sync / 云端同步**: Sync Todo tasks with Notion or other platforms. (Todo 清单支持多端同步或对接 Notion API)。
3. **Analytics / 数据统计**: Charting daily streaks and code runtime complexity using egui plots. (使用图表展示每日打卡热力图)。

目录更新后要修改以下内容
src/
├── app.rs               # 主应用层，负责处理路由和全局状态的调度
├── lib.rs               # 模块导出定义
├── main.rs              # 应用程序入口
├── leetcode/            # [模块] LeetCode 打卡功能
│   ├── mod.rs           # 模块统一出口
│   ├── state.rs         # 数据层：存放结构体、序列化逻辑 (如 LeetCodeState)
│   ├── ui.rs            # 视图层：存放 egui 相关的页面渲染逻辑
│   └── api.rs           # 接口层：为以后发起网络请求（如提交代码、获取题目）预留
└── todo/                # [模块] 日常 Todo 清单功能
    ├── mod.rs           # 模块统一出口
    ├── state.rs         # 数据层：存放 TodoItem 和状态控制逻辑
    ├── ui.rs            # 视图层：存放待办事项的渲染和交互事件
    └── api.rs           # 接口层：为以后对接云端或本地 SQLite 同步预留