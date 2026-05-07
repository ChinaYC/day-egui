use super::state::{TodoItem, TodoState};

impl TodoState {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        self.ensure_builtin_sections_and_settings();
        self.migrate_items_without_section();

        let mut state_changed = false;
        // egui 的常见写法：用局部变量收集“本帧 UI 事件”，循环结束后再统一修改 self.items。
        // 这样可以避免在 iter_mut() 遍历时直接对 Vec 做结构性修改（remove/insert）。
        let mut delete_confirmed: Option<uuid::Uuid> = None;
        let mut reorder_request: Option<(uuid::Uuid, uuid::Uuid, usize)> = None;
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

                egui::ComboBox::from_id_salt("new_task_section")
                    .selected_text(
                        self.new_task_section
                            .and_then(|id| self.section_name(id).map(|s| s.to_string()))
                            .unwrap_or_else(|| "选择分区 (Section)".to_string()),
                    )
                    .show_ui(ui, |ui| {
                        for section in &self.sections {
                            ui.selectable_value(
                                &mut self.new_task_section,
                                Some(section.id),
                                section.name.clone(),
                            );
                        }
                    });
                
                if ui.button("➕ 添加任务 (Add Task)").clicked()
                    || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                {
                    if !self.new_task_title.trim().is_empty() {
                        let desc = if self.new_task_description.trim().is_empty() {
                            None
                        } else {
                            Some(self.new_task_description.clone())
                        };
                        let section_id = self.new_task_section;
                        self.items
                            .push(TodoItem::new(self.new_task_title.clone(), desc, section_id));
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

        ui.horizontal(|ui| {
            if ui
                .selectable_label(self.active_section.is_none(), "全部 (All)")
                .clicked()
            {
                self.active_section = None;
                self.dragging_item = None;
                self.dragging_section = None;
                self.drag_target_index = None;
                self.item_to_delete = None;
            }

            for section in &self.sections {
                if ui
                    .selectable_label(self.active_section == Some(section.id), section.name.clone())
                    .clicked()
                {
                    self.active_section = Some(section.id);
                    self.dragging_item = None;
                    self.dragging_section = None;
                    self.drag_target_index = None;
                    self.item_to_delete = None;
                }
            }
        });
        ui.add_space(8.0);
        
        egui::ScrollArea::vertical().id_salt("todo_list_scroll").show(ui, |ui| {
            let sections_to_render: Vec<uuid::Uuid> = if let Some(active) = self.active_section {
                vec![active]
            } else {
                self.sections.iter().map(|s| s.id).collect()
            };

            for section_id in sections_to_render {
                let section_name = self
                    .section_name(section_id)
                    .unwrap_or("未知分区 (Unknown)")
                    .to_string();

                ui.add_space(4.0);
                ui.label(egui::RichText::new(section_name).strong());
                ui.add_space(6.0);

                let indices: Vec<usize> = self
                    .items
                    .iter()
                    .enumerate()
                    .filter_map(|(i, item)| (item.section_id == Some(section_id)).then_some(i))
                    .collect();

                for (visible_index, item_index) in indices.iter().copied().enumerate() {
                    let item = &mut self.items[item_index];
                    let item_id = item.id;
                    let mut drag_handle_response: Option<egui::Response> = None;

                    ui.vertical(|ui| {
                        let row_response = ui
                            .horizontal(|ui| {
                                drag_handle_response = Some(
                                    ui.add(
                                        egui::Label::new(
                                            egui::RichText::new("≡")
                                                .size(14.0)
                                                .color(egui::Color32::GRAY),
                                        )
                                        // Sense::click_and_drag() 表示这个控件既能点击也能拖拽，
                                        // 返回的 Response 会提供 drag_started/dragged/drag_stopped 等 API。
                                        .sense(egui::Sense::click_and_drag()),
                                    ),
                                );

                                if let Some(drag_handle_response) = &drag_handle_response {
                                    // drag_started(): 本帧从“未拖拽”变为“开始拖拽”的那一刻（只会为 true 一帧）
                                    if drag_handle_response.drag_started() {
                                        self.dragging_item = Some(item_id);
                                        self.dragging_section = Some(section_id);
                                        self.drag_target_index = Some(visible_index);
                                        self.item_to_delete = None;
                                    }
                                }

                                if ui.checkbox(&mut item.completed, "").changed() {
                                    state_changed = true;
                                }

                                let title_text = if item.completed {
                                    egui::RichText::new(&item.title)
                                        .strikethrough()
                                        .color(egui::Color32::DARK_GRAY)
                                } else {
                                    egui::RichText::new(&item.title)
                                };

                                ui.label(title_text);

                                if item.is_automated {
                                    ui.label(
                                        egui::RichText::new("🤖 自动 (Auto)")
                                            .size(10.0)
                                            .color(egui::Color32::LIGHT_GREEN),
                                    );
                                }

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if self.item_to_delete == Some(item_id) {
                                            if ui.button("取消 (Cancel)").clicked() {
                                                self.item_to_delete = None;
                                            }
                                            if ui.button("删除 (Delete)").clicked() {
                                                delete_confirmed = Some(item_id);
                                                self.item_to_delete = None;
                                            }
                                        } else if ui.button("🗑️").clicked() {
                                            // 二次确认：第一次点🗑️只进入“待确认”状态，不会立刻删除。
                                            self.item_to_delete = Some(item_id);
                                        }
                                        ui.label(
                                            egui::RichText::new(&item.created_at)
                                                .size(10.0)
                                                .color(egui::Color32::GRAY),
                                        );
                                    },
                                );
                            })
                            .response;

                        // contains_pointer()：即使另一个控件正在被拖拽，hovered() 可能为 false；
                        // 对拖拽目标判定更适合用 contains_pointer()。
                        if self.dragging_item.is_some()
                            && self.dragging_section == Some(section_id)
                            && row_response.contains_pointer()
                        {
                            if let Some(pointer_pos) = ui.ctx().pointer_hover_pos() {
                                // 这里用鼠标在 row 的上半部/下半部，决定插入到当前 visible_index 之前还是之后，
                                // 最终把它表达成“目标插入索引”（同分区内的相对索引）。
                                let insert_index = if pointer_pos.y > row_response.rect.center().y {
                                    visible_index.saturating_add(1)
                                } else {
                                    visible_index
                                };
                                self.drag_target_index = Some(insert_index);
                            } else {
                                self.drag_target_index = Some(visible_index);
                            }
                        }

                        if let Some(drag_handle_response) = &drag_handle_response {
                            // drag_stopped(): 拖拽结束的那一帧（只会为 true 一帧），适合在这里提交重排请求。
                            if self.dragging_item == Some(item_id)
                                && self.dragging_section == Some(section_id)
                                && drag_handle_response.drag_stopped()
                            {
                                if let Some(target_index) = self.drag_target_index {
                                    reorder_request = Some((item_id, section_id, target_index));
                                }
                                self.dragging_item = None;
                                self.dragging_section = None;
                                self.drag_target_index = None;
                            }
                        }

                        if let Some(desc) = &item.description {
                            if !desc.is_empty() {
                                ui.horizontal(|ui| {
                                    ui.add_space(24.0); // indent to match text
                                    ui.label(
                                        egui::RichText::new(desc)
                                            .size(12.0)
                                            .color(egui::Color32::DARK_GRAY),
                                    );
                                });
                            }
                        }
                    });
                }

                ui.add_space(8.0);
            }
        });

