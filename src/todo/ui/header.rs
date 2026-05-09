use crate::todo::state::{FilterAutomated, FilterReminder, FilterStatus, SortMode};

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
        }

        ui.label("📁 保存位置 (Save Location):");
        let path_display = match &state.save_folder {
            Some(p) => p.clone(),
            None => "默认配置 (Default Storage)".to_string(),
        };
        ui.label(egui::RichText::new(path_display).color(egui::Color32::LIGHT_BLUE));

        if ui.button("更改 (Change)").clicked() {
            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                state.save_folder = Some(folder.to_string_lossy().to_string());
                state.load_from_file();
                *state_changed = true;
            }
        }

        if state.save_folder.is_some() && ui.button("重置 (Reset)").clicked() {
            state.save_folder = None;
            *state_changed = true;
        }
    });

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

            egui::ComboBox::from_id_salt("new_task_section")
                .selected_text(
                    state
                        .new_task_section
                        .and_then(|id| state.section_name(id).map(|s| s.to_string()))
                        .unwrap_or_else(|| "选择分区 (Section)".to_string()),
                )
                .show_ui(ui, |ui| {
                    for section in &state.sections {
                        ui.selectable_value(
                            &mut state.new_task_section,
                            Some(section.id),
                            section.name.clone(),
                        );
                    }
                });

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
                            state.new_task_error_msg =
                                Some("到期日格式应为 YYYY-MM-DD".to_string());
                            return;
                        };
                        Some(parsed)
                    };

                    let reminder_at = if state.new_task_reminder.trim().is_empty() {
                        None
                    } else {
                        let Some(parsed) =
                            crate::todo::reminders::parse_local_datetime_to_utc(
                                &state.new_task_reminder,
                            )
                        else {
                            state.new_task_error_msg =
                                Some("提醒时间格式应为 YYYY-MM-DD HH:MM".to_string());
                            return;
                        };
                        if parsed <= chrono::Utc::now() {
                            state.new_task_error_msg = Some("提醒时间需为未来时间".to_string());
                            return;
                        }
                        Some(parsed)
                    };

                    let mut item = TodoItem::new(state.new_task_title.clone(), desc, section_id);
                    item.due_at = due_at;
                    item.reminder_at = reminder_at;
                    item.reminder_sent = item.reminder_at.is_none();
                    item.priority = state.new_task_priority.min(3);
                    state.items.push(item);
                    state.new_task_title.clear();
                    state.new_task_description.clear();
                    state.new_task_due.clear();
                    state.new_task_reminder.clear();
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
            ui.label("到期日 (Due, YYYY-MM-DD, 可选)");
            ui.add_space(12.0);
            ui.text_edit_singleline(&mut state.new_task_reminder);
            ui.label("提醒 (Reminder, YYYY-MM-DD HH:MM, 可选)");
        });

        if let Some(err) = &state.new_task_error_msg {
            ui.add_space(4.0);
            ui.label(egui::RichText::new(err).color(egui::Color32::RED));
        }
    });

    ui.add_space(16.0);
    ui.separator();
}
