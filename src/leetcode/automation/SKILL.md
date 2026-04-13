# LeetCode Daily Check-in Skill

## 概述

本技能用于自动化完成 LeetCode 每日打卡任务，包括：
- 获取每日一题题目
- 从官方题解复制 Rust 代码
- 提交代码并领取积分
- 同步打卡状态到嘀嗒清单习惯

## 前置条件

1. 已登录 LeetCode 账号 (leetcode.cn)
2. 已登录嘀嗒清单账号 (dida365.com)
3. 浏览器已启动并连接到 CDP (Chrome DevTools Protocol)
4. 每日一题使用 Rust 语言完成

## 完整流程步骤

### 步骤 1: 启动浏览器并访问 LeetCode

```javascript
// 启动浏览器
browser({ action: "start" })

// 访问 LeetCode 首页
browser({ action: "open", targetUrl: "https://leetcode.cn" })

// 等待页面加载
wait(3000)
```

**关键操作**: 通过首页 Daily Question 按钮获取今日题目信息

### 步骤 2: 获取今日题目信息

```javascript
// 获取页面快照
snapshot = browser({ action: "snapshot", compact: true })

// 解析今日题目标题和链接
// 通常在页面中显示为 "Daily Question" 按钮或链接
// 例如: "3740. 三个相等元素之间的最小距离 I"
```

**关键信息提取**:
- 题目编号和名称
- 题目链接 (含 envType=daily-question 参数)
- 连续打卡天数

### 步骤 3: 访问题解页面查找官方题解

```javascript
// 构建题解页面 URL
// 格式: https://leetcode.cn/problems/{problem-slug}/solutions/
browser({ 
  action: "open", 
  targetUrl: "https://leetcode.cn/problems/minimum-distance-between-three-equal-elements-i/solutions/" 
})

// 等待加载
wait(3000)

// 获取题解列表
snapshot = browser({ action: "snapshot", compact: true })
```

**关键操作**: 
- 找到 "力扣官方题解" 链接
- 点击进入官方题解详情页

### 步骤 4: 获取 Rust 语言代码

```javascript
// 进入官方题解详情页
browser({ action: "act", request: { kind: "click", ref: "官方题解链接ref" } })
wait(3000)

// 点击 Rust 语言选项
browser({ action: "act", request: { kind: "click", ref: "Rust按钮ref" } })
wait(2000)

// 获取 Rust 代码快照
snapshot = browser({ action: "snapshot" })

// 从快照中提取 Rust 代码块
// 代码通常在 <code> 元素或 monaco 编辑器中
```

**关键代码结构**:
```rust
impl Solution {
    pub fn function_name(params) -> return_type {
        // 官方题解代码
    }
}
```

### 步骤 5: 回到题目页面填入代码

```javascript
// 访问题目页面
browser({ 
  action: "open", 
  targetUrl: "https://leetcode.cn/problems/{problem-slug}/" 
})
wait(2000)

// 通过 JavaScript 注入代码到 Monaco 编辑器
browser({ 
  action: "act", 
  request: { 
    kind: "evaluate", 
    fn: `() => {
      const code = \`//day编写\n
impl Solution {
    pub fn function_name(params) -> return_type {
        // 从题解复制的代码
    }
}\`;

      // 方法 1: 使用 Monaco 编辑器 API
      if (typeof monaco !== 'undefined' && monaco.editor) {
        const models = monaco.editor.getModels();
        if (models.length > 0) {
          models[0].setValue(code);
          return 'Monaco code set';
        }
      }
      
      // 方法 2: 使用 textarea 作为后备
      const textarea = document.querySelector('textarea');
      if (textarea) {
        textarea.value = code;
        textarea.dispatchEvent(new Event('input', { bubbles: true }));
        return 'Textarea code set';
      }
      
      return 'Editor not found';
    }`
  } 
})
```

**关键技术点**:
- LeetCode 使用 Monaco Editor 作为代码编辑器
- 通过 `monaco.editor.getModels()` 获取编辑器实例
- 使用 `setValue()` 方法设置代码内容
- 必须添加 `//day编写` 注释作为标记

### 步骤 6: 提交代码

```javascript
// 获取页面快照确认代码已填入
snapshot = browser({ action: "snapshot", compact: true })

// 点击提交按钮
browser({ action: "act", request: { kind: "click", ref: "提交按钮ref" } })

// 等待判题结果
wait(5000)

// 获取提交结果
snapshot = browser({ action: "snapshot", compact: true })
```

**结果判断**:
- 显示 "恭喜完成今日打卡任务" = 打卡成功
- 显示 "解答错误" = 需要检查代码
- 显示 "执行出错" = 代码有语法或运行时错误

### 步骤 7: 领取积分

```javascript
// 点击领取积分按钮
browser({ action: "act", request: { kind: "click", ref: "领取积分按钮ref" } })
```

### 步骤 8: 同步到嘀嗒清单

