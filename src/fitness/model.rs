use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Sex {
    Male,
    Female,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum TrainingCondition {
    Equipment,
    Swimming,
    Home,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum FitnessPhase {
    FatLoss,
    MuscleGain,
    Maintenance,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum MetricLevel {
    Green,
    Yellow,
    Red,
    Unknown,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PlanMetric {
    pub key: String,
    pub name: String,
    pub value_text: String,
    pub reference_text: String,
    pub level: MetricLevel,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct FitnessProfileInput {
    pub height_cm: Option<f32>,
    pub weight_kg: Option<f32>,
    pub body_fat_pct: Option<f32>,
    pub visceral_fat_level: Option<f32>,
    pub skeletal_muscle_kg: Option<f32>,
    pub age: Option<u8>,
    pub sex: Option<Sex>,
    pub training_condition: Option<TrainingCondition>,
    pub training_time_window: Option<String>,
    pub weekly_training_days_goal: Option<u8>,
    pub data_date: Option<String>,
}

impl Default for FitnessProfileInput {
    fn default() -> Self {
        Self {
            height_cm: None,
            weight_kg: None,
            body_fat_pct: None,
            visceral_fat_level: None,
            skeletal_muscle_kg: None,
            age: None,
            sex: None,
            training_condition: None,
            training_time_window: None,
            weekly_training_days_goal: Some(3),
            data_date: Some(default_today_string()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FitnessProfile {
    pub id: Uuid,
    pub name: String,
    #[serde(default = "utc_now")]
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub input: FitnessProfileInput,
    #[serde(default)]
    pub last_plan: Option<FitnessPlan>,
    #[serde(default)]
    pub completed_workout_dates: Vec<String>,
}

impl FitnessProfile {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            created_at: utc_now(),
            input: FitnessProfileInput::default(),
            last_plan: None,
            completed_workout_dates: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FitnessPlan {
    pub generated_at: DateTime<Utc>,
    pub start_date: String,
    pub bmi: Option<f32>,
    pub health_summary: Vec<String>,
    #[serde(default)]
    pub metrics: Vec<PlanMetric>,
    pub phase: FitnessPhase,
    pub need_daily_training: bool,
    pub recommended_weekly_training_days: u8,
    pub weekly_training_plan: Vec<DailyWorkoutPlan>,
    pub weekly_diet_suggestions: DietPlan,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DailyWorkoutPlan {
    pub day_index: u8,
    pub title: String,
    pub duration_min: u16,
    pub workout: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DietPlan {
    pub daily_protein_g_per_kg: f32,
    pub daily_fat_g_per_kg: f32,
    pub daily_carbs_g_per_kg: f32,
    pub structure: Vec<String>,
}

fn utc_now() -> DateTime<Utc> {
    Utc::now()
}

fn default_today_string() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}
