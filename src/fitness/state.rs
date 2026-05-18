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
    #[serde(skip)]
    pub error_expires_at: Option<std::time::Instant>,
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
            error_expires_at: None,
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

    pub fn set_error(&mut self, msg: impl Into<String>) {
        self.error_msg = Some(msg.into());
        self.error_expires_at = Some(std::time::Instant::now() + std::time::Duration::from_secs(5));
    }

    pub fn clear_error(&mut self) {
        self.error_msg = None;
        self.error_expires_at = None;
    }

    pub fn update_error_timeout(&mut self) {
        if let Some(expires_at) = self.error_expires_at {
            if std::time::Instant::now() > expires_at {
                self.error_msg = None;
                self.error_expires_at = None;
            }
        }
    }
}
