use super::super::TodoState;

pub fn show(state: &mut TodoState, ui: &mut egui::Ui, state_changed: &mut bool) {
    let Some(item_id) = state.editing_reminder else {
        return;
    };

    let item = state.items.iter().find(|i| i.id == item_id);
    let item_title = item
        .map(|i| i.title.clone())
        .unwrap_or_default();
    let item_completed = item.map(|i| i.completed).unwrap_or(false);

    let mut open = true;
    egui::Window::new("提醒 (Reminder)")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ui.ctx(), |ui| {
            ui.label(format!("任务: {}", item_title));
            ui.add_space(8.0);
            ui.label("输入提醒时间：YYYY-MM-DD HH:MM");
            ui.text_edit_singleline(&mut state.reminder_input);
            if let Some(err) = &state.reminder_error_msg {
                ui.label(egui::RichText::new(err).color(egui::Color32::RED));
            }

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("取消 (Cancel)").clicked() {
                    state.editing_reminder = None;
                    state.reminder_error_msg = None;
                }

                if ui.button("清除提醒 (Clear)").clicked() {
                    if let Some(item) = state.items.iter_mut().find(|i| i.id == item_id) {
                        item.reminder_at = None;
                        item.reminder_sent = false;
                        *state_changed = true;
                    }
                    state.editing_reminder = None;
                    state.reminder_error_msg = None;
                }

                if ui.button("保存 (Save)").clicked() {
                    if item_completed {
                        state.reminder_error_msg =
                            Some("已完成任务不会提醒，请先取消完成状态".to_string());
                        return;
                    }

                    let Some(parsed) =
                        crate::todo::reminders::parse_local_datetime_to_utc(&state.reminder_input)
                    else {
                        state.reminder_error_msg =
                            Some("时间格式不正确，请用 YYYY-MM-DD HH:MM".to_string());
                        return;
                    };

                    if parsed <= chrono::Utc::now() {
                        state.reminder_error_msg =
                            Some("提醒时间已过去，请设置未来时间".to_string());
                        return;
                    }

                    if let Some(item) = state.items.iter_mut().find(|i| i.id == item_id) {
                        item.reminder_at = Some(parsed);
                        item.reminder_sent = false;
                        *state_changed = true;
                    }
                    state.editing_reminder = None;
                    state.reminder_error_msg = None;
                }
            });
        });

    if !open {
        state.editing_reminder = None;
        state.reminder_error_msg = None;
    }
}
