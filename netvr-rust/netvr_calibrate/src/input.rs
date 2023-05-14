use netvr_data::{net::CalibrationSample, Quaternion, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Default, Clone, Debug)]
pub struct CalibrationResult {
    pub translation: Vec3,
    pub rotation: Quaternion,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CalibrationInput {
    pub target: Vec<CalibrationSample>,
    pub target_name: String,
    pub reference: Vec<CalibrationSample>,
    pub reference_name: String,
}
