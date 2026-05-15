use chrono::{Duration, NaiveDate, TimeZone, Utc};

use super::FitnessState;
use super::model::{
    DailyWorkoutPlan, DietPlan, FitnessPhase, FitnessPlan, MetricLevel, PlanMetric, Sex,
    TrainingCondition,
};
use crate::fitness::health;
use crate::todo::{TodoItem, TodoState};

pub fn show(
    state: &mut FitnessState,
    ui: &mut egui::Ui,
    todo: &mut TodoState,
    state_changed: &mut bool,
) {
    ui.heading("健身训练 (Fitness)");
    ui.add_space(8.0);

    if state.selected_profile.is_none() {
        state.selected_profile = state.profiles.first().map(|p| p.id);
    }

    ui.columns(2, |cols| {
        cols[0].set_width(240.0);
        show_profiles_panel(state, &mut cols[0], state_changed);
        show_profile_detail(state, &mut cols[1], todo, state_changed);
    });

    if let Some(err) = &state.error_msg {
        ui.add_space(6.0);
        ui.label(egui::RichText::new(err).color(egui::Color32::RED));
    }
}

fn show_profiles_panel(state: &mut FitnessState, ui: &mut egui::Ui, state_changed: &mut bool) {
    ui.label(egui::RichText::new("人员 (Profiles)").strong());
    ui.add_space(6.0);

    ui.horizontal(|ui| {
        ui.text_edit_singleline(&mut state.new_profile_name);
        if ui.button("新增").clicked() {
            let name = state.new_profile_name.trim().to_string();
            if name.is_empty() {
                state.error_msg = Some("姓名不能为空".to_string());
                return;
            }
            let profile = super::model::FitnessProfile::new(name);
            state.selected_profile = Some(profile.id);
            state.profiles.push(profile);
            state.new_profile_name.clear();
            state.error_msg = None;
            *state_changed = true;
        }
    });

    ui.add_space(8.0);
    egui::ScrollArea::vertical()
        .id_salt("fitness_profiles_scroll")
        .show(ui, |ui| {
            let mut to_delete: Option<uuid::Uuid> = None;
            for p in &state.profiles {
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(state.selected_profile == Some(p.id), p.name.clone())
                        .clicked()
                    {
                        state.selected_profile = Some(p.id);
                        state.error_msg = None;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🗑️").clicked() {
                            to_delete = Some(p.id);
                        }
                    });
                });
            }
            if let Some(id) = to_delete {
                state.profiles.retain(|p| p.id != id);
                if state.selected_profile == Some(id) {
                    state.selected_profile = state.profiles.first().map(|p| p.id);
                }
                *state_changed = true;
            }
        });

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);

    egui::CollapsingHeader::new("JSON 导入/导出")
        .id_salt("fitness_json_panel")
        .default_open(false)
        .show(ui, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.button("复制模板").clicked() {
                    let template = super::model::FitnessProfileInput::default();
                    if let Ok(s) = serde_json::to_string_pretty(&template) {
                        ui.ctx().copy_text(s);
                        state.error_msg = None;
                    } else {
                        state.error_msg = Some("模板生成失败".to_string());
                    }
                }

                if ui.button("复制当前资料 JSON").clicked() {
                    let Some(p) = state.selected_profile() else {
                        state.error_msg = Some("请先选择人员".to_string());
                        return;
                    };
                    if let Ok(s) = serde_json::to_string_pretty(&p.input) {
                        ui.ctx().copy_text(s);
                        state.error_msg = None;
                    } else {
                        state.error_msg = Some("JSON 导出失败".to_string());
                    }
                }

                if ui.button("复制当前计划 JSON").clicked() {
                    let Some(p) = state.selected_profile() else {
                        state.error_msg = Some("请先选择人员".to_string());
                        return;
                    };
                    let Some(plan) = &p.last_plan else {
                        state.error_msg = Some("当前人员还没有生成计划".to_string());
                        return;
                    };
                    if let Ok(s) = serde_json::to_string_pretty(plan) {
                        ui.ctx().copy_text(s.clone());
                        state.last_export_json = s;
                        state.error_msg = None;
                    } else {
                        state.error_msg = Some("JSON 导出失败".to_string());
                    }
                }
            });

            ui.add_space(6.0);
            ui.label("粘贴资料 JSON 后点击导入：");
            ui.add(egui::TextEdit::multiline(&mut state.import_json).desired_rows(6));
            ui.horizontal(|ui| {
                if ui.button("导入到当前人员").clicked() {
                    let import_json = state.import_json.clone();
                    let input = match serde_json::from_str::<super::model::FitnessProfileInput>(
                        &import_json,
                    ) {
                        Ok(v) => v,
                        Err(_) => {
                            state.error_msg = Some("JSON 解析失败，请确认格式正确".to_string());
                            return;
                        }
                    };
                    let Some(p) = state.selected_profile_mut() else {
                        state.error_msg = Some("请先选择人员".to_string());
                        return;
                    };
                    p.input = input;
                    state.error_msg = None;
                    *state_changed = true;
                }
                if ui.button("清空").clicked() {
                    state.import_json.clear();
                }
            });
        });
}

