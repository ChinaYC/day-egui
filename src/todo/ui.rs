use super::state::{TodoItem, TodoState};

impl TodoState {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let mut state_changed = false;
        
        ui.heading("日常 Todo 清单 (Daily Todo List)");

        let auto_count = self.get_today_automated_count();
        if auto_count > 0 {
            ui.add_space(4.0);
            ui.label(egui::RichText::new(format!("🤖 今日自动化完成任务数: {} (Automated tasks today)", auto_count))
                .color(egui::Color32::LIGHT_GREEN));
        }

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label("📁 保存位置 (Save Location):");
            let path_display = match &self.save_folder {
                Some(p) => p.clone(),
                None => "默认配置 (Default Storage)".to_string(),
            };
            ui.label(egui::RichText::new(path_display).color(egui::Color32::LIGHT_BLUE));
            
            if ui.button("更改 (Change)").clicked() {
                #[cfg(not(any(target_os = "android", target_os = "ios", target_arch = "wasm32")))]
                {
                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                        self.save_folder = Some(folder.to_string_lossy().to_string());
                        self.load_from_file();
                        state_changed = true;
                    }
                }
                #[cfg(any(target_os = "android", target_os = "ios", target_arch = "wasm32"))]
                {
                    self.error_msg = Some("此平台暂不支持自定义保存路径 (Custom save path not supported on this platform)".to_string());
                }
            }
            
            if self.save_folder.is_some() && ui.button("重置 (Reset)").clicked() {
                self.save_folder = None;
                state_changed = true;
            }
        });
        
        if let Some(err) = &self.error_msg {
            ui.label(egui::RichText::new(err).color(egui::Color32::RED));
            if ui.button("清除错误 (Clear)").clicked() {
                self.error_msg = None;
            }
        }

        ui.add_space(8.0);
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let response = ui.text_edit_singleline(&mut self.new_task_title);
                ui.label("任务标题 (Title)");
                
                if ui.button("➕ 添加任务 (Add Task)").clicked()
                    || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                {
                    if !self.new_task_title.trim().is_empty() {
                        let desc = if self.new_task_description.trim().is_empty() {
                            None
                        } else {
                            Some(self.new_task_description.clone())
                        };
                        let category = if self.new_task_category.trim().is_empty() {
                            None
                        } else {
                            Some(self.new_task_category.clone())
                        };
                        let reminder = if self.new_task_reminder.trim().is_empty() {
                            None
                        } else {
                            Some(self.new_task_reminder.clone())
                        };
                        
                        let tags: Vec<String> = self.new_task_tags
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();

                        let mut new_item = TodoItem::new(self.new_task_title.clone(), desc, category, reminder);
                        new_item.tags = tags;
                        
                        self.items.push(new_item);
                        self.new_task_title.clear();
                        self.new_task_description.clear();
                        self.new_task_category.clear();
                        self.new_task_reminder.clear();
                        self.new_task_tags.clear();
                        state_changed = true;
                    }
                }
            });
            
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("描述:");
                ui.text_edit_singleline(&mut self.new_task_description);
                
                ui.add_space(10.0);
                ui.label("标签(用逗号分隔):");
                ui.text_edit_singleline(&mut self.new_task_tags);
            });
        });

        ui.add_space(16.0);
        ui.horizontal(|ui| {
            ui.label("🏷 标签过滤:");
            let mut filter_text = self.filter_tag.clone().unwrap_or_default();
            if ui.text_edit_singleline(&mut filter_text).changed() {
                if filter_text.trim().is_empty() {
                    self.filter_tag = None;
                } else {
                    self.filter_tag = Some(filter_text.clone());
                }
            }
            if ui.button("清除").clicked() {
                self.filter_tag = None;
            }
        });
        
        ui.separator();
        
        egui::ScrollArea::vertical().id_salt("todo_list_scroll").show(ui, |ui| {
            let mut visible_items: Vec<(usize, &mut TodoItem)> = self.items.iter_mut().enumerate()
                .filter(|(_, item)| !item.is_deleted)
                .filter(|(_, item)| {
                    if let Some(filter) = &self.filter_tag {
                        item.tags.iter().any(|t| t.contains(filter))
                    } else {
                        true
                    }
                })
                .collect();
                
            // 倒序排列，最后加的在最前
            visible_items.reverse();

            for (index, item) in visible_items {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        if ui.checkbox(&mut item.completed, "").changed() {
                            state_changed = true;
                        }
                        
                        let title_text = if item.completed {
                            egui::RichText::new(&item.title).strikethrough().color(egui::Color32::DARK_GRAY)
                        } else {
                            egui::RichText::new(&item.title)
                        };
                        
                        ui.label(title_text);
                        
                        if item.is_automated {
                            ui.label(egui::RichText::new("🤖 自动 (Auto)").size(10.0).color(egui::Color32::LIGHT_GREEN));
                        }
                        
                        for tag in &item.tags {
                            ui.label(egui::RichText::new(format!("[{}]", tag)).size(10.0).color(egui::Color32::GOLD));
                        }
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("🗑️").clicked() {
                                self.item_to_delete = Some(index);
                            }
                            ui.label(egui::RichText::new(&item.created_at).size(10.0).color(egui::Color32::GRAY));
                        });
                    });
                    
                    if let Some(desc) = &item.description {
                        if !desc.is_empty() {
                            ui.horizontal(|ui| {
                                ui.add_space(24.0); // indent to match text
                                ui.label(egui::RichText::new(desc).size(12.0).color(egui::Color32::DARK_GRAY));
                            });
                        }
                    }
                });
            }
        });
        
        // 删除二次确认弹窗
        if let Some(index) = self.item_to_delete {
            let mut is_open = true;
            let item_title = self.items.get(index).map(|i| i.title.clone()).unwrap_or_default();
            
            egui::Window::new("确认删除 (Confirm Deletion)")
                .collapsible(false)
                .resizable(false)
                .open(&mut is_open)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.label(format!("确定要删除任务 \"{}\" 吗？\n(Are you sure you want to delete this task?)", item_title));
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("取消 (Cancel)").clicked() {
                            self.item_to_delete = None;
                        }
                        if ui.button("确定 (Confirm)").clicked() {
                            if index < self.items.len() {
                                self.items[index].is_deleted = true; // 逻辑删除
                                state_changed = true;
                            }
                            self.item_to_delete = None;
                        }
                    });
                });
                
            if !is_open {
                self.item_to_delete = None;
            }
        }
        
        if state_changed {
            self.save_to_file();
        }
    }
}
