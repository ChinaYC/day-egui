use anyhow::Result;
use std::sync::{Arc, Mutex};

// 导入重构后的模块
use super::get_daily::get_daily_problem_url;
use super::submit::submit_code;

use std::sync::atomic::{AtomicBool, Ordering};
use tracing::info;

// 日志工具
pub fn add_log(logs: &Arc<Mutex<Vec<String>>>, msg: &str) {
    let mut logs_lock = logs.lock().unwrap();
    logs_lock.push(msg.to_string());
    if logs_lock.len() > 500 {
        let overflow = logs_lock.len() - 500;
        logs_lock.drain(0..overflow);
    }
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
    info!(user = true, "正在启动浏览器... (Starting browser...)");

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
            info!(user = true, "启动新浏览器实例 (Starting new browser instance)");
            let new_b = super::browser::launch_browser()?;
            *browser_lock = Some(new_b.clone());
            new_b
        } else {
            info!(user = true, "复用已有浏览器实例 (Reusing existing browser instance)");
            existing_browser.unwrap()
        }
    };

    check_cancel(&cancel_flag)?;
    // 1. 登录流程
    let active_tab = super::login::ensure_login(&browser_inst, &logs, &cancel_flag)?;

    check_cancel(&cancel_flag)?;
    // 2. 获取每日一题 URL 及打卡状态
    let (problem_url, title, is_solved, consecutive_days) =
        get_daily_problem_url(&active_tab, &logs, &cancel_flag)?;

    // 更新 UI 状态
    *problem_title_ref.lock().unwrap() = title.clone();

    let mut display_status = if is_solved {
        "今日已打卡".to_string()
    } else {
        "今日未打卡".to_string()
    };
    if !consecutive_days.is_empty() {
        display_status = format!("{} (连续 {} 天)", display_status, consecutive_days);
    }

    *checkin_status_ref.lock().unwrap() = display_status.clone();
    *daily_problem_url_ref.lock().unwrap() = problem_url.clone();

    if is_solved {
        info!(user = true, "✅ 检测到今日已打卡，流程结束 (Daily problem already solved)");
        return Ok(String::new());
    }

    check_cancel(&cancel_flag)?;
    // 3. 提取题解代码
    let (code, lang) = super::solution::extract_solution_code(&active_tab, &problem_url, &logs, &cancel_flag)?;

    check_cancel(&cancel_flag)?;
    // 4. 直接在题解页面右侧编辑器填入代码并提交
    submit_code(&active_tab, &code, &lang, &logs, &cancel_flag)?;

    let mut final_status = format!("今日已打卡 ({})", lang);
    if !consecutive_days.is_empty() {
        final_status = format!("今日已打卡 ({}) (连续 {} 天)", lang, consecutive_days);
    }
    *checkin_status_ref.lock().unwrap() = final_status;

    Ok(code)
}
