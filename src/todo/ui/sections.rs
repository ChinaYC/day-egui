use super::super::TodoState;

pub fn show(state: &mut TodoState, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if ui
            .selectable_label(state.view_mode == crate::todo::state::TodoViewMode::Tasks, "任务")
            .clicked()
        {
            state.view_mode = crate::todo::state::TodoViewMode::Tasks;
            reset_per_section_interactions(state);
        }
        if ui
            .selectable_label(
                state.view_mode == crate::todo::state::TodoViewMode::Planner,
                "周计划",
            )
            .clicked()
        {
            state.view_mode = crate::todo::state::TodoViewMode::Planner;
            reset_per_section_interactions(state);
        }
        if ui
            .selectable_label(state.view_mode == crate::todo::state::TodoViewMode::Trash, "回收站")
            .clicked()
        {
            state.view_mode = crate::todo::state::TodoViewMode::Trash;
            reset_per_section_interactions(state);
        }
    });
    ui.add_space(6.0);

    if state.view_mode == crate::todo::state::TodoViewMode::Tasks {
        ui.horizontal(|ui| {
            use crate::todo::state::TaskSmartView;

            if ui
                .selectable_label(state.smart_view == TaskSmartView::All, "全部")
                .clicked()
            {
                state.smart_view = TaskSmartView::All;
                reset_per_section_interactions(state);
            }
            if ui
                .selectable_label(state.smart_view == TaskSmartView::Inbox, "收件箱")
                .clicked()
            {
                state.smart_view = TaskSmartView::Inbox;
                reset_per_section_interactions(state);
            }
            if ui
                .selectable_label(state.smart_view == TaskSmartView::Today, "今天")
                .clicked()
            {
                state.smart_view = TaskSmartView::Today;
                reset_per_section_interactions(state);
            }
            if ui
                .selectable_label(state.smart_view == TaskSmartView::Next7Days, "未来7天")
                .clicked()
            {
                state.smart_view = TaskSmartView::Next7Days;
                reset_per_section_interactions(state);
            }
        });
        ui.add_space(6.0);
    }

    if state.view_mode != crate::todo::state::TodoViewMode::Tasks {
        ui.add_space(8.0);
        return;
    }

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
    state.editing_reminder = None;
    state.reminder_error_msg = None;
    state.editing_task = None;
    state.edit_error_msg = None;
}
