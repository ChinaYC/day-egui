use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use headless_chrome::Browser;

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub struct LeetCodeState {
    pub daily_problem_url: String,
    pub problem_title: String,
    pub checkin_status: String,
    pub checkin_date: String,
    pub language: String,
    
    #[serde(skip)]
    pub solution_code: Arc<Mutex<String>>,
    
    #[serde(skip)]
    pub last_submitted: Arc<Mutex<Option<String>>>,
    
    #[serde(skip)]
    pub is_running: Arc<Mutex<bool>>,
    
    #[serde(skip)]
    pub logs: Arc<Mutex<Vec<String>>>,
    
    #[serde(skip)]
    pub cancel_flag: Arc<AtomicBool>,

    #[serde(skip)]
    pub browser_instance: Arc<Mutex<Option<Browser>>>,
    
    #[serde(skip)]
    pub status_message: Arc<Mutex<String>>,

    #[serde(skip)]
    pub newly_completed_task: Option<(String, String)>,
}

impl LeetCodeState {
    /// Syncs background task state to UI state and returns a new task if one was completed.
    /// Returns: Option<(Task Title, Task Source, Option<Description>)>
    pub fn sync_background_state(&mut self) -> Option<(String, String, Option<String>)> {
        let mut newly_completed = None;
        
        let status_arc = Arc::clone(&self.status_message);
        if let Ok(mut msg) = status_arc.try_lock() {
            if msg.starts_with("SYNC:") {
                let parts: Vec<&str> = msg.split('|').collect();
                if parts.len() >= 3 {
                    self.problem_title = parts[1].to_string();
                    let new_status = parts[2].to_string();
                    
                    let mut url = String::new();
                    if parts.len() >= 4 {
                        url = parts[3].to_string();
                        self.daily_problem_url = url.clone();
                    }

                    // 记录打卡完成的新任务
                    if new_status.contains("已打卡") {
                        let description = if url.is_empty() {
                            format!("状态: {}", new_status)
                        } else {
                            format!("题目链接: {}\n状态: {}", url, new_status)
                        };
                        
                        newly_completed = Some((
                            format!("LeetCode 每日打卡: {}", self.problem_title),
                            "LeetCode".to_string(),
                            Some(description)
                        ));
                    }
                    
                    self.checkin_status = new_status;
                    self.checkin_date = chrono::Local::now().format("%Y-%m-%d").to_string();
                    
                    *msg = String::new(); // 清除消息避免重复处理
                }
            }
        }
        
        newly_completed
    }
}