fn show_profile_detail(
    state: &mut FitnessState,
    ui: &mut egui::Ui,
    todo: &mut TodoState,
    state_changed: &mut bool,
) {
    ui.horizontal(|ui| {
        ui.label("人员：");
        let selected_name = state
            .selected_profile()
            .map(|p| p.name.as_str())
            .unwrap_or("未选择");
        egui::ComboBox::from_id_salt("fitness_profile_select")
            .selected_text(selected_name)
            .show_ui(ui, |ui| {
                for p in &state.profiles {
                    ui.selectable_value(&mut state.selected_profile, Some(p.id), p.name.clone());
                }
            });
    });
    ui.add_space(8.0);

    let Some(profile_id) = state.selected_profile else {
        ui.label("请选择左侧人员后开始填写。");
        return;
    };
    let Some(profile_index) = state.profiles.iter().position(|p| p.id == profile_id) else {
        ui.label("请选择左侧人员后开始填写。");
        return;
    };
    let mut error: Option<String> = None;

    {
        let profile = &mut state.profiles[profile_index];

        ui.label(egui::RichText::new(format!("资料：{}", profile.name)).strong());
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.label("身高(cm)：");
            ui.add(egui::DragValue::new(profile.input.height_cm.get_or_insert(0.0)).speed(0.5));
            ui.add_space(10.0);
            ui.label("体重(kg)：");
            ui.add(egui::DragValue::new(profile.input.weight_kg.get_or_insert(0.0)).speed(0.5));
        });

        ui.add_space(6.0);
        ui.separator();
        ui.add_space(6.0);

        ui.label(egui::RichText::new("健康指标（当前值 vs 参考）").strong());
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("●").color(egui::Color32::from_rgb(46, 204, 113)));
            ui.label(egui::RichText::new("正常").color(egui::Color32::GRAY));
            ui.add_space(8.0);
            ui.label(egui::RichText::new("●").color(egui::Color32::from_rgb(241, 196, 15)));
            ui.label(egui::RichText::new("注意").color(egui::Color32::GRAY));
            ui.add_space(8.0);
            ui.label(egui::RichText::new("●").color(egui::Color32::from_rgb(231, 76, 60)));
            ui.label(egui::RichText::new("风险").color(egui::Color32::GRAY));
            ui.add_space(8.0);
            ui.label(egui::RichText::new("○").color(egui::Color32::GRAY));
            ui.label(egui::RichText::new("未填写").color(egui::Color32::GRAY));
        });
        let bmi_v = health::bmi(profile.input.height_cm, profile.input.weight_kg);
        let bmi_i = health::bmi_indicator(bmi_v);
        let bf_i = health::body_fat_indicator(profile.input.sex, profile.input.body_fat_pct);
        let vf_i = health::visceral_fat_indicator(profile.input.visceral_fat_level);
        let sm_i =
            health::skeletal_muscle_indicator(profile.input.sex, profile.input.skeletal_muscle_kg);

        ui.add_space(4.0);
        egui::Grid::new("fitness_metrics_grid")
            .num_columns(3)
            .spacing(egui::vec2(10.0, 6.0))
            .show(ui, |ui| {
                metric_row(ui, &bmi_i);
                metric_row(ui, &bf_i);
                metric_row(ui, &vf_i);
                metric_row(ui, &sm_i);
            });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label("创建数据时间：");
            let date_str = profile
                .input
                .data_date
                .get_or_insert_with(|| chrono::Local::now().format("%Y-%m-%d").to_string());
            ui.text_edit_singleline(date_str);
            ui.add_space(6.0);
            if ui.button("今天").clicked() {
                *date_str = chrono::Local::now().format("%Y-%m-%d").to_string();
            }
            if ui.button("明天").clicked() {
                *date_str = (chrono::Local::now().date_naive() + chrono::Duration::days(1))
                    .format("%Y-%m-%d")
                    .to_string();
            }
        });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label("体脂率(%)：");
            ui.add(egui::DragValue::new(profile.input.body_fat_pct.get_or_insert(0.0)).speed(0.1));
            ui.add_space(10.0);
            ui.label("内脏脂肪等级：");
            ui.add(
                egui::DragValue::new(profile.input.visceral_fat_level.get_or_insert(0.0))
                    .speed(0.1),
            );
            ui.add_space(10.0);
            ui.label("骨骼肌量(kg)：");
            ui.add(
                egui::DragValue::new(profile.input.skeletal_muscle_kg.get_or_insert(0.0))
                    .speed(0.1),
            );
        });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label("年龄：");
            let mut age = profile.input.age.unwrap_or(0) as i32;
            if ui.add(egui::DragValue::new(&mut age).speed(1)).changed() {
                profile.input.age = Some(age.max(0).min(120) as u8);
            }
            ui.add_space(10.0);
            ui.label("性别：");
            let mut sex = profile.input.sex.unwrap_or(Sex::Male);
            egui::ComboBox::from_id_salt("fitness_sex")
                .selected_text(match sex {
                    Sex::Male => "男",
                    Sex::Female => "女",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut sex, Sex::Male, "男");
                    ui.selectable_value(&mut sex, Sex::Female, "女");
                });
            profile.input.sex = Some(sex);
        });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label("健身条件：");
            let mut cond = profile
                .input
                .training_condition
                .unwrap_or(TrainingCondition::Equipment);
            egui::ComboBox::from_id_salt("fitness_condition")
                .selected_text(match cond {
                    TrainingCondition::Equipment => "器械",
                    TrainingCondition::Swimming => "游泳",
                    TrainingCondition::Home => "家庭",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut cond, TrainingCondition::Equipment, "器械");
                    ui.selectable_value(&mut cond, TrainingCondition::Swimming, "游泳");
                    ui.selectable_value(&mut cond, TrainingCondition::Home, "家庭");
                });
            profile.input.training_condition = Some(cond);
            ui.add_space(10.0);
            ui.label("训练时间段：");
            let s = profile
                .input
                .training_time_window
                .get_or_insert_with(|| "晚间".to_string());
            ui.text_edit_singleline(s);
        });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label("每周训练天数目标：");
            let mut days = profile.input.weekly_training_days_goal.unwrap_or(3) as i32;
            if ui.add(egui::DragValue::new(&mut days).speed(1)).changed() {
                profile.input.weekly_training_days_goal = Some(days.max(1).min(7) as u8);
            }
        });

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui.button("生成分析与计划").clicked() {
                match generate_plan(&profile.input) {
                    Ok(plan) => {
                        profile.last_plan = Some(plan);
                        *state_changed = true;
                    }
                    Err(e) => {
                        error = Some(e);
                    }
                }
            }

            if ui.button("生成并同步到 Todo").clicked() {
                match generate_plan(&profile.input) {
                    Ok(plan) => {
                        sync_plan_to_todo(todo, &profile.name, &plan);
                        profile.last_plan = Some(plan);
                        *state_changed = true;
                    }
                    Err(e) => {
                        error = Some(e);
                    }
                }
            }
        });

        ui.add_space(10.0);
        if let Some(plan) = &profile.last_plan {
            ui.separator();
            ui.add_space(8.0);
            ui.label(egui::RichText::new("计划与执行").strong());
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label(format!(
                    "阶段：{} / 建议训练天数：{}",
                    phase_text(plan.phase),
                    plan.recommended_weekly_training_days
                ));
                ui.add_space(12.0);
                ui.label(format!(
                    "开始日期：{} / 生成时间：{}",
                    plan.start_date,
                    plan.generated_at
                        .with_timezone(&chrono::Local)
                        .format("%Y-%m-%d %H:%M")
                ));
            });

            if let Some(bmi) = plan.bmi {
                ui.add_space(4.0);
                ui.label(format!("BMI：{bmi:.1}"));
            }

            if !plan.metrics.is_empty() {
                ui.add_space(8.0);
                ui.label(egui::RichText::new("指标解读").strong());
                egui::Grid::new("fitness_plan_metrics_grid")
                    .num_columns(3)
                    .spacing(egui::vec2(10.0, 6.0))
                    .show(ui, |ui| {
                        for m in &plan.metrics {
                            plan_metric_row(ui, m);
                        }
                    });
            }

            ui.add_space(8.0);
            ui.label(egui::RichText::new("建议摘要").strong());
            for line in &plan.health_summary {
                ui.label(format!("• {line}"));
            }

            let start = NaiveDate::parse_from_str(&plan.start_date, "%Y-%m-%d")
                .unwrap_or_else(|_| chrono::Local::now().date_naive());
            let today = chrono::Local::now().date_naive();

            ui.add_space(10.0);
            egui::CollapsingHeader::new("训练计划（可勾选完成 / 一键同步到 Todo）")
                .id_salt("fitness_training_plan")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("同步未完成到 Todo").clicked() {
                            let folder_id = get_or_create_fitness_folder(todo);
                            let section_name = format!("健身 - {}", profile.name);
                            let section_id =
                                get_or_create_section_in_folder(todo, &section_name, folder_id);

                            let mut changed_any = false;
                            for day in &plan.weekly_training_plan {
                                if day.title.contains("休息") {
                                    continue;
                                }
                                let date = start + Duration::days(day.day_index as i64);
                                let date_str = date.format("%Y-%m-%d").to_string();
                                if profile.completed_workout_dates.iter().any(|d| d == &date_str) {
                                    continue;
                                }
                                if sync_day_to_todo_with_section(todo, day, date, section_id) {
                                    changed_any = true;
                                }
                            }
                            if changed_any {
                                todo.save_to_file();
                                *state_changed = true;
                            }
                        }

                        if ui.button("清空完成标记").clicked() {
                            profile.completed_workout_dates.clear();
                            *state_changed = true;
                        }
                    });

                    ui.add_space(6.0);
                    egui::ScrollArea::vertical()
                        .id_salt("fitness_week_plan_scroll")
                        .max_height(360.0)
                        .show(ui, |ui| {
                            for day in &plan.weekly_training_plan {
                                let date = start + Duration::days(day.day_index as i64);
                                let date_str = date.format("%Y-%m-%d").to_string();
                                let is_today = date == today;

                                let mut done = profile
                                    .completed_workout_dates
                                    .iter()
                                    .any(|d| d == &date_str);

                                let header = format!(
                                    "{}  {}（{}min）{}",
                                    date_str,
                                    day.title,
                                    day.duration_min,
                                    if is_today { "  ← 今天" } else { "" }
                                );

                                egui::CollapsingHeader::new(
                                    egui::RichText::new(header).strong().color(if is_today {
                                        egui::Color32::from_rgb(46, 204, 113)
                                    } else {
                                        ui.visuals().text_color()
                                    }),
                                )
                                .id_salt(format!("fitness_day_{}_{}", profile.id, day.day_index))
                                .default_open(is_today)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        if ui.button("复制训练").clicked() {
                                            ui.ctx().copy_text(day.workout.join("\n"));
                                            state.error_msg = None;
                                        }

                                        if !day.title.contains("休息")
                                            && ui.button("同步到 Todo").clicked()
                                        {
                                            if sync_day_to_todo(todo, &profile.name, day, date) {
                                                *state_changed = true;
                                            }
                                        }

                                        if ui.checkbox(&mut done, "完成").changed() {
                                            if done {
                                                if !profile
                                                    .completed_workout_dates
                                                    .iter()
                                                    .any(|d| d == &date_str)
                                                {
                                                    profile.completed_workout_dates.push(
                                                        date_str.clone(),
                                                    );
                                                }
                                            } else {
                                                profile.completed_workout_dates.retain(|d| {
                                                    d != &date_str
                                                });
                                            }
                                            *state_changed = true;
                                        }
                                    });

                                    ui.add_space(4.0);
                                    for (idx, step) in day.workout.iter().enumerate() {
                                        ui.label(format!("{}. {step}", idx + 1));
                                    }
                                });

                                ui.add_space(6.0);
                            }
                        });
                });

            ui.add_space(10.0);
            egui::CollapsingHeader::new("饮食建议（按体重自动换算）")
                .id_salt("fitness_diet_plan")
                .default_open(true)
                .show(ui, |ui| {
                    let diet = &plan.weekly_diet_suggestions;
                    let w = profile.input.weight_kg.unwrap_or(0.0);
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "蛋白：{:.1} g/kg",
                            diet.daily_protein_g_per_kg
                        ));
                        ui.add_space(10.0);
                        ui.label(format!("脂肪：{:.1} g/kg", diet.daily_fat_g_per_kg));
                        ui.add_space(10.0);
                        ui.label(format!("碳水：{:.1} g/kg", diet.daily_carbs_g_per_kg));
                    });
                    if w > 0.0 {
                        ui.add_space(4.0);
                        ui.label(format!(
                            "按体重 {:.1}kg 估算：蛋白 {:.0}g / 脂肪 {:.0}g / 碳水 {:.0}g",
                            w,
                            diet.daily_protein_g_per_kg * w,
                            diet.daily_fat_g_per_kg * w,
                            diet.daily_carbs_g_per_kg * w
                        ));
                    }
                    ui.add_space(6.0);
                    for line in &diet.structure {
                        ui.label(format!("• {line}"));
                    }
                });
        }
    }

    state.error_msg = error;
}