        if let Some(index) = delete_confirmed {
            if let Some(pos) = self.items.iter().position(|i| i.id == index) {
                self.items.remove(pos);
                state_changed = true;
            }
        }

        if let Some((dragged_id, section_id, target_index)) = reorder_request {
            // 同一个分区内部排序：通过 remove + insert 实现。
            // 关键点：target_index 是“同分区内的插入点”，因此需要先把它映射到全局 Vec 的插入位置。
            if self.reorder_item_in_section(dragged_id, section_id, target_index) {
                state_changed = true;
            }
        }
        
        egui::Area::new("todo_settings_button".into())
            .anchor(egui::Align2::LEFT_BOTTOM, [10.0, -10.0])
            .show(ui.ctx(), |ui| {
                if ui.button("⚙").clicked() {
                    self.show_settings = true;
                }
            });

        if self.show_settings {
            let mut open = true;
            egui::Window::new("设置 (Settings)")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.label("自动任务默认分区 (Automated tasks go to):");
                    let mut selected = self.settings.automated_section_id;
                    egui::ComboBox::from_id_salt("automated_section_select")
                        .selected_text(
                            selected
                                .and_then(|id| self.section_name(id).map(|s| s.to_string()))
                                .unwrap_or_else(|| "未设置 (Unset)".to_string()),
                        )
                        .show_ui(ui, |ui| {
                            for section in &self.sections {
                                ui.selectable_value(
                                    &mut selected,
                                    Some(section.id),
                                    section.name.clone(),
                                );
                            }
                        });
                    if selected != self.settings.automated_section_id {
                        self.settings.automated_section_id = selected;
                        state_changed = true;
                    }

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.label("新建分区 (Create section):");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.new_section_name);
                        if ui.button("创建 (Create)").clicked() {
                            let name = self.new_section_name.trim().to_string();
                            if !name.is_empty() {
                                self.sections.push(super::state::TodoSection::new(name));
                                self.new_section_name.clear();
                                state_changed = true;
                            }
                        }
                    });
                });

            if !open {
                self.show_settings = false;
            }
        }

        if state_changed {
            self.save_to_file();
        }
    }

    fn reorder_item_in_section(
        &mut self,
        dragged_id: uuid::Uuid,
        section_id: uuid::Uuid,
        target_index: usize,
    ) -> bool {
        let from_index = match self.items.iter().position(|i| i.id == dragged_id) {
            Some(i) => i,
            None => return false,
        };

        if self.items.get(from_index).and_then(|i| i.section_id) != Some(section_id) {
            return false;
        }

        let indices_before: Vec<usize> = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(i, it)| (it.section_id == Some(section_id)).then_some(i))
            .collect();
        let from_pos = match indices_before.iter().position(|&i| i == from_index) {
            Some(p) => p,
            None => return false,
        };

        // target_index 是“同分区内的插入点”，但它的计算发生在 remove 之前；
        // 因此当目标插入点在被拖拽元素之后时，remove 会让插入点左移一格。
        let mut target_pos = target_index.min(indices_before.len());
        if target_pos > from_pos {
            target_pos = target_pos.saturating_sub(1);
        }

        let item = self.items.remove(from_index);

        let indices_after: Vec<usize> = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(i, it)| (it.section_id == Some(section_id)).then_some(i))
            .collect();

        let insert_global_index = if indices_after.is_empty() {
            self.items.len()
        } else if target_pos >= indices_after.len() {
            indices_after.last().copied().unwrap_or(self.items.len()) + 1
        } else {
            indices_after[target_pos]
        };

        let insert_global_index = insert_global_index.min(self.items.len());
        self.items.insert(insert_global_index, item);
        true
    }
}
