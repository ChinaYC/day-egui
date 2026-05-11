use crate::todo::state::{FilterAutomated, FilterReminder, FilterStatus, SortMode, TodoViewMode};

use super::super::{TodoItem, TodoState};

pub fn show(state: &mut TodoState, ui: &mut egui::Ui, state_changed: &mut bool) {
    ui.heading("日常 Todo 清单 (Daily Todo List)");

    let auto_count = state.get_today_automated_count();
    if auto_count > 0 {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(format!(
                "🤖 今日自动化完成任务数: {} (Automated tasks today)",
                auto_count
            ))
            .color(egui::Color32::LIGHT_GREEN),
        );
    }

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        let undo_enabled = !state.undo_stack.is_empty();
        if ui
            .add_enabled(undo_enabled, egui::Button::new("↩ 撤销 (Undo)"))
            .clicked()
        {
            if state.undo_last_action() {
                *state_changed = true;
            }
        }

        ui.add_space(8.0);
        let can_bulk = state.view_mode == TodoViewMode::Tasks;
        if ui
            .add_enabled(
                can_bulk,
                egui::Button::new("✅ 批量").selected(state.selection_mode),
            )
            .clicked()
        {
            state.selection_mode = !state.selection_mode;
            if !state.selection_mode {
                state.selected_items.clear();
                state.batch_tag_input.clear();
            }
        }
        if state.selection_mode {
            ui.label(format!("已选 {}", state.selected_items.len()));
            if ui.button("清空选择").clicked() {
                state.selected_items.clear();
            }
        }

        ui.add_space(8.0);
        let search_response = ui.add(
            egui::TextEdit::singleline(&mut state.search_query)
                .hint_text("🔍 搜索标题/备注 (Search)")
                .desired_width(220.0),
        );
        if !state.search_query.is_empty() && ui.button("✖").clicked() {
            state.search_query.clear();
            search_response.request_focus();
        }

        ui.add_space(8.0);
        ui.label("状态：");
        egui::ComboBox::from_id_salt("filter_status")
            .selected_text(match state.filter_status {
                FilterStatus::All => "全部",
                FilterStatus::Active => "未完成",
                FilterStatus::Completed => "已完成",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.filter_status, FilterStatus::All, "全部");
                ui.selectable_value(&mut state.filter_status, FilterStatus::Active, "未完成");
                ui.selectable_value(&mut state.filter_status, FilterStatus::Completed, "已完成");
            });

        ui.label("来源：");
        egui::ComboBox::from_id_salt("filter_automated")
            .selected_text(match state.filter_automated {
                FilterAutomated::All => "全部",
                FilterAutomated::AutomatedOnly => "自动",
                FilterAutomated::ManualOnly => "手动",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.filter_automated, FilterAutomated::All, "全部");
                ui.selectable_value(
                    &mut state.filter_automated,
                    FilterAutomated::AutomatedOnly,
                    "自动",
                );
                ui.selectable_value(
                    &mut state.filter_automated,
                    FilterAutomated::ManualOnly,
                    "手动",
                );
            });

        ui.label("提醒：");
        egui::ComboBox::from_id_salt("filter_reminder")
            .selected_text(match state.filter_reminder {
                FilterReminder::All => "全部",
                FilterReminder::WithReminder => "有提醒",
                FilterReminder::WithoutReminder => "无提醒",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.filter_reminder, FilterReminder::All, "全部");
                ui.selectable_value(
                    &mut state.filter_reminder,
                    FilterReminder::WithReminder,
                    "有提醒",
                );
                ui.selectable_value(
                    &mut state.filter_reminder,
                    FilterReminder::WithoutReminder,
                    "无提醒",
                );
            });

        let mut tags: Vec<String> = {
            let mut set: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
            for it in &state.items {
                if it.deleted_at.is_some() {
                    continue;
                }
                for t in &it.tags {
                    let t = t.trim();
                    if !t.is_empty() {
                        set.insert(t.to_string());
                    }
                }
            }
            set.into_iter().collect()
        };
        tags.sort();

        ui.label("标签：");
        ui.menu_button(
            match &state.active_tag {
                Some(t) => format!("标签：#{t}"),
                None => "全部".to_string(),
            },
            |ui| {
                if ui.button("全部").clicked() {
                    state.active_tag = None;
                    state.selection_mode = false;
                    state.selected_items.clear();
                    ui.close();
                }
                ui.separator();
                for t in &tags {
                    if ui.button(format!("#{t}")).clicked() {
                        state.active_tag = Some(t.clone());
                        state.selection_mode = false;
                        state.selected_items.clear();
                        ui.close();
                    }
                }
            },
        );

        ui.label("排序：");
        egui::ComboBox::from_id_salt("sort_mode")
            .selected_text(match state.sort_mode {
                SortMode::Manual => "手动排序",
                SortMode::Priority => "按优先级",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.sort_mode, SortMode::Manual, "手动排序");
                ui.selectable_value(&mut state.sort_mode, SortMode::Priority, "按优先级");
            });

        if ui.button("重置筛选").clicked() {
            state.search_query.clear();
            state.filter_status = FilterStatus::All;
            state.filter_automated = FilterAutomated::All;
            state.filter_reminder = FilterReminder::All;
            state.sort_mode = SortMode::Manual;
            state.active_tag = None;
        }

        ui.label("📁 保存位置 (Save Location):");
        let path_display = match &state.save_folder {
            Some(p) => p.clone(),
            None => "默认配置 (Default Storage)".to_string(),
        };
        ui.label(egui::RichText::new(path_display).color(egui::Color32::LIGHT_BLUE));

        if ui.button("更改 (Change)").clicked() {
            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                state.switch_storage_folder(Some(folder.to_string_lossy().to_string()));
                *state_changed = true;
            }
        }

        if state.save_folder.is_some() && ui.button("重置 (Reset)").clicked() {
            state.switch_storage_folder(None);
            *state_changed = true;
        }
    });

    if state.view_mode == TodoViewMode::Tasks
        && state.selection_mode
        && !state.selected_items.is_empty()
    {
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

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if ui.button("完成").clicked() {
                apply_complete(state, true, state_changed);
            }
            if ui.button("取消完成").clicked() {
                apply_complete(state, false, state_changed);
            }
            if ui.button("删除").clicked() {
                apply_delete(state, state_changed);
            }

            ui.menu_button("移动到清单", |ui| {
                for (folder_id, folder_name) in &folders {
                    ui.collapsing(folder_name.clone(), |ui| {
                        for (section_id, section_name, _folder) in sections
                            .iter()
                            .filter(|(_, _, folder)| *folder == Some(*folder_id))
                        {
                            if ui.button(section_name.clone()).clicked() {
                                apply_move_section(state, *section_id, state_changed);
                                ui.close();
                            }
                        }
                    });
                }
                ui.separator();
                for (section_id, section_name, folder) in sections.iter() {
                    if folder.is_some() {
                        continue;
                    }
                    if ui.button(section_name.clone()).clicked() {
                        apply_move_section(state, *section_id, state_changed);
                        ui.close();
                    }
                }
            });

            ui.menu_button("优先级", |ui| {
                for (p, label) in [(0u8, "P0"), (1, "P1"), (2, "P2"), (3, "P3")] {
                    if ui.button(label).clicked() {
                        apply_priority(state, p, state_changed);
                        ui.close();
                    }
                }
            });

            ui.menu_button("加标签", |ui| {
                ui.text_edit_singleline(&mut state.batch_tag_input);
                if ui.button("应用").clicked() {
                    let input = state.batch_tag_input.clone();
                    apply_add_tags(state, &input, state_changed);
                    ui.close();
                }
            });
        });
    }

    if let Some(err) = &state.error_msg {
        ui.label(egui::RichText::new(err).color(egui::Color32::RED));
        if ui.button("清除错误 (Clear)").clicked() {
            state.error_msg = None;
        }
    }

    ui.add_space(8.0);
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            let response = ui.text_edit_singleline(&mut state.new_task_title);
            ui.label("任务标题 (Title)");

            ui.label("清单：");
            egui::ComboBox::from_id_salt("new_task_section")
                .selected_text(
                    state
                        .new_task_section
                        .and_then(|id| state.section_name(id).map(|s| s.to_string()))
                        .unwrap_or_else(|| "选择分区 (Section)".to_string()),
                )
                .show_ui(ui, |ui| {
                    for section in &state.sections {
                        let label = section
                            .folder_id
                            .and_then(|fid| state.folder_name(fid).map(|f| f.to_string()))
                            .map(|f| format!("{f} / {}", section.name))
                            .unwrap_or_else(|| section.name.clone());
                        ui.selectable_value(&mut state.new_task_section, Some(section.id), label);
                    }
                });

            ui.label("优先级：");
            egui::ComboBox::from_id_salt("new_task_priority")
                .selected_text(match state.new_task_priority.min(3) {
                    0 => "P0",
                    1 => "P1",
                    2 => "P2",
                    _ => "P3",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut state.new_task_priority, 0, "P0");
                    ui.selectable_value(&mut state.new_task_priority, 1, "P1");
                    ui.selectable_value(&mut state.new_task_priority, 2, "P2");
                    ui.selectable_value(&mut state.new_task_priority, 3, "P3");
                });

            if ui.button("➕ 添加任务 (Add Task)").clicked()
                || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
            {
                if !state.new_task_title.trim().is_empty() {
                    let (title, tags) =
                        parse_title_and_tags(&state.new_task_title, &state.new_task_tags);
                    let desc = if state.new_task_description.trim().is_empty() {
                        None
                    } else {
                        Some(state.new_task_description.clone())
                    };
                    let section_id = state.new_task_section;

                    let due_at = if state.new_task_due.trim().is_empty() {
                        None
                    } else {
                        let Some(parsed) =
                            crate::todo::reminders::parse_local_date_to_utc_end_of_day(
                                &state.new_task_due,
                            )
                        else {
                            state.new_task_error_msg = Some(
                                "到期日格式：YYYY-MM-DD / MM-DD / 5月10日 / 今天 / 明天"
                                    .to_string(),
                            );
                            return;
                        };
                        Some(parsed)
                    };

                    let mut reminder_repeat = None;
                    let reminder_at = if state.new_task_reminder.trim().is_empty() {
                        None
                    } else {
                        let Some((parsed, repeat)) =
                            crate::todo::reminders::parse_local_reminder_to_utc_and_repeat(
                                &state.new_task_reminder,
                            )
                        else {
                            state.new_task_error_msg = Some(
                                "提醒格式：YYYY-MM-DD HH:MM / 今天 20:00 / 周二 9:00 / 每周二 9:00 / 20:00 / +2h"
                                    .to_string(),
                            );
                            return;
                        };
                        if parsed <= chrono::Utc::now() {
                            state.new_task_error_msg = Some("提醒时间需为未来时间".to_string());
                            return;
                        }
                        reminder_repeat = repeat;
                        Some(parsed)
                    };

                    let mut item = TodoItem::new(title, desc, section_id);
                    item.due_at = due_at;
                    item.reminder_at = reminder_at;
                    item.reminder_sent = item.reminder_at.is_none();
                    item.reminder_repeat = reminder_repeat;
                    item.priority = state.new_task_priority.min(3);
                    item.tags = tags;
                    state.items.push(item);
                    state.new_task_title.clear();
                    state.new_task_description.clear();
                    state.new_task_due.clear();
                    state.new_task_reminder.clear();
                    state.new_task_tags.clear();
                    state.new_task_error_msg = None;
                    *state_changed = true;
                }
            }
        });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut state.new_task_description);
            ui.label("任务备注 (Description, 可选)");
        });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut state.new_task_due);
            ui.label("到期日 (Due, YYYY-MM-DD/今天/明天/周二, 可选)");
            ui.add_space(12.0);
            ui.text_edit_singleline(&mut state.new_task_reminder);
            ui.label("提醒 (Reminder, 支持周几/相对时间, 可选)");
            ui.add_space(12.0);
            ui.text_edit_singleline(&mut state.new_task_tags);
            ui.label("标签 (Tags, 可选)");
        });

        if let Some(err) = &state.new_task_error_msg {
            ui.add_space(4.0);
            ui.label(egui::RichText::new(err).color(egui::Color32::RED));
        }
    });

    ui.add_space(16.0);
    ui.separator();
}

