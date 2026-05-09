use super::super::TodoState;

pub fn show(state: &mut TodoState, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if ui
            .selectable_label(
                state.view_mode == crate::todo::state::TodoViewMode::Tasks,
                "任务",
            )
            .clicked()
        {
            state.view_mode = crate::todo::state::TodoViewMode::Tasks;
            state.selection_mode = false;
            state.selected_items.clear();
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
            state.selection_mode = false;
            state.selected_items.clear();
            reset_per_section_interactions(state);
        }
        if ui
            .selectable_label(
                state.view_mode == crate::todo::state::TodoViewMode::Trash,
                "回收站",
            )
            .clicked()
        {
            state.view_mode = crate::todo::state::TodoViewMode::Trash;
            state.selection_mode = false;
            state.selected_items.clear();
            reset_per_section_interactions(state);
        }
    });
    ui.add_space(6.0);

    if state.view_mode == crate::todo::state::TodoViewMode::Tasks {
        ui.horizontal(|ui| {
            use crate::todo::state::TaskSmartView;

            ui.label("快捷视图：");
            if ui
                .selectable_label(state.smart_view == TaskSmartView::All, "全部")
                .clicked()
            {
                state.smart_view = TaskSmartView::All;
                state.selection_mode = false;
                state.selected_items.clear();
                reset_per_section_interactions(state);
            }
            if ui
                .selectable_label(state.smart_view == TaskSmartView::Inbox, "收件箱")
                .clicked()
            {
                state.smart_view = TaskSmartView::Inbox;
                state.selection_mode = false;
                state.selected_items.clear();
                reset_per_section_interactions(state);
            }
            if ui
                .selectable_label(state.smart_view == TaskSmartView::Today, "今天")
                .clicked()
            {
                state.smart_view = TaskSmartView::Today;
                state.selection_mode = false;
                state.selected_items.clear();
                reset_per_section_interactions(state);
            }
            if ui
                .selectable_label(state.smart_view == TaskSmartView::Next7Days, "未来7天")
                .clicked()
            {
                state.smart_view = TaskSmartView::Next7Days;
                state.selection_mode = false;
                state.selected_items.clear();
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
        ui.label("清单范围：");
        let current = if let Some(section_id) = state.active_section {
            state
                .section_name(section_id)
                .map(|s| format!("清单：{s}"))
                .unwrap_or_else(|| "清单：未知".to_string())
        } else if let Some(folder_id) = state.active_folder {
            state
                .folder_name(folder_id)
                .map(|s| format!("文件夹：{s}"))
                .unwrap_or_else(|| "文件夹：未知".to_string())
        } else {
            "清单：全部".to_string()
        };

        let folders: Vec<(uuid::Uuid, String)> = state
            .folders
            .iter()
            .map(|f| (f.id, f.name.clone()))
            .collect();
        let sections: Vec<(uuid::Uuid, String, Option<uuid::Uuid>)> = state
            .sections
            .iter()
            .map(|s| (s.id, s.name.clone(), s.folder_id))
            .collect();

        ui.menu_button(current, |ui| {
            if ui.button("全部").clicked() {
                state.active_section = None;
                state.active_folder = None;
                state.selection_mode = false;
                state.selected_items.clear();
                reset_per_section_interactions(state);
                ui.close();
            }

            ui.separator();

            for (folder_id, folder_name) in &folders {
                ui.collapsing(folder_name.clone(), |ui| {
                    if ui.button("查看该文件夹全部任务").clicked() {
                        state.active_section = None;
                        state.active_folder = Some(*folder_id);
                        state.selection_mode = false;
                        state.selected_items.clear();
                        reset_per_section_interactions(state);
                        ui.close();
                    }
                    for (section_id, section_name, _folder) in sections
                        .iter()
                        .filter(|(_, _, folder)| *folder == Some(*folder_id))
                    {
                        if ui.button(section_name.clone()).clicked() {
                            state.active_section = Some(*section_id);
                            state.active_folder = None;
                            state.selection_mode = false;
                            state.selected_items.clear();
                            reset_per_section_interactions(state);
                            ui.close();
                        }
                    }
                });
            }

            ui.separator();

            ui.collapsing("未归类", |ui| {
                for (section_id, section_name, folder) in &sections {
                    if folder.is_some() {
                        continue;
                    }
                    if ui.button(section_name.clone()).clicked() {
                        state.active_section = Some(*section_id);
                        state.active_folder = None;
                        state.selection_mode = false;
                        state.selected_items.clear();
                        reset_per_section_interactions(state);
                        ui.close();
                    }
                }
            });
        });
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
