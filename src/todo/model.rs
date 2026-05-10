use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ReminderRepeat {
    Weekly { weekday: u8, hour: u8, minute: u8 },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TodoItem {
    pub id: Uuid,
    pub title: String,
    pub completed: bool,
    pub created_at: String,

    // 分区：任务属于哪个分区（分区本身在 TodoState.sections 里定义）
    // 用 Option 兼容旧版本数据：旧文件里没有这个字段时会走默认值 None，再在加载后做迁移补全。
    #[serde(default)]
    pub section_id: Option<Uuid>,

    // 提醒：到点后触发系统通知/日历提醒
    // 这里用 Utc 存储，跨时区/跨设备更稳定；UI 展示时再按本地时区格式化。
    #[serde(default)]
    pub reminder_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub reminder_sent: bool,
    #[serde(default)]
    pub reminder_repeat: Option<ReminderRepeat>,

    #[serde(default)]
    pub due_at: Option<DateTime<Utc>>,

    // 优先级：0(P0最高) ~ 3(P3最低)
    #[serde(default = "default_priority")]
    pub priority: u8,

    // 回收站：软删除后会带上删除时间；列表默认隐藏，回收站视图可恢复/彻底删除。
    #[serde(default)]
    pub deleted_at: Option<DateTime<Utc>>,

    #[serde(default)]
    pub is_automated: bool,
    #[serde(default)]
    pub automated_source: Option<String>,
    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub tags: Vec<String>,
}

impl TodoItem {
    pub fn new(title: String, description: Option<String>, section_id: Option<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            completed: false,
            created_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            section_id,
            reminder_at: None,
            reminder_sent: false,
            reminder_repeat: None,
            due_at: None,
            priority: default_priority(),
            deleted_at: None,
            is_automated: false,
            automated_source: None,
            description,
            tags: Vec::new(),
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
            reminder_at: None,
            reminder_sent: false,
            reminder_repeat: None,
            due_at: None,
            priority: 3,
            deleted_at: None,
            is_automated: true,
            automated_source: Some(source),
            description,
            tags: Vec::new(),
        }
    }

    pub fn reminder_at_local_string(&self) -> Option<String> {
        self.reminder_at.map(|utc| {
            utc.with_timezone(&Local)
                .format("%Y-%m-%d %H:%M")
                .to_string()
        })
    }

    pub fn reminder_display_string(&self) -> Option<String> {
        if let Some(rule) = self.reminder_repeat {
            match rule {
                ReminderRepeat::Weekly {
                    weekday,
                    hour,
                    minute,
                } => {
                    let day = match weekday {
                        0 => "周一",
                        1 => "周二",
                        2 => "周三",
                        3 => "周四",
                        4 => "周五",
                        5 => "周六",
                        _ => "周日",
                    };
                    return Some(format!("每{day} {:02}:{:02}", hour, minute));
                }
            }
        }
        self.reminder_at_local_string()
    }

    pub fn due_at_local_string(&self) -> Option<String> {
        self.due_at
            .map(|utc| utc.with_timezone(&Local).format("%Y-%m-%d").to_string())
    }

    pub fn priority_label(&self) -> &'static str {
        match self.priority.min(3) {
            0 => "P0",
            1 => "P1",
            2 => "P2",
            _ => "P3",
        }
    }
}

fn default_priority() -> u8 {
    2
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TodoSection {
    pub id: Uuid,
    pub name: String,

    #[serde(default)]
    pub folder_id: Option<Uuid>,
}

impl TodoSection {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            folder_id: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TodoFolder {
    pub id: Uuid,
    pub name: String,
}

impl TodoFolder {
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
    // 字体大小倍率（1.0 = 默认），用于提升可读性。
    #[serde(default = "default_font_scale")]
    pub font_scale: f32,
}

impl Default for TodoSettings {
    fn default() -> Self {
        Self {
            automated_section_id: None,
            font_scale: default_font_scale(),
        }
    }
}

fn default_font_scale() -> f32 {
    1.0
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TodoStorage {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub items: Vec<TodoItem>,
    pub sections: Vec<TodoSection>,
    #[serde(default)]
    pub folders: Vec<TodoFolder>,
    pub settings: TodoSettings,
    pub planner: TodoPlanner,
}

fn default_schema_version() -> u32 {
    1
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct PlannerEntry {
    pub text: String,
    pub done: bool,
}

impl Default for PlannerEntry {
    fn default() -> Self {
        Self {
            text: String::new(),
            done: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct TodoPlanner {
    pub date: String,
    pub long_term: Vec<PlannerEntry>,
    pub weekly: Vec<PlannerEntry>,
    pub daily: Vec<Vec<PlannerEntry>>,
    pub reward: String,
    pub notes: String,
}

impl Default for TodoPlanner {
    fn default() -> Self {
        let mut daily: Vec<Vec<PlannerEntry>> = Vec::new();
        for _ in 0..7 {
            daily.push(vec![
                PlannerEntry::default(),
                PlannerEntry::default(),
                PlannerEntry::default(),
            ]);
        }
        Self {
            date: String::new(),
            long_term: vec![
                PlannerEntry::default(),
                PlannerEntry::default(),
                PlannerEntry::default(),
            ],
            weekly: vec![
                PlannerEntry::default(),
                PlannerEntry::default(),
                PlannerEntry::default(),
            ],
            daily,
            reward: String::new(),
            notes: String::new(),
        }
    }
}
