use super::model::{FitnessPhase, MetricLevel, PlanMetric, Sex};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum IndicatorLevel {
    Good,
    Warn,
    Bad,
    Unknown,
}

pub struct MetricIndicator {
    pub name: &'static str,
    pub value_text: String,
    pub reference_text: String,
    pub level: IndicatorLevel,
}

pub fn build_plan_metrics(
    sex: Option<Sex>,
    height_cm: Option<f32>,
    weight_kg: Option<f32>,
    body_fat_pct: Option<f32>,
    visceral_fat_level: Option<f32>,
    skeletal_muscle_kg: Option<f32>,
) -> Vec<PlanMetric> {
    let bmi_ind = bmi_indicator(bmi(height_cm, weight_kg));
    let bf_ind = body_fat_indicator(sex, body_fat_pct);
    let vf_ind = visceral_fat_indicator(visceral_fat_level);
    let sm_ind = skeletal_muscle_indicator(sex, skeletal_muscle_kg);

    vec![
        indicator_to_plan_metric("bmi", &bmi_ind),
        indicator_to_plan_metric("body_fat_pct", &bf_ind),
        indicator_to_plan_metric("visceral_fat_level", &vf_ind),
        indicator_to_plan_metric("skeletal_muscle_kg", &sm_ind),
    ]
}

fn indicator_to_plan_metric(key: &'static str, ind: &MetricIndicator) -> PlanMetric {
    PlanMetric {
        key: key.to_string(),
        name: ind.name.to_string(),
        value_text: ind.value_text.clone(),
        reference_text: ind.reference_text.clone(),
        level: match ind.level {
            IndicatorLevel::Good => MetricLevel::Green,
            IndicatorLevel::Warn => MetricLevel::Yellow,
            IndicatorLevel::Bad => MetricLevel::Red,
            IndicatorLevel::Unknown => MetricLevel::Unknown,
        },
    }
}

pub fn bmi(height_cm: Option<f32>, weight_kg: Option<f32>) -> Option<f32> {
    let h_cm = height_cm?;
    let w = weight_kg?;
    if h_cm <= 0.0 || w <= 0.0 {
        return None;
    }
    let h = h_cm / 100.0;
    Some(w / (h * h))
}

pub fn bmi_indicator(bmi: Option<f32>) -> MetricIndicator {
    let Some(v) = bmi else {
        return MetricIndicator {
            name: "BMI",
            value_text: "未填写".to_string(),
            reference_text: "参考：18.5-23.9".to_string(),
            level: IndicatorLevel::Unknown,
        };
    };

    let (level, ref_text) = if v < 18.5 {
        (IndicatorLevel::Warn, "偏低：< 18.5")
    } else if v < 24.0 {
        (IndicatorLevel::Good, "正常：18.5-23.9")
    } else if v < 28.0 {
        (IndicatorLevel::Warn, "超重：24.0-27.9")
    } else {
        (IndicatorLevel::Bad, "肥胖：≥ 28.0")
    };

    MetricIndicator {
        name: "BMI",
        value_text: format!("{v:.1}"),
        reference_text: format!("参考：{ref_text}"),
        level,
    }
}

