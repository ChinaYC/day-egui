# 更新说明 / Changelog

本文件记录每个版本（尤其是 0.1.x 这种小版本）的改动点，方便在 GitHub Release 页面快速了解更新内容。

格式参考 Keep a Changelog，并尽量用用户视角描述“新增/优化/修复”。

## [Unreleased]

### 新增
- 

### 优化
- 

### 修复
- 

## [0.1.0] - 2026-05-11

### 新增
- Todo：任务支持 `updated_at` 更新时间戳，并在旧数据加载时自动补齐。
- Todo：新增导入能力（支持导入 `todos.json` / 导出的 `jsonl`），并提供合并逻辑。
- Todo：冲突解决窗口（同 ID 且双方都在上次同步后修改时触发），支持“用本地 / 用导入 / 保留两份”。
- 发布：GitHub Actions Release 自动构建并上传可下载产物（Windows/macOS/Linux/Web/Android）。

### 优化
- Todo：完成/删除/恢复/改提醒/编辑等所有修改操作会刷新 `updated_at`，为后续同步与冲突判断提供可靠基础。

