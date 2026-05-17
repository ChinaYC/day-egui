use super::super::TodoState;
use crate::todo::ThemeMode;

pub fn show(state: &mut TodoState, ui: &mut egui::Ui, state_changed: &mut bool) {
    egui::Area::new("todo_settings_button".into())
        .anchor(egui::Align2::LEFT_BOTTOM, [10.0, -10.0])
        .show(ui.ctx(), |ui| {
            if ui.button("⚙").clicked() {
                state.show_settings = true;
            }
        });

    show_import_conflict_window(state, ui, state_changed);

    if !state.show_settings {
        return;
    }

    let auto_id = state.get_or_create_section_id_by_name("自动 (Auto)");
    let manual_id = state.get_or_create_section_id_by_name("手动 (Manual)");

    let mut open = true;
    egui::Window::new("设置 (Settings)")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ui.ctx(), |ui| {
            ui.label("字体大小 (Font size):");
            let mut font_scale = state.settings.font_scale;
            if ui
                .add(
                    egui::Slider::new(&mut font_scale, 0.8..=1.6)
                        .show_value(true)
                        .clamping(egui::SliderClamping::Always),
                )
                .changed()
            {
                state.settings.font_scale = font_scale;
                *state_changed = true;
            }

            ui.add_space(10.0);
            ui.label("主题 (Theme):");
            let mut theme_mode = state.settings.theme_mode;
            egui::ComboBox::from_id_salt("theme_mode_select")
                .selected_text(match theme_mode {
                    ThemeMode::System => "跟随系统",
                    ThemeMode::Light => "白色",
                    ThemeMode::Dark => "黑色",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut theme_mode, ThemeMode::System, "跟随系统");
                    ui.selectable_value(&mut theme_mode, ThemeMode::Light, "白色");
                    ui.selectable_value(&mut theme_mode, ThemeMode::Dark, "黑色");
                });
            if theme_mode != state.settings.theme_mode {
                state.settings.theme_mode = theme_mode;
                *state_changed = true;
            }

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);

            ui.label("菜单配置 (Menu configuration):");
            ui.vertical(|ui| {
                let mut move_up = None;
                let mut move_down = None;
                let len = state.settings.sidebar_items.len();

                for (i, item) in state.settings.sidebar_items.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        if ui.checkbox(&mut item.visible, "").changed() {
                            *state_changed = true;
                        }
                        ui.label(&item.name);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if i < len - 1 {
                                if ui.button("↓").clicked() {
                                    move_down = Some(i);
                                }
                            }
                            if i > 0 {
                                if ui.button("↑").clicked() {
                                    move_up = Some(i);
                                }
                            }
                        });
                    });
                }

                if let Some(i) = move_up {
                    state.settings.sidebar_items.swap(i, i - 1);
                    *state_changed = true;
                }
                if let Some(i) = move_down {
                    state.settings.sidebar_items.swap(i, i + 1);
                    *state_changed = true;
                }
            });

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);

            ui.label("自动任务默认分区 (Automated tasks go to):");
            let mut selected = state.settings.automated_section_id;
            egui::ComboBox::from_id_salt("automated_section_select")
                .selected_text(
                    selected
                        .and_then(|id| state.section_name(id).map(|s| s.to_string()))
                        .unwrap_or_else(|| "未设置 (Unset)".to_string()),
                )
                .show_ui(ui, |ui| {
                    for section in &state.sections {
                        ui.selectable_value(&mut selected, Some(section.id), section.name.clone());
                    }
                });
            if selected != state.settings.automated_section_id {
                state.settings.automated_section_id = selected;
                *state_changed = true;
            }

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);

            ui.label("文件夹 (Folders):");
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut state.new_folder_name);
                if ui.button("创建文件夹").clicked() {
                    let name = state.new_folder_name.trim().to_string();
                    if name.is_empty() {
                        state.folder_manage_error_msg = Some("文件夹名称不能为空".to_string());
                        return;
                    }
                    if state.folders.iter().any(|f| f.name == name) {
                        state.folder_manage_error_msg = Some("文件夹名称已存在".to_string());
                        return;
                    }
                    state.folders.push(super::super::TodoFolder::new(name));
                    state.new_folder_name.clear();
                    state.folder_manage_error_msg = None;
                    *state_changed = true;
                }
            });
            if let Some(err) = &state.folder_manage_error_msg {
                ui.label(egui::RichText::new(err).color(egui::Color32::RED));
            }

            let folders: Vec<(uuid::Uuid, String)> = state
                .folders
                .iter()
                .map(|f| (f.id, f.name.clone()))
                .collect();
            for (folder_id, folder_name) in folders {
                ui.horizontal(|ui| {
                    ui.label(folder_name);
                    if ui.button("重命名").clicked() {
                        state.folder_to_rename = Some(folder_id);
                        state.folder_rename_input =
                            state.folder_name(folder_id).unwrap_or_default().to_string();
                        state.folder_manage_error_msg = None;
                    }
                    if ui.button("删除").clicked() {
                        state.folder_to_delete = Some(folder_id);
                        state.folder_manage_error_msg = None;
                    }
                });
            }

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);

            ui.label("新建分区 (Create section):");
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut state.new_section_name);
                if ui.button("创建 (Create)").clicked() {
                    let name = state.new_section_name.trim().to_string();
                    if !name.is_empty() {
                        state.sections.push(super::super::TodoSection::new(name));
                        state.new_section_name.clear();
                        *state_changed = true;
                    }
                }
            });

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);

            ui.label("分区管理 (Manage sections):");
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
            for (section_id, section_name, section_folder) in sections {
                ui.horizontal(|ui| {
                    ui.label(section_name);

                    let mut folder_id = section_folder;
                    egui::ComboBox::from_id_salt(format!("section_folder_{section_id}"))
                        .selected_text(
                            folder_id
                                .and_then(|id| state.folder_name(id).map(|s| s.to_string()))
                                .unwrap_or_else(|| "未归类".to_string()),
                        )
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut folder_id, None, "未归类");
                            for (fid, fname) in &folders {
                                ui.selectable_value(&mut folder_id, Some(*fid), fname.clone());
                            }
                        });
                    if folder_id != section_folder {
                        if let Some(section) =
                            state.sections.iter_mut().find(|s| s.id == section_id)
                        {
                            section.folder_id = folder_id;
                            *state_changed = true;
                        }
                    }

                    let can_manage = section_id != auto_id && section_id != manual_id;

                    if ui
                        .add_enabled(can_manage, egui::Button::new("重命名"))
                        .clicked()
                    {
                        state.section_to_rename = Some(section_id);
                        state.section_rename_input = state
                            .section_name(section_id)
                            .unwrap_or_default()
                            .to_string();
                        state.section_manage_error_msg = None;
                    }
                    if ui
                        .add_enabled(can_manage, egui::Button::new("删除"))
                        .clicked()
                    {
                        state.section_to_delete = Some(section_id);
                        state.section_delete_move_to = Some(manual_id);
                        state.section_manage_error_msg = None;
                    }
                });
            }

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);

            ui.label("数据工具 (Data tools):");
            if let Some(folder) = state.get_save_folder_path() {
                ui.horizontal(|ui| {
                    ui.label("数据目录：");
                    let folder_str = folder.to_string_lossy().to_string();
                    ui.label(
                        egui::RichText::new(folder_str.clone()).color(egui::Color32::LIGHT_BLUE),
                    );
                    if ui.button("复制路径").clicked() {
                        ui.ctx().copy_text(folder_str);
                        state.show_snackbar("已复制路径");
                    }
                });
                ui.add_space(4.0);
            } else {
                ui.label("数据目录：Web 版本暂无本地目录");
                ui.add_space(4.0);
            }

            if let Some(t) = state.settings.last_sync_time {
                let local = t.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M").to_string();
                ui.label(format!("上次导入时间：{local}"));
            } else {
                ui.label("上次导入时间：无");
            }

            ui.horizontal(|ui| {
                ui.checkbox(&mut state.import_manual_conflicts, "导入时手动解决冲突");

                if ui.button("导入任务 (Import)").clicked() {
                    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
                    {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Todo", &["json", "jsonl"])
                            .pick_file()
                        {
                            if state.start_import_from_file(path) {
                                *state_changed = true;
                            }
                        }
                    }
                    #[cfg(target_os = "android")]
                    {
                        state.show_snackbar("Android 版本暂不支持导入");
                    }
                    #[cfg(target_arch = "wasm32")]
                    {
                        state.show_snackbar("Web 版本暂不支持导入");
                    }
                }
            });

            ui.horizontal(|ui| {
                if ui.button("导出任务 JSONL").clicked() {
                    if let Some(path) = state.export_jsonl(false) {
                        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                            state.show_snackbar(format!("已导出: {name}"));
                        } else {
                            state.show_snackbar("已导出");
                        }
                    }
                }
                if ui.button("导出回收站 JSONL").clicked() {
                    if let Some(path) = state.export_jsonl(true) {
                        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                            state.show_snackbar(format!("已导出: {name}"));
                        } else {
                            state.show_snackbar("已导出");
                        }
                    }
                }
                if ui.button("导出提醒 reminders.ics").clicked() {
                    super::super::reminders::write_reminders_ics(state);
                    state.show_snackbar("已导出 reminders.ics");
                }
            });
        });

    if !open {
        state.show_settings = false;
    }

    if let Some(section_id) = state.section_to_rename {
        let mut rename_open = true;
        egui::Window::new("重命名分区 (Rename section)")
            .open(&mut rename_open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                ui.label("新名称 (New name):");
                ui.text_edit_singleline(&mut state.section_rename_input);
                if let Some(err) = &state.section_manage_error_msg {
                    ui.label(egui::RichText::new(err).color(egui::Color32::RED));
                }

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("取消").clicked() {
                        state.section_to_rename = None;
                        state.section_manage_error_msg = None;
                    }
                    if ui.button("保存").clicked() {
                        let name = state.section_rename_input.trim().to_string();
                        if name.is_empty() {
                            state.section_manage_error_msg = Some("分区名称不能为空".to_string());
                            return;
                        }
                        if state
                            .sections
                            .iter()
                            .any(|s| s.id != section_id && s.name == name)
                        {
                            state.section_manage_error_msg = Some("分区名称已存在".to_string());
                            return;
                        }
                        if let Some(section) =
                            state.sections.iter_mut().find(|s| s.id == section_id)
                        {
                            section.name = name;
                            *state_changed = true;
                        }
                        state.section_to_rename = None;
                        state.section_manage_error_msg = None;
                    }
                });
            });
        if !rename_open {
            state.section_to_rename = None;
            state.section_manage_error_msg = None;
        }
    }

    if let Some(section_id) = state.section_to_delete {
        let mut delete_open = true;
        egui::Window::new("删除分区 (Delete section)")
            .open(&mut delete_open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                let section_name = state.section_name(section_id).unwrap_or("未知分区");
                ui.label(format!(
                    "确认删除分区：{}\n该分区下的任务将移动到：",
                    section_name
                ));

                let mut move_to = state.section_delete_move_to.or(Some(manual_id));
                egui::ComboBox::from_id_salt("delete_section_move_to")
                    .selected_text(
                        move_to
                            .and_then(|id| state.section_name(id).map(|s| s.to_string()))
                            .unwrap_or_else(|| "请选择".to_string()),
                    )
                    .show_ui(ui, |ui| {
                        for section in &state.sections {
                            if section.id == section_id {
                                continue;
                            }
                            ui.selectable_value(
                                &mut move_to,
                                Some(section.id),
                                section.name.clone(),
                            );
                        }
                    });
                state.section_delete_move_to = move_to;

                if let Some(err) = &state.section_manage_error_msg {
                    ui.label(egui::RichText::new(err).color(egui::Color32::RED));
                }

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("取消").clicked() {
                        state.section_to_delete = None;
                        state.section_manage_error_msg = None;
                    }
                    if ui.button("确认删除").clicked() {
                        let Some(move_to) = state.section_delete_move_to else {
                            state.section_manage_error_msg = Some("请选择迁移目标分区".to_string());
                            return;
                        };
                        if move_to == section_id {
                            state.section_manage_error_msg =
                                Some("迁移目标不能是当前分区".to_string());
                            return;
                        }

                        for item in &mut state.items {
                            if item.section_id == Some(section_id) {
                                item.section_id = Some(move_to);
                                item.touch();
                            }
                        }
                        state.sections.retain(|s| s.id != section_id);

                        if state.settings.automated_section_id == Some(section_id) {
                            state.settings.automated_section_id = Some(auto_id);
                        }
                        if state.new_task_section == Some(section_id) {
                            state.new_task_section = Some(move_to);
                        }
                        if state.active_section == Some(section_id) {
                            state.active_section = None;
                        }

                        state.ensure_builtin_sections_and_settings();
                        *state_changed = true;

                        state.section_to_delete = None;
                        state.section_delete_move_to = None;
                        state.section_manage_error_msg = None;
                    }
                });
            });

        if !delete_open {
            state.section_to_delete = None;
            state.section_manage_error_msg = None;
        }
    }

    if let Some(folder_id) = state.folder_to_rename {
        let mut rename_open = true;
        egui::Window::new("重命名文件夹 (Rename folder)")
            .open(&mut rename_open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                ui.label("新名称 (New name):");
                ui.text_edit_singleline(&mut state.folder_rename_input);
                if let Some(err) = &state.folder_manage_error_msg {
                    ui.label(egui::RichText::new(err).color(egui::Color32::RED));
                }

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("取消").clicked() {
                        state.folder_to_rename = None;
                        state.folder_manage_error_msg = None;
                    }
                    if ui.button("保存").clicked() {
                        let name = state.folder_rename_input.trim().to_string();
                        if name.is_empty() {
                            state.folder_manage_error_msg = Some("文件夹名称不能为空".to_string());
                            return;
                        }
                        if state
                            .folders
                            .iter()
                            .any(|f| f.id != folder_id && f.name == name)
                        {
                            state.folder_manage_error_msg = Some("文件夹名称已存在".to_string());
                            return;
                        }
                        if let Some(folder) = state.folders.iter_mut().find(|f| f.id == folder_id) {
                            folder.name = name;
                            *state_changed = true;
                        }
                        state.folder_to_rename = None;
                        state.folder_manage_error_msg = None;
                    }
                });
            });
        if !rename_open {
            state.folder_to_rename = None;
            state.folder_manage_error_msg = None;
        }
    }

    if let Some(folder_id) = state.folder_to_delete {
        let mut delete_open = true;
        egui::Window::new("删除文件夹 (Delete folder)")
            .open(&mut delete_open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                let folder_name = state.folder_name(folder_id).unwrap_or("未知文件夹");
                ui.label(format!(
                    "确认删除文件夹：{}\n该文件夹下的清单将移动到：未归类",
                    folder_name
                ));
                if let Some(err) = &state.folder_manage_error_msg {
                    ui.label(egui::RichText::new(err).color(egui::Color32::RED));
                }

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("取消").clicked() {
                        state.folder_to_delete = None;
                        state.folder_manage_error_msg = None;
                    }
                    if ui.button("确认删除").clicked() {
                        for section in &mut state.sections {
                            if section.folder_id == Some(folder_id) {
                                section.folder_id = None;
                            }
                        }
                        state.folders.retain(|f| f.id != folder_id);
                        if state.active_folder == Some(folder_id) {
                            state.active_folder = None;
                        }
                        state.folder_to_delete = None;
                        state.folder_manage_error_msg = None;
                        *state_changed = true;
                    }
                });
            });

        if !delete_open {
            state.folder_to_delete = None;
            state.folder_manage_error_msg = None;
        }
    }
}

