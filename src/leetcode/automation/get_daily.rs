use super::daily::check_cancel;
use super::utils::ElementUtils;
use anyhow::Result;
use headless_chrome::Tab;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tracing::{debug, info};

pub fn get_daily_problem_url(
    tab: &Arc<Tab>,
    _logs: &Arc<Mutex<Vec<String>>>,
    cancel_flag: &Arc<AtomicBool>,
) -> Result<(String, String, bool, String)> {
    info!(user = true, "在首页尝试获取每日一题链接及打卡状态...");

    let mut attempts = 0;
    let max_attempts = 15; // 最多等待 15 秒

    loop {
        check_cancel(cancel_flag)?;
        attempts += 1;

        let js_code = r#"
            (function() {
                let url = "";
                let title = "未知题目";
                let is_solved = false;
                let consecutive_days = "";
                
                // 尝试获取顶部导航栏的连续打卡天数
                const dayLinks = document.querySelectorAll('a[href*="envId="]');
                for (let d of dayLinks) {
                    let text = d.innerText.trim();
                    if (text && !text.includes('每日') && text.match(/^\d+$/)) {
                        consecutive_days = text;
                        break;
                    }
                }
                
                const links = document.querySelectorAll('a');
                for (let a of links) {
                    // 首页的每日一题链接带有 envType=daily-question
                    if (a.href.includes('envType=daily-question')) {
                        // 查找 h3 标签获取纯净题目名称，必须包含 h3 才是真正的每日一题卡片
                        let h3 = a.querySelector('h3');
                        if (h3) {
                            url = a.href;
                            title = h3.innerText.replace(/\n/g, ' ').trim();
                            
                            // 检查是否已经打卡
                            let html = a.innerHTML.toLowerCase();
                            if (html.includes('green') || html.includes('success') || html.includes('check') || html.includes('已完成')) {
                                is_solved = true;
                            }
                            
                            return JSON.stringify({ url: url, title: title, is_solved: is_solved, consecutive_days: consecutive_days });
                        }
                    }
                }
                return JSON.stringify({ url: "", title: "", is_solved: false, consecutive_days: consecutive_days });
            })();
            "#;
        
        debug!("执行获取每日一题 JS: {}", js_code);
        let json_str = ElementUtils::eval_string(tab, js_code)?;
        debug!("获取每日一题结果响应: {}", json_str);

        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap_or_default();
        let problem_url = parsed["url"].as_str().unwrap_or("").to_string();

        if !problem_url.is_empty() {
            let problem_title = parsed["title"].as_str().unwrap_or("").to_string();
            let is_solved = parsed["is_solved"].as_bool().unwrap_or(false);
            let consecutive_days = parsed["consecutive_days"]
                .as_str()
                .unwrap_or("")
                .to_string();

            info!(
                user = true,
                "获取到题目: {} (是否完成: {}, 连续打卡: {}天)",
                problem_title, is_solved, consecutive_days
            );
            return Ok((problem_url, problem_title, is_solved, consecutive_days));
        }

        if attempts >= max_attempts {
            return Err(anyhow::anyhow!("未找到每日一题链接，请检查页面结构"));
        }

        if attempts == 1 {
            info!(user = true, "等待每日一题内容加载...");
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
