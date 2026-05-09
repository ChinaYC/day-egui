use egui::{Color32, CornerRadius, Margin, Pos2, Rect, Stroke, Vec2};

use super::super::model::{PlannerEntry, TodoPlanner};
use super::super::TodoState;

pub fn show(state: &mut TodoState, ui: &mut egui::Ui, state_changed: &mut bool) {
    normalize_planner(&mut state.planner);

    let bg = Color32::from_rgb(232, 245, 236);
    let border = Color32::from_rgb(128, 185, 145);
    let accent = Color32::from_rgb(28, 116, 62);
    let soft = Color32::from_rgb(215, 236, 223);

    egui::Frame::NONE
        .fill(bg)
        .stroke(Stroke::new(2.0, border))
        .corner_radius(CornerRadius::same(18))
        .inner_margin(Margin::same(18))
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("planner_scroll")
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("⏳ 把大任务拆成小目标，按重要程度逐个完成")
                                .strong()
                                .size(20.0)
                                .color(accent),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("填入今天日期").clicked() {
                                state.planner.date =
                                    chrono::Local::now().format("%Y / %m / %d").to_string();
                                *state_changed = true;
                            }
                            if ui.small_button("清空本周").clicked() {
                                let date = state.planner.date.clone();
                                state.planner = TodoPlanner::default();
                                state.planner.date = date;
                                *state_changed = true;
                            }
                        });
                    });

                    ui.add_space(12.0);

                    let long_term_resp = section_frame(ui, soft, border, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("✏ 我的长期目标")
                                    .strong()
                                    .size(18.0)
                                    .color(accent),
                            );
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new("日期：")
                                            .strong()
                                            .color(accent),
                                    );
                                    let r = ui.add(
                                        egui::TextEdit::singleline(&mut state.planner.date)
                                            .hint_text("**** / ** / **")
                                            .desired_width(140.0),
                                    );
                                    if r.changed() {
                                        *state_changed = true;
                                    }
                                });
                            });
                        });
                        ui.add_space(8.0);

                        for i in 0..3 {
                            ui.horizontal(|ui| {
                                number_badge(ui, i + 1, accent, border);
                                let r = ui.add(
                                    egui::TextEdit::singleline(&mut state.planner.long_term[i].text)
                                        .desired_width(ui.available_width()),
                                );
                                if r.changed() {
                                    *state_changed = true;
                                }
                            });
                            ui.add_space(6.0);
                        }
                    });
                    dashed_border(ui, long_term_resp, border);

                    ui.add_space(12.0);

                    let weekly_resp = section_frame(ui, soft, border, |ui| {
                        ui.label(
                            egui::RichText::new("🧾 本周目标")
                                .strong()
                                .size(18.0)
                                .color(accent),
                        );
                        ui.add_space(8.0);
                        for i in 0..3 {
                            ui.horizontal(|ui| {
                                ring_badge(ui, accent, border);
                                let r = ui.add(
                                    egui::TextEdit::singleline(&mut state.planner.weekly[i].text)
                                        .desired_width(ui.available_width()),
                                );
                                if r.changed() {
                                    *state_changed = true;
                                }
                            });
                            ui.add_space(6.0);
                        }
                    });
                    dashed_border(ui, weekly_resp, border);

                    ui.add_space(16.0);

                    let width = ui.available_width();
                    let cols = if width >= 980.0 {
                        3
                    } else if width >= 660.0 {
                        2
                    } else {
                        1
                    };

                    render_days(ui, cols, soft, border, accent, state, state_changed);

                    ui.add_space(14.0);
                    ui.separator();
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new(
                            "回顾一周，有汗水更有进步。为自己的坚持点赞，下周继续加油！",
                        )
                        .strong()
                        .color(accent),
                    );
                });
        });
}

fn normalize_planner(planner: &mut TodoPlanner) {
    ensure_len_entries(&mut planner.long_term, 3);
    ensure_len_entries(&mut planner.weekly, 3);
    if planner.daily.len() < 7 {
        while planner.daily.len() < 7 {
            planner.daily.push(vec![
                PlannerEntry::default(),
                PlannerEntry::default(),
                PlannerEntry::default(),
            ]);
        }
    } else if planner.daily.len() > 7 {
        planner.daily.truncate(7);
    }
    for day in &mut planner.daily {
        ensure_len_entries(day, 3);
    }
}

fn ensure_len_entries(list: &mut Vec<PlannerEntry>, len: usize) {
    if list.len() < len {
        while list.len() < len {
            list.push(PlannerEntry::default());
        }
    } else if list.len() > len {
        list.truncate(len);
    }
}