fn metric_row(ui: &mut egui::Ui, ind: &health::MetricIndicator) {
    ui.label(ind.name);
    let (color, symbol) = match ind.level {
        health::IndicatorLevel::Good => (egui::Color32::from_rgb(46, 204, 113), "●"),
        health::IndicatorLevel::Warn => (egui::Color32::from_rgb(241, 196, 15), "●"),
        health::IndicatorLevel::Bad => (egui::Color32::from_rgb(231, 76, 60), "●"),
        health::IndicatorLevel::Unknown => (egui::Color32::GRAY, "○"),
    };
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(symbol).color(color));
        ui.label(egui::RichText::new(ind.value_text.clone()).color(color));
    });
    ui.label(egui::RichText::new(ind.reference_text.clone()).color(egui::Color32::GRAY));
    ui.end_row();
}

fn plan_metric_row(ui: &mut egui::Ui, metric: &PlanMetric) {
    ui.label(metric.name.clone());
    let (color, symbol) = match metric.level {
        MetricLevel::Green => (egui::Color32::from_rgb(46, 204, 113), "●"),
        MetricLevel::Yellow => (egui::Color32::from_rgb(241, 196, 15), "●"),
        MetricLevel::Red => (egui::Color32::from_rgb(231, 76, 60), "●"),
        MetricLevel::Unknown => (egui::Color32::GRAY, "○"),
    };
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(symbol).color(color));
        ui.label(egui::RichText::new(metric.value_text.clone()).color(color));
    });
    ui.label(egui::RichText::new(metric.reference_text.clone()).color(egui::Color32::GRAY));
    ui.end_row();
}

