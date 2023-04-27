use netvr_data::{net::CalibrationSample, Quaternion, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug)]
pub struct CalibrationResult {
    pub translation: Vec3,
    pub rotation: Quaternion,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CalibrationInput {
    pub target: Vec<CalibrationSample>,
    pub reference: Vec<CalibrationSample>,
}