fn parse_title_and_tags(title: &str, extra: &str) -> (String, Vec<String>) {
    let mut tags: Vec<String> = Vec::new();
    let mut parts: Vec<&str> = Vec::new();

    for token in title.split_whitespace() {
        if let Some(t) = token.strip_prefix('#') {
            let t = t.trim_matches(|c: char| c == ',' || c == '，' || c == ';' || c == '；');
            if !t.is_empty() && !tags.iter().any(|x| x.eq_ignore_ascii_case(t)) {
                tags.push(t.to_string());
            }
        } else {
            parts.push(token);
        }
    }

    for raw in extra.split(|c: char| c == ',' || c == '，' || c.is_whitespace() || c == '#') {
        let t = raw.trim();
        if t.is_empty() {
            continue;
        }
        if !tags.iter().any(|x| x.eq_ignore_ascii_case(t)) {
            tags.push(t.to_string());
        }
    }

    (parts.join(" "), tags)
}

fn selected_ids(state: &TodoState) -> Vec<uuid::Uuid> {
    state.selected_items.iter().copied().collect()
}

fn apply_complete(state: &mut TodoState, completed: bool, state_changed: &mut bool) {
    for id in selected_ids(state) {
        if let Some(item) = state.items.iter_mut().find(|i| i.id == id) {
            if item.deleted_at.is_some() {
                continue;
            }
            let before = item.clone();
            item.completed = completed;
            if item.completed {
                item.reminder_sent = true;
            } else if let Some(rule) = item.reminder_repeat {
                if let Some(next) =
                    crate::todo::reminders::next_reminder_from_repeat(rule, chrono::Utc::now())
                {
                    item.reminder_at = Some(next);
                }
                item.reminder_sent = false;
            } else if item.reminder_at.is_some() {
                item.reminder_sent = false;
            }
            item.touch();
            state.push_undo_replace_item(id, before);
            *state_changed = true;
        }
    }
}