fn phase_text(phase: FitnessPhase) -> &'static str {
    match phase {
        FitnessPhase::FatLoss => "减脂",
        FitnessPhase::MuscleGain => "增肌",
        FitnessPhase::Maintenance => "维持",
    }
}

fn generate_plan(input: &super::model::FitnessProfileInput) -> Result<FitnessPlan, String> {
    let start_date = parse_start_date(input.data_date.as_deref())?;
    let height_cm = input.height_cm.filter(|v| *v > 0.0).ok_or("请填写身高")?;
    let weight_kg = input.weight_kg.filter(|v| *v > 0.0).ok_or("请填写体重")?;
    let bmi = health::bmi(Some(height_cm), Some(weight_kg)).ok_or("身高或体重不正确")?;

    let mut summary: Vec<String> = Vec::new();
    let bmi_ind = health::bmi_indicator(Some(bmi));
    summary.push(format!(
        "BMI：{}（{}）",
        bmi_ind.value_text, bmi_ind.reference_text
    ));

    if let Some(v) = input.body_fat_pct {
        if v > 0.0 {
            summary.push(format!("体脂率: {:.1}%", v));
        }
    }
    if let Some(v) = input.visceral_fat_level {
        if v > 0.0 {
            if v >= 12.0 {
                summary.push("内脏脂肪偏高，建议提高有氧+控制精制碳水".to_string());
            } else {
                summary.push("内脏脂肪水平可接受".to_string());
            }
        }
    }

    let sex = input.sex.unwrap_or(Sex::Male);
    let phase = health::decide_phase(sex, bmi, input.body_fat_pct, input.skeletal_muscle_kg);
    let metrics = health::build_plan_metrics(
        input.sex,
        input.height_cm,
        input.weight_kg,
        input.body_fat_pct,
        input.visceral_fat_level,
        input.skeletal_muscle_kg,
    );

    let goal_days = input.weekly_training_days_goal.unwrap_or(3).clamp(1, 7);
    let recommended_days = match phase {
        FitnessPhase::FatLoss => goal_days.max(4).min(6),
        FitnessPhase::MuscleGain => goal_days.max(3).min(5),
        FitnessPhase::Maintenance => goal_days.max(2).min(4),
    };
    let need_daily = matches!(phase, FitnessPhase::FatLoss) && recommended_days >= 6;

    let condition = input
        .training_condition
        .unwrap_or(TrainingCondition::Equipment);
    let weekly_training_plan = build_week_plan(condition, recommended_days);
    let weekly_diet_suggestions = build_diet_plan(phase);

    Ok(FitnessPlan {
        generated_at: Utc::now(),
        start_date: start_date.format("%Y-%m-%d").to_string(),
        bmi: Some(bmi),
        health_summary: summary,
        metrics,
        phase,
        need_daily_training: need_daily,
        recommended_weekly_training_days: recommended_days,
        weekly_training_plan,
        weekly_diet_suggestions,
    })
}

