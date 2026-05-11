# 发布与打包（GitHub Releases）

本项目已配置 GitHub Actions 自动构建多平台二进制并发布到 GitHub Releases。

## 触发方式 A：推送 tag（推荐）

在本地创建并推送 tag（tag 必须以 `v` 开头，匹配 `v*`）：

```bash
git tag v0.1.0
git push origin v0.1.0
```

推送后到 GitHub 仓库的 Actions 页面查看 `Release` 工作流运行，完成后到 Releases 页面下载产物。

## 触发方式 C：GitHub 网页 “Create a new release”

你也可以直接在 GitHub Releases 页面点击 “Create a new release” 并发布。
发布完成后会自动触发 `Release` 工作流（监听 `release: published`），构建并把产物上传到该 Release 的 Assets。

## 触发方式 B：GitHub 网页手动触发（无需本地 git）

1. 打开仓库 → Actions
2. 选择 `Release` 工作流
3. 点击 `Run workflow`
4. 在 `version` 输入框填入要发布的 tag（例如 `v0.1.0`），然后运行

工作流结束后会自动创建对应 tag 的 Release，并上传三端产物：
- `EfficiencyTool-linux-x86_64-musl.tar.gz`（Linux 静态版，更通用）
- `EfficiencyTool-windows-x86_64.zip`
- `EfficiencyTool-macos-aarch64.tar.gz`（内含 `EfficiencyTool.app`）
- `EfficiencyTool-macos-x86_64.tar.gz`（内含 `EfficiencyTool.app`）
- `EfficiencyTool-android-arm64.apk`
- `EfficiencyTool-web.tar.gz`（Web 版静态文件，可本地解压打开）

下载地址：
- Release 页面：`https://github.com/<owner>/<repo>/releases/tag/<tag>`
- 产物直链：`https://github.com/<owner>/<repo>/releases/download/<tag>/<asset-file-name>`

## 常见问题

### 1) 为什么没有触发？

- tag 没有以 `v` 开头（需要 `v0.1.0` 这种格式）
- 只创建了 tag 但没有推送（需要 `git push origin v0.1.0`）
- 手动触发时 `version` 为空或不合法

### 2) 在哪里看日志？

仓库 → Actions → 进入对应的 `Release` workflow run，即可看到每个平台的编译日志与上传步骤。
