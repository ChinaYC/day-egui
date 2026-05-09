use super::super::TodoState;

pub fn show(state: &mut TodoState, ui: &mut egui::Ui, state_changed: &mut bool) {
    egui::Area::new("todo_settings_button".into())
        .anchor(egui::Align2::LEFT_BOTTOM, [10.0, -10.0])
        .show(ui.ctx(), |ui| {
            if ui.button("⚙").clicked() {
                state.show_settings = true;
            }
        });

    if !state.show_settings {
        return;
    }

    let auto_id = state.get_or_create_section_id_by_name("自动 (Auto)");
    let manual_id = state.get_or_create_section_id_by_name("手动 (Manual)");

    let mut open = true;
    egui::Window::new("设置 (Settings)")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ui.ctx(), |ui| {
            ui.label("字体大小 (Font size):");
            let mut font_scale = state.settings.font_scale;
            if ui
                .add(
                    egui::Slider::new(&mut font_scale, 0.8..=1.6)
                        .show_value(true)
                        .clamping(egui::SliderClamping::Always),
                )
                .changed()
            {
                state.settings.font_scale = font_scale;
                *state_changed = true;
            }

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);

            ui.label("自动任务默认分区 (Automated tasks go to):");
            let mut selected = state.settings.automated_section_id;
            egui::ComboBox::from_id_salt("automated_section_select")
                .selected_text(
                    selected
                        .and_then(|id| state.section_name(id).map(|s| s.to_string()))
                        .unwrap_or_else(|| "未设置 (Unset)".to_string()),
                )
                .show_ui(ui, |ui| {
                    for section in &state.sections {
                        ui.selectable_value(&mut selected, Some(section.id), section.name.clone());
                    }
                });
            if selected != state.settings.automated_section_id {
                state.settings.automated_section_id = selected;
                *state_changed = true;
            }

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);

            ui.label("新建分区 (Create section):");
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut state.new_section_name);
                if ui.button("创建 (Create)").clicked() {
                    let name = state.new_section_name.trim().to_string();
                    if !name.is_empty() {
                        state.sections.push(super::super::TodoSection::new(name));
                        state.new_section_name.clear();
                        *state_changed = true;
                    }
                }
            });

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);

            ui.label("分区管理 (Manage sections):");
            let sections: Vec<(uuid::Uuid, String)> = state
                .sections
                .iter()
                .map(|s| (s.id, s.name.clone()))
                .collect();
            for (section_id, section_name) in sections {
                ui.horizontal(|ui| {
                    ui.label(section_name);

                    if section_id == auto_id || section_id == manual_id {
                        ui.add_enabled(false, egui::Button::new("重命名"));
                        ui.add_enabled(false, egui::Button::new("删除"));
                        return;
                    }

                    if ui.button("重命名").clicked() {
                        state.section_to_rename = Some(section_id);
                        state.section_rename_input = state
                            .section_name(section_id)
                            .unwrap_or_default()
                            .to_string();
                        state.section_manage_error_msg = None;
                    }
                    if ui.button("删除").clicked() {
                        state.section_to_delete = Some(section_id);
                        state.section_delete_move_to = Some(manual_id);
                        state.section_manage_error_msg = None;
                    }
                });
            }

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);

            ui.label("数据工具 (Data tools):");
            ui.horizontal(|ui| {
                if ui.button("导出任务 JSONL").clicked() {
                    if let Some(path) = state.export_jsonl(false) {
                        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                            state.show_snackbar(format!("已导出: {name}"));
                        } else {
                            state.show_snackbar("已导出");
                        }
                    }
                }
                if ui.button("导出回收站 JSONL").clicked() {
                    if let Some(path) = state.export_jsonl(true) {
                        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                            state.show_snackbar(format!("已导出: {name}"));
                        } else {
                            state.show_snackbar("已导出");
                        }
                    }
                }
            });
        });

    if !open {
        state.show_settings = false;
    }

    if let Some(section_id) = state.section_to_rename {
        let mut rename_open = true;
        egui::Window::new("重命名分区 (Rename section)")
            .open(&mut rename_open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                ui.label("新名称 (New name):");
                ui.text_edit_singleline(&mut state.section_rename_input);
                if let Some(err) = &state.section_manage_error_msg {
                    ui.label(egui::RichText::new(err).color(egui::Color32::RED));
                }

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("取消").clicked() {
                        state.section_to_rename = None;
                        state.section_manage_error_msg = None;
                    }
                    if ui.button("保存").clicked() {
                        let name = state.section_rename_input.trim().to_string();
                        if name.is_empty() {
                            state.section_manage_error_msg = Some("分区名称不能为空".to_string());
                            return;
                        }
                        if state.sections.iter().any(|s| s.id != section_id && s.name == name) {
                            state.section_manage_error_msg = Some("分区名称已存在".to_string());
                            return;
                        }
                        if let Some(section) = state.sections.iter_mut().find(|s| s.id == section_id)
                        {
                            section.name = name;
                            *state_changed = true;
                        }
                        state.section_to_rename = None;
                        state.section_manage_error_msg = None;
                    }
                });
            });
        if !rename_open {
            state.section_to_rename = None;
            state.section_manage_error_msg = None;
        }
    }

    if let Some(section_id) = state.section_to_delete {
        let mut delete_open = true;
        egui::Window::new("删除分区 (Delete section)")
            .open(&mut delete_open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                let section_name = state.section_name(section_id).unwrap_or("未知分区");
                ui.label(format!(
                    "确认删除分区：{}\n该分区下的任务将移动到：",
                    section_name
                ));

                let mut move_to = state.section_delete_move_to.or(Some(manual_id));
                egui::ComboBox::from_id_salt("delete_section_move_to")
                    .selected_text(
                        move_to
                            .and_then(|id| state.section_name(id).map(|s| s.to_string()))
                            .unwrap_or_else(|| "请选择".to_string()),
                    )
                    .show_ui(ui, |ui| {
                        for section in &state.sections {
                            if section.id == section_id {
                                continue;
                            }
                            ui.selectable_value(&mut move_to, Some(section.id), section.name.clone());
                        }
                    });
                state.section_delete_move_to = move_to;

                if let Some(err) = &state.section_manage_error_msg {
                    ui.label(egui::RichText::new(err).color(egui::Color32::RED));
                }

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("取消").clicked() {
                        state.section_to_delete = None;
                        state.section_manage_error_msg = None;
                    }
                    if ui.button("确认删除").clicked() {
                        let Some(move_to) = state.section_delete_move_to else {
                            state.section_manage_error_msg = Some("请选择迁移目标分区".to_string());
                            return;
                        };
                        if move_to == section_id {
                            state.section_manage_error_msg = Some("迁移目标不能是当前分区".to_string());
                            return;
                        }

                        for item in &mut state.items {
                            if item.section_id == Some(section_id) {
                                item.section_id = Some(move_to);
                            }
                        }
                        state.sections.retain(|s| s.id != section_id);

                        if state.settings.automated_section_id == Some(section_id) {
                            state.settings.automated_section_id = Some(auto_id);
                        }
                        if state.new_task_section == Some(section_id) {
                            state.new_task_section = Some(move_to);
                        }
                        if state.active_section == Some(section_id) {
                            state.active_section = None;
                        }

                        state.ensure_builtin_sections_and_settings();
                        *state_changed = true;

                        state.section_to_delete = None;
                        state.section_delete_move_to = None;
                        state.section_manage_error_msg = None;
                    }
                });
            });

        if !delete_open {
            state.section_to_delete = None;
            state.section_manage_error_msg = None;
        }
    }
}