fn build_week_plan(cond: TrainingCondition, days: u8) -> Vec<DailyWorkoutPlan> {
    let mut out: Vec<DailyWorkoutPlan> = Vec::new();
    let mut train_days_left = days.min(7) as i32;

    for i in 0..7u8 {
        if train_days_left <= 0 {
            out.push(DailyWorkoutPlan {
                day_index: i,
                title: "休息/拉伸".to_string(),
                duration_min: 20,
                workout: vec![
                    "轻松步行 20-40 分钟".to_string(),
                    "拉伸 10 分钟".to_string(),
                ],
            });
            continue;
        }
        train_days_left -= 1;

        let (title, workout, duration) = match cond {
            TrainingCondition::Swimming => (
                "游泳+核心".to_string(),
                vec![
                    "热身 10 分钟".to_string(),
                    "游泳 30-45 分钟（间歇/耐力）".to_string(),
                    "核心 10 分钟（平板/死虫）".to_string(),
                ],
                60,
            ),
            TrainingCondition::Home => (
                "家庭全身".to_string(),
                vec![
                    "热身 8 分钟".to_string(),
                    "深蹲/弓步 4x12".to_string(),
                    "俯卧撑/哑铃推 4x10".to_string(),
                    "划船/弹力带 4x12".to_string(),
                    "核心 8 分钟".to_string(),
                ],
                50,
            ),
            TrainingCondition::Equipment => match i % 3 {
                0 => (
                    "下肢力量".to_string(),
                    vec![
                        "热身 8 分钟".to_string(),
                        "深蹲/腿举 4x8-12".to_string(),
                        "硬拉/腿弯举 3x8-12".to_string(),
                        "小腿/臀中肌 3x12-15".to_string(),
                        "有氧 15-25 分钟".to_string(),
                    ],
                    70,
                ),
                1 => (
                    "上肢力量".to_string(),
                    vec![
                        "热身 8 分钟".to_string(),
                        "卧推/推举 4x8-12".to_string(),
                        "划船/下拉 4x8-12".to_string(),
                        "手臂/肩后束 3x12-15".to_string(),
                        "有氧 10-20 分钟".to_string(),
                    ],
                    70,
                ),
                _ => (
                    "全身+有氧".to_string(),
                    vec![
                        "热身 8 分钟".to_string(),
                        "全身复合动作 4 组（深蹲/推/拉）".to_string(),
                        "间歇有氧 20 分钟".to_string(),
                        "拉伸 10 分钟".to_string(),
                    ],
                    60,
                ),
            },
        };

        out.push(DailyWorkoutPlan {
            day_index: i,
            title,
            duration_min: duration,
            workout,
        });
    }

    out
}

