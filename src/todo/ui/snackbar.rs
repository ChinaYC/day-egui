use super::super::TodoState;

pub fn show(state: &mut TodoState, ui: &mut egui::Ui, state_changed: &mut bool) {
    let (message, expires_at) = match state.snackbar.as_ref() {
        Some(s) => (s.message.clone(), s.expires_at),
        None => return,
    };

    if std::time::Instant::now() >= expires_at {
        state.snackbar = None;
        return;
    }

    let can_undo = !state.undo_stack.is_empty();
    egui::Area::new("todo_snackbar".into())
        .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -18.0])
        .show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style())
                .fill(egui::Color32::from_rgba_premultiplied(30, 30, 30, 220))
                .corner_radius(egui::CornerRadius::same(10))
                .inner_margin(egui::Margin::symmetric(12, 8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(message.as_str()).color(egui::Color32::WHITE),
                        );
                        ui.add_space(12.0);
                        if ui
                            .add_enabled(
                                can_undo,
                                egui::Button::new(
                                    egui::RichText::new("撤销").color(egui::Color32::LIGHT_BLUE),
                                ),
                            )
                            .clicked()
                        {
                            if state.undo_last_action() {
                                *state_changed = true;
                            }
                            state.snackbar = None;
                        }
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("✖").color(egui::Color32::GRAY),
                                )
                                .frame(false),
                            )
                            .clicked()
                        {
                            state.snackbar = None;
                        }
                    });
                });
        });
}
