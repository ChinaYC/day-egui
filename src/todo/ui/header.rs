use super::super::{TodoItem, TodoState};

pub fn show(state: &mut TodoState, ui: &mut egui::Ui, state_changed: &mut bool) {
    ui.heading("日常 Todo 清单 (Daily Todo List)");

    let auto_count = state.get_today_automated_count();
    if auto_count > 0 {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(format!(
                "🤖 今日自动化完成任务数: {} (Automated tasks today)",
                auto_count
            ))
            .color(egui::Color32::LIGHT_GREEN),
        );
    }

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.label("📁 保存位置 (Save Location):");
        let path_display = match &state.save_folder {
            Some(p) => p.clone(),
            None => "默认配置 (Default Storage)".to_string(),
        };
        ui.label(egui::RichText::new(path_display).color(egui::Color32::LIGHT_BLUE));

        if ui.button("更改 (Change)").clicked() {
            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                state.save_folder = Some(folder.to_string_lossy().to_string());
                state.load_from_file();
                *state_changed = true;
            }
        }

        if state.save_folder.is_some() && ui.button("重置 (Reset)").clicked() {
            state.save_folder = None;
            *state_changed = true;
        }
    });

    if let Some(err) = &state.error_msg {
        ui.label(egui::RichText::new(err).color(egui::Color32::RED));
        if ui.button("清除错误 (Clear)").clicked() {
            state.error_msg = None;
        }
    }

    ui.add_space(8.0);
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            let response = ui.text_edit_singleline(&mut state.new_task_title);
            ui.label("任务标题 (Title)");

            egui::ComboBox::from_id_salt("new_task_section")
                .selected_text(
                    state
                        .new_task_section
                        .and_then(|id| state.section_name(id).map(|s| s.to_string()))
                        .unwrap_or_else(|| "选择分区 (Section)".to_string()),
                )
                .show_ui(ui, |ui| {
                    for section in &state.sections {
                        ui.selectable_value(
                            &mut state.new_task_section,
                            Some(section.id),
                            section.name.clone(),
                        );
                    }
                });

            if ui.button("➕ 添加任务 (Add Task)").clicked()
                || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
            {
                if !state.new_task_title.trim().is_empty() {
                    let desc = if state.new_task_description.trim().is_empty() {
                        None
                    } else {
                        Some(state.new_task_description.clone())
                    };
                    let section_id = state.new_task_section;
                    state
                        .items
                        .push(TodoItem::new(state.new_task_title.clone(), desc, section_id));
                    state.new_task_title.clear();
                    state.new_task_description.clear();
                    *state_changed = true;
                }
            }
        });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut state.new_task_description);
            ui.label("任务备注 (Description, 可选)");
        });
    });

    ui.add_space(16.0);
    ui.separator();
}

