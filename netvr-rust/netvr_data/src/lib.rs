use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Device {
    id: u32,
    x: f32,
    y: f32,
    z: f32,
    qx: f32,
    qy: f32,
    qz: f32,
    qw: f32,
}
