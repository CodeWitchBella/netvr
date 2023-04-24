use std::{collections::HashMap, fmt::Debug};

use serde::{Deserialize, Serialize};

use crate::Pose;

#[derive(Serialize, Deserialize)]
pub struct DiscoveryResponse {
    header: [u8; 5],
}

impl DiscoveryResponse {
    pub fn validate_header(&self) -> bool {
        self.header == [b'n', b'e', b't', b'v', b'r']
    }
}

impl Default for DiscoveryResponse {
    fn default() -> Self {
        Self {
            header: [b'n', b'e', b't', b'v', b'r'],
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct ConfigurationDown {
    pub snap: RemoteConfigurationSnapshotSet,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct LocalStateSnapshot {
    pub controllers: Vec<Pose>,
    pub views: Vec<Pose>,
    pub required_configuration: u32,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub enum ActionType {
    Boolean = 1,
    Float = 2,
    Vector2f = 3,
    Pose = 4,
    VibrationOutput = 100,
    #[default]
    Unknown,
}

impl From<openxr_sys::ActionType> for ActionType {
    fn from(action_type: openxr_sys::ActionType) -> Self {
        match action_type {
            openxr_sys::ActionType::BOOLEAN_INPUT => ActionType::Boolean,
            openxr_sys::ActionType::FLOAT_INPUT => ActionType::Float,
            openxr_sys::ActionType::VECTOR2F_INPUT => ActionType::Vector2f,
            openxr_sys::ActionType::POSE_INPUT => ActionType::Pose,
            openxr_sys::ActionType::VIBRATION_OUTPUT => ActionType::VibrationOutput,
            _ => ActionType::Unknown,
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct RemoteAction {
    #[serde(rename = "type")]
    pub ty: ActionType,
    pub name: String,
    pub localized_name: String,
    pub binding: String,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct RemoteInteractionProfile {
    pub path: String,
    pub bindings: Vec<RemoteAction>,
    #[serde(skip)]
    pub path_handle: u64,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct RemoteConfigurationSnapshot {
    pub version: u32,
    pub interaction_profiles: Vec<RemoteInteractionProfile>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct RemoteConfigurationSnapshotSet {
    pub clients: HashMap<ClientId, RemoteConfigurationSnapshot>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ConfigurationUp {
    Hello,
    ConfigurationSnapshot(RemoteConfigurationSnapshot),
}

pub type ClientId = u32;

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct RemoteStateSnapshotSet {
    /// Makes sure that we do not apply older snapshots, if they arrive out of
    /// order.
    pub order: usize,
    pub clients: HashMap<ClientId, LocalStateSnapshot>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Heartbeat {
    _buf: [u8; 5],
}
impl Default for Heartbeat {
    fn default() -> Self {
        Self {
            _buf: [b'h', b'e', b'l', b'l', b'o'],
        }
    }
}
