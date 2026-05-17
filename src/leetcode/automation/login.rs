use super::utils::ElementUtils;
use anyhow::Result;
use headless_chrome::{Browser, Tab};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{debug, info};

pub fn ensure_login(
    browser: &Browser,
    _logs: &Arc<Mutex<Vec<String>>>,
    cancel_flag: &Arc<std::sync::atomic::AtomicBool>,
) -> Result<Arc<Tab>> {
    debug!("开始登录检查流程");
    // 智能获取初始标签页，优先寻找空白页 (about:blank) 进行复用，避免多出多余的空窗口
    let tab = {
        let mut target_tab = None;
        for i in 0..30 {
            let tabs = browser.get_tabs().lock().unwrap().clone();
            debug!("第 {} 次尝试获取标签页，当前标签页数量: {}", i, tabs.len());
            for t in &tabs {
                let url = t.get_url();
                debug!("检查标签页 URL: {}", url);
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
            debug!("未找到空白页，回退到最后一个标签页");
            tabs.last()
                .cloned()
                .unwrap_or_else(|| {
                    debug!("创建新标签页");
                    browser.new_tab().unwrap()
                })
        })
    };

    info!(user = true, "等待页面加载... (Waiting for page load)");
    // 忽略导航过程中的网络错误，有时候部分资源加载失败会报错但页面其实已经出来了
    let _ = tab.navigate_to("https://leetcode.cn/");
    if !ElementUtils::wait_for_element(&tab, "body", Duration::from_secs(15)) {
        info!("页面主体等待超时，继续尝试...");
    }

    // 尝试点击登录按钮，触发登录弹窗或跳转页面
    let _ = ElementUtils::eval_bool(
        &tab,
        r#"
        (function() {
            let loginBtn = document.querySelector('a[href*="/accounts/login"]');
            if (loginBtn) {
                loginBtn.click();
                return true;
            }
            return false;
        })();
        "#,
    );

    info!(user = true, "检查是否需要登录... (Checking login status)");
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
            let is_logged_in = ElementUtils::eval_bool(
                &t,
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
            )
            .unwrap_or(false);

            if is_logged_in {
                info!(user = true, "已登录！ (Logged in!)");
                return Ok(t);
            }
        }

        std::thread::sleep(Duration::from_secs(2));
        info!(user = true, "请在弹出的浏览器中完成登录... (Please log in via the browser window...)");
    }
}
