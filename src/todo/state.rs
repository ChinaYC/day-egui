use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::model::{TodoItem, TodoSection, TodoSettings, TodoStorage};

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct TodoState {
    pub items: Vec<TodoItem>,
    pub sections: Vec<TodoSection>,
    pub settings: TodoSettings,
    pub new_task_title: String,
    pub new_task_description: String,

    pub save_folder: Option<String>,

    #[serde(skip)]
    pub item_to_delete: Option<Uuid>,
    #[serde(skip)]
    pub dragging_item: Option<Uuid>,
    #[serde(skip)]
    pub dragging_section: Option<Uuid>,
    #[serde(skip)]
    pub drag_target_index: Option<usize>,

    #[serde(skip)]
    pub active_section: Option<Uuid>,
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
    pub last_font_scale: f32,

    #[serde(skip)]
    pub initial_loaded: bool,
    #[serde(skip)]
    pub error_msg: Option<String>,
}

impl Default for TodoState {
    fn default() -> Self {
        let mut state = Self {
            items: Vec::new(),
            sections: Vec::new(),
            settings: TodoSettings::default(),
            new_task_title: String::new(),
            new_task_description: String::new(),
            save_folder: None,
            item_to_delete: None,
            dragging_item: None,
            dragging_section: None,
            drag_target_index: None,
            active_section: None,
            new_section_name: String::new(),
            new_task_section: None,
            show_settings: false,
            editing_reminder: None,
            reminder_input: String::new(),
            reminder_error_msg: None,
            section_to_rename: None,
            section_rename_input: String::new(),
            section_to_delete: None,
            section_delete_move_to: None,
            section_manage_error_msg: None,
            last_font_scale: 1.0,
            initial_loaded: false,
            error_msg: None,
        };
        state.ensure_builtin_sections_and_settings();
        state
    }
}

impl TodoState {
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

    pub fn section_name(&self, section_id: Uuid) -> Option<&str> {
        self.sections
            .iter()
            .find(|s| s.id == section_id)
            .map(|s| s.name.as_str())
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
                item.section_id = Some(if item.is_automated { auto_id } else { manual_id });
            }
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
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(storage) = serde_json::from_str::<TodoStorage>(&content) {
                        self.items = storage.items;
                        self.sections = storage.sections;
                        self.settings = storage.settings;
                    } else if let Ok(items) = serde_json::from_str::<Vec<TodoItem>>(&content) {
                        self.items = items;
                    } else {
                        self.error_msg = Some(
                            "无法解析本地数据文件 (Failed to parse local data file)".to_string(),
                        );
                    }
                } else {
                    self.error_msg =
                        Some("无法读取本地数据文件 (Failed to read local data file)".to_string());
                }
            }
        }
        self.ensure_builtin_sections_and_settings();
        self.migrate_items_without_section();
        self.initial_loaded = true;
    }

    pub fn save_to_file(&self) {
        if let Some(path) = self.get_save_path() {
            if let Some(folder) = path.parent() {
                let _ = std::fs::create_dir_all(folder);
            }

            let storage = TodoStorage {
                items: self.items.clone(),
                sections: self.sections.clone(),
                settings: self.settings.clone(),
            };

            if let Ok(content) = serde_json::to_string_pretty(&storage) {
                if let Err(e) = std::fs::write(&path, content) {
                    println!("Failed to save todo items: {}", e);
                }
            }
        }
        super::reminders::write_reminders_ics(self);
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
            .filter(|item| item.is_automated && item.completed && item.created_at.starts_with(&today))
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