fn show_import_conflict_window(state: &mut TodoState, ui: &mut egui::Ui, state_changed: &mut bool) {
    if state.pending_import.is_none() {
        return;
    };

    let mut open = true;
    let mut choose_latest = false;
    let mut cancel = false;
    let mut apply = false;
    egui::Window::new("冲突解决 (Resolve conflicts)")
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ui.ctx(), |ui| {
            let count = state
                .pending_import
                .as_ref()
                .map(|p| p.conflicts.len())
                .unwrap_or(0);
            ui.label(format!("冲突条目：{}", count));
            ui.add_space(8.0);

            egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                if let Some(pending) = state.pending_import.as_mut() {
                    for c in &mut pending.conflicts {
                        ui.separator();
                        let local_time = c
                            .local
                            .updated_at_utc()
                            .with_timezone(&chrono::Local)
                            .format("%Y-%m-%d %H:%M")
                            .to_string();
                        let incoming_time = c
                            .incoming
                            .updated_at_utc()
                            .with_timezone(&chrono::Local)
                            .format("%Y-%m-%d %H:%M")
                            .to_string();

                        ui.label(format!("任务 ID: {}", c.id));
                        ui.label(format!("本地：{}  (更新 {local_time})", c.local.title));
                        ui.label(format!("导入：{}  (更新 {incoming_time})", c.incoming.title));

                        ui.horizontal(|ui| {
                            ui.radio_value(
                                &mut c.choice,
                                crate::todo::state::ImportConflictChoice::UseLocal,
                                "用本地",
                            );
                            ui.radio_value(
                                &mut c.choice,
                                crate::todo::state::ImportConflictChoice::UseIncoming,
                                "用导入",
                            );
                            ui.radio_value(
                                &mut c.choice,
                                crate::todo::state::ImportConflictChoice::DuplicateIncoming,
                                "保留两份",
                            );
                        });
                    }
                }
            });

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("全部用较新").clicked() {
                    choose_latest = true;
                }
                if ui.button("取消导入").clicked() {
                    cancel = true;
                }
                if ui.button("应用导入").clicked() {
                    apply = true;
                }
            });
        });

    if !open {
        state.pending_import = None;
        return;
    }

    if cancel {
        state.pending_import = None;
        return;
    }

    if choose_latest {
        state.choose_latest_for_all_conflicts();
    }

    if apply {
        if state.apply_pending_import() {
            *state_changed = true;
        }
    }
}
