use chrono::Local;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct TodoItem {
    pub id: Uuid,
    pub title: String,
    pub completed: bool,
    pub created_at: String,
    // 任务所属分区（分区本身在 TodoState.sections 里定义）
    // 用 Option 兼容旧版本数据：旧文件里没有这个字段时会走默认值 None，再在加载后做迁移补全。
    #[serde(default)]
    pub section_id: Option<Uuid>,
    #[serde(default)]
    pub is_automated: bool,
    #[serde(default)]
    pub automated_source: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

impl TodoItem {
    pub fn new(title: String, description: Option<String>, section_id: Option<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            completed: false,
            created_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            section_id,
            is_automated: false,
            automated_source: None,
            description,
        }
    }

    pub fn new_automated(
        title: String,
        source: String,
        description: Option<String>,
        section_id: Option<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            completed: true,
            created_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            section_id,
            is_automated: true,
            automated_source: Some(source),
            description,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TodoSection {
    pub id: Uuid,
    pub name: String,
}

impl TodoSection {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct TodoSettings {
    // 自动任务默认落到哪个分区（齿轮设置里可改）
    pub automated_section_id: Option<Uuid>,
}

impl Default for TodoSettings {
    fn default() -> Self {
        Self {
            automated_section_id: None,
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct TodoStorage {
    items: Vec<TodoItem>,
    sections: Vec<TodoSection>,
    settings: TodoSettings,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct TodoState {
    pub items: Vec<TodoItem>,
    pub sections: Vec<TodoSection>,
    pub settings: TodoSettings,
    pub new_task_title: String,
    pub new_task_description: String,
    
    // 自定义保存路径
    pub save_folder: Option<String>,
    
    #[serde(skip)]
    // 删除二次确认：存放“待确认删除”的任务 id（用 Uuid 绑定具体条目，避免列表排序/增删导致误删）
    pub item_to_delete: Option<Uuid>,

    #[serde(skip)]
    // 拖拽排序：当前正在拖拽的任务 id
    pub dragging_item: Option<Uuid>,

    #[serde(skip)]
    // 拖拽排序：当前拖拽发生在哪个分区里（用于 All 视图里分区隔离排序）
    pub dragging_section: Option<Uuid>,

    #[serde(skip)]
    // 拖拽排序：目标插入位置（语义是“插入到这个 index 之前/之后”，由 UI 计算得出）
    pub drag_target_index: Option<usize>,

    #[serde(skip)]
    // UI：当前选中的分区；None 表示“全部”
    pub active_section: Option<Uuid>,

    #[serde(skip)]
    // UI：新建分区的输入框内容
    pub new_section_name: String,

    #[serde(skip)]
    // UI：新增任务时选择的目标分区
    pub new_task_section: Option<Uuid>,

    #[serde(skip)]
    // UI：是否打开设置窗口（由左下角齿轮按钮控制）
    pub show_settings: bool,
    
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
            initial_loaded: false,
            error_msg: None,
        };
        state.ensure_builtin_sections_and_settings();
        state
    }
}

impl TodoState {
    // 内置分区：满足“自动/手动/全部”的需求，其中“全部”是视图，不落盘为分区。
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

    // 兼容旧数据：如果 item.section_id 缺失，就按 is_automated 自动归类到内置分区。
    pub fn migrate_items_without_section(&mut self) {
        let auto_id = self.get_or_create_section_id_by_name("自动 (Auto)");
        let manual_id = self.get_or_create_section_id_by_name("手动 (Manual)");

        for item in &mut self.items {
            if item.section_id.is_none() {
                item.section_id = Some(if item.is_automated { auto_id } else { manual_id });
            }
        }
    }

    pub fn get_save_path(&self) -> Option<std::path::PathBuf> {
        self.save_folder.as_ref().map(|f| std::path::Path::new(f).join("todos.json"))
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
                        self.error_msg = Some("无法解析本地数据文件 (Failed to parse local data file)".to_string());
                    }
                } else {
                    self.error_msg = Some("无法读取本地数据文件 (Failed to read local data file)".to_string());
                }
            }
        }
        self.ensure_builtin_sections_and_settings();
        self.migrate_items_without_section();
        self.initial_loaded = true;
    }

    pub fn save_to_file(&self) {
        if let Some(path) = self.get_save_path() {
            let storage = TodoStorage {
                items: self.items.clone(),
                sections: self.sections.clone(),
                settings: self.settings.clone(),
            };
            if let Ok(content) = serde_json::to_string_pretty(&storage) {
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
