use crate::fitness::FitnessState;
use crate::leetcode::LeetCodeState;
use crate::todo::{ThemeMode, TodoState};

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Copy)]
pub enum AppRoute {
    LeetCode,
    Todo,
    Fitness,
}

impl Default for AppRoute {
    fn default() -> Self {
        Self::LeetCode
    }
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    route: AppRoute,
    leetcode_state: LeetCodeState,
    todo_state: TodoState,
    fitness_state: FitnessState,
    #[serde(skip)]
    system_visuals: Option<egui::Visuals>,
    show_sidebar: bool,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            route: AppRoute::default(),
            leetcode_state: LeetCodeState::default(),
            todo_state: TodoState::default(),
            fitness_state: FitnessState::default(),
            system_visuals: None,
            show_sidebar: true,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        crate::theme::setup_fonts(&cc.egui_ctx);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        let mut app: Self = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };

        // Initialize Tracing with the logs Arc from leetcode_state
        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        {
            use std::sync::Once;
            static INIT_LOGGER: Once = Once::new();
            let logs = app.leetcode_state.logs.clone();
            INIT_LOGGER.call_once(|| {
                crate::leetcode::automation::logger::init_tracing(logs);
            });
        }

        app.system_visuals = Some(cc.egui_ctx.global_style().visuals.clone());
        app
    }
}

impl eframe::App for TemplateApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Ensure initial load if not loaded yet
        if !self.todo_state.initial_loaded {
            self.todo_state.load_from_file();
        }

        let visuals = match self.todo_state.settings.theme_mode {
            ThemeMode::System => self.system_visuals.clone(),
            ThemeMode::Light => Some(egui::Visuals::light()),
            ThemeMode::Dark => Some(egui::Visuals::dark()),
        };
        if let Some(visuals) = visuals {
            ui.ctx().set_visuals(visuals);
        }

        // 提醒轮询：让应用在空闲时也能“到点触发通知”。
        // eframe/egui 在没有交互时可能降低刷新频率，这里主动请求定时重绘用于检查提醒。
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_secs(1));
        self.todo_state.poll_reminders_and_persist_if_needed();

        // Sync background tasks and update Todo list
        if let Some((task_title, source, description)) = self.leetcode_state.sync_background_state()
        {
            use crate::todo::TodoItem;

            if !self.todo_state.has_today_automated_task(&source) {
                // 自动任务落到哪个分区由 Todo 设置控制；默认是“自动 (Auto)”分区。
                let section_id = Some(self.todo_state.automated_target_section_id());
                self.todo_state.items.push(TodoItem::new_automated(
                    task_title,
                    source,
                    description,
                    section_id,
                ));
                self.todo_state.save_to_file();
            }
        }

        // Apply some styling inspired by Cupertino
        let mut style = (*ui.ctx().global_style()).clone();
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.window_margin = egui::Margin::same(12);
        style.visuals.window_corner_radius = egui::CornerRadius::same(8);

        let font_scale = self.todo_state.settings.font_scale;
        if font_scale.is_finite() && font_scale > 0.0 {
            let ratio = font_scale / self.todo_state.last_font_scale.max(0.01);
            if (ratio - 1.0).abs() > f32::EPSILON {
                for font_id in style.text_styles.values_mut() {
                    font_id.size *= ratio;
                }
                self.todo_state.last_font_scale = font_scale;
            }
        }

        ui.ctx().set_global_style(style);

        let is_mobile = ui.ctx().viewport_rect().width() < 600.0;

        // 开启 egui 开发者调试面板（仅在非移动端显示，或根据需要开启）
        // if !is_mobile {
        //     egui::Window::new("🛠 调试面板 (Debugger)")
        //         .default_pos([ui.ctx().viewport_rect().width() - 350.0, 20.0])
        //         .default_size([300.0, 500.0])
        //         .vscroll(true)
        //         .open(&mut true)
        //         .show(ui.ctx(), |ui| {
        //             ui.ctx().clone().inspection_ui(ui);
        //         });
        // }

        // 顶部导航栏 (提供侧边栏切换按钮)
        egui::Panel::top("top_panel").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("☰").clicked() {
                    self.show_sidebar = !self.show_sidebar;
                }

                if is_mobile || !self.show_sidebar {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let title = self
                            .todo_state
                            .settings
                            .sidebar_items
                            .iter()
                            .find(|item| match item.route_key.as_str() {
                                "leetcode" => self.route == AppRoute::LeetCode,
                                "todo" => self.route == AppRoute::Todo,
                                "fitness" => self.route == AppRoute::Fitness,
                                _ => false,
                            })
                            .map(|item| item.name.as_str())
                            .unwrap_or("⚡ 效率工具");
                        ui.heading(title);
                    });
                }
            });
        });

        // 侧边栏
        if self.show_sidebar {
            egui::Panel::left("left_panel")
                .resizable(false)
                .exact_size(if is_mobile {
                    ui.ctx().viewport_rect().width() * 0.7
                } else {
                    200.0
                })
                .show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("⚡ 效率工具");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("✕").clicked() {
                                self.show_sidebar = false;
                            }
                        });
                    });
                    ui.add_space(20.0);

                    ui.vertical_centered_justified(|ui| {
                        let sidebar_items = self.todo_state.settings.sidebar_items.clone();
                        for item in sidebar_items {
                            if !item.visible {
                                continue;
                            }

                            let target_route = match item.route_key.as_str() {
                                "leetcode" => AppRoute::LeetCode,
                                "todo" => AppRoute::Todo,
                                "fitness" => AppRoute::Fitness,
                                _ => continue,
                            };

                            if ui
                                .selectable_label(self.route == target_route, &item.name)
                                .clicked()
                            {
                                self.route = target_route;
                                if is_mobile {
                                    self.show_sidebar = false;
                                }
                            }
                            ui.add_space(8.0);
                        }
                    });

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            ui.label("v0.1.0 Offline");
                        });
                    });
                });
        }

        egui::CentralPanel::default().show_inside(ui, |ui| match self.route {
            AppRoute::LeetCode => {
                self.leetcode_state.ui(ui, &self.todo_state);
            }
            AppRoute::Todo => {
                self.todo_state.ui(ui);
            }
            AppRoute::Fitness => {
                let mut changed = false;
                crate::fitness::ui::show(
                    &mut self.fitness_state,
                    ui,
                    &mut self.todo_state,
                    &mut changed,
                );
            }
        });
    }
}
