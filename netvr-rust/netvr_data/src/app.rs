use serde::{Deserialize, Serialize};

use crate::Pose;

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Snapshot {
    pub objects: Vec<Pose>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AppUp {
    Init(Snapshot),
    Grab(u32),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AppDown {
    Release(u32),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AppDatagramUp {
    SetPose(usize, Pose),
}