fn section_frame<R>(
    ui: &mut egui::Ui,
    fill: Color32,
    stroke: Color32,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::Response {
    egui::Frame::NONE
        .fill(fill)
        .stroke(Stroke::new(1.0, stroke))
        .corner_radius(CornerRadius::same(14))
        .inner_margin(Margin::same(14))
        .show(ui, add_contents)
        .response
}

fn dashed_border(ui: &egui::Ui, rect: egui::Response, color: Color32) {
    let stroke = Stroke::new(1.0, color);
    dashed_round_rect(ui.painter(), rect.rect, 14.0, stroke, 8.0, 6.0);
}

fn dashed_round_rect(
    painter: &egui::Painter,
    rect: Rect,
    rounding: f32,
    stroke: Stroke,
    dash: f32,
    gap: f32,
) {
    let inner = rect.shrink(3.0);
    let left = inner.left() + rounding;
    let right = inner.right() - rounding;
    let top = inner.top() + rounding;
    let bottom = inner.bottom() - rounding;

    dashed_line(
        painter,
        Pos2::new(left, inner.top()),
        Pos2::new(right, inner.top()),
        stroke,
        dash,
        gap,
    );
    dashed_line(
        painter,
        Pos2::new(left, inner.bottom()),
        Pos2::new(right, inner.bottom()),
        stroke,
        dash,
        gap,
    );
    dashed_line(
        painter,
        Pos2::new(inner.left(), top),
        Pos2::new(inner.left(), bottom),
        stroke,
        dash,
        gap,
    );
    dashed_line(
        painter,
        Pos2::new(inner.right(), top),
        Pos2::new(inner.right(), bottom),
        stroke,
        dash,
        gap,
    );
}

fn dashed_line(
    painter: &egui::Painter,
    from: Pos2,
    to: Pos2,
    stroke: Stroke,
    dash: f32,
    gap: f32,
) {
    let delta = to - from;
    let len = delta.length();
    if len <= 0.01 {
        return;
    }
    let dir = delta / len;
    let step = dash + gap;
    let mut t = 0.0;
    while t < len {
        let a = from + dir * t;
        let b = from + dir * (t + dash).min(len);
        painter.line_segment([a, b], stroke);
        t += step;
    }
}

fn number_badge(ui: &mut egui::Ui, n: usize, text: Color32, stroke: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(22.0, 22.0), egui::Sense::hover());
    ui.painter()
        .circle_filled(rect.center(), 10.0, Color32::from_rgb(245, 252, 247));
    ui.painter()
        .circle_stroke(rect.center(), 10.0, Stroke::new(1.5, stroke));
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        format!("{n}"),
        egui::FontId::proportional(14.0),
        text,
    );
}

fn ring_badge(ui: &mut egui::Ui, text: Color32, stroke: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(22.0, 22.0), egui::Sense::hover());
    ui.painter()
        .circle_stroke(rect.center(), 10.0, Stroke::new(2.0, stroke));
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "◌",
        egui::FontId::proportional(14.0),
        text,
    );
}

fn render_days(
    ui: &mut egui::Ui,
    cols: usize,
    fill: Color32,
    stroke: Color32,
    accent: Color32,
    state: &mut TodoState,
    state_changed: &mut bool,
) {
    let labels = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"];

    let mut blocks: Vec<DayBlock<'_>> = Vec::new();
    for (idx, label) in labels.iter().enumerate() {
        blocks.push(DayBlock::Day { index: idx, title: *label });
    }
    blocks.push(DayBlock::Reward);
    blocks.push(DayBlock::Notes);

    let mut row = 0usize;
    while row * cols < blocks.len() {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;
            for c in 0..cols {
                let i = row * cols + c;
                if i >= blocks.len() {
                    break;
                }
                let w = (ui.available_width() - 12.0 * (cols.saturating_sub(1) as f32))
                    / cols as f32;
                ui.set_min_width(w);
                match blocks[i] {
                    DayBlock::Day { index, title } => {
                        let resp = day_card(ui, fill, stroke, accent, title, |ui| {
                            for j in 0..3 {
                                let entry = &mut state.planner.daily[index][j];
                                ui.horizontal(|ui| {
                                    let mut done = entry.done;
                                    let r = ui.checkbox(&mut done, "");
                                    if r.changed() {
                                        entry.done = done;
                                        *state_changed = true;
                                    }
                                    let r = ui.add(
                                        egui::TextEdit::singleline(&mut entry.text)
                                            .desired_width(ui.available_width()),
                                    );
                                    if r.changed() {
                                        *state_changed = true;
                                    }
                                });
                                ui.add_space(4.0);
                            }
                        });
                        dashed_border(ui, resp, stroke);
                    }
                    DayBlock::Reward => {
                        let resp = day_card(ui, fill, stroke, accent, "🧩 奖励", |ui| {
                            let r = ui.add(
                                egui::TextEdit::multiline(&mut state.planner.reward)
                                    .desired_width(ui.available_width())
                                    .desired_rows(5),
                            );
                            if r.changed() {
                                *state_changed = true;
                            }
                        });
                        dashed_border(ui, resp, stroke);
                    }
                    DayBlock::Notes => {
                        let resp = day_card(ui, fill, stroke, accent, "🗒 备注", |ui| {
                            let r = ui.add(
                                egui::TextEdit::multiline(&mut state.planner.notes)
                                    .desired_width(ui.available_width())
                                    .desired_rows(5),
                            );
                            if r.changed() {
                                *state_changed = true;
                            }
                        });
                        dashed_border(ui, resp, stroke);
                    }
                }
            }
        });
        ui.add_space(12.0);
        row += 1;
    }
}

fn day_card<R>(
    ui: &mut egui::Ui,
    fill: Color32,
    stroke: Color32,
    accent: Color32,
    title: &str,
    add: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::Response {
    egui::Frame::NONE
        .fill(fill)
        .stroke(Stroke::new(1.0, stroke))
        .corner_radius(CornerRadius::same(14))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(title).strong().color(accent));
            ui.add_space(8.0);
            add(ui)
        })
        .response
}

#[derive(Clone, Copy)]
enum DayBlock<'a> {
    Day { index: usize, title: &'a str },
    Reward,
    Notes,
}
