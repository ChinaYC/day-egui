use anyhow::Result;
use headless_chrome::{Browser, Tab};
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn add_log(logs: &Arc<Mutex<Vec<String>>>, msg: &str) {
    let now = chrono::Local::now().format("%H:%M:%S").to_string();
    let mut logs_lock = logs.lock().unwrap();
    logs_lock.push(format!("[{}] {}", now, msg));
}

///等待元素出现
fn wait_for_element(tab: &Tab, selector: &str,timeout: Duration) -> Result<()> {
    let start = std::time::Instant::now();
    loop {
        if start.elapsed() > timeout {
            return Err(anyhow::anyhow!("元素未出现 (Element not found)"));
        }

        // 检查元素是否存在
       let exists = tab.evaluate(&format!("document.querySelector('{}') !== null", selector), false)?
            .value
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
    
        if exists {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(200));
    }
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
    // 忽略导航过程中的网络错误，有时候部分资源加载失败会报错但页面其实已经出来了
    let _ = tab.navigate_to("https://leetcode.cn/");
    std::thread::sleep(Duration::from_secs(5)); // 给页面留出足够的渲染时间

    // 尝试点击登录按钮，触发登录弹窗或跳转页面
    let _ = tab.evaluate(
        r#"
        (function() {
            let loginBtn = document.querySelector('a[href*="/accounts/login"]');
            if (loginBtn) {
                loginBtn.click();
            }
        })();
        "#,
        false
    );

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
                    // 如果存在明确的登录链接，说明未登录
                    if (document.querySelector('a[href*="/accounts/login"]')) return false;
                    
                    if (document.cookie.includes('LEETCODE_SESSION')) return true;
                    
                    // 检查页面是否包含用户头像或菜单等已登录标志
                    if (document.querySelector('nav') && !document.querySelector('a[href*="/accounts/login"]')) {
                        // 简单判断：没有登录按钮且页面加载完成
                        return true;
                    }
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
