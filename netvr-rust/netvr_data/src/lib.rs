mod framing;
pub mod handle_serializer;

use std::collections::HashMap;

use net::{ClientId, RemoteConfigurationSnapshot, StateSnapshot};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<openxr_sys::Vector3f> for Vec3 {
    fn from(v: openxr_sys::Vector3f) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

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

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Pose {
    pub position: Vec3,
    pub orientation: Quaternion,
}

impl From<openxr_sys::Posef> for Pose {
    fn from(v: openxr_sys::Posef) -> Self {
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

#[derive(Serialize, Deserialize)]
pub struct JustInstance {
    #[serde(with = "handle_serializer::instance")]
    pub instance: openxr_sys::Instance,
}

#[derive(Serialize, Deserialize)]
pub struct InstanceAndSession {
    #[serde(with = "handle_serializer::instance")]
    pub instance: openxr_sys::Instance,
    #[serde(with = "handle_serializer::session")]
    pub session: openxr_sys::Session,
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

/// This structure is not meant to be used directly but rather as a holder for
/// all other structures that are used for serialization. This is to make sure
/// that required code is generated for all structures without having to update
/// the list in build.rs.
#[derive(Serialize, Deserialize)]
pub struct CodegenRoot(
    pub ReadRemoteDevicesOutput,
    pub JustInstance,
    pub Nothing,
    pub InstanceAndSession,
    pub RemoteSnapshot,
);

pub mod net;
pub use bincode;
pub use framing::*;
pub use serde;
