use anyhow::Result;
use std::sync::{Arc, Mutex};
use super::{browser, login};

// 导入重构后的模块
use super::get_daily::get_daily_problem_url;
use super::solution::extract_solution_code;
use super::submit::submit_code;

use std::sync::atomic::{AtomicBool, Ordering};

// 日志工具
pub fn add_log(logs: &Arc<Mutex<Vec<String>>>, msg: &str) {
    let now = chrono::Local::now().format("%H:%M:%S").to_string();
    let mut logs_lock = logs.lock().unwrap();
    logs_lock.push(format!("[{}] {}", now, msg));
}

pub fn check_cancel(cancel_flag: &AtomicBool) -> Result<()> {
    if cancel_flag.load(Ordering::Relaxed) {
        return Err(anyhow::anyhow!("已手动停止 (Stopped by user)"));
    }
    Ok(())
}

pub fn run_daily_flow(
    logs: Arc<Mutex<Vec<String>>>,
    cancel_flag: Arc<AtomicBool>,
    browser_instance: Arc<Mutex<Option<headless_chrome::Browser>>>,
    problem_title_ref: Arc<Mutex<String>>,
    checkin_status_ref: Arc<Mutex<String>>,
    daily_problem_url_ref: Arc<Mutex<String>>,
) -> Result<String> {
    check_cancel(&cancel_flag)?;
    add_log(&logs, "正在启动浏览器... (Starting browser...)");
    
    // 复用或创建浏览器实例
    let browser_inst = {
        let mut browser_lock = browser_instance.lock().unwrap();
        let mut needs_new = true;
        let mut existing_browser = None;
        
        if let Some(b) = browser_lock.as_ref() {
            // 只要能获取版本号，就说明浏览器底层连接还活着
            if b.get_version().is_ok() {
                needs_new = false;
                existing_browser = Some(b.clone());
            }
        }
        
        if needs_new {
            add_log(&logs, "启动新浏览器实例 (Starting new browser instance)");
            let new_b = browser::launch_browser()?;
            *browser_lock = Some(new_b.clone());
            new_b
        } else {
            add_log(&logs, "复用已有浏览器实例 (Reusing existing browser instance)");
            existing_browser.unwrap()
        }
    };

    check_cancel(&cancel_flag)?;
    // 1. 登录流程
    let active_tab = login::ensure_login(&browser_inst, &logs, &cancel_flag)?;

    check_cancel(&cancel_flag)?;
    // 2. 获取每日一题 URL 及打卡状态
    let (problem_url, title, is_solved, consecutive_days) = get_daily_problem_url(&active_tab, &logs, &cancel_flag)?;
    
    // 更新 UI 状态
    *problem_title_ref.lock().unwrap() = title.clone();
    
    let mut display_status = if is_solved { "今日已打卡".to_string() } else { "今日未打卡".to_string() };
    if !consecutive_days.is_empty() {
        display_status = format!("{} (连续 {} 天)", display_status, consecutive_days);
    }
    
    *checkin_status_ref.lock().unwrap() = display_status.clone();
    *daily_problem_url_ref.lock().unwrap() = problem_url.clone();

    if is_solved {
        add_log(&logs, "✅ 检测到今日已打卡，流程结束 (Daily problem already solved)");
        return Ok(String::new());
    }

    check_cancel(&cancel_flag)?;
    // 3. 提取题解代码
    let (code, lang) = extract_solution_code(&active_tab, &problem_url, &logs, &cancel_flag)?;

    check_cancel(&cancel_flag)?;
    // 4. 回到题目页面填入代码并提交
    submit_code(&active_tab, &problem_url, &code, &lang, &logs, &cancel_flag)?;

    let mut final_status = "今日已打卡".to_string();
    if !consecutive_days.is_empty() {
        // 如果是刚刚打卡成功，连续天数可能增加了 1，这里只是简单展示，为了严谨可以不显示天数，
        // 或者简单追加之前获取的天数。这里我们保留天数显示，如果需要精确可再次抓取。
        final_status = format!("今日已打卡 (连续 {} 天)", consecutive_days);
    }
    *checkin_status_ref.lock().unwrap() = final_status;

    Ok(code)
}