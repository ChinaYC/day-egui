use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::model::{TodoFolder, TodoItem, TodoPlanner, TodoSection, TodoSettings, TodoStorage};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write as _};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TodoViewMode {
    Tasks,
    Planner,
    Trash,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TaskSmartView {
    All,
    Inbox,
    Today,
    Next7Days,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FilterStatus {
    All,
    Active,
    Completed,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FilterAutomated {
    All,
    AutomatedOnly,
    ManualOnly,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FilterReminder {
    All,
    WithReminder,
    WithoutReminder,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    Manual,
    Priority,
}

pub struct Snackbar {
    pub message: String,
    pub expires_at: Instant,
}

#[derive(Clone)]
pub enum UndoAction {
    ReplaceItem { id: Uuid, before: TodoItem },
    ReinsertItem { index: usize, item: TodoItem },
    ReinsertMany { items: Vec<(usize, TodoItem)> },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ImportConflictChoice {
    UseLocal,
    UseIncoming,
    DuplicateIncoming,
}

pub struct ImportConflict {
    pub id: Uuid,
    pub local: TodoItem,
    pub incoming: TodoItem,
    pub choice: ImportConflictChoice,
}

pub struct PendingImport {
    pub incoming_hash: String,
    pub baseline_time: Option<chrono::DateTime<chrono::Utc>>,
    pub items: Vec<TodoItem>,
    pub sections: Vec<TodoSection>,
    pub folders: Vec<TodoFolder>,
    pub conflicts: Vec<ImportConflict>,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct TodoState {
    pub items: Vec<TodoItem>,
    pub sections: Vec<TodoSection>,
    pub folders: Vec<TodoFolder>,
    pub settings: TodoSettings,
    pub planner: TodoPlanner,
    pub new_task_title: String,
    pub new_task_description: String,
    pub new_task_due: String,
    pub new_task_reminder: String,
    pub new_task_tags: String,

    pub save_folder: Option<String>,

    #[serde(skip)]
    pub item_to_delete: Option<Uuid>,
    #[serde(skip)]
    pub delete_is_permanent: bool,
    #[serde(skip)]
    pub confirm_clear_trash: bool,
    #[serde(skip)]
    pub dragging_item: Option<Uuid>,
    #[serde(skip)]
    pub dragging_section: Option<Uuid>,
    #[serde(skip)]
    pub drag_target_index: Option<usize>,

    #[serde(skip)]
    pub active_section: Option<Uuid>,
    #[serde(skip)]
    pub active_folder: Option<Uuid>,
    #[serde(skip)]
    pub active_tag: Option<String>,
    #[serde(skip)]
    pub new_section_name: String,
    #[serde(skip)]
    pub new_task_section: Option<Uuid>,
    #[serde(skip)]
    pub show_settings: bool,

    #[serde(skip)]
    pub editing_reminder: Option<Uuid>,
    #[serde(skip)]
    pub reminder_input: String,
    #[serde(skip)]
    pub reminder_error_msg: Option<String>,

    #[serde(skip)]
    pub editing_task: Option<Uuid>,
    #[serde(skip)]
    pub edit_title_input: String,
    #[serde(skip)]
    pub edit_desc_input: String,
    #[serde(skip)]
    pub edit_tags_input: String,
    #[serde(skip)]
    pub edit_section_input: Option<Uuid>,
    #[serde(skip)]
    pub edit_due_input: String,
    #[serde(skip)]
    pub edit_reminder_input: String,
    #[serde(skip)]
    pub edit_priority_input: u8,
    #[serde(skip)]
    pub edit_error_msg: Option<String>,

    #[serde(skip)]
    pub selection_mode: bool,
    #[serde(skip)]
    pub selected_items: std::collections::BTreeSet<Uuid>,
    #[serde(skip)]
    pub batch_tag_input: String,

    #[serde(skip)]
    pub view_mode: TodoViewMode,
    #[serde(skip)]
    pub smart_view: TaskSmartView,
    #[serde(skip)]
    pub search_query: String,
    #[serde(skip)]
    pub filter_status: FilterStatus,
    #[serde(skip)]
    pub filter_automated: FilterAutomated,
    #[serde(skip)]
    pub filter_reminder: FilterReminder,
    #[serde(skip)]
    pub sort_mode: SortMode,

    #[serde(skip)]
    pub undo_stack: Vec<UndoAction>,

    #[serde(skip)]
    pub snackbar: Option<Snackbar>,

    #[serde(skip)]
    pub new_task_error_msg: Option<String>,
    #[serde(skip)]
    pub new_task_priority: u8,

    #[serde(skip)]
    pub section_to_rename: Option<Uuid>,
    #[serde(skip)]
    pub section_rename_input: String,
    #[serde(skip)]
    pub section_to_delete: Option<Uuid>,
    #[serde(skip)]
    pub section_delete_move_to: Option<Uuid>,
    #[serde(skip)]
    pub section_manage_error_msg: Option<String>,

    #[serde(skip)]
    pub new_folder_name: String,
    #[serde(skip)]
    pub folder_to_rename: Option<Uuid>,
    #[serde(skip)]
    pub folder_rename_input: String,
    #[serde(skip)]
    pub folder_to_delete: Option<Uuid>,
    #[serde(skip)]
    pub folder_manage_error_msg: Option<String>,

    #[serde(skip)]
    pub last_font_scale: f32,

    #[serde(skip)]
    pub initial_loaded: bool,
    #[serde(skip)]
    pub error_msg: Option<String>,

    #[serde(skip)]
    pub import_manual_conflicts: bool,
    #[serde(skip)]
    pub pending_import: Option<PendingImport>,
    #[serde(skip)]
    pub import_error_msg: Option<String>,
}

impl Default for TodoState {
    fn default() -> Self {
        let mut state = Self {
            items: Vec::new(),
            sections: Vec::new(),
            folders: Vec::new(),
            settings: TodoSettings::default(),
            planner: TodoPlanner::default(),
            new_task_title: String::new(),
            new_task_description: String::new(),
            new_task_due: String::new(),
            new_task_reminder: String::new(),
            new_task_tags: String::new(),
            save_folder: None,
            item_to_delete: None,
            delete_is_permanent: false,
            confirm_clear_trash: false,
            dragging_item: None,
            dragging_section: None,
            drag_target_index: None,
            active_section: None,
            active_folder: None,
            active_tag: None,
            new_section_name: String::new(),
            new_task_section: None,
            show_settings: false,
            editing_reminder: None,
            reminder_input: String::new(),
            reminder_error_msg: None,
            editing_task: None,
            edit_title_input: String::new(),
            edit_desc_input: String::new(),
            edit_tags_input: String::new(),
            edit_section_input: None,
            edit_due_input: String::new(),
            edit_reminder_input: String::new(),
            edit_priority_input: 2,
            edit_error_msg: None,
            view_mode: TodoViewMode::Tasks,
            smart_view: TaskSmartView::All,
            search_query: String::new(),
            filter_status: FilterStatus::All,
            filter_automated: FilterAutomated::All,
            filter_reminder: FilterReminder::All,
            sort_mode: SortMode::Manual,
            undo_stack: Vec::new(),
            snackbar: None,
            new_task_error_msg: None,
            new_task_priority: 2,
            section_to_rename: None,
            section_rename_input: String::new(),
            section_to_delete: None,
            section_delete_move_to: None,
            section_manage_error_msg: None,
            new_folder_name: String::new(),
            folder_to_rename: None,
            folder_rename_input: String::new(),
            folder_to_delete: None,
            folder_manage_error_msg: None,
            last_font_scale: 1.0,
            initial_loaded: false,
            error_msg: None,
            selection_mode: false,
            selected_items: std::collections::BTreeSet::new(),
            batch_tag_input: String::new(),
            import_manual_conflicts: false,
            pending_import: None,
            import_error_msg: None,
        };
        state.ensure_builtin_sections_and_settings();
        state
    }
}

impl TodoState {
    pub fn show_snackbar(&mut self, message: impl Into<String>) {
        self.snackbar = Some(Snackbar {
            message: message.into(),
            expires_at: Instant::now() + Duration::from_secs(4),
        });
    }

    pub fn dismiss_snackbar(&mut self) {
        self.snackbar = None;
    }

    pub fn push_undo_replace_item(&mut self, id: Uuid, before: TodoItem) {
        self.undo_stack.push(UndoAction::ReplaceItem { id, before });
        if self.undo_stack.len() > 50 {
            self.undo_stack
                .drain(0..self.undo_stack.len().saturating_sub(50));
        }
    }

    pub fn undo_last_action(&mut self) -> bool {
        let Some(action) = self.undo_stack.pop() else {
            return false;
        };

        match action {
            UndoAction::ReplaceItem { id, before } => {
                if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
                    *item = before;
                    return true;
                }
            }
            UndoAction::ReinsertItem { index, item } => {
                let idx = index.min(self.items.len());
                self.items.insert(idx, item);
                return true;
            }
            UndoAction::ReinsertMany { mut items } => {
                items.sort_by_key(|(idx, _)| *idx);
                for (index, item) in items {
                    let idx = index.min(self.items.len());
                    self.items.insert(idx, item);
                }
                return true;
            }
        }

        false
    }

    pub fn ensure_builtin_sections_and_settings(&mut self) {
        let auto_id = self.get_or_create_section_id_by_name("自动 (Auto)");
        let manual_id = self.get_or_create_section_id_by_name("手动 (Manual)");

        if self.settings.automated_section_id.is_none()
            || self
                .settings
                .automated_section_id
                .is_some_and(|id| self.section_name(id).is_none())
        {
            self.settings.automated_section_id = Some(auto_id);
        }

        if self.new_task_section.is_none()
            || self
                .new_task_section
                .is_some_and(|id| self.section_name(id).is_none())
        {
            self.new_task_section = Some(manual_id);
        }
    }

    pub fn switch_storage_folder(&mut self, folder: Option<String>) {
        self.save_folder = folder;
        self.reset_runtime_state_for_storage_switch();
        self.load_from_file();
    }

    fn reset_runtime_state_for_storage_switch(&mut self) {
        self.item_to_delete = None;
        self.delete_is_permanent = false;
        self.confirm_clear_trash = false;
        self.dragging_item = None;
        self.dragging_section = None;
        self.drag_target_index = None;

        self.active_section = None;
        self.active_folder = None;
        self.active_tag = None;
        self.new_section_name.clear();
        self.new_task_section = None;
        self.show_settings = false;

        self.editing_reminder = None;
        self.reminder_input.clear();
        self.reminder_error_msg = None;

        self.editing_task = None;
        self.edit_title_input.clear();
        self.edit_desc_input.clear();
        self.edit_tags_input.clear();
        self.edit_section_input = None;
        self.edit_due_input.clear();
        self.edit_reminder_input.clear();
        self.edit_priority_input = 2;
        self.edit_error_msg = None;

        self.selection_mode = false;
        self.selected_items.clear();
        self.batch_tag_input.clear();

        self.view_mode = TodoViewMode::Tasks;
        self.smart_view = TaskSmartView::All;
        self.search_query.clear();
        self.filter_status = FilterStatus::All;
        self.filter_automated = FilterAutomated::All;
        self.filter_reminder = FilterReminder::All;
        self.sort_mode = SortMode::Manual;

        self.undo_stack.clear();
        self.new_task_error_msg = None;
        self.error_msg = None;

        self.new_task_title.clear();
        self.new_task_description.clear();
        self.new_task_due.clear();
        self.new_task_reminder.clear();
        self.new_task_tags.clear();
        self.new_task_priority = 2;

        self.pending_import = None;
        self.import_error_msg = None;
    }

    fn reset_persistent_state_for_storage_switch(&mut self) {
        self.items.clear();
        self.sections.clear();
        self.folders.clear();
        self.settings = TodoSettings::default();
        self.planner = TodoPlanner::default();
    }

    pub fn section_name(&self, section_id: Uuid) -> Option<&str> {
        self.sections
            .iter()
            .find(|s| s.id == section_id)
            .map(|s| s.name.as_str())
    }

    pub fn folder_name(&self, folder_id: Uuid) -> Option<&str> {
        self.folders
            .iter()
            .find(|f| f.id == folder_id)
            .map(|f| f.name.as_str())
    }

    pub fn get_or_create_section_id_by_name(&mut self, name: &str) -> Uuid {
        if let Some(section) = self.sections.iter().find(|s| s.name == name) {
            return section.id;
        }
        let section = TodoSection::new(name.to_string());
        let id = section.id;
        self.sections.push(section);
        id
    }

    pub fn automated_target_section_id(&mut self) -> Uuid {
        self.ensure_builtin_sections_and_settings();
        self.settings
            .automated_section_id
            .unwrap_or_else(|| self.get_or_create_section_id_by_name("自动 (Auto)"))
    }

    pub fn migrate_items_without_section(&mut self) {
        let auto_id = self.get_or_create_section_id_by_name("自动 (Auto)");
        let manual_id = self.get_or_create_section_id_by_name("手动 (Manual)");

        for item in &mut self.items {
            if item.section_id.is_none() {
                item.section_id = Some(if item.is_automated {
                    auto_id
                } else {
                    manual_id
                });
            }
        }
    }

    pub fn migrate_items_without_updated_at(&mut self) {
        for item in &mut self.items {
            item.ensure_updated_at();
        }
    }

    pub fn migrate_items_with_invalid_section(&mut self) {
        let auto_id = self.get_or_create_section_id_by_name("自动 (Auto)");
        let manual_id = self.get_or_create_section_id_by_name("手动 (Manual)");

        let existing: std::collections::HashSet<Uuid> =
            self.sections.iter().map(|s| s.id).collect();

        for item in &mut self.items {
            let Some(section_id) = item.section_id else {
                continue;
            };
            if existing.contains(&section_id) {
                continue;
            }
            item.section_id = Some(if item.is_automated { auto_id } else { manual_id });
        }
    }

    pub fn get_save_folder_path(&self) -> Option<std::path::PathBuf> {
        if let Some(folder) = &self.save_folder {
            return Some(std::path::PathBuf::from(folder));
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let base = dirs::data_local_dir().or_else(dirs::config_dir)?;
            return Some(base.join("eframe_template").join("todo"));
        }

        #[cfg(target_arch = "wasm32")]
        {
            None
        }
    }

    pub fn get_save_path(&self) -> Option<std::path::PathBuf> {
        self.get_save_folder_path()
            .map(|folder| folder.join("todos.json"))
    }

    pub fn load_from_file(&mut self) {
        if let Some(path) = self.get_save_path() {
            if path.exists() {
                let storage_res = File::open(&path).ok().and_then(|f| {
                    serde_json::from_reader::<_, TodoStorage>(BufReader::new(f)).ok()
                });

                if let Some(storage) = storage_res {
                    self.items = storage.items;
                    self.sections = storage.sections;
                    self.folders = storage.folders;
                    self.settings = storage.settings;
                    self.planner = storage.planner;
                } else {
                    let items_res = File::open(&path).ok().and_then(|f| {
                        serde_json::from_reader::<_, Vec<TodoItem>>(BufReader::new(f)).ok()
                    });

                    if let Some(items) = items_res {
                        self.items = items;
                    } else {
                        self.error_msg = Some(
                            "无法解析本地数据文件 (Failed to parse local data file)".to_string(),
                        );
                    }
                }
            } else if self.save_folder.is_some() {
                self.reset_persistent_state_for_storage_switch();
            }
        }
        self.ensure_builtin_sections_and_settings();
        self.migrate_items_without_section();
        self.migrate_items_without_updated_at();
        self.migrate_items_with_invalid_section();
        self.initial_loaded = true;
    }

    pub fn save_to_file(&self) {
        if let Some(path) = self.get_save_path() {
            if let Some(folder) = path.parent() {
                let _ = std::fs::create_dir_all(folder);
            }

            let storage = TodoStorage {
                schema_version: 1,
                items: self.items.clone(),
                sections: self.sections.clone(),
                folders: self.folders.clone(),
                settings: self.settings.clone(),
                planner: self.planner.clone(),
            };

            let Some(folder) = path.parent() else {
                return;
            };
            let Ok(mut tmp) = tempfile::NamedTempFile::new_in(folder) else {
                return;
            };
            {
                let mut writer = BufWriter::new(&mut tmp);
                if serde_json::to_writer_pretty(&mut writer, &storage).is_err() {
                    return;
                }
                if writer.write_all(b"\n").is_err() {
                    return;
                }
                if writer.flush().is_err() {
                    return;
                }
            }
            let _ = tmp.persist(path);
        }
        super::reminders::write_reminders_ics(self);
    }

    pub fn export_jsonl(&mut self, only_trash: bool) -> Option<std::path::PathBuf> {
        #[derive(Serialize)]
        struct ExportItem<'a> {
            id: uuid::Uuid,
            title: &'a str,
            completed: bool,
            created_at: &'a str,
            section: &'a str,
            priority: u8,
            due_at: Option<chrono::DateTime<chrono::Utc>>,
            reminder_at: Option<chrono::DateTime<chrono::Utc>>,
            deleted: bool,
            deleted_at: Option<chrono::DateTime<chrono::Utc>>,
            is_automated: bool,
            automated_source: Option<&'a str>,
            description: Option<&'a str>,
        }

        let folder = self.get_save_folder_path()?;
        let _ = std::fs::create_dir_all(&folder);

        let file_name = if only_trash {
            "todos.trash.export.jsonl"
        } else {
            "todos.export.jsonl"
        };
        let path = folder.join(file_name);

        let Ok(file) = File::create(&path) else {
            self.show_snackbar("导出失败");
            return None;
        };
        let mut writer = BufWriter::new(file);

        let manual_id = self.get_or_create_section_id_by_name("手动 (Manual)");
        for item in &self.items {
            let deleted = item.deleted_at.is_some();
            if only_trash {
                if !deleted {
                    continue;
                }
            } else if deleted {
                continue;
            }

            let section_id = item.section_id.unwrap_or(manual_id);
            let section = self
                .section_name(section_id)
                .unwrap_or("未知分区 (Unknown)");
            let export = ExportItem {
                id: item.id,
                title: item.title.as_str(),
                completed: item.completed,
                created_at: item.created_at.as_str(),
                section,
                priority: item.priority.min(3),
                due_at: item.due_at,
                reminder_at: item.reminder_at,
                deleted,
                deleted_at: item.deleted_at,
                is_automated: item.is_automated,
                automated_source: item.automated_source.as_deref(),
                description: item.description.as_deref(),
            };

            if serde_json::to_writer(&mut writer, &export).is_err() {
                self.show_snackbar("导出失败");
                return None;
            }
            if writer.write_all(b"\n").is_err() {
                self.show_snackbar("导出失败");
                return None;
            }
        }

        if writer.flush().is_err() {
            self.show_snackbar("导出失败");
            return None;
        }

        Some(path)
    }

    pub fn start_import_from_file(&mut self, path: std::path::PathBuf) -> bool {
        let Ok(bytes) = std::fs::read(&path) else {
            self.show_snackbar("导入失败：无法读取文件");
            return false;
        };

        let incoming_hash = hash_bytes(&bytes);
        if self.settings.last_sync_hash.as_deref() == Some(incoming_hash.as_str()) {
            self.show_snackbar("该文件已导入过");
            return false;
        }

        let Ok(parsed) = parse_import_bytes(&bytes) else {
            self.show_snackbar("导入失败：无法解析文件");
            return false;
        };

        let mut items = parsed.items;
        for item in &mut items {
            item.ensure_updated_at();
        }

        let baseline_time = self.settings.last_sync_time;
        let mut conflicts: Vec<ImportConflict> = Vec::new();

        let mut local_index: std::collections::HashMap<Uuid, usize> =
            std::collections::HashMap::new();
        for (idx, item) in self.items.iter().enumerate() {
            local_index.insert(item.id, idx);
        }

        for incoming in &items {
            let Some(&idx) = local_index.get(&incoming.id) else {
                continue;
            };
            let local = &self.items[idx];
            let Some(baseline) = baseline_time else {
                continue;
            };
            if local.updated_at_utc() > baseline && incoming.updated_at_utc() > baseline {
                let choice = if incoming.updated_at_utc() >= local.updated_at_utc() {
                    ImportConflictChoice::UseIncoming
                } else {
                    ImportConflictChoice::UseLocal
                };
                conflicts.push(ImportConflict {
                    id: incoming.id,
                    local: local.clone(),
                    incoming: incoming.clone(),
                    choice,
                });
            }
        }

        let pending = PendingImport {
            incoming_hash,
            baseline_time,
            items,
            sections: parsed.sections,
            folders: parsed.folders,
            conflicts,
        };

        if self.import_manual_conflicts && !pending.conflicts.is_empty() {
            self.pending_import = Some(pending);
            return false;
        }

        self.apply_import(pending);
        true
    }

    pub fn apply_pending_import(&mut self) -> bool {
        let Some(pending) = self.pending_import.take() else {
            return false;
        };
        self.apply_import(pending);
        true
    }

    pub fn choose_latest_for_all_conflicts(&mut self) {
        let Some(pending) = &mut self.pending_import else {
            return;
        };
        for c in &mut pending.conflicts {
            c.choice = if c.incoming.updated_at_utc() >= c.local.updated_at_utc() {
                ImportConflictChoice::UseIncoming
            } else {
                ImportConflictChoice::UseLocal
            };
        }
    }

    fn apply_import(&mut self, mut pending: PendingImport) {
        for s in pending.sections.drain(..) {
            if !self.sections.iter().any(|x| x.id == s.id) {
                self.sections.push(s);
            }
        }
        for f in pending.folders.drain(..) {
            if !self.folders.iter().any(|x| x.id == f.id) {
                self.folders.push(f);
            }
        }

        let mut conflict_by_id: std::collections::HashMap<Uuid, ImportConflictChoice> =
            std::collections::HashMap::new();
        for c in &pending.conflicts {
            conflict_by_id.insert(c.id, c.choice);
        }

        let mut local_index: std::collections::HashMap<Uuid, usize> =
            std::collections::HashMap::new();
        for (idx, item) in self.items.iter().enumerate() {
            local_index.insert(item.id, idx);
        }

        let baseline_time = pending.baseline_time;

        for mut incoming in pending.items.drain(..) {
            incoming.ensure_updated_at();

            let Some(&idx) = local_index.get(&incoming.id) else {
                self.items.push(incoming);
                continue;
            };

            let local = self.items.get(idx).cloned();
            let Some(local_item) = local else {
                continue;
            };

            if let Some(choice) = conflict_by_id.get(&incoming.id).copied() {
                match choice {
                    ImportConflictChoice::UseLocal => {}
                    ImportConflictChoice::UseIncoming => {
                        self.items[idx] = incoming;
                    }
                    ImportConflictChoice::DuplicateIncoming => {
                        let mut dup = incoming;
                        dup.id = Uuid::new_v4();
                        self.items.push(dup);
                    }
                }
                continue;
            }

            let replace = if let Some(baseline) = baseline_time {
                !(local_item.updated_at_utc() > baseline && incoming.updated_at_utc() > baseline)
                    && incoming.updated_at_utc() > local_item.updated_at_utc()
            } else {
                incoming.updated_at_utc() > local_item.updated_at_utc()
            };

            if replace {
                self.items[idx] = incoming;
            }
        }

        self.ensure_builtin_sections_and_settings();
        self.migrate_items_without_section();
        self.migrate_items_without_updated_at();
        self.migrate_items_with_invalid_section();

        self.settings.last_sync_time = Some(chrono::Utc::now());
        self.settings.last_sync_hash = Some(pending.incoming_hash);
        self.show_snackbar("导入完成");
    }

    pub fn poll_reminders_and_persist_if_needed(&mut self) {
        if super::reminders::poll_due_reminders_and_notify(self) {
            self.save_to_file();
        }
    }

    pub fn get_today_automated_count(&self) -> usize {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        self.items
            .iter()
            .filter(|item| {
                item.is_automated && item.completed && item.created_at.starts_with(&today)
            })
            .count()
    }

    pub fn has_today_automated_task(&self, source: &str) -> bool {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        self.items.iter().any(|item| {
            item.is_automated
                && item.completed
                && item.created_at.starts_with(&today)
                && item.automated_source.as_deref() == Some(source)
        })
    }
}

struct ParsedImport {
    items: Vec<TodoItem>,
    sections: Vec<TodoSection>,
    folders: Vec<TodoFolder>,
}

fn hash_bytes(bytes: &[u8]) -> String {
    use std::hash::{Hash as _, Hasher as _};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn parse_import_bytes(bytes: &[u8]) -> Result<ParsedImport, ()> {
    if let Ok(storage) = serde_json::from_slice::<TodoStorage>(bytes) {
        return Ok(ParsedImport {
            items: storage.items,
            sections: storage.sections,
            folders: storage.folders,
        });
    }

    if let Ok(items) = serde_json::from_slice::<Vec<TodoItem>>(bytes) {
        return Ok(ParsedImport {
            items,
            sections: Vec::new(),
            folders: Vec::new(),
        });
    }

    #[derive(Deserialize)]
    struct JsonlItem {
        id: uuid::Uuid,
        title: String,
        completed: bool,
        created_at: String,
        section: String,
        priority: u8,
        due_at: Option<chrono::DateTime<chrono::Utc>>,
        reminder_at: Option<chrono::DateTime<chrono::Utc>>,
        deleted: bool,
        deleted_at: Option<chrono::DateTime<chrono::Utc>>,
        is_automated: bool,
        automated_source: Option<String>,
        description: Option<String>,
    }

    let text = std::str::from_utf8(bytes).map_err(|_| ())?;
    let mut sections: Vec<TodoSection> = Vec::new();
    let mut section_map: std::collections::HashMap<String, uuid::Uuid> =
        std::collections::HashMap::new();
    let mut items: Vec<TodoItem> = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(row) = serde_json::from_str::<JsonlItem>(line) else {
            return Err(());
        };

        let section_id = if let Some(&id) = section_map.get(&row.section) {
            id
        } else {
            let section = TodoSection::new(row.section.clone());
            let id = section.id;
            sections.push(section);
            section_map.insert(row.section.clone(), id);
            id
        };

        let deleted_at = if row.deleted { row.deleted_at } else { None };
        let reminder_sent = row.completed || row.reminder_at.is_none();

        items.push(TodoItem {
            id: row.id,
            title: row.title,
            completed: row.completed,
            created_at: row.created_at,
            updated_at: None,
            section_id: Some(section_id),
            reminder_at: row.reminder_at,
            reminder_sent,
            reminder_repeat: None,
            due_at: row.due_at,
            priority: row.priority.min(3),
            deleted_at,
            is_automated: row.is_automated,
            automated_source: row.automated_source,
            description: row.description,
            tags: Vec::new(),
        });
    }

    Ok(ParsedImport {
        items,
        sections,
        folders: Vec::new(),
    })
}
