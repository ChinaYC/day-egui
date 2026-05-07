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

    let mut open = true;
    egui::Window::new("设置 (Settings)")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ui.ctx(), |ui| {
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
        });

    if !open {
        state.show_settings = false;
    }
}

