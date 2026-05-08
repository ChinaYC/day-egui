use uuid::Uuid;

use super::super::TodoState;

pub struct ListEvents {
    pub delete_confirmed: Option<Uuid>,
    pub reorder_request: Option<(Uuid, Uuid, usize)>,
    pub open_reminder_for: Option<Uuid>,
}

pub fn show(state: &mut TodoState, ui: &mut egui::Ui, state_changed: &mut bool) -> ListEvents {
    let mut delete_confirmed: Option<Uuid> = None;
    let mut reorder_request: Option<(Uuid, Uuid, usize)> = None;
    let mut open_reminder_for: Option<Uuid> = None;

    if state.dragging_item.is_none() {
        state.drag_target_index = None;
        state.dragging_section = None;
    }

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
                    .filter_map(|(i, item)| (item.section_id == Some(section_id)).then_some(i))
                    .collect();

                for (visible_index, item_index) in indices.iter().copied().enumerate() {
                    let item = &mut state.items[item_index];
                    let item_id = item.id;
                    let mut drag_handle_response: Option<egui::Response> = None;

                    ui.vertical(|ui| {
                        let row_response = ui
                            .horizontal(|ui| {
                                drag_handle_response = Some(
                                    ui.add(
                                        egui::Label::new(
                                            egui::RichText::new("≡")
                                                .size(14.0)
                                                .color(egui::Color32::GRAY),
                                        )
                                        .sense(egui::Sense::click_and_drag()),
                                    ),
                                );

                                if let Some(drag_handle_response) = &drag_handle_response {
                                    // drag_started() 只会 true 一帧：在这里记录“正在拖拽哪个任务”。
                                    if drag_handle_response.drag_started() {
                                        state.dragging_item = Some(item_id);
                                        state.dragging_section = Some(section_id);
                                        state.drag_target_index = Some(visible_index);
                                        state.item_to_delete = None;
                                        state.editing_reminder = None;
                                    }
                                }

                                if ui.checkbox(&mut item.completed, "").changed() {
                                    if item.completed {
                                        // UX：任务完成后默认不再提醒，避免“做完了还弹通知”。
                                        item.reminder_sent = true;
                                    } else if item.reminder_at.is_some() {
                                        // UX：如果把任务从完成改回未完成，之前设置的提醒重新生效。
                                        item.reminder_sent = false;
                                    }
                                    *state_changed = true;
                                }

                                let title_text = if item.completed {
                                    egui::RichText::new(&item.title)
                                        .strikethrough()
                                        .color(egui::Color32::DARK_GRAY)
                                } else {
                                    egui::RichText::new(&item.title)
                                };
                                ui.label(title_text);

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
                                        if state.item_to_delete == Some(item_id) {
                                            if ui.button("取消 (Cancel)").clicked() {
                                                state.item_to_delete = None;
                                            }
                                            if ui.button("删除 (Delete)").clicked() {
                                                delete_confirmed = Some(item_id);
                                                state.item_to_delete = None;
                                            }
                                        } else if ui.button("🗑️").clicked() {
                                            state.item_to_delete = Some(item_id);
                                        }

                                        if ui.button("⏰").clicked() {
                                            open_reminder_for = Some(item_id);
                                        }
                                        ui.menu_button("📁", |ui| {
                                            for section in &state.sections {
                                                if ui.button(section.name.clone()).clicked() {
                                                    item.section_id = Some(section.id);
                                                    *state_changed = true;
                                                    ui.close();
                                                }
                                            }
                                        });
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
                        if state.dragging_item.is_some()
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
                            if state.dragging_item == Some(item_id)
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

    ListEvents {
        delete_confirmed,
        reorder_request,
        open_reminder_for,
    }
}
