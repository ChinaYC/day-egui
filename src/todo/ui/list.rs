use uuid::Uuid;

use crate::todo::state::{FilterAutomated, FilterReminder, FilterStatus, TodoViewMode};

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

    if state.view_mode == TodoViewMode::Trash {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("回收站 (Trash)").strong());
            if ui.button("清空回收站").clicked() {
                state.confirm_clear_trash = true;
            }
        });
        ui.add_space(6.0);
    }

    if state.confirm_clear_trash {
        let mut open = true;
        egui::Window::new("清空回收站 (Clear trash)")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                ui.label("确认彻底删除回收站内所有任务？此操作不可撤销。");
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("取消").clicked() {
                        state.confirm_clear_trash = false;
                    }
                    if ui.button("确认清空").clicked() {
                        clear_trash_confirmed = true;
                        state.confirm_clear_trash = false;
                    }
                });
            });
        if !open {
            state.confirm_clear_trash = false;
        }
    }

    let query = state.search_query.trim().to_lowercase();
    let mut undo_replacements: Vec<(Uuid, crate::todo::model::TodoItem)> = Vec::new();

    egui::ScrollArea::vertical()
        .id_salt("todo_list_scroll")
        .show(ui, |ui| {
            let sections_to_render: Vec<Uuid> = if let Some(active) = state.active_section {
                vec![active]
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

                for (visible_index, item_index) in indices.iter().copied().enumerate() {
                    let item = &mut state.items[item_index];
                    let item_id = item.id;
                    let mut drag_handle_response: Option<egui::Response> = None;
                    let can_interact = state.view_mode == TodoViewMode::Tasks && item.deleted_at.is_none();

                    ui.vertical(|ui| {
                        let row_response = ui
                            .horizontal(|ui| {
                                let mut completed = item.completed;
                                let completed_before = item.completed;
                                let reminder_sent_before = item.reminder_sent;

                                drag_handle_response = Some(
                                    ui.add_enabled(
                                        can_interact,
                                        egui::Label::new(
                                            egui::RichText::new("≡")
                                                .size(14.0)
                                                .color(egui::Color32::GRAY),
                                        )
                                        .sense(egui::Sense::click_and_drag()),
                                    ),
                                );

                                if can_interact {
                                    if let Some(drag_handle_response) = &drag_handle_response {
                                        if drag_handle_response.drag_started() {
                                            state.dragging_item = Some(item_id);
                                            state.dragging_section = Some(section_id);
                                            state.drag_target_index = Some(visible_index);
                                            state.item_to_delete = None;
                                            state.delete_is_permanent = false;
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
                                    open_editor_for = Some(item_id);
                                }

                                if item.is_automated {
                                    ui.label(
                                        egui::RichText::new("🤖 自动 (Auto)")
                                            .size(10.0)
                                            .color(egui::Color32::LIGHT_GREEN),
                                    );
                                }

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if state.view_mode == TodoViewMode::Trash {
                                            if ui.button("恢复").clicked() {
                                                restore_confirmed = Some(item_id);
                                            }

                                            if state.item_to_delete == Some(item_id)
                                                && state.delete_is_permanent
                                            {
                                                if ui.button("取消").clicked() {
                                                    state.item_to_delete = None;
                                                }
                                                if ui.button("确认删除").clicked() {
                                                    purge_confirmed = Some(item_id);
                                                    state.item_to_delete = None;
                                                    state.delete_is_permanent = false;
                                                }
                                            } else if ui.button("彻底删除").clicked() {
                                                state.item_to_delete = Some(item_id);
                                                state.delete_is_permanent = true;
                                            }
                                        } else {
                                            if state.item_to_delete == Some(item_id)
                                                && !state.delete_is_permanent
                                            {
                                                if ui.button("取消 (Cancel)").clicked() {
                                                    state.item_to_delete = None;
                                                }
                                                if ui.button("删除 (Delete)").clicked() {
                                                    delete_confirmed = Some(item_id);
                                                    state.item_to_delete = None;
                                                }
                                            } else if ui.button("🗑️").clicked() {
                                                state.item_to_delete = Some(item_id);
                                                state.delete_is_permanent = false;
                                            }

                                            if ui.button("⏰").clicked() {
                                                open_reminder_for = Some(item_id);
                                            }

                                            let mut moved_section = item.section_id;
                                            ui.menu_button("📁", |ui| {
                                                for section in &state.sections {
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
                                                egui::RichText::new(reminder_text)
                                                    .size(10.0)
                                                    .color(egui::Color32::GRAY),
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
                        if can_interact
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
                            if can_interact
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
        TodoViewMode::Trash => {
            if item.deleted_at.is_none() {
                return false;
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
