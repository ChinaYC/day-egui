use anyhow::Result;
use headless_chrome::Tab;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};

pub struct ElementUtils;

impl ElementUtils {
    /// 在指定时间内等待元素出现并返回其是否存在的布尔值
    pub fn wait_for_element(tab: &Arc<Tab>, selector: &str, timeout: Duration) -> bool {
        let start = Instant::now();
        while start.elapsed() < timeout {
            if let Ok(res) = tab.evaluate(&format!("!!document.querySelector('{}')", selector), false) {
                if res.value.and_then(|v| v.as_bool()).unwrap_or(false) {
                    return true;
                }
            }
            std::thread::sleep(Duration::from_millis(200));
        }
        false
    }

    /// 执行 JS 并获取字符串结果
    pub fn eval_string(tab: &Arc<Tab>, js: &str) -> Result<String> {
        let res = tab.evaluate(js, false).map_err(|e| anyhow::anyhow!(e))?;
        Ok(res.value.and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_default())
    }

    /// 执行 JS 并获取布尔结果
    pub fn eval_bool(tab: &Arc<Tab>, js: &str) -> Result<bool> {
        let res = tab.evaluate(js, false).map_err(|e| anyhow::anyhow!(e))?;
        Ok(res.value.and_then(|v| v.as_bool()).unwrap_or(false))
    }

    /// 点击元素，支持更复杂的点击逻辑（如触发事件）
    pub fn click(tab: &Arc<Tab>, selector: &str) -> Result<bool> {
        let js = format!(
            r#"(function() {{
                const el = document.querySelector('{}');
                if (el) {{
                    el.click();
                    return true;
                }}
                return false;
            }})()"#,
            selector
        );
        Self::eval_bool(tab, &js)
    }

    /// 根据文本内容查找并点击按钮
    pub fn click_by_text(tab: &Arc<Tab>, tag: &str, text: &str) -> Result<bool> {
        let js = format!(
            r#"(function() {{
                const els = Array.from(document.querySelectorAll('{}'));
                const target = els.find(el => el.innerText.includes('{}'));
                if (target) {{
                    target.click();
                    return true;
                }}
                return false;
            }})()"#,
            tag, text
        );
        Self::eval_bool(tab, &js)
    }
}

/// 统一的日志记录宏封装
pub fn log_info(_logs: &Arc<std::sync::Mutex<Vec<String>>>, msg: &str) {
    info!(user = true, "{}", msg);
}

pub fn log_error(_logs: &Arc<std::sync::Mutex<Vec<String>>>, msg: &str) {
    error!(user = true, "{}", msg);
}

pub fn log_warn(_logs: &Arc<std::sync::Mutex<Vec<String>>>, msg: &str) {
    warn!(user = true, "{}", msg);
}