fn build_diet_plan(phase: FitnessPhase) -> DietPlan {
    match phase {
        FitnessPhase::FatLoss => DietPlan {
            daily_protein_g_per_kg: 1.8,
            daily_fat_g_per_kg: 0.8,
            daily_carbs_g_per_kg: 2.0,
            structure: vec![
                "每餐优先蛋白：瘦肉/鱼虾/蛋/奶/豆制品".to_string(),
                "主食选择低 GI：米饭减量+杂粮/土豆/燕麦".to_string(),
                "蔬菜每日至少 500g，保证纤维".to_string(),
                "减少含糖饮料与油炸食品".to_string(),
            ],
        },
        FitnessPhase::MuscleGain => DietPlan {
            daily_protein_g_per_kg: 2.0,
            daily_fat_g_per_kg: 1.0,
            daily_carbs_g_per_kg: 4.0,
            structure: vec![
                "训练前后补碳水+蛋白：香蕉/面包+酸奶/乳清".to_string(),
                "主食足量：米饭/面/土豆/燕麦".to_string(),
                "蛋白分配到 3-4 餐".to_string(),
                "睡前补充慢消化蛋白：牛奶/酸奶".to_string(),
            ],
        },
        FitnessPhase::Maintenance => DietPlan {
            daily_protein_g_per_kg: 1.6,
            daily_fat_g_per_kg: 0.9,
            daily_carbs_g_per_kg: 3.0,
            structure: vec![
                "均衡三大营养：蛋白/碳水/脂肪".to_string(),
                "每周 1-2 次高蛋白餐巩固训练效果".to_string(),
                "保持蔬菜与水果摄入".to_string(),
                "保证睡眠与饮水".to_string(),
            ],
        },
    }
}

