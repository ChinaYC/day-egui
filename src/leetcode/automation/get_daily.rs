use anyhow::Result;
use headless_chrome::Tab;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::sync::atomic::AtomicBool;
use super::daily::{add_log, check_cancel};

pub fn get_daily_problem_url(tab: &Arc<Tab>, logs: &Arc<Mutex<Vec<String>>>, cancel_flag: &Arc<AtomicBool>) -> Result<(String, String, bool, String)> {
    add_log(logs, "访问 LeetCode 首页...");
    tab.navigate_to("https://leetcode.cn").map_err(|e| anyhow::anyhow!(e))?;
    std::thread::sleep(Duration::from_secs(3));

    check_cancel(cancel_flag)?;
    add_log(logs, "尝试获取每日一题链接及打卡状态...");
    
    // 我们提取四个信息：
    // 1. 每日一题的 url
    // 2. 题目名称
    // 3. 是否已完成 (通常力扣打卡完成后，链接旁边会有打勾图标)
    // 4. 连续打卡天数
    let eval_res = tab.evaluate(
        r#"
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
                    // （因为页面顶部可能还有包含 envType=daily-question 的打卡天数火焰图标）
                    let h3 = a.querySelector('h3');
                    if (h3) {
                        url = a.href;
                        title = h3.innerText.replace(/\n/g, ' ').trim();
                        
                        // 检查是否已经打卡
                        // 未打卡时，通常有 stroke-lc-gray-60 等灰色的 circle。已打卡时通常会有绿色（green）、check 等标志。
                        let html = a.innerHTML.toLowerCase();
                        if (html.includes('green') || html.includes('success') || html.includes('check') || html.includes('已完成')) {
                            is_solved = true;
                        } else {
                            // 进一步检查 circle 是否没有灰色
                            if (!html.includes('stroke-lc-gray') && !html.includes('rgba(0,0,0,0.03)')) {
                                // 可能是已完成的其它状态
                            }
                        }
                        
                        return JSON.stringify({ url: url, title: title, is_solved: is_solved, consecutive_days: consecutive_days });
                    }
                }
            }
            return JSON.stringify({ url: "", title: "", is_solved: false, consecutive_days: consecutive_days });
        })();
        "#,
        false
    ).map_err(|e| anyhow::anyhow!(e))?;

    let json_str = eval_res.value
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| r#"{"url":"","title":"","is_solved":false,"consecutive_days":""}"#.to_string());
        
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap_or_default();
    
    let problem_url = parsed["url"].as_str().unwrap_or("").to_string();
    let problem_title = parsed["title"].as_str().unwrap_or("").to_string();
    let is_solved = parsed["is_solved"].as_bool().unwrap_or(false);
    let consecutive_days = parsed["consecutive_days"].as_str().unwrap_or("").to_string();
    
    if problem_url.is_empty() {
        add_log(logs, "❌ 未找到每日一题链接，请检查页面结构");
        return Err(anyhow::anyhow!("未找到每日一题链接"));
    }

    add_log(logs, &format!("获取到题目: {} (是否完成: {}, 连续打卡: {}天)", problem_title, is_solved, consecutive_days));
    Ok((problem_url, problem_title, is_solved, consecutive_days))
}
