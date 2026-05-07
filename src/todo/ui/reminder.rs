use super::super::TodoState;

pub fn show(state: &mut TodoState, ui: &mut egui::Ui, state_changed: &mut bool) {
    let Some(item_id) = state.editing_reminder else {
        return;
    };

    let item_title = state
        .items
        .iter()
        .find(|i| i.id == item_id)
        .map(|i| i.title.clone())
        .unwrap_or_default();

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

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("取消 (Cancel)").clicked() {
                    state.editing_reminder = None;
                }

                if ui.button("清除提醒 (Clear)").clicked() {
                    if let Some(item) = state.items.iter_mut().find(|i| i.id == item_id) {
                        item.reminder_at = None;
                        item.reminder_sent = false;
                        *state_changed = true;
                    }
                    state.editing_reminder = None;
                }

                if ui.button("保存 (Save)").clicked() {
                    let parsed =
                        crate::todo::reminders::parse_local_datetime_to_utc(&state.reminder_input);
                    if let Some(item) = state.items.iter_mut().find(|i| i.id == item_id) {
                        item.reminder_at = parsed;
                        item.reminder_sent = false;
                        *state_changed = true;
                    }
                    state.editing_reminder = None;
                }
            });
        });

    if !open {
        state.editing_reminder = None;
    }
}

