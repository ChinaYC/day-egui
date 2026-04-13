use anyhow::Result;
use headless_chrome::{Browser, LaunchOptions, Tab};
use std::sync::Arc;
use std::time::Duration;

pub fn launch_browser() -> Result<Browser> {
    // 设置持久化的用户数据目录，使得登录状态能够保留
    // 这里将其放在用户的 home 目录下的 .leetcode_automation 文件夹中
    let user_data_dir = if let Some(mut path) = dirs::home_dir() {
        path.push(".leetcode_automation");
        Some(path)
    } else {
        None
    };

    let browser = Browser::new(
        LaunchOptions::default_builder()
            .headless(false)
            .user_data_dir(user_data_dir)
            .ignore_certificate_errors(false) // 禁用忽略证书错误，去除浏览器顶部的黄条警告
            .ignore_default_args(vec![
                std::ffi::OsStr::new("--enable-automation"),
                std::ffi::OsStr::new("--password-store=basic"),
                std::ffi::OsStr::new("--use-mock-keychain"),
            ])
            .idle_browser_timeout(std::time::Duration::from_secs(600))
            .build()
            .map_err(|e| anyhow::anyhow!(e))?,
    )
    .map_err(|e| anyhow::anyhow!(e))?;

    Ok(browser)
}

pub fn wait_for_element_with_text<'a>(
    tab: &'a Arc<Tab>,
    selector: &str,
    text: &str,
    timeout: Duration,
    cancel_flag: &Arc<std::sync::atomic::AtomicBool>,
) -> Result<()> {
    let start = std::time::Instant::now();
    loop {
        if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(anyhow::anyhow!("已手动停止 (Stopped by user)"));
        }
        if start.elapsed() > timeout {
            return Err(anyhow::anyhow!("Timeout waiting for element with text: {}", text));
        }
        
        let script = format!(
            r#"
            (function() {{
                const elements = document.querySelectorAll(`{}`);
                for (let el of elements) {{
                    if (el.innerText && el.innerText.includes(`{}`)) {{
                        return true;
                    }}
                }}
                return false;
            }})();
            "#,
            selector.replace('`', "\\`"),
            text.replace('`', "\\`")
        );
        
        let found = tab.evaluate(&script, false)
            .map(|v| v.value.and_then(|val| val.as_bool()).unwrap_or(false))
            .unwrap_or(false);
            
        if found {
            return Ok(());
        }
        
        std::thread::sleep(Duration::from_millis(500));
    }
}