fn sync_plan_to_todo(todo: &mut TodoState, person: &str, plan: &FitnessPlan) {
    let folder_id = get_or_create_fitness_folder(todo);
    let section_name = format!("健身 - {person}");
    let section_id = get_or_create_section_in_folder(todo, &section_name, folder_id);

    let today = NaiveDate::parse_from_str(&plan.start_date, "%Y-%m-%d")
        .unwrap_or_else(|_| chrono::Local::now().date_naive());
    let mut changed_any = false;
    for day in &plan.weekly_training_plan {
        if day.title.contains("休息") {
            continue;
        }
        let date = today + chrono::Duration::days(day.day_index as i64);
        if sync_day_to_todo_with_section(todo, day, date, section_id) {
            changed_any = true;
        }
    }

    if changed_any {
        todo.save_to_file();
    }
}

fn sync_day_to_todo(
    todo: &mut TodoState,
    person: &str,
    day: &DailyWorkoutPlan,
    date: NaiveDate,
) -> bool {
    let folder_id = get_or_create_fitness_folder(todo);
    let section_name = format!("健身 - {person}");
    let section_id = get_or_create_section_in_folder(todo, &section_name, folder_id);
    let changed = sync_day_to_todo_with_section(todo, day, date, section_id);
    if changed {
        todo.save_to_file();
    }
    changed
}

