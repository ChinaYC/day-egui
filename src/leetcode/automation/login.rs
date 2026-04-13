use anyhow::Result;
use headless_chrome::{Browser, Tab};
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn add_log(logs: &Arc<Mutex<Vec<String>>>, msg: &str) {
    let now = chrono::Local::now().format("%H:%M:%S").to_string();
    let mut logs_lock = logs.lock().unwrap();
    logs_lock.push(format!("[{}] {}", now, msg));
}

pub fn ensure_login(browser: &Browser, logs: &Arc<Mutex<Vec<String>>>, cancel_flag: &Arc<std::sync::atomic::AtomicBool>) -> Result<Arc<Tab>> {
    // 智能获取初始标签页，优先寻找空白页 (about:blank) 进行复用，避免多出多余的空窗口
    let tab = {
        let mut target_tab = None;
        for _ in 0..30 {
            let tabs = browser.get_tabs().lock().unwrap().clone();
            for t in &tabs {
                let url = t.get_url();
                if url == "about:blank" || url.contains("newtab") || url.contains("new-tab") {
                    target_tab = Some(t.clone());
                    break;
                }
            }
            if target_tab.is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        target_tab.unwrap_or_else(|| {
            let tabs = browser.get_tabs().lock().unwrap().clone();
            tabs.last().cloned().unwrap_or_else(|| browser.new_tab().unwrap())
        })
    };
    
    add_log(logs, "等待页面加载... (Waiting for page load)");
    tab.navigate_to("https://leetcode.cn/").map_err(|e: anyhow::Error| e)?;
    std::thread::sleep(Duration::from_secs(3));

    add_log(logs, "检查是否需要登录... (Checking login status)");
    let timeout = Duration::from_secs(300); 
    let start = std::time::Instant::now();
    
    loop {
        if cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(anyhow::anyhow!("已手动停止 (Stopped by user)"));
        }
        
        if start.elapsed() > timeout {
            return Err(anyhow::anyhow!("登录超时 (Login timeout)"));
        }

        // 检查所有已打开的标签页，防止第三方登录在新的标签页中打开或跳转
        let tabs = browser.get_tabs().lock().unwrap().clone();
        for t in tabs {
            let is_logged_in = t.evaluate(
                r#"
                (function() {
                    if (document.cookie.includes('LEETCODE_SESSION')) return true;
                    let text = document.body.innerText;
                    if (text.includes('Plus 会员') && !text.includes('登录 / 注册')) return true;
                    return false;
                })();
                "#,
                false
            )
            .map(|v| v.value.and_then(|val| val.as_bool()).unwrap_or(false))
            .unwrap_or(false);

            if is_logged_in {
                add_log(logs, "已登录！ (Logged in!)");
                return Ok(t);
            }
        }

        std::thread::sleep(Duration::from_secs(2));
        add_log(logs, "请在弹出的浏览器中完成登录... (Please log in via the browser window...)");
    }
}
