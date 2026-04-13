use chrono::Local;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct TodoItem {
    pub id: Uuid,
    pub title: String,
    pub completed: bool,
    pub created_at: String,
    #[serde(default)]
    pub is_automated: bool,
    #[serde(default)]
    pub automated_source: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

impl TodoItem {
    pub fn new(title: String, description: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            completed: false,
            created_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            is_automated: false,
            automated_source: None,
            description,
        }
    }

    pub fn new_automated(title: String, source: String, description: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            completed: true,
            created_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            is_automated: true,
            automated_source: Some(source),
            description,
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TodoState {
    pub items: Vec<TodoItem>,
    pub new_task_title: String,
    pub new_task_description: String,
    
    // 自定义保存路径
    pub save_folder: Option<String>,
    
    #[serde(skip)]
    pub item_to_delete: Option<usize>,
    
    #[serde(skip)]
    pub initial_loaded: bool,
    
    #[serde(skip)]
    pub error_msg: Option<String>,
}

impl TodoState {
    pub fn get_save_path(&self) -> Option<std::path::PathBuf> {
        self.save_folder.as_ref().map(|f| std::path::Path::new(f).join("todos.json"))
    }

    pub fn load_from_file(&mut self) {
        if let Some(path) = self.get_save_path() {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(items) = serde_json::from_str(&content) {
                        self.items = items;
                    } else {
                        self.error_msg = Some("无法解析本地数据文件 (Failed to parse local data file)".to_string());
                    }
                } else {
                    self.error_msg = Some("无法读取本地数据文件 (Failed to read local data file)".to_string());
                }
            }
        }
        self.initial_loaded = true;
    }

    pub fn save_to_file(&self) {
        if let Some(path) = self.get_save_path() {
            if let Ok(content) = serde_json::to_string_pretty(&self.items) {
                if let Err(e) = std::fs::write(path, content) {
                    println!("Failed to save todo items: {}", e);
                }
            }
        }
    }

    pub fn get_today_automated_count(&self) -> usize {
        let today = Local::now().format("%Y-%m-%d").to_string();
        self.items.iter()
            .filter(|item| item.is_automated && item.completed && item.created_at.starts_with(&today))
            .count()
    }

    pub fn has_today_automated_task(&self, source: &str) -> bool {
        let today = Local::now().format("%Y-%m-%d").to_string();
        self.items.iter()
            .any(|item| item.is_automated && item.completed && item.created_at.starts_with(&today) && item.automated_source.as_deref() == Some(source))
    }
}