fn sync_day_to_todo_with_section(
    todo: &mut TodoState,
    day: &DailyWorkoutPlan,
    date: NaiveDate,
    section_id: uuid::Uuid,
) -> bool {
    let due_at = local_date_to_utc_end_of_day(date);
    let title = format!("健身：{}（{}min）", day.title, day.duration_min);

    if todo.items.iter().any(|item| {
        item.deleted_at.is_none()
            && item.title == title
            && item.due_at == due_at
            && item.section_id == Some(section_id)
    }) {
        return false;
    }

    let mut item = TodoItem::new(title, Some(day.workout.join("\n")), Some(section_id));
    item.due_at = due_at;
    todo.items.push(item);
    true
}

fn parse_start_date(input: Option<&str>) -> Result<NaiveDate, String> {
    let s = input.unwrap_or("").trim();
    if s.is_empty() {
        return Ok(chrono::Local::now().date_naive());
    }
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|_| "创建数据时间格式应为 YYYY-MM-DD".to_string())
}

fn local_date_to_utc_end_of_day(date: chrono::NaiveDate) -> Option<chrono::DateTime<chrono::Utc>> {
    let naive = date.and_hms_opt(23, 59, 0)?;
    let local = chrono::Local.from_local_datetime(&naive).single()?;
    Some(local.with_timezone(&chrono::Utc))
}

fn get_or_create_fitness_folder(todo: &mut TodoState) -> uuid::Uuid {
    if let Some(f) = todo.folders.iter().find(|f| f.name == "健身") {
        return f.id;
    }
    let folder = crate::todo::TodoFolder::new("健身".to_string());
    let id = folder.id;
    todo.folders.push(folder);
    id
}

fn get_or_create_section_in_folder(
    todo: &mut TodoState,
    name: &str,
    folder_id: uuid::Uuid,
) -> uuid::Uuid {
    if let Some(s) = todo.sections.iter().find(|s| s.name == name) {
        return s.id;
    }
    let mut section = crate::todo::TodoSection::new(name.to_string());
    section.folder_id = Some(folder_id);
    let id = section.id;
    todo.sections.push(section);
    id
}
