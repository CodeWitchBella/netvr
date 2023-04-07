use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
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

#[derive(Serialize, Deserialize, Default)]
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
    pub id: u32,
    pub pos: Vec3,
    pub rot: Quaternion,
}

#[derive(Serialize, Deserialize, Default)]
pub struct RemoteDevices {
    pub devices: Vec<RemoteDevice>,
}
