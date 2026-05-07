use super::state::{TodoItem, TodoState};

impl TodoState {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let mut state_changed = false;
        let mut delete_confirmed: Option<uuid::Uuid> = None;
        let mut reorder_request: Option<(uuid::Uuid, usize)> = None;
        if self.dragging_item.is_none() {
            self.drag_target_index = None;
        }
        
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
                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                    self.save_folder = Some(folder.to_string_lossy().to_string());
                    self.load_from_file();
                    state_changed = true;
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
                        self.items.push(TodoItem::new(self.new_task_title.clone(), desc));
                        self.new_task_title.clear();
                        self.new_task_description.clear();
                        state_changed = true;
                    }
                }
            });
            
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.new_task_description);
                ui.label("任务备注 (Description, 可选)");
            });
        });

        ui.add_space(16.0);
        ui.separator();
        
        egui::ScrollArea::vertical().id_salt("todo_list_scroll").show(ui, |ui| {
            for (index, item) in self.items.iter_mut().enumerate() {
                let item_id = item.id;
                let mut drag_handle_response: Option<egui::Response> = None;
                ui.vertical(|ui| {
                    let row_response = ui.horizontal(|ui| {
                        drag_handle_response = Some(
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new("≡")
                                        .size(14.0)
                                        .color(egui::Color32::GRAY),
                                )
                                .sense(egui::Sense::click_and_drag()),
                            ),
                        );

                        if let Some(drag_handle_response) = &drag_handle_response {
                            if drag_handle_response.drag_started() {
                                self.dragging_item = Some(item_id);
                                self.drag_target_index = Some(index);
                                self.item_to_delete = None;
                            }
                        }

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
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if self.item_to_delete == Some(item_id) {
                                if ui.button("取消 (Cancel)").clicked() {
                                    self.item_to_delete = None;
                                }
                                if ui.button("删除 (Delete)").clicked() {
                                    delete_confirmed = Some(item_id);
                                    self.item_to_delete = None;
                                }
                            } else if ui.button("🗑️").clicked() {
                                self.item_to_delete = Some(item_id);
                            }
                            ui.label(egui::RichText::new(&item.created_at).size(10.0).color(egui::Color32::GRAY));
                        });
                    }).response;

                    if self.dragging_item.is_some() && row_response.contains_pointer() {
                        if let Some(pointer_pos) = ui.ctx().pointer_hover_pos() {
                            let insert_index = if pointer_pos.y > row_response.rect.center().y {
                                index.saturating_add(1)
                            } else {
                                index
                            };
                            self.drag_target_index = Some(insert_index);
                        } else {
                            self.drag_target_index = Some(index);
                        }
                    }

                    if let Some(drag_handle_response) = &drag_handle_response {
                        if self.dragging_item == Some(item_id) && drag_handle_response.drag_stopped() {
                            if let Some(target_index) = self.drag_target_index {
                                reorder_request = Some((item_id, target_index));
                            }
                            self.dragging_item = None;
                            self.drag_target_index = None;
                        }
                    }
                    
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

        if let Some(index) = delete_confirmed {
            if let Some(pos) = self.items.iter().position(|i| i.id == index) {
                self.items.remove(pos);
                state_changed = true;
            }
        }

        if let Some((dragged_id, target_index)) = reorder_request {
            if let Some(from_index) = self.items.iter().position(|i| i.id == dragged_id) {
                let mut to_index = target_index.min(self.items.len());
                if to_index > from_index {
                    to_index = to_index.saturating_sub(1);
                }
                if from_index != to_index && to_index <= self.items.len().saturating_sub(1) {
                    let item = self.items.remove(from_index);
                    self.items.insert(to_index, item);
                    state_changed = true;
                }
            }
        }
        
        if state_changed {
            self.save_to_file();
        }
    }
}