fn apply_delete(state: &mut TodoState, state_changed: &mut bool) {
    for id in selected_ids(state) {
        if let Some(item) = state.items.iter_mut().find(|i| i.id == id) {
            if item.deleted_at.is_some() {
                continue;
            }
            let before = item.clone();
            item.deleted_at = Some(chrono::Utc::now());
            item.reminder_sent = true;
            item.touch();
            state.push_undo_replace_item(id, before);
            *state_changed = true;
        }
    }
    state.show_snackbar("已删除");
}

fn apply_move_section(state: &mut TodoState, section_id: uuid::Uuid, state_changed: &mut bool) {
    for id in selected_ids(state) {
        if let Some(item) = state.items.iter_mut().find(|i| i.id == id) {
            if item.deleted_at.is_some() {
                continue;
            }
            let before = item.clone();
            item.section_id = Some(section_id);
            item.touch();
            state.push_undo_replace_item(id, before);
            *state_changed = true;
        }
    }
}

fn apply_priority(state: &mut TodoState, priority: u8, state_changed: &mut bool) {
    let p = priority.min(3);
    for id in selected_ids(state) {
        if let Some(item) = state.items.iter_mut().find(|i| i.id == id) {
            if item.deleted_at.is_some() {
                continue;
            }
            let before = item.clone();
            item.priority = p;
            item.touch();
            state.push_undo_replace_item(id, before);
            *state_changed = true;
        }
    }
}

fn apply_add_tags(state: &mut TodoState, input: &str, state_changed: &mut bool) {
    let tags = parse_tags_only(input);
    if tags.is_empty() {
        return;
    }
    for id in selected_ids(state) {
        if let Some(item) = state.items.iter_mut().find(|i| i.id == id) {
            if item.deleted_at.is_some() {
                continue;
            }
            let before = item.clone();
            for t in &tags {
                if !item.tags.iter().any(|x| x.eq_ignore_ascii_case(t)) {
                    item.tags.push(t.clone());
                }
            }
            item.touch();
            state.push_undo_replace_item(id, before);
            *state_changed = true;
        }
    }
}

fn parse_tags_only(input: &str) -> Vec<String> {
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
