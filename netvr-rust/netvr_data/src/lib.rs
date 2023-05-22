pub mod app;
mod framing;
pub mod handle_serializer;

use std::collections::HashMap;

use net::{ClientId, RemoteConfigurationSnapshot, StateSnapshot};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[cfg(not(target_arch = "wasm32"))]
impl From<openxr_sys::Vector3f> for Vec3 {
    fn from(v: openxr_sys::Vector3f) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<Vec3> for openxr_sys::Vector3f {
    fn from(v: Vec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Default for Quaternion {
    fn default() -> Self {
        Self {
            x: Default::default(),
            y: Default::default(),
            z: Default::default(),
            w: 1.,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<openxr_sys::Quaternionf> for Quaternion {
    fn from(v: openxr_sys::Quaternionf) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
            w: v.w,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<Quaternion> for openxr_sys::Quaternionf {
    fn from(v: Quaternion) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
            w: v.w,
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct RemoteDevice {
    // TODO: consider changing to u64 to be able to comfortably fit client id +
    // device id
    pub id: u32,
    pub pos: Vec3,
    pub rot: Quaternion,
    pub user_path: String,
    pub interaction_profile: String,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct Pose {
    pub position: Vec3,
    pub orientation: Quaternion,
}

#[cfg(not(target_arch = "wasm32"))]
impl From<openxr_sys::Posef> for Pose {
    fn from(v: openxr_sys::Posef) -> Self {
        Self {
            position: v.position.into(),
            orientation: v.orientation.into(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<Pose> for openxr_sys::Posef {
    fn from(v: Pose) -> Self {
        Self {
            position: v.position.into(),
            orientation: v.orientation.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct ReadRemoteDevicesOutput {
    pub devices: Vec<RemoteDevice>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize, Deserialize)]
pub struct JustInstance {
    #[serde(with = "handle_serializer::instance")]
    pub instance: openxr_sys::Instance,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize, Deserialize)]
pub struct InstanceAndSession {
    #[serde(with = "handle_serializer::instance")]
    pub instance: openxr_sys::Instance,
    #[serde(with = "handle_serializer::session")]
    pub session: openxr_sys::Session,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize, Deserialize)]
pub struct StartInput {
    #[serde(with = "handle_serializer::instance")]
    pub instance: openxr_sys::Instance,
    #[serde(with = "handle_serializer::session")]
    pub session: openxr_sys::Session,
    pub data_directory: String,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct RemoteClientSnapshot {
    pub configuration: RemoteConfigurationSnapshot,
    pub state: StateSnapshot,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct RemoteSnapshot {
    pub clients: HashMap<ClientId, RemoteClientSnapshot>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Nothing(u8);

#[derive(Serialize, Deserialize, Default)]
pub struct OnlyString(pub String);

#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize, Deserialize)]
pub struct InitRemoteObjectsInput {
    #[serde(with = "handle_serializer::instance")]
    pub instance: openxr_sys::Instance,
    #[serde(with = "handle_serializer::session")]
    pub session: openxr_sys::Session,
    pub snapshot: app::Snapshot,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize, Deserialize)]
pub struct GrabInput {
    #[serde(with = "handle_serializer::instance")]
    pub instance: openxr_sys::Instance,
    #[serde(with = "handle_serializer::session")]
    pub session: openxr_sys::Session,
    pub object_id: u32,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize, Deserialize)]
pub struct SetPoseInput {
    #[serde(with = "handle_serializer::instance")]
    pub instance: openxr_sys::Instance,
    #[serde(with = "handle_serializer::session")]
    pub session: openxr_sys::Session,
    pub object_id: u32,
    pub pose: Pose,
}

/// This structure is not meant to be used directly but rather as a holder for
/// all other structures that are used for serialization. This is to make sure
/// that required code is generated for all structures without having to update
/// the list in build.rs.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize, Deserialize)]
pub struct CodegenRoot(
    pub ReadRemoteDevicesOutput,
    pub JustInstance,
    pub Nothing,
    pub InstanceAndSession,
    pub RemoteSnapshot,
    pub StartInput,
    pub Snapshot,
    pub InitRemoteObjectsInput,
    pub GrabInput,
    pub SetPoseInput,
    pub OnlyString,
);

pub mod net;
pub use app::Snapshot;
pub use bincode;
#[cfg(not(target_arch = "wasm32"))]
pub use framing::*;
pub use serde;
