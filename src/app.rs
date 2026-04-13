use crate::leetcode::LeetCodeState;
use crate::todo::TodoState;

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Copy)]
pub enum AppRoute {
    LeetCode,
    Todo,
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
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            route: AppRoute::default(),
            leetcode_state: LeetCodeState::default(),
            todo_state: TodoState::default(),
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
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
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

        // Sync background tasks and update Todo list
        if let Some((task_title, source, description)) = self.leetcode_state.sync_background_state() {
            use crate::todo::TodoItem;
            
            if !self.todo_state.has_today_automated_task(&source) {
                self.todo_state.items.push(TodoItem::new_automated(task_title, source, description));
                self.todo_state.save_to_file();
            }
        }

        // Apply some styling inspired by Cupertino
        let mut style = (*ui.ctx().global_style()).clone();
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.window_margin = egui::Margin::same(12);
        style.visuals.window_corner_radius = egui::CornerRadius::same(8);
        ui.ctx().set_global_style(style);

        egui::Panel::left("left_panel")
            .resizable(false)
            .exact_size(200.0)
            .show_inside(ui, |ui| {
                ui.heading("⚡ 效率工具");
                ui.add_space(20.0);

                ui.vertical_centered_justified(|ui| {
                    if ui.selectable_label(self.route == AppRoute::LeetCode, "LeetCode 刷题").clicked() {
                        self.route = AppRoute::LeetCode;
                    }
                    ui.add_space(8.0);
                    if ui.selectable_label(self.route == AppRoute::Todo, "Todo 清单").clicked() {
                        self.route = AppRoute::Todo;
                    }
                });

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.label("v0.1.0 Offline");
                    });
                });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            match self.route {
                AppRoute::LeetCode => {
                    self.leetcode_state.ui(ui, &self.todo_state);
                }
                AppRoute::Todo => {
                    self.todo_state.ui(ui);
                }
            }
        });
    }
}
