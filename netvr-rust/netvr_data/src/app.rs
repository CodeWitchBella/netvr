use serde::{Deserialize, Serialize};

use crate::Pose;

/// List of all poses of all objects in the scene
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Snapshot {
    pub objects: Vec<Pose>,
}

/// Messages that the client can send to the server about the syncrhonized objects
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AppUp {
    Init(Snapshot),
    Grab(u32),
}

/// Messages that the server can send to the client about the synchronized objects
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AppDown {
    Release(u32),
}

/// Datagrams that the client can send to the server about the synchronized objects
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AppDatagramUp {
    SetPose(usize, Pose),
}
