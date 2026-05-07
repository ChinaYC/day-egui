use super::super::TodoState;

pub fn show(state: &mut TodoState, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if ui
            .selectable_label(state.active_section.is_none(), "全部 (All)")
            .clicked()
        {
            state.active_section = None;
            reset_per_section_interactions(state);
        }

        let sections: Vec<(uuid::Uuid, String)> = state
            .sections
            .iter()
            .map(|s| (s.id, s.name.clone()))
            .collect();
        for (section_id, section_name) in sections {
            if ui
                .selectable_label(state.active_section == Some(section_id), section_name)
                .clicked()
            {
                state.active_section = Some(section_id);
                reset_per_section_interactions(state);
            }
        }
    });

    ui.add_space(8.0);
}

fn reset_per_section_interactions(state: &mut TodoState) {
    state.dragging_item = None;
    state.dragging_section = None;
    state.drag_target_index = None;
    state.item_to_delete = None;
    state.editing_reminder = None;
}