pub fn body_fat_indicator(sex: Option<Sex>, body_fat_pct: Option<f32>) -> MetricIndicator {
    let Some(v) = body_fat_pct.filter(|v| *v > 0.0) else {
        return MetricIndicator {
            name: "体脂率",
            value_text: "未填写".to_string(),
            reference_text: "参考：男 10-20 / 女 18-30".to_string(),
            level: IndicatorLevel::Unknown,
        };
    };

    let (level, ref_text) = match sex.unwrap_or(Sex::Male) {
        Sex::Male => {
            if v < 10.0 {
                (IndicatorLevel::Warn, "偏低：< 10")
            } else if v <= 20.0 {
                (IndicatorLevel::Good, "正常：10-20")
            } else if v <= 25.0 {
                (IndicatorLevel::Warn, "偏高：20-25")
            } else {
                (IndicatorLevel::Bad, "过高：> 25")
            }
        }
        Sex::Female => {
            if v < 18.0 {
                (IndicatorLevel::Warn, "偏低：< 18")
            } else if v <= 30.0 {
                (IndicatorLevel::Good, "正常：18-30")
            } else if v <= 35.0 {
                (IndicatorLevel::Warn, "偏高：30-35")
            } else {
                (IndicatorLevel::Bad, "过高：> 35")
            }
        }
    };

    MetricIndicator {
        name: "体脂率",
        value_text: format!("{v:.1}%"),
        reference_text: format!("参考：{ref_text}"),
        level,
    }
}

pub fn visceral_fat_indicator(visceral_fat_level: Option<f32>) -> MetricIndicator {
    let Some(v) = visceral_fat_level.filter(|v| *v > 0.0) else {
        return MetricIndicator {
            name: "内脏脂肪",
            value_text: "未填写".to_string(),
            reference_text: "参考：< 10".to_string(),
            level: IndicatorLevel::Unknown,
        };
    };

    let (level, ref_text) = if v < 10.0 {
        (IndicatorLevel::Good, "正常：< 10")
    } else if v < 15.0 {
        (IndicatorLevel::Warn, "偏高：10-14")
    } else {
        (IndicatorLevel::Bad, "过高：≥ 15")
    };

    MetricIndicator {
        name: "内脏脂肪",
        value_text: format!("{v:.1}"),
        reference_text: format!("参考：{ref_text}"),
        level,
    }
}

pub fn skeletal_muscle_indicator(
    sex: Option<Sex>,
    skeletal_muscle_kg: Option<f32>,
) -> MetricIndicator {
    let Some(v) = skeletal_muscle_kg.filter(|v| *v > 0.0) else {
        return MetricIndicator {
            name: "骨骼肌量",
            value_text: "未填写".to_string(),
            reference_text: "参考：男 ≥ 30 / 女 ≥ 22".to_string(),
            level: IndicatorLevel::Unknown,
        };
    };

    let (level, ref_text) = match sex.unwrap_or(Sex::Male) {
        Sex::Male => {
            if v >= 30.0 {
                (IndicatorLevel::Good, "良好：≥ 30kg")
            } else if v >= 27.0 {
                (IndicatorLevel::Warn, "可提升：27-29.9kg")
            } else {
                (IndicatorLevel::Bad, "偏低：< 27kg")
            }
        }
        Sex::Female => {
            if v >= 22.0 {
                (IndicatorLevel::Good, "良好：≥ 22kg")
            } else if v >= 19.0 {
                (IndicatorLevel::Warn, "可提升：19-21.9kg")
            } else {
                (IndicatorLevel::Bad, "偏低：< 19kg")
            }
        }
    };

    MetricIndicator {
        name: "骨骼肌量",
        value_text: format!("{v:.1}kg"),
        reference_text: format!("参考：{ref_text}"),
        level,
    }
}

pub fn decide_phase(
    sex: Sex,
    bmi: f32,
    body_fat_pct: Option<f32>,
    skeletal_muscle_kg: Option<f32>,
) -> FitnessPhase {
    let bf = body_fat_pct.unwrap_or(0.0);
    let muscle = skeletal_muscle_kg.unwrap_or(0.0);

    let bf_high = match sex {
        Sex::Male => bf > 20.0,
        Sex::Female => bf > 30.0,
    };
    let muscle_low = muscle > 0.0
        && match sex {
            Sex::Male => muscle < 28.0,
            Sex::Female => muscle < 20.0,
        };

    if bmi >= 24.0 || bf_high {
        FitnessPhase::FatLoss
    } else if muscle_low {
        FitnessPhase::MuscleGain
    } else {
        FitnessPhase::Maintenance
    }
}