```javascript
// 访问嘀嗒清单习惯页面
browser({ 
  action: "open", 
  targetUrl: "https://dida365.com/webapp/#q/all/habit" 
})
wait(3000)

// 获取习惯列表
snapshot = browser({ action: "snapshot", compact: true })

// 查找 leetcode 习惯
// 如果显示 "X 天 Y 天"，说明今日已打卡
// 如果显示 "X 天 0 天"，说明今日未打卡，需要点击
```

**关键判断**:
- 习惯项显示格式: `{习惯名} {总天数} {当前连续天数}`
- 如果当前连续天数 > 0，表示今日已完成
- 如果当前连续天数 = 0，表示今日未完成，需要点击习惯项打卡

## 常见问题与解决方案

### 问题 1: Monaco 编辑器代码注入失败

**现象**: 代码没有填入编辑器

**解决方案**:
```javascript
// 尝试多种方式注入代码
function injectCode(code) {
  // 方式 1: Monaco API
  if (typeof monaco !== 'undefined' && monaco.editor) {
    const models = monaco.editor.getModels();
    if (models.length > 0) {
      models[0].setValue(code);
      return true;
    }
  }
  
  // 方式 2: 查找 textarea
  const textarea = document.querySelector('textarea.monaco-editor');
  if (textarea) {
    textarea.value = code;
    textarea.dispatchEvent(new Event('input', { bubbles: true }));
    return true;
  }
  
  // 方式 3: 直接操作 DOM
  const editorElement = document.querySelector('.monaco-editor');
  if (editorElement) {
    // 触发编辑器 focus 后再注入
    editorElement.click();
    // 使用 clipboard API 或键盘事件
  }
  
  return false;
}
```

### 问题 2: 题解页面没有 Rust 代码

**现象**: 官方题解只提供 C++/Python/Java 等语言

**解决方案**:
1. 优先选择有 Rust 版本的题解（如灵茶山艾府的题解通常有 Rust）
2. 如果没有 Rust 版本，将 C++ 代码手动转换为 Rust
3. 关键转换规则:
   - `vector<int>` → `Vec<i32>`
   - `int` → `i32`
   - `long long` → `i64`
   - `std::min` → `.min()`
   - 数组索引用 `usize` 类型

### 问题 3: 代码提交后部分测试用例失败

**现象**: 通过 X/Y 个测试用例，部分失败

**解决方案**:
1. 查看失败的测试用例
2. 常见原因:
   - 整数溢出: 使用 `i64` 代替 `i32`，最后转回
   - 取模运算: 负数取模要处理 `(x % MOD + MOD) % MOD`
   - 边界条件: 空数组、单元素数组等

### 问题 4: 浏览器节点离线导致自动化失败

**现象**: Cron 任务执行时报 "no available browser nodes"

**解决方案**:
1. 确保 Windows 电脑开机并运行 ToClaw 客户端
2. 或者手动执行打卡流程
3. 设置多个备用设备

## 完整自动化脚本示例

```javascript
// 完整 LeetCode 每日打卡流程
async function leetcodeDailyCheckin() {
  // 1. 启动浏览器
  await browser({ action: "start" });
  
  // 2. 访问 LeetCode
  await browser({ action: "open", targetUrl: "https://leetcode.cn" });
  await wait(3000);
  
  // 3. 获取今日题目
  const homeSnapshot = await browser({ action: "snapshot", compact: true });
  // 解析题目链接...
  
  // 4. 访问题目和题解
  await browser({ action: "open", targetUrl: problemUrl });
  await wait(2000);
  await browser({ action: "open", targetUrl: solutionUrl });
  await wait(3000);
  
  // 5. 获取 Rust 代码
  // ... 解析代码 ...
  const rustCode = `//day编写\n${extractedCode}`;
  
  // 6. 填入代码
  await browser({ 
    action: "open", 
    targetUrl: problemUrl 
  });
  await wait(2000);
  
  await browser({ 
    action: "act", 
    request: { 
      kind: "evaluate", 
      fn: `() => {
        const code = \`${rustCode}\`;
        if (typeof monaco !== 'undefined' && monaco.editor) {
          const models = monaco.editor.getModels();
          if (models.length > 0) {
            models[0].setValue(code);
            return 'Code injected';
          }
        }
        return 'Failed';
      }`
    } 
  });
  
  // 7. 提交
  await browser({ action: "act", request: { kind: "click", ref: "提交按钮ref" } });
  await wait(5000);
  
  // 8. 领取积分
  const resultSnapshot = await browser({ action: "snapshot", compact: true });
  if (resultSnapshot.contains("恭喜完成今日打卡任务")) {
    await browser({ action: "act", request: { kind: "click", ref: "领取积分按钮ref" } });
  }
  
  // 9. 嘀嗒清单打卡
  await browser({ action: "open", targetUrl: "https://dida365.com/webapp/#q/all/habit" });
  await wait(3000);
  
  return "打卡完成!";
}
```

## 参考资源

- LeetCode 中国站: https://leetcode.cn
- 嘀嗒清单: https://dida365.com
- Monaco Editor API: https://microsoft.github.io/monaco-editor/api/

## 更新日志

- 2026-04-10: 创建 skill 文档，整理完整打卡流程
