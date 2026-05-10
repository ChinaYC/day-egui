use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::model::{FitnessPlan, FitnessProfile};

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct FitnessState {
    pub profiles: Vec<FitnessProfile>,

    #[serde(skip)]
    pub selected_profile: Option<Uuid>,
    #[serde(skip)]
    pub new_profile_name: String,
    #[serde(skip)]
    pub import_json: String,
    #[serde(skip)]
    pub last_export_json: String,
    #[serde(skip)]
    pub error_msg: Option<String>,
}

impl Default for FitnessState {
    fn default() -> Self {
        Self {
            profiles: Vec::new(),
            selected_profile: None,
            new_profile_name: String::new(),
            import_json: String::new(),
            last_export_json: String::new(),
            error_msg: None,
        }
    }
}

impl FitnessState {
    pub fn selected_profile_mut(&mut self) -> Option<&mut FitnessProfile> {
        let id = self.selected_profile?;
        self.profiles.iter_mut().find(|p| p.id == id)
    }

    pub fn selected_profile(&self) -> Option<&FitnessProfile> {
        let id = self.selected_profile?;
        self.profiles.iter().find(|p| p.id == id)
    }

    pub fn set_last_plan(&mut self, profile_id: Uuid, plan: FitnessPlan) {
        if let Some(p) = self.profiles.iter_mut().find(|p| p.id == profile_id) {
            p.last_plan = Some(plan);
        }
    }
}
