use std::{collections::HashMap, fmt::Debug};

use serde::{Deserialize, Serialize};

use crate::{app, Pose};

/// Response from discovery server
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

/// Space to be used for calibration sample collection
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum BaseSpace {
    Server,
    Stage,
}

/// What is sent for changing calibration from server to clients
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ConfigurationDown {
    Snapshot(ConfigurationSnapshotSet),
    SetServerSpacePose(Pose),
    TriggerCalibration(String, CalibrationConfiguration, BaseSpace),
    RequestSample(String, BaseSpace),
    StopCalibration,
    ChangeName(String),
}

/// Controller data
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Controller {
    pub interaction_profile: u8,
    pub user_path: u8,
    pub pose: Pose,
}

/// State snapshot
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct StateSnapshot {
    pub controllers: Vec<Controller>,
    pub view: Pose,
    pub required_configuration: u32,
}

/// Type of OpenXR acction but serializable
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

#[cfg(not(target_arch = "wasm32"))]
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

/// Remote OpenXR action data
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct RemoteAction {
    #[serde(rename = "type")]
    pub ty: ActionType,
    pub name: String,
    pub localized_name: String,
    pub binding: String,
}

/// Remote interaction profile data
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct RemoteInteractionProfile {
    pub path: String,
    pub bindings: Vec<RemoteAction>,
}

/// Data of a remote configuration.
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct RemoteConfigurationSnapshot {
    pub version: u32,
    pub user_paths: Vec<String>,
    pub interaction_profiles: Vec<RemoteInteractionProfile>,
    pub name: String,
}

/// Data of all remote configurations.
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct ConfigurationSnapshotSet {
    pub clients: HashMap<ClientId, RemoteConfigurationSnapshot>,
}

/// What is sent when local configuration changes to notify the server
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ConfigurationUp {
    Hello,
    ConfigurationSnapshot(RemoteConfigurationSnapshot),
}

/// Sample used for calibration
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CalibrationSample {
    pub flags: u64,
    pub pose: Pose,
    pub prev_flags: Option<u64>,
    pub prev_pose: Option<Pose>,
    /// This is the time that we used when requesting the sample.
    pub nanos: i64,
    /// This is when the sample was requested.
    pub now_nanos: i64,
}

/// Configratiuon options for calibration
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct CalibrationConfiguration {
    pub sample_count: usize,
    pub sample_interval_nanos: i64,
}

/// Id of a client
pub type ClientId = u32;

/// All remote snapshots + order to make sure stale snapshots are not applied.
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct RemoteStateSnapshotSet {
    /// Makes sure that we do not apply older snapshots, if they arrive out of
    /// order.
    pub order: usize,
    pub clients: HashMap<ClientId, StateSnapshot>,
}

/// What is sent from server to client over unrealiable channel
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum DatagramDown {
    App(app::Snapshot),
    State(RemoteStateSnapshotSet),
}

/// Whati s sent from client to server over unrealiable channel
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum DatagramUp {
    State(StateSnapshot),
    App(app::AppDatagramUp),
}

/// Sent periodically to check that connection is still alive
/// ... this was triumph, I'm making a note here: huge success
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
