use super::state::LeetCodeState;
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::atomic::Ordering;

impl LeetCodeState {
    pub fn ui(&mut self, ui: &mut egui::Ui, todo_state: &crate::todo::TodoState) {
        // 限制打卡开关，方便开发调试使用
        const ENABLE_CHECKIN_LIMIT: bool = false;

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        if self.checkin_date != today && !self.checkin_status.is_empty() {
            self.checkin_status = "待检测".to_string();
        }

        // 如果 Todo 中已有今天的自动化任务，或者当前状态本身就是已打卡，说明今天打过卡了
        let already_checked_in_today = 
            self.checkin_status.contains("已打卡") || 
            todo_state.has_today_automated_task("LeetCode");
            
        // 如果发现 Todo 里有但当前状态没有更新，自动更正状态（通常是在重启应用后发生）
        if already_checked_in_today && !self.checkin_status.contains("已打卡") {
            self.checkin_status = "今日已打卡 (从 Todo 清单恢复)".to_string();
            self.checkin_date = today.clone();
        }
        
        ui.heading("LeetCode 每日打卡 (LeetCode Daily Check-in)");
        
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label("题目名称 (Problem Title):");
            ui.text_edit_singleline(&mut self.problem_title);
        });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label("打卡状态 (Check-in Status):");
            let color = if self.checkin_status.contains("已打卡") {
                egui::Color32::GREEN
            } else if self.checkin_status.contains("待检测") {
                egui::Color32::from_rgb(255, 165, 0) // Orange
            } else {
                egui::Color32::RED
            };
            ui.label(egui::RichText::new(if self.checkin_status.is_empty() { "待检测" } else { &self.checkin_status })
                .color(color));
        });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label("题目链接 (Problem URL):");
            ui.text_edit_singleline(&mut self.daily_problem_url);
        });

        ui.add_space(8.0);
        ui.label("解答代码 (Solution Code):");
        egui::ScrollArea::vertical().id_salt("leetcode_code_scroll").max_height(300.0).show(ui, |ui| {
            let mut code = self.solution_code.lock().unwrap_or_else(|e| e.into_inner());
            ui.add(
                egui::TextEdit::multiline(&mut *code)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .desired_rows(15),
            );
        });

        ui.add_space(16.0);
        ui.horizontal(|ui| {
            let is_running = *self.is_running.lock().unwrap_or_else(|e| e.into_inner());
            let button_width = ui.available_width() * 0.4;
            
            if is_running {
                let btn = egui::Button::new(egui::RichText::new("⏹ 停止 (Stop)").size(16.0).color(egui::Color32::RED))
                    .min_size(egui::vec2(button_width, 30.0));
                    
                if ui.add(btn).clicked() {
                    self.cancel_flag.store(true, Ordering::Relaxed);
                    let mut logs = self.logs.lock().unwrap();
                    let now = chrono::Local::now().format("%H:%M:%S").to_string();
                    logs.push(format!("[{}] 正在尝试停止... (Stopping...)", now));
                }
            } else {
                let btn = egui::Button::new(egui::RichText::new("🚀 开始打卡 (Start Check-in)").size(16.0))
                    .min_size(egui::vec2(button_width, 30.0))
                    .sense(egui::Sense::click());
                
                //根据ENABLE_CHECKIN_LIMIT判断是否启用打卡按钮
                let button_chickin_enabled = if ENABLE_CHECKIN_LIMIT {
                    !already_checked_in_today
                } else {
                    true
                };
                let response = ui.add_enabled(button_chickin_enabled, btn);
                
                if ENABLE_CHECKIN_LIMIT&&already_checked_in_today {
                    // 我们只保留手动跟随鼠标的 Fallback Tooltip，移除可能造成重影的默认 hover_text
                    if response.rect.contains(ui.input(|i| i.pointer.hover_pos().unwrap_or_default())) {
                        #[allow(deprecated)]
                        egui::Tooltip::new(response.id.with("fallback"), ui.ctx().clone(), egui::PopupAnchor::Pointer, ui.layer_id())
                            .show(|ui| {
                                ui.label("今日已完成打卡，请明天再来 (Today's check-in is already completed)");
                            });
                    }
                }
                
                if response.clicked() {
                        self.cancel_flag.store(false, Ordering::Relaxed);
                        let is_running_clone = Arc::clone(&self.is_running);
                        let logs_clone = Arc::clone(&self.logs);
                        let code_clone = Arc::clone(&self.solution_code);
                        let last_submitted_clone = Arc::clone(&self.last_submitted);
                        let cancel_flag_clone = Arc::clone(&self.cancel_flag);
                        let browser_instance_clone = Arc::clone(&self.browser_instance);
                        
                        // We need to pass back strings to the UI state safely across threads
                        // using Arc<Mutex<String>> wrapper for problem_title and checkin_status
                        let problem_title_ref = Arc::new(Mutex::new(self.problem_title.clone()));
                        let checkin_status_ref = Arc::new(Mutex::new(self.checkin_status.clone()));
                        let daily_problem_url_ref = Arc::new(Mutex::new(self.daily_problem_url.clone()));
                        
                        let problem_title_clone = Arc::clone(&problem_title_ref);
                        let checkin_status_clone = Arc::clone(&checkin_status_ref);
                        let daily_problem_url_clone = Arc::clone(&daily_problem_url_ref);
                        let sync_status_clone = Arc::clone(&self.status_message);
                        
                        *is_running_clone.lock().unwrap() = true;
                        
                        // Clear previous logs and append start message
                        {
                            let mut logs = logs_clone.lock().unwrap();
                            logs.clear();
                            let now = chrono::Local::now().format("%H:%M:%S").to_string();
                            logs.push(format!("[{}] 启动浏览器... (Starting browser...)", now));
                        }

                        let ctx = ui.ctx().clone();

                        #[cfg(not(target_arch = "wasm32"))]
                        thread::spawn(move || {
                            use crate::leetcode::automation;
                            
                            let logs_for_automation = Arc::clone(&logs_clone);
                            match automation::run_daily_flow(
                                logs_for_automation, 
                                cancel_flag_clone, 
                                browser_instance_clone,
                                problem_title_clone.clone(),
                                checkin_status_clone.clone(),
                                daily_problem_url_clone.clone(),
                            ) {
                                Ok(code) => {
                                    // Sync back
                                    if let (Ok(title), Ok(status), Ok(url)) = (problem_title_clone.lock(), checkin_status_clone.lock(), daily_problem_url_clone.lock()) {
                                        *sync_status_clone.lock().unwrap() = format!("SYNC:|{}|{}|{}", *title, *status, *url);
                                    }
                                    if !code.is_empty() {
                                        *code_clone.lock().unwrap() = code.clone();
                                    }
                                    let mut logs = logs_clone.lock().unwrap();
                                    let now = chrono::Local::now().format("%H:%M:%S").to_string();
                                    logs.push(format!("[{}] 成功 (Success)! 代码长度: {}", now, code.len()));
                                    
                                    *last_submitted_clone.lock().unwrap() = Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
                                }
                                Err(e) => {
                                    // Sync back
                                    if let (Ok(title), Ok(status), Ok(url)) = (problem_title_clone.lock(), checkin_status_clone.lock(), daily_problem_url_clone.lock()) {
                                        *sync_status_clone.lock().unwrap() = format!("SYNC:|{}|{}|{}", *title, *status, *url);
                                    }
                                    let mut logs = logs_clone.lock().unwrap();
                                    let now = chrono::Local::now().format("%H:%M:%S").to_string();
                                    logs.push(format!("[{}] 错误 (Error): {}", now, e));
                                }
                            }
                            *is_running_clone.lock().unwrap() = false;
                            
                            // 强制刷新 UI 使得能够立即响应 SYNC 并添加到 Todo
                            ctx.request_repaint();
                        });
                        
                        #[cfg(target_arch = "wasm32")]
                        {
                            let mut logs = logs_clone.lock().unwrap();
                            let now = chrono::Local::now().format("%H:%M:%S").to_string();
                            logs.push(format!("[{}] WASM 暂不支持自动化操作 (WASM not supported for automation)", now));
                            *is_running_clone.lock().unwrap() = false;
                        }
                    }
            }
            
            let copy_btn = egui::Button::new("📋 复制答案 (Copy Answer)").min_size(egui::vec2(200.0, 30.0)).sense(egui::Sense::click());
            let copy_response = ui.add(copy_btn);
            
            if copy_response.rect.contains(ui.input(|i| i.pointer.hover_pos().unwrap_or_default())) {
                #[allow(deprecated)]
                egui::Tooltip::new(copy_response.id.with("fallback"), ui.ctx().clone(), egui::PopupAnchor::Pointer, ui.layer_id())
                    .show(|ui| {
                        ui.label("点击将代码复制到剪贴板 (Click to copy code to clipboard)");
                    });
            }

            if copy_response.clicked() {
                let code = self.solution_code.lock().unwrap_or_else(|e| e.into_inner()).clone();
                ui.ctx().copy_text(code);
            }
        });

        let logs = self.logs.lock().unwrap_or_else(|e| e.into_inner()).clone();
        if !logs.is_empty() {
            ui.add_space(8.0);
            ui.label("执行日志 (Run Logs):");
            egui::ScrollArea::vertical().id_salt("leetcode_logs_scroll").max_height(100.0).stick_to_bottom(true).show(ui, |ui| {
                for log in logs {
                    ui.label(egui::RichText::new(log).color(egui::Color32::LIGHT_BLUE));
                }
            });
        }

        let last_sub = self.last_submitted.lock().unwrap_or_else(|e| e.into_inner()).clone();
        if let Some(time) = last_sub {
            ui.add_space(8.0);
            ui.label(egui::RichText::new(format!("最后提交时间 (Last Submitted): {}", time)).color(egui::Color32::LIGHT_GREEN));
        }
    }
}
