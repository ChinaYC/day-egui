use uuid::Uuid;

use crate::todo::state::{
    FilterAutomated, FilterReminder, FilterStatus, SortMode, TaskSmartView, TodoViewMode,
};

use super::super::TodoState;

pub struct ListEvents {
    pub delete_confirmed: Option<Uuid>,
    pub restore_confirmed: Option<Uuid>,
    pub purge_confirmed: Option<Uuid>,
    pub clear_trash_confirmed: bool,
    pub reorder_request: Option<(Uuid, Uuid, usize)>,
    pub open_reminder_for: Option<Uuid>,
    pub open_editor_for: Option<Uuid>,
}

pub fn show(state: &mut TodoState, ui: &mut egui::Ui, state_changed: &mut bool) -> ListEvents {
    let mut delete_confirmed: Option<Uuid> = None;
    let mut restore_confirmed: Option<Uuid> = None;
    let mut purge_confirmed: Option<Uuid> = None;
    let mut clear_trash_confirmed = false;
    let mut reorder_request: Option<(Uuid, Uuid, usize)> = None;
    let mut open_reminder_for: Option<Uuid> = None;
    let mut open_editor_for: Option<Uuid> = None;

    if state.dragging_item.is_none() {
        state.drag_target_index = None;
        state.dragging_section = None;
    }
    if state.sort_mode != SortMode::Manual {
        state.dragging_item = None;
        state.dragging_section = None;
        state.drag_target_index = None;
    }

    if state.view_mode == TodoViewMode::Trash {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("回收站 (Trash)").strong());
            if ui.button("清空回收站").clicked() {
                clear_trash_confirmed = true;
            }
        });
        ui.add_space(6.0);
    }

    let query = state.search_query.trim().to_lowercase();
    let mut undo_replacements: Vec<(Uuid, crate::todo::model::TodoItem)> = Vec::new();

    egui::ScrollArea::vertical()
        .id_salt("todo_list_scroll")
        .show(ui, |ui| {
            let sections_to_render: Vec<Uuid> = if let Some(active) = state.active_section {
                vec![active]
            } else if let Some(folder_id) = state.active_folder {
                state
                    .sections
                    .iter()
                    .filter(|s| s.folder_id == Some(folder_id))
                    .map(|s| s.id)
                    .collect()
            } else {
                state.sections.iter().map(|s| s.id).collect()
            };

            for section_id in sections_to_render {
                let section_name = state
                    .section_name(section_id)
                    .unwrap_or("未知分区 (Unknown)")
                    .to_string();

                ui.add_space(4.0);
                ui.label(egui::RichText::new(section_name).strong());
                ui.add_space(6.0);

                let indices: Vec<usize> = state
                    .items
                    .iter()
                    .enumerate()
                    .filter_map(|(i, item)| {
                        if item.section_id != Some(section_id) {
                            return None;
                        }
                        if !item_matches_filters(state, item, &query) {
                            return None;
                        }
                        Some(i)
                    })
                    .collect();
                let mut indices = indices;
                if state.sort_mode == SortMode::Priority && state.view_mode == TodoViewMode::Tasks {
                    indices.sort_by_key(|&i| {
                        let item = &state.items[i];
                        let completed = if item.completed { 1u8 } else { 0u8 };
                        let p = item.priority.min(3);
                        let due_ts = item.due_at.map(|d| d.timestamp()).unwrap_or(i64::MAX);
                        (completed, p, due_ts, i)
                    });
                }

                for (visible_index, item_index) in indices.iter().copied().enumerate() {
                    let item = &mut state.items[item_index];
                    let item_id = item.id;
                    let mut drag_handle_response: Option<egui::Response> = None;
                    let can_interact = state.view_mode == TodoViewMode::Tasks && item.deleted_at.is_none();
                    let can_drag =
                        can_interact && state.sort_mode == SortMode::Manual && !state.selection_mode;

                    ui.vertical(|ui| {
                        let row_response = ui
                            .horizontal(|ui| {
                                let mut completed = item.completed;
                                let completed_before = item.completed;
                                let reminder_sent_before = item.reminder_sent;

                                if state.selection_mode && can_interact {
                                    let mut selected = state.selected_items.contains(&item_id);
                                    if ui.checkbox(&mut selected, "").changed() {
                                        if selected {
                                            state.selected_items.insert(item_id);
                                        } else {
                                            state.selected_items.remove(&item_id);
                                        }
                                    }
                                }

                                drag_handle_response = Some(
                                    ui.add_enabled(
                                        can_drag,
                                        egui::Label::new(
                                            egui::RichText::new("≡")
                                                .size(14.0)
                                                .color(egui::Color32::GRAY),
                                        )
                                        .sense(egui::Sense::click_and_drag()),
                                    ),
                                );

                                if can_drag {
                                    if let Some(drag_handle_response) = &drag_handle_response {
                                        if drag_handle_response.drag_started() {
                                            state.dragging_item = Some(item_id);
                                            state.dragging_section = Some(section_id);
                                            state.drag_target_index = Some(visible_index);
                                            state.editing_reminder = None;
                                            state.editing_task = None;
                                        }
                                    }
                                }

                                if ui
                                    .add_enabled(
                                        can_interact,
                                        egui::Checkbox::new(&mut completed, ""),
                                    )
                                    .changed()
                                {
                                    let before = item.clone();
                                    item.completed = completed;
                                    if item.completed {
                                        item.reminder_sent = true;
                                    } else if item.reminder_at.is_some() {
                                        item.reminder_sent = false;
                                    }
                                    undo_replacements.push((item_id, before));
                                    *state_changed = true;
                                } else {
                                    item.completed = completed_before;
                                    item.reminder_sent = reminder_sent_before;
                                }

                                let title_text = if item.completed {
                                    egui::RichText::new(&item.title)
                                        .strikethrough()
                                        .color(egui::Color32::DARK_GRAY)
                                } else {
                                    egui::RichText::new(&item.title)
                                };
                                let title_response = ui.add(
                                    egui::Label::new(title_text).sense(egui::Sense::click()),
                                );
                                if can_interact && title_response.clicked() {
                                    if state.selection_mode {
                                        if state.selected_items.contains(&item_id) {
                                            state.selected_items.remove(&item_id);
                                        } else {
                                            state.selected_items.insert(item_id);
                                        }
                                    } else {
                                        open_editor_for = Some(item_id);
                                    }
                                }

                                let p = item.priority.min(3);
                                let (p_text, p_color) = match p {
                                    0 => ("P0", egui::Color32::LIGHT_RED),
                                    1 => ("P1", egui::Color32::from_rgb(255, 180, 80)),
                                    2 => ("P2", egui::Color32::LIGHT_BLUE),
                                    _ => ("P3", egui::Color32::GRAY),
                                };
                                ui.label(
                                    egui::RichText::new(p_text)
                                        .size(10.0)
                                        .color(p_color),
                                );

                                if item.is_automated {
                                    ui.label(
                                        egui::RichText::new("🤖 自动 (Auto)")
                                            .size(10.0)
                                            .color(egui::Color32::LIGHT_GREEN),
                                    );
                                }

                                if !item.tags.is_empty() {
                                    for tag in item.tags.iter().take(3) {
                                        ui.label(
                                            egui::RichText::new(format!("#{tag}"))
                                                .size(10.0)
                                                .color(egui::Color32::from_rgb(120, 170, 255)),
                                        );
                                    }
                                    if item.tags.len() > 3 {
                                        ui.label(
                                            egui::RichText::new("…")
                                                .size(10.0)
                                                .color(egui::Color32::GRAY),
                                        );
                                    }
                                }

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if state.view_mode == TodoViewMode::Trash {
                                            if ui.button("恢复").clicked() {
                                                restore_confirmed = Some(item_id);
                                            }

                                            if ui.button("彻底删除").clicked() {
                                                purge_confirmed = Some(item_id);
                                            }
                                        } else {
                                            if ui.button("🗑️").clicked() {
                                                delete_confirmed = Some(item_id);
                                            }

                                            if ui.button("⏰").clicked() {
                                                open_reminder_for = Some(item_id);
                                            }

                                            let mut moved_section = item.section_id;
                                            ui.menu_button("📁", |ui| {
                                                for folder in &state.folders {
                                                    ui.collapsing(folder.name.clone(), |ui| {
                                                        for section in state
                                                            .sections
                                                            .iter()
                                                            .filter(|s| s.folder_id == Some(folder.id))
                                                        {
                                                            if ui.button(section.name.clone()).clicked() {
                                                                moved_section = Some(section.id);
                                                                ui.close();
                                                            }
                                                        }
                                                    });
                                                }
                                                ui.separator();
                                                for section in state.sections.iter().filter(|s| s.folder_id.is_none()) {
                                                    if ui.button(section.name.clone()).clicked() {
                                                        moved_section = Some(section.id);
                                                        ui.close();
                                                    }
                                                }
                                            });
                                            if moved_section != item.section_id {
                                                let before = item.clone();
                                                item.section_id = moved_section;
                                                undo_replacements.push((item_id, before));
                                                *state_changed = true;
                                            }
                                        }

                                        if let Some(reminder_text) = item.reminder_at_local_string()
                                        {
                                            ui.label(
                                                egui::RichText::new(format!("⏰ {reminder_text}"))
                                                    .size(10.0)
                                                    .color(egui::Color32::GRAY),
                                            );
                                        }

                                        if let Some(due_text) = item.due_at_local_string() {
                                            let overdue = is_overdue(item);
                                            let color = if overdue {
                                                egui::Color32::LIGHT_RED
                                            } else {
                                                egui::Color32::GRAY
                                            };
                                            ui.label(
                                                egui::RichText::new(format!("📅 {due_text}"))
                                                    .size(10.0)
                                                    .color(color),
                                            );
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

                        // contains_pointer() 在拖拽中比 hovered() 更可靠（hovered 可能被拖拽控件“抢占”）。
                        if can_drag
                            && state.dragging_item.is_some()
                            && state.dragging_section == Some(section_id)
                            && row_response.contains_pointer()
                        {
                            if let Some(pointer_pos) = ui.ctx().pointer_hover_pos() {
                                // 把“鼠标在当前行上半/下半”的信息翻译成“插入点索引”。
                                let insert_index = if pointer_pos.y > row_response.rect.center().y {
                                    visible_index.saturating_add(1)
                                } else {
                                    visible_index
                                };
                                state.drag_target_index = Some(insert_index);
                            } else {
                                state.drag_target_index = Some(visible_index);
                            }
                        }

                        if let Some(drag_handle_response) = &drag_handle_response {
                            // drag_stopped() 只会 true 一帧：在这里提交重排请求，不直接改 Vec。
                            if can_drag
                                && state.dragging_item == Some(item_id)
                                && state.dragging_section == Some(section_id)
                                && drag_handle_response.drag_stopped()
                            {
                                if let Some(target_index) = state.drag_target_index {
                                    reorder_request = Some((item_id, section_id, target_index));
                                }
                                state.dragging_item = None;
                                state.dragging_section = None;
                                state.drag_target_index = None;
                            }
                        }

                        if let Some(desc) = &item.description {
                            if !desc.is_empty() {
                                ui.horizontal(|ui| {
                                    ui.add_space(24.0);
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

    for (id, before) in undo_replacements {
        state.push_undo_replace_item(id, before);
    }

    ListEvents {
        delete_confirmed,
        restore_confirmed,
        purge_confirmed,
        clear_trash_confirmed,
        reorder_request,
        open_reminder_for,
        open_editor_for,
    }
}

fn item_matches_filters(state: &TodoState, item: &crate::todo::model::TodoItem, query: &str) -> bool {
    match state.view_mode {
        TodoViewMode::Tasks => {
            if item.deleted_at.is_some() {
                return false;
            }
        }
        TodoViewMode::Planner => {
            return false;
        }
        TodoViewMode::Trash => {
            if item.deleted_at.is_none() {
                return false;
            }
        }
    }

    if state.view_mode == TodoViewMode::Tasks {
        match state.smart_view {
            TaskSmartView::All => {}
            TaskSmartView::Inbox => {
                if item.due_at.is_some() {
                    return false;
                }
            }
            TaskSmartView::Today => {
                let Some(due) = item.due_at else {
                    return false;
                };
                let today = chrono::Local::now().date_naive();
                let due_date = due.with_timezone(&chrono::Local).date_naive();
                if due_date != today {
                    return false;
                }
            }
            TaskSmartView::Next7Days => {
                let Some(due) = item.due_at else {
                    return false;
                };
                let today = chrono::Local::now().date_naive();
                let end = today + chrono::Duration::days(6);
                let due_date = due.with_timezone(&chrono::Local).date_naive();
                if due_date < today || due_date > end {
                    return false;
                }
            }
        }
    }

    match state.filter_status {
        FilterStatus::All => {}
        FilterStatus::Active => {
            if item.completed {
                return false;
            }
        }
        FilterStatus::Completed => {
            if !item.completed {
                return false;
            }
        }
    }

    match state.filter_automated {
        FilterAutomated::All => {}
        FilterAutomated::AutomatedOnly => {
            if !item.is_automated {
                return false;
            }
        }
        FilterAutomated::ManualOnly => {
            if item.is_automated {
                return false;
            }
        }
    }

    match state.filter_reminder {
        FilterReminder::All => {}
        FilterReminder::WithReminder => {
            if item.reminder_at.is_none() {
                return false;
            }
        }
        FilterReminder::WithoutReminder => {
            if item.reminder_at.is_some() {
                return false;
            }
        }
    }

    if let Some(tag) = &state.active_tag {
        let tag_l = tag.to_lowercase();
        if !item.tags.iter().any(|t| t.to_lowercase() == tag_l) {
            return false;
        }
    }

    if !query.is_empty() {
        let title = item.title.to_lowercase();
        if title.contains(query) {
            return true;
        }
        if let Some(desc) = &item.description {
            if desc.to_lowercase().contains(query) {
                return true;
            }
        }
        return false;
    }

    true
}

fn is_overdue(item: &crate::todo::model::TodoItem) -> bool {
    if item.completed || item.deleted_at.is_some() {
        return false;
    }
    let Some(due) = item.due_at else {
        return false;
    };
    due < chrono::Utc::now()
}
