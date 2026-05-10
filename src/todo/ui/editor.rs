use super::super::TodoState;

pub fn show(state: &mut TodoState, ui: &mut egui::Ui, state_changed: &mut bool) {
    let Some(item_id) = state.editing_task else {
        return;
    };

    let item = state.items.iter().find(|i| i.id == item_id);
    let item_title = item.map(|i| i.title.clone()).unwrap_or_default();
    let item_deleted = item.map(|i| i.deleted_at.is_some()).unwrap_or(false);
    let item_completed = item.map(|i| i.completed).unwrap_or(false);

    let mut open = true;
    egui::Window::new("编辑任务 (Edit task)")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ui.ctx(), |ui| {
            ui.label(format!("任务: {}", item_title));
            if item_deleted {
                ui.label(
                    egui::RichText::new("回收站任务不可编辑，请先恢复")
                        .color(egui::Color32::YELLOW),
                );
            }

            ui.add_space(8.0);
            ui.label("标题 (Title):");
            ui.text_edit_singleline(&mut state.edit_title_input);

            ui.add_space(6.0);
            ui.label("备注 (Description):");
            ui.text_edit_multiline(&mut state.edit_desc_input);

            ui.add_space(6.0);
            ui.label("标签 (Tags, 用逗号分隔或 #tag):");
            ui.text_edit_singleline(&mut state.edit_tags_input);

            ui.add_space(6.0);
            ui.label("分区 (Section):");
            egui::ComboBox::from_id_salt("edit_task_section")
                .selected_text(
                    state
                        .edit_section_input
                        .and_then(|id| state.section_name(id).map(|s| s.to_string()))
                        .unwrap_or_else(|| "未选择".to_string()),
                )
                .show_ui(ui, |ui| {
                    for section in &state.sections {
                        let label = section
                            .folder_id
                            .and_then(|fid| state.folder_name(fid).map(|f| f.to_string()))
                            .map(|f| format!("{f} / {}", section.name))
                            .unwrap_or_else(|| section.name.clone());
                        ui.selectable_value(&mut state.edit_section_input, Some(section.id), label);
                    }
                });

            ui.add_space(6.0);
            ui.label("优先级 (Priority):");
            let mut p = state.edit_priority_input.min(3);
            egui::ComboBox::from_id_salt("edit_task_priority")
                .selected_text(match p {
                    0 => "P0 (最高)",
                    1 => "P1",
                    2 => "P2",
                    _ => "P3 (最低)",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut p, 0, "P0 (最高)");
                    ui.selectable_value(&mut p, 1, "P1");
                    ui.selectable_value(&mut p, 2, "P2");
                    ui.selectable_value(&mut p, 3, "P3 (最低)");
                });
            state.edit_priority_input = p;

            ui.add_space(6.0);
            ui.label("到期日 (Due, YYYY-MM-DD/今天/明天/周二):");
            ui.text_edit_singleline(&mut state.edit_due_input);

            ui.add_space(6.0);
            ui.label("提醒 (Reminder, 支持周几/相对时间):");
            ui.text_edit_singleline(&mut state.edit_reminder_input);

            if let Some(err) = &state.edit_error_msg {
                ui.add_space(6.0);
                ui.label(egui::RichText::new(err).color(egui::Color32::RED));
            }

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("关闭 (Close)").clicked() {
                    state.editing_task = None;
                    state.edit_error_msg = None;
                }

                if ui.button("保存 (Save)").clicked() {
                    if item_deleted {
                        state.edit_error_msg = Some("请先恢复任务再编辑".to_string());
                        return;
                    }

                    let title = state.edit_title_input.trim().to_string();
                    if title.is_empty() {
                        state.edit_error_msg = Some("标题不能为空".to_string());
                        return;
                    }

                    let desc = state.edit_desc_input.trim().to_string();
                    let description = if desc.is_empty() { None } else { Some(desc) };

                    let tags = parse_tags_input(&state.edit_tags_input);

                    let due_at = if state.edit_due_input.trim().is_empty() {
                        None
                    } else {
                        let Some(parsed) =
                            crate::todo::reminders::parse_local_date_to_utc_end_of_day(
                                &state.edit_due_input,
                            )
                        else {
                            state.edit_error_msg = Some(
                                "到期日格式：YYYY-MM-DD / MM-DD / 5月10日 / 今天 / 明天"
                                    .to_string(),
                            );
                            return;
                        };
                        Some(parsed)
                    };

                    let mut reminder_repeat = None;
                    let reminder_at = if state.edit_reminder_input.trim().is_empty() {
                        None
                    } else {
                        if item_completed {
                            state.edit_error_msg =
                                Some("已完成任务不会提醒，请先取消完成状态".to_string());
                            return;
                        }
                        let Some((parsed, repeat)) =
                            crate::todo::reminders::parse_local_reminder_to_utc_and_repeat(
                                &state.edit_reminder_input,
                            )
                        else {
                            state.edit_error_msg = Some(
                                "提醒格式：YYYY-MM-DD HH:MM / 今天 20:00 / 明天 9:00 / 20:00 / +2h"
                                    .to_string(),
                            );
                            return;
                        };
                        if parsed <= chrono::Utc::now() {
                            state.edit_error_msg =
                                Some("提醒时间已过去，请设置未来时间".to_string());
                            return;
                        }
                        reminder_repeat = repeat;
                        Some(parsed)
                    };

                    if let Some(item) = state.items.iter_mut().find(|i| i.id == item_id) {
                        let before = item.clone();
                        item.title = title;
                        item.description = description;
                        item.section_id = state.edit_section_input;
                        item.due_at = due_at;
                        item.priority = state.edit_priority_input.min(3);
                        item.reminder_at = reminder_at;
                        item.reminder_sent = item.completed || item.reminder_at.is_none();
                        item.reminder_repeat = if item.reminder_at.is_some() {
                            reminder_repeat
                        } else {
                            None
                        };
                        item.tags = tags;
                        state.push_undo_replace_item(item_id, before);
                        *state_changed = true;
                    }

                    state.editing_task = None;
                    state.edit_error_msg = None;
                }
            });
        });

    if !open {
        state.editing_task = None;
        state.edit_error_msg = None;
    }
}

fn parse_tags_input(input: &str) -> Vec<String> {
    let mut tags: Vec<String> = Vec::new();
    for raw in input.split(|c: char| c == ',' || c == '，' || c.is_whitespace() || c == '#') {
        let t = raw.trim();
        if t.is_empty() {
            continue;
        }
        if !tags.iter().any(|x| x.eq_ignore_ascii_case(t)) {
            tags.push(t.to_string());
        }
    }
    tags
}
