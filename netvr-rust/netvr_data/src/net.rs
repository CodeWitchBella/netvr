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

#[derive(Serialize, Deserialize)]
pub struct ConfigurationDown {
    header: [u8; 5],
    pub port: u16,
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

pub trait ExtraMarker {}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Action<ActionExtra> {
    #[serde(rename = "type")]
    pub ty: ActionType,
    pub name: String,
    pub localized_name: String,
    pub binding: String,
    pub extra: ActionExtra,
}

impl<T: ExtraMarker> From<Action<T>> for Action<()> {
    fn from(action: Action<T>) -> Self {
        Self {
            ty: action.ty,
            name: action.name,
            localized_name: action.localized_name,
            binding: action.binding,
            extra: (),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct InteractionProfile<ActionExtra> {
    pub path: String,
    pub bindings: Vec<Action<ActionExtra>>,
    #[serde(skip)]
    pub path_handle: u64,
}

impl<T: ExtraMarker> From<InteractionProfile<T>> for InteractionProfile<()> {
    fn from(profile: InteractionProfile<T>) -> Self {
        Self {
            path: profile.path,
            bindings: profile.bindings.into_iter().map(|b| b.into()).collect(),
            path_handle: profile.path_handle,
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct LocalConfigurationSnapshot<ActionExtra> {
    pub version: u32,
    pub interaction_profiles: Vec<InteractionProfile<ActionExtra>>,
}

impl<T: ExtraMarker> From<LocalConfigurationSnapshot<T>> for LocalConfigurationSnapshot<()> {
    fn from(snapshot: LocalConfigurationSnapshot<T>) -> Self {
        Self {
            version: snapshot.version,
            interaction_profiles: snapshot
                .interaction_profiles
                .into_iter()
                .map(|p| p.into())
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct RemoteConfigurationSnapshot {
    pub clients: HashMap<ClientId, LocalConfigurationSnapshot<()>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ConfigurationUp {
    Hello,
    ConfigurationSnapshot(LocalConfigurationSnapshot<()>),
}

pub type ClientId = u32;

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct RemoteStateSnapshot {
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
