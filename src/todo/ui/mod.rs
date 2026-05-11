mod editor;
mod header;
mod list;
mod planner;
mod reminder;
mod sections;
mod settings;
mod snackbar;

use super::TodoState;

impl TodoState {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        self.ensure_builtin_sections_and_settings();
        self.migrate_items_without_section();

        let mut state_changed = false;

        if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Z)) {
            if self.undo_last_action() {
                state_changed = true;
            }
        }

        sections::show(self, ui);

        match self.view_mode {
            crate::todo::state::TodoViewMode::Planner => {
                planner::show(self, ui, &mut state_changed);
            }
            crate::todo::state::TodoViewMode::Tasks | crate::todo::state::TodoViewMode::Trash => {
                header::show(self, ui, &mut state_changed);
                let events = list::show(self, ui, &mut state_changed);

                if let Some(item_id) = events.open_editor_for {
                    if self.editing_task != Some(item_id) {
                        self.editing_task = Some(item_id);
                        self.edit_error_msg = None;
                        if let Some(item) = self.items.iter().find(|i| i.id == item_id) {
                            self.edit_title_input = item.title.clone();
                            self.edit_desc_input = item.description.clone().unwrap_or_default();
                            self.edit_tags_input = item.tags.join(", ");
                            self.edit_section_input = item.section_id;
                            self.edit_due_input = item.due_at_local_string().unwrap_or_default();
                            self.edit_reminder_input =
                                item.reminder_display_string().unwrap_or_default();
                            self.edit_priority_input = item.priority;
                        }
                    }
                }

                if let Some(item_id) = events.open_reminder_for {
                    if self.editing_reminder != Some(item_id) {
                        self.editing_reminder = Some(item_id);
                        self.reminder_error_msg = None;
                        self.reminder_input = self
                            .items
                            .iter()
                            .find(|i| i.id == item_id)
                            .and_then(|i| i.reminder_display_string())
                            .unwrap_or_default();
                    }
                }

                if let Some(item_id) = events.delete_confirmed {
                    if let Some(item) = self.items.iter_mut().find(|i| i.id == item_id) {
                        let before = item.clone();
                        item.deleted_at = Some(chrono::Utc::now());
                        item.reminder_sent = true;
                        item.touch();
                        self.push_undo_replace_item(item_id, before);
                        self.show_snackbar("已删除");
                        self.selected_items.remove(&item_id);
                        state_changed = true;
                    }
                }

                if let Some(item_id) = events.restore_confirmed {
                    if let Some(item) = self.items.iter_mut().find(|i| i.id == item_id) {
                        let before = item.clone();
                        item.deleted_at = None;
                        if item.reminder_at.is_some() && !item.completed {
                            if let Some(rule) = item.reminder_repeat {
                                if let Some(next) =
                                    crate::todo::reminders::next_reminder_from_repeat(
                                        rule,
                                        chrono::Utc::now(),
                                    )
                                {
                                    item.reminder_at = Some(next);
                                    item.reminder_sent = false;
                                }
                            } else {
                                item.reminder_sent = false;
                            }
                        }
                        item.touch();
                        self.push_undo_replace_item(item_id, before);
                        self.selected_items.remove(&item_id);
                        state_changed = true;
                    }
                }

                if let Some(item_id) = events.purge_confirmed {
                    if let Some(pos) = self.items.iter().position(|i| i.id == item_id) {
                        let item = self.items.remove(pos);
                        self.undo_stack
                            .push(crate::todo::state::UndoAction::ReinsertItem {
                                index: pos,
                                item,
                            });
                        self.show_snackbar("已彻底删除");
                        self.selected_items.remove(&item_id);
                        state_changed = true;
                    }
                }

                if events.clear_trash_confirmed {
                    let mut removed: Vec<(usize, crate::todo::model::TodoItem)> = Vec::new();
                    let mut i = 0;
                    while i < self.items.len() {
                        if self.items[i].deleted_at.is_some() {
                            let id = self.items[i].id;
                            let item = self.items.remove(i);
                            removed.push((i, item));
                            self.selected_items.remove(&id);
                        } else {
                            i += 1;
                        }
                    }
                    if !removed.is_empty() {
                        self.undo_stack
                            .push(crate::todo::state::UndoAction::ReinsertMany { items: removed });
                        self.show_snackbar("回收站已清空");
                        state_changed = true;
                    }
                }

                if let Some((dragged_id, section_id, target_index)) = events.reorder_request {
                    if reorder_item_in_section(self, dragged_id, section_id, target_index) {
                        state_changed = true;
                    }
                }

                editor::show(self, ui, &mut state_changed);
                reminder::show(self, ui, &mut state_changed);
            }
        }

        settings::show(self, ui, &mut state_changed);
        snackbar::show(self, ui, &mut state_changed);

        if state_changed {
            self.save_to_file();
        }
    }
}

fn reorder_item_in_section(
    state: &mut TodoState,
    dragged_id: uuid::Uuid,
    section_id: uuid::Uuid,
    target_index: usize,
) -> bool {
    let from_index = match state.items.iter().position(|i| i.id == dragged_id) {
        Some(i) => i,
        None => return false,
    };

    if state.items.get(from_index).and_then(|i| i.section_id) != Some(section_id) {
        return false;
    }

    let indices_before: Vec<usize> = state
        .items
        .iter()
        .enumerate()
        .filter_map(|(i, it)| (it.section_id == Some(section_id)).then_some(i))
        .collect();
    let from_pos = match indices_before.iter().position(|&i| i == from_index) {
        Some(p) => p,
        None => return false,
    };

    let mut target_pos = target_index.min(indices_before.len());
    if target_pos > from_pos {
        target_pos = target_pos.saturating_sub(1);
    }

    let item = state.items.remove(from_index);

    let indices_after: Vec<usize> = state
        .items
        .iter()
        .enumerate()
        .filter_map(|(i, it)| (it.section_id == Some(section_id)).then_some(i))
        .collect();

    let insert_global_index = if indices_after.is_empty() {
        state.items.len()
    } else if target_pos >= indices_after.len() {
        indices_after.last().copied().unwrap_or(state.items.len()) + 1
    } else {
        indices_after[target_pos]
    };

    let insert_global_index = insert_global_index.min(state.items.len());
    state.items.insert(insert_global_index, item);
    true
}
